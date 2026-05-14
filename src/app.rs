//! Test session state, scoring, and lifecycle.
//!
//! `App` holds both configuration (`source`, `limit`, `lang`, `difficulty`)
//! and per-run state (`typed`, counters, timestamps). The invariant
//! `colors.len() == target.len()` is maintained by all constructors and
//! relied on by the renderer.

use std::time::Instant;

use rand::seq::SliceRandom;
use ratatui::style::Color;

use crate::cli::{
    limits_for, next_in, prev_in, Difficulty, Lang, Limit, Source, DIFFICULTIES, LANGS, SOURCES,
};
use crate::code;
use crate::words;

const CHARS_PER_WORD: f64 = 5.0;
const MIN_ELAPSED_FOR_WPM: f64 = 0.1;
const CODE_CHARS_PER_SECOND: usize = 25;
const CODE_BUFFER_FLOOR: usize = 2000;
const TEXT_POOL_FOR_TIME: usize = 500;

pub struct App {
    pub target: Vec<char>,
    pub typed: Vec<char>,
    pub colors: Vec<Color>,
    pub correct: usize,
    pub incorrect: usize,
    pub keystrokes: usize,
    pub start: Option<Instant>,
    pub finished: bool,
    pub end_elapsed: Option<f64>,
    pub source: Source,
    pub limit: Limit,
    pub lang: Lang,
    pub difficulty: Difficulty,
    pub target_words: usize,
}

impl App {
    pub fn new(source: Source, limit: Limit, lang: Lang, difficulty: Difficulty) -> Self {
        let (target_str, colors, target_words) = match source {
            Source::Code => build_code(lang, limit),
            Source::Text => build_text(difficulty, limit),
        };
        Self {
            target: target_str.chars().collect(),
            typed: Vec::new(),
            colors,
            correct: 0,
            incorrect: 0,
            keystrokes: 0,
            start: None,
            finished: false,
            end_elapsed: None,
            source,
            limit,
            lang,
            difficulty,
            target_words,
        }
    }

    /// Recreate the session with the current config, discarding typing
    /// progress.
    pub fn restart(&mut self) {
        *self = App::new(self.source, self.limit, self.lang, self.difficulty);
    }

    pub fn cycle_limit_next(&mut self) {
        self.limit = next_in(limits_for(self.source), self.limit);
        self.restart();
    }

    pub fn cycle_limit_prev(&mut self) {
        self.limit = prev_in(limits_for(self.source), self.limit);
        self.restart();
    }

    pub fn cycle_source_next(&mut self) {
        let new_source = next_in(SOURCES, self.source);
        self.source = new_source;
        self.limit = clamp_limit(self.limit, new_source);
        self.restart();
    }

    pub fn cycle_source_prev(&mut self) {
        let new_source = prev_in(SOURCES, self.source);
        self.source = new_source;
        self.limit = clamp_limit(self.limit, new_source);
        self.restart();
    }

    /// Lang for Code, difficulty for Text.
    pub fn cycle_secondary_next(&mut self) {
        match self.source {
            Source::Code => self.lang = next_in(LANGS, self.lang),
            Source::Text => self.difficulty = next_in(DIFFICULTIES, self.difficulty),
        }
        self.restart();
    }

    pub fn cycle_secondary_prev(&mut self) {
        match self.source {
            Source::Code => self.lang = prev_in(LANGS, self.lang),
            Source::Text => self.difficulty = prev_in(DIFFICULTIES, self.difficulty),
        }
        self.restart();
    }

    pub fn elapsed(&self) -> f64 {
        if let Some(e) = self.end_elapsed {
            return e;
        }
        self.start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0)
    }

    fn finish(&mut self) {
        if self.finished {
            return;
        }
        let live = self.start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
        let frozen = match self.limit.time_secs() {
            Some(n) => live.min(n),
            None => live,
        };
        self.end_elapsed = Some(frozen);
        self.finished = true;
    }

    pub fn time_left(&self) -> f64 {
        self.limit
            .time_secs()
            .map(|n| (n - self.elapsed()).max(0.0))
            .unwrap_or(0.0)
    }

    pub fn wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < MIN_ELAPSED_FOR_WPM {
            return 0.0;
        }
        (self.correct as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    pub fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < MIN_ELAPSED_FOR_WPM {
            return 0.0;
        }
        (self.typed.len() as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            return 100.0;
        }
        (self.correct as f64 / total as f64) * 100.0
    }

    /// Count of whitespace-terminated words in `target` that the user has
    /// fully typed past.
    pub fn words_done(&self) -> usize {
        let n = self.typed.len().min(self.target.len());
        if n == 0 {
            return 0;
        }
        let mut count = 0usize;
        let mut in_word = false;
        for ch in &self.target[..n] {
            if ch.is_whitespace() {
                if in_word {
                    count += 1;
                    in_word = false;
                }
            } else {
                in_word = true;
            }
        }
        count
    }

    pub fn push(&mut self, c: char) {
        if self.finished {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        let idx = self.typed.len();
        if idx >= self.target.len() {
            return;
        }
        if self.target[idx] == c {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }
        self.keystrokes += 1;
        self.typed.push(c);
        self.check_done();
    }

    pub fn backspace(&mut self) {
        if let Some(c) = self.typed.pop() {
            let idx = self.typed.len();
            if self.target[idx] == c {
                self.correct = self.correct.saturating_sub(1);
            } else {
                self.incorrect = self.incorrect.saturating_sub(1);
            }
        }
    }

    fn check_done(&mut self) {
        match self.limit {
            Limit::Time(n) => {
                if self.elapsed() >= f64::from(n) {
                    self.finish();
                }
            }
            Limit::Count(_) => {
                if self.words_done() >= self.target_words || self.typed.len() >= self.target.len() {
                    self.finish();
                }
            }
            Limit::Snippet => {
                if self.typed.len() >= self.target.len() {
                    self.finish();
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if self.finished {
            return;
        }
        if let Some(n) = self.limit.time_secs() {
            if self.start.is_some() && self.elapsed() >= n {
                self.finish();
            }
        }
    }

    pub fn is_code(&self) -> bool {
        matches!(self.source, Source::Code)
    }
}

fn clamp_limit(current: Limit, source: Source) -> Limit {
    let allowed = limits_for(source);
    if allowed.contains(&current) {
        current
    } else {
        allowed[0]
    }
}

fn build_code(lang: Lang, limit: Limit) -> (String, Vec<Color>, usize) {
    let s = match limit {
        Limit::Snippet => code::pick_sample(lang).to_string(),
        Limit::Time(secs) => code::long_sample(
            lang,
            (secs as usize * CODE_CHARS_PER_SECOND).max(CODE_BUFFER_FLOOR),
        ),
        Limit::Count(_) => code::long_sample(lang, CODE_BUFFER_FLOOR),
    };
    let colors = code::highlight(&s, lang);
    (s, colors, 0)
}

fn build_text(difficulty: Difficulty, limit: Limit) -> (String, Vec<Color>, usize) {
    let pool = words::pool(difficulty);
    let mut rng = rand::thread_rng();
    let count = match limit {
        Limit::Count(n) => n as usize,
        _ => TEXT_POOL_FOR_TIME,
    };
    let picked: Vec<String> = (0..count)
        .map(|_| pool.choose(&mut rng).unwrap().clone())
        .collect();
    let mut s = picked.join(" ");
    if matches!(limit, Limit::Count(_)) {
        s.push(' ');
    }
    let len = s.chars().count();
    (s, vec![Color::Gray; len], count)
}

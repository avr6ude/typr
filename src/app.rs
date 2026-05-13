use std::time::Instant;

use rand::seq::SliceRandom;
use ratatui::style::Color;

use crate::cli::{Difficulty, Lang, Limit, Source};
use crate::code;
use crate::words;

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
            Source::Code => {
                let s = match limit {
                    Limit::Snippet => code::pick_sample(lang).to_string(),
                    Limit::Time(secs) => code::long_sample(lang, (secs as usize * 25).max(2000)),
                    Limit::Count(_) => code::long_sample(lang, 2000),
                };
                let colors = code::highlight(&s, lang);
                (s, colors, 0)
            }
            Source::Text => {
                let pool = words::pool(difficulty);
                let mut rng = rand::thread_rng();
                let count = match limit {
                    Limit::Count(n) => n as usize,
                    _ => 500,
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
        };
        let target: Vec<char> = target_str.chars().collect();
        Self {
            target,
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
        let live = self
            .start
            .map(|s| s.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        let frozen = match self.limit {
            Limit::Time(n) => live.min(n as f64),
            _ => live,
        };
        self.end_elapsed = Some(frozen);
        self.finished = true;
    }

    pub fn time_left(&self) -> f64 {
        match self.limit {
            Limit::Time(n) => (n as f64 - self.elapsed()).max(0.0),
            _ => 0.0,
        }
    }

    pub fn wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < 0.1 {
            return 0.0;
        }
        (self.correct as f64 / 5.0) / (secs / 60.0)
    }

    pub fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < 0.1 {
            return 0.0;
        }
        (self.typed.len() as f64 / 5.0) / (secs / 60.0)
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            return 100.0;
        }
        (self.correct as f64 / total as f64) * 100.0
    }

    pub fn words_done(&self) -> usize {
        if self.typed.is_empty() {
            return 0;
        }
        let upto: String = self.target[..self.typed.len().min(self.target.len())]
            .iter()
            .collect();
        let raw = upto.split_whitespace().count();
        if self.typed.last() == Some(&' ') {
            raw
        } else {
            raw.saturating_sub(1)
        }
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

    pub fn check_done(&mut self) {
        match self.limit {
            Limit::Time(n) => {
                if self.elapsed() >= n as f64 {
                    self.finish();
                }
            }
            Limit::Count(_) => {
                if self.words_done() >= self.target_words
                    || self.typed.len() >= self.target.len()
                {
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
        if let Limit::Time(n) = self.limit {
            if self.start.is_some() && self.elapsed() >= n as f64 {
                self.finish();
            }
        }
    }

    pub fn is_code(&self) -> bool {
        matches!(self.source, Source::Code)
    }
}

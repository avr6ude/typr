use std::time::Instant;

use rand::seq::SliceRandom;

use crate::cli::Mode;

const WORDS: &str = include_str!("words.txt");

pub struct App {
    pub target: Vec<char>,
    pub typed: Vec<char>,
    pub correct: usize,
    pub incorrect: usize,
    pub start: Option<Instant>,
    pub finished: bool,
    pub end_elapsed: Option<f64>,
    pub mode: Mode,
    pub amount: u32,
    pub target_words: usize,
}

impl App {
    pub fn new(mode: Mode, amount: u32) -> Self {
        let pool: Vec<&str> = WORDS.split_whitespace().collect();
        let mut rng = rand::thread_rng();
        let count = match mode {
            Mode::Time => 500,
            Mode::Words => amount as usize,
        };
        let picked: Vec<String> = (0..count)
            .map(|_| pool.choose(&mut rng).unwrap().to_string())
            .collect();
        let target: Vec<char> = picked.join(" ").chars().collect();
        Self {
            target,
            typed: Vec::new(),
            correct: 0,
            incorrect: 0,
            start: None,
            finished: false,
            end_elapsed: None,
            mode,
            amount,
            target_words: count,
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
        let frozen = match self.mode {
            Mode::Time => live.min(self.amount as f64),
            Mode::Words => live,
        };
        self.end_elapsed = Some(frozen);
        self.finished = true;
    }

    pub fn time_left(&self) -> f64 {
        match self.mode {
            Mode::Time => (self.amount as f64 - self.elapsed()).max(0.0),
            Mode::Words => 0.0,
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
        match self.mode {
            Mode::Time => {
                if self.elapsed() >= self.amount as f64 {
                    self.finish();
                }
            }
            Mode::Words => {
                if self.words_done() >= self.target_words {
                    self.finish();
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if matches!(self.mode, Mode::Time)
            && self.start.is_some()
            && self.elapsed() >= self.amount as f64
        {
            self.finished = true;
        }
    }
}

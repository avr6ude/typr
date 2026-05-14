//! CLI arguments, config enums (source, lang, limit, difficulty), and small
//! list-rotation utilities used by the event loop.

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Source {
    Text,
    Code,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Difficulty {
    E200,
    E1k,
    E5k,
    E10k,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    Rust,
    Python,
    Js,
    Go,
}

/// A test-completion condition.
///
/// `Time(secs)` ends when the clock runs out, `Count(words)` ends after the
/// user has typed N whitespace-separated words, `Snippet` ends when the user
/// types through the entire target.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Limit {
    Time(u32),
    Count(u32),
    Snippet,
}

impl Limit {
    pub fn time_secs(&self) -> Option<f64> {
        match self {
            Limit::Time(n) => Some(f64::from(*n)),
            _ => None,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "typr", about = "CLI typing test", version)]
pub struct Cli {
    #[arg(short, long, value_enum, default_value_t = Source::Text)]
    pub source: Source,

    #[arg(short, long, value_enum, default_value_t = Lang::Rust)]
    pub lang: Lang,

    #[arg(short, long, value_enum, default_value_t = Difficulty::E200)]
    pub difficulty: Difficulty,

    /// Test limit. Examples: `30s`, `60s`, `25w` (text only), `snippet` (code only).
    #[arg(short = 'L', long, value_parser = parse_limit, default_value = "30s")]
    pub limit: Limit,
}

fn parse_limit(s: &str) -> Result<Limit, String> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("snippet") {
        return Ok(Limit::Snippet);
    }
    if let Some(n) = s.strip_suffix('s') {
        return n.parse::<u32>().map(Limit::Time).map_err(|e| e.to_string());
    }
    if let Some(n) = s.strip_suffix('w') {
        return n
            .parse::<u32>()
            .map(Limit::Count)
            .map_err(|e| e.to_string());
    }
    s.parse::<u32>().map(Limit::Time).map_err(|e| e.to_string())
}

pub const SOURCES: &[Source] = &[Source::Text, Source::Code];
pub const LANGS: &[Lang] = &[Lang::Rust, Lang::Python, Lang::Js, Lang::Go];
pub const DIFFICULTIES: &[Difficulty] = &[
    Difficulty::E200,
    Difficulty::E1k,
    Difficulty::E5k,
    Difficulty::E10k,
];

pub const LIMITS_TEXT: &[Limit] = &[
    Limit::Time(15),
    Limit::Time(30),
    Limit::Time(60),
    Limit::Time(120),
    Limit::Count(10),
    Limit::Count(25),
    Limit::Count(50),
    Limit::Count(100),
];

pub const LIMITS_CODE: &[Limit] = &[
    Limit::Time(15),
    Limit::Time(30),
    Limit::Time(60),
    Limit::Time(120),
    Limit::Snippet,
];

pub fn limits_for(s: Source) -> &'static [Limit] {
    match s {
        Source::Text => LIMITS_TEXT,
        Source::Code => LIMITS_CODE,
    }
}

pub fn source_label(s: Source) -> &'static str {
    match s {
        Source::Text => "text",
        Source::Code => "code",
    }
}

pub fn lang_label(l: Lang) -> &'static str {
    match l {
        Lang::Rust => "rust",
        Lang::Python => "python",
        Lang::Js => "js",
        Lang::Go => "go",
    }
}

pub fn difficulty_label(d: Difficulty) -> &'static str {
    match d {
        Difficulty::E200 => "200",
        Difficulty::E1k => "1k",
        Difficulty::E5k => "5k",
        Difficulty::E10k => "10k",
    }
}

pub fn limit_label(l: Limit) -> String {
    match l {
        Limit::Time(n) => format!("{n}s"),
        Limit::Count(n) => format!("{n}"),
        Limit::Snippet => "snippet".to_string(),
    }
}

/// Returns the next element in `list` after `current`, wrapping at the end.
/// If `current` is not in the list, returns the first element.
pub fn next_in<T: PartialEq + Copy>(list: &[T], current: T) -> T {
    if list.is_empty() {
        return current;
    }
    let pos = list.iter().position(|x| *x == current).unwrap_or(0);
    list[(pos + 1) % list.len()]
}

/// Returns the previous element in `list`, wrapping at the start.
/// If `current` is not in the list, returns the first element.
pub fn prev_in<T: PartialEq + Copy>(list: &[T], current: T) -> T {
    if list.is_empty() {
        return current;
    }
    let pos = list.iter().position(|x| *x == current).unwrap_or(0);
    list[(pos + list.len() - 1) % list.len()]
}

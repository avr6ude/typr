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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Limit {
    Time(u32),
    Count(u32),
    Snippet,
}

#[derive(Parser, Debug)]
#[command(name = "typr", about = "CLI typing test")]
pub struct Cli {
    #[arg(short, long, value_enum, default_value_t = Source::Text)]
    pub source: Source,

    #[arg(short, long, value_enum, default_value_t = Lang::Rust)]
    pub lang: Lang,

    #[arg(short, long, value_enum, default_value_t = Difficulty::E200)]
    pub difficulty: Difficulty,
}

pub const SOURCES: &[Source] = &[Source::Text, Source::Code];
pub const LANGS: &[Lang] = &[Lang::Rust, Lang::Python, Lang::Js, Lang::Go];
pub const DIFFICULTIES: &[Difficulty] = &[
    Difficulty::E200,
    Difficulty::E1k,
    Difficulty::E5k,
    Difficulty::E10k,
];

pub fn difficulty_label(d: Difficulty) -> &'static str {
    match d {
        Difficulty::E200 => "200",
        Difficulty::E1k => "1k",
        Difficulty::E5k => "5k",
        Difficulty::E10k => "10k",
    }
}

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

pub fn limit_label(l: Limit) -> String {
    match l {
        Limit::Time(n) => format!("{}s", n),
        Limit::Count(n) => format!("{}", n),
        Limit::Snippet => "snippet".to_string(),
    }
}

pub fn next_in<T: PartialEq + Copy>(list: &[T], current: T) -> T {
    let pos = list.iter().position(|x| *x == current).unwrap_or(0);
    list[(pos + 1) % list.len()]
}

pub fn prev_in<T: PartialEq + Copy>(list: &[T], current: T) -> T {
    let pos = list.iter().position(|x| *x == current).unwrap_or(0);
    list[(pos + list.len() - 1) % list.len()]
}

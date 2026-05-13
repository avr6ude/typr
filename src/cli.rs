use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    Time,
    Words,
    Code,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    Rust,
    Python,
    Js,
    Go,
}

#[derive(Parser, Debug)]
#[command(name = "typr", about = "CLI typing test")]
pub struct Cli {
    #[arg(short, long, value_enum, default_value_t = Mode::Time)]
    pub mode: Mode,

    #[arg(short, long, default_value_t = 30)]
    pub amount: u32,

    #[arg(short, long, value_enum)]
    pub lang: Option<Lang>,
}

#[derive(Copy, Clone, Debug)]
pub struct Preset {
    pub mode: Mode,
    pub amount: u32,
    pub lang: Option<Lang>,
}

pub const PRESETS: &[Preset] = &[
    Preset { mode: Mode::Time, amount: 15, lang: None },
    Preset { mode: Mode::Time, amount: 30, lang: None },
    Preset { mode: Mode::Time, amount: 60, lang: None },
    Preset { mode: Mode::Time, amount: 120, lang: None },
    Preset { mode: Mode::Words, amount: 10, lang: None },
    Preset { mode: Mode::Words, amount: 25, lang: None },
    Preset { mode: Mode::Words, amount: 50, lang: None },
    Preset { mode: Mode::Words, amount: 100, lang: None },
    Preset { mode: Mode::Code, amount: 0, lang: Some(Lang::Rust) },
    Preset { mode: Mode::Code, amount: 0, lang: Some(Lang::Python) },
    Preset { mode: Mode::Code, amount: 0, lang: Some(Lang::Js) },
    Preset { mode: Mode::Code, amount: 0, lang: Some(Lang::Go) },
];

pub fn preset_index(mode: Mode, amount: u32, lang: Option<Lang>) -> usize {
    PRESETS
        .iter()
        .position(|p| p.mode == mode && p.amount == amount && p.lang == lang)
        .unwrap_or(1)
}

pub fn lang_label(l: Lang) -> &'static str {
    match l {
        Lang::Rust => "rust",
        Lang::Python => "python",
        Lang::Js => "js",
        Lang::Go => "go",
    }
}

pub fn type_label(m: Mode) -> &'static str {
    match m {
        Mode::Time => "time",
        Mode::Words => "words",
        Mode::Code => "code",
    }
}

pub fn amount_label(p: &Preset) -> String {
    match p.mode {
        Mode::Time => format!("{}s", p.amount),
        Mode::Words => format!("{}", p.amount),
        Mode::Code => lang_label(p.lang.unwrap_or(Lang::Rust)).to_string(),
    }
}

pub const TYPES: &[Mode] = &[Mode::Time, Mode::Words, Mode::Code];

fn group(mode: Mode) -> Vec<usize> {
    PRESETS
        .iter()
        .enumerate()
        .filter(|(_, p)| p.mode == mode)
        .map(|(i, _)| i)
        .collect()
}

pub fn next_in_type(current: usize) -> usize {
    let g = group(PRESETS[current].mode);
    let pos = g.iter().position(|&i| i == current).unwrap_or(0);
    g[(pos + 1) % g.len()]
}

pub fn prev_in_type(current: usize) -> usize {
    let g = group(PRESETS[current].mode);
    let pos = g.iter().position(|&i| i == current).unwrap_or(0);
    g[(pos + g.len() - 1) % g.len()]
}

pub fn next_type(current: usize) -> usize {
    let cur = PRESETS[current].mode;
    let pos = TYPES.iter().position(|t| *t == cur).unwrap_or(0);
    let next = TYPES[(pos + 1) % TYPES.len()];
    group(next)[0]
}

pub fn prev_type(current: usize) -> usize {
    let cur = PRESETS[current].mode;
    let pos = TYPES.iter().position(|t| *t == cur).unwrap_or(0);
    let prev = TYPES[(pos + TYPES.len() - 1) % TYPES.len()];
    group(prev)[0]
}

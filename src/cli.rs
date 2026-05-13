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

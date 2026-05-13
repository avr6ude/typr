use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    Time,
    Words,
}

#[derive(Parser, Debug)]
#[command(name = "typr", about = "CLI typing test")]
pub struct Cli {
    #[arg(short, long, value_enum, default_value_t = Mode::Time)]
    pub mode: Mode,

    #[arg(short, long, default_value_t = 30)]
    pub amount: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Preset {
    pub mode: Mode,
    pub amount: u32,
}

pub const PRESETS: &[Preset] = &[
    Preset { mode: Mode::Time, amount: 15 },
    Preset { mode: Mode::Time, amount: 30 },
    Preset { mode: Mode::Time, amount: 60 },
    Preset { mode: Mode::Time, amount: 120 },
    Preset { mode: Mode::Words, amount: 10 },
    Preset { mode: Mode::Words, amount: 25 },
    Preset { mode: Mode::Words, amount: 50 },
    Preset { mode: Mode::Words, amount: 100 },
];

pub fn preset_index(mode: Mode, amount: u32) -> usize {
    PRESETS
        .iter()
        .position(|p| p.mode == mode && p.amount == amount)
        .unwrap_or(1)
}

//! Code samples + syntect highlighting.
//!
//! Samples live in `samples/<lang>.txt`, separated by lines containing only
//! `---`. `SyntaxSet` and `ThemeSet` are loaded once and cached.

use std::sync::OnceLock;

use rand::seq::SliceRandom;
use ratatui::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::cli::Lang;

static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
static THEMES: OnceLock<ThemeSet> = OnceLock::new();
static RUST_SAMPLES: OnceLock<Vec<&'static str>> = OnceLock::new();
static PYTHON_SAMPLES: OnceLock<Vec<&'static str>> = OnceLock::new();
static JS_SAMPLES: OnceLock<Vec<&'static str>> = OnceLock::new();
static GO_SAMPLES: OnceLock<Vec<&'static str>> = OnceLock::new();

const RUST_RAW: &str = include_str!("samples/rust.txt");
const PYTHON_RAW: &str = include_str!("samples/python.txt");
const JS_RAW: &str = include_str!("samples/js.txt");
const GO_RAW: &str = include_str!("samples/go.txt");

fn syntaxes() -> &'static SyntaxSet {
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn themes() -> &'static ThemeSet {
    THEMES.get_or_init(ThemeSet::load_defaults)
}

fn theme() -> &'static Theme {
    let ts = themes();
    ts.themes
        .get("base16-ocean.dark")
        .or_else(|| ts.themes.values().next())
        .expect("at least one theme bundled")
}

fn split_samples(raw: &'static str) -> Vec<&'static str> {
    raw.split("\n---\n").map(str::trim).collect()
}

fn pool(lang: Lang) -> &'static [&'static str] {
    let cell = match lang {
        Lang::Rust => &RUST_SAMPLES,
        Lang::Python => &PYTHON_SAMPLES,
        Lang::Js => &JS_SAMPLES,
        Lang::Go => &GO_SAMPLES,
    };
    let raw = match lang {
        Lang::Rust => RUST_RAW,
        Lang::Python => PYTHON_RAW,
        Lang::Js => JS_RAW,
        Lang::Go => GO_RAW,
    };
    cell.get_or_init(|| split_samples(raw))
}

pub fn pick_sample(lang: Lang) -> &'static str {
    let mut rng = rand::thread_rng();
    pool(lang).choose(&mut rng).copied().unwrap_or("")
}

/// Concatenate random samples with `\n\n` separators until reaching
/// `target_chars` characters.
pub fn long_sample(lang: Lang, target_chars: usize) -> String {
    let mut rng = rand::thread_rng();
    let p = pool(lang);
    let mut out = String::new();
    let mut len = 0usize;
    while len < target_chars {
        let Some(s) = p.choose(&mut rng).copied() else {
            break;
        };
        if !out.is_empty() {
            out.push_str("\n\n");
            len += 2;
        }
        out.push_str(s);
        len += s.chars().count();
    }
    out
}

/// Returns one [`Color`] per character of `text`, guaranteed to have length
/// equal to `text.chars().count()`. Falls back to [`Color::Gray`] for any
/// characters syntect produces no style for.
pub fn highlight(text: &str, lang: Lang) -> Vec<Color> {
    let total = text.chars().count();
    let ps = syntaxes();
    let ext = match lang {
        Lang::Rust => "rs",
        Lang::Python => "py",
        Lang::Js => "js",
        Lang::Go => "go",
    };
    let syntax = ps
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, theme());

    let mut colors: Vec<Color> = Vec::with_capacity(total);
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, ps).unwrap_or_default();
        for (style, s) in ranges {
            let c = style.foreground;
            let color = Color::Rgb(c.r, c.g, c.b);
            colors.extend(std::iter::repeat(color).take(s.chars().count()));
        }
    }
    if colors.len() < total {
        colors.resize(total, Color::Gray);
    } else {
        colors.truncate(total);
    }
    colors
}

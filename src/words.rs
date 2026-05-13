use std::sync::OnceLock;

use serde::Deserialize;

use crate::cli::Difficulty;

const E200_JSON: &str = include_str!("english.json");
const E1K_JSON: &str = include_str!("english_1k.json");
const E5K_JSON: &str = include_str!("english_5k.json");
const E10K_JSON: &str = include_str!("english_10k.json");

#[derive(Deserialize)]
struct LangFile {
    words: Vec<String>,
}

static E200: OnceLock<Vec<String>> = OnceLock::new();
static E1K: OnceLock<Vec<String>> = OnceLock::new();
static E5K: OnceLock<Vec<String>> = OnceLock::new();
static E10K: OnceLock<Vec<String>> = OnceLock::new();

const FALLBACK: &[&str] = &[
    "the", "be", "of", "and", "to", "in", "have", "it", "that", "for", "they", "with", "as",
    "not", "on", "at", "this", "but", "or", "from",
];

fn parse(s: &str) -> Vec<String> {
    let parsed: Vec<String> = serde_json::from_str::<LangFile>(s)
        .map(|l| l.words)
        .unwrap_or_default();
    if parsed.is_empty() {
        FALLBACK.iter().map(|w| (*w).to_string()).collect()
    } else {
        parsed
    }
}

pub fn pool(d: Difficulty) -> &'static [String] {
    match d {
        Difficulty::E200 => E200.get_or_init(|| parse(E200_JSON)),
        Difficulty::E1k => E1K.get_or_init(|| parse(E1K_JSON)),
        Difficulty::E5k => E5K.get_or_init(|| parse(E5K_JSON)),
        Difficulty::E10k => E10K.get_or_init(|| parse(E10K_JSON)),
    }
}

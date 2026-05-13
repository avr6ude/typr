use rand::seq::SliceRandom;
use ratatui::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::cli::Lang;

const RUST: &[&str] = &[
    "fn fibonacci(n: u32) -> u64 {\n    let mut a: u64 = 0;\n    let mut b: u64 = 1;\n    for _ in 0..n {\n        let next = a + b;\n        a = b;\n        b = next;\n    }\n    a\n}",
    "fn sum_even(nums: &[i32]) -> i32 {\n    nums.iter().filter(|n| **n % 2 == 0).sum()\n}",
    "struct Point { x: f64, y: f64 }\n\nimpl Point {\n    fn distance(&self, other: &Point) -> f64 {\n        let dx = self.x - other.x;\n        let dy = self.y - other.y;\n        (dx * dx + dy * dy).sqrt()\n    }\n}",
];

const PYTHON: &[&str] = &[
    "def quicksort(arr):\n    if len(arr) <= 1:\n        return arr\n    pivot = arr[len(arr) // 2]\n    left = [x for x in arr if x < pivot]\n    mid = [x for x in arr if x == pivot]\n    right = [x for x in arr if x > pivot]\n    return quicksort(left) + mid + quicksort(right)",
    "def fib(n):\n    a, b = 0, 1\n    for _ in range(n):\n        a, b = b, a + b\n    return a",
    "class Counter:\n    def __init__(self):\n        self.count = 0\n    def inc(self, by=1):\n        self.count += by\n        return self.count",
];

const JS: &[&str] = &[
    "function debounce(fn, delay) {\n    let timer = null;\n    return function(...args) {\n        clearTimeout(timer);\n        timer = setTimeout(() => fn.apply(this, args), delay);\n    };\n}",
    "const groupBy = (arr, key) => arr.reduce((acc, item) => {\n    (acc[item[key]] = acc[item[key]] || []).push(item);\n    return acc;\n}, {});",
    "async function fetchUser(id) {\n    const res = await fetch(`/api/users/${id}`);\n    if (!res.ok) throw new Error('not found');\n    return await res.json();\n}",
];

const GO: &[&str] = &[
    "func sortAndPrint(nums []int) {\n    sort.Ints(nums)\n    for i, v := range nums {\n        fmt.Printf(\"%d: %d\\n\", i, v)\n    }\n}",
    "func contains(s []string, target string) bool {\n    for _, v := range s {\n        if v == target {\n            return true\n        }\n    }\n    return false\n}",
    "type Stack struct {\n    data []int\n}\n\nfunc (s *Stack) Push(v int) {\n    s.data = append(s.data, v)\n}",
];

fn pool(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Rust => RUST,
        Lang::Python => PYTHON,
        Lang::Js => JS,
        Lang::Go => GO,
    }
}

pub fn pick_sample(lang: Lang) -> &'static str {
    let mut rng = rand::thread_rng();
    pool(lang).choose(&mut rng).copied().unwrap()
}

pub fn long_sample(lang: Lang, target_chars: usize) -> String {
    let mut rng = rand::thread_rng();
    let p = pool(lang);
    let mut out = String::new();
    while out.chars().count() < target_chars {
        let s = p.choose(&mut rng).copied().unwrap();
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(s);
    }
    out
}

pub fn highlight(text: &str, lang: Lang) -> Vec<Color> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let ext = match lang {
        Lang::Rust => "rs",
        Lang::Python => "py",
        Lang::Js => "js",
        Lang::Go => "go",
    };
    let syntax = ps
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    let mut colors: Vec<Color> = Vec::with_capacity(text.chars().count());
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, &ps).unwrap_or_default();
        for (style, s) in ranges {
            let c = style.foreground;
            let color = Color::Rgb(c.r, c.g, c.b);
            for _ in s.chars() {
                colors.push(color);
            }
        }
    }
    colors
}

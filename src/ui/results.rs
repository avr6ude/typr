//! Post-run results screen: big WPM glyph + summary stats.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::Limit;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " results ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let wpm_lines: Vec<Line> = big_number(&format!("{:.0}", app.wpm()))
        .into_iter()
        .map(|s| {
            Line::from(Span::styled(
                s,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(wpm_lines).alignment(Alignment::Center),
        rows[1],
    );

    f.render_widget(
        Paragraph::new("wpm")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        rows[2],
    );

    let lim_label = match app.limit {
        Limit::Snippet => ("source".to_string(), "snippet".to_string()),
        _ => ("time".to_string(), format!("{:.1}s", app.elapsed())),
    };
    let g1: Vec<Span> = vec![
        kv("raw", format!("{:.1}", app.raw_wpm()), Color::Gray),
        sep(),
        kv("accuracy", format!("{:.1}%", app.accuracy()), Color::Green),
        sep(),
        kv(&lim_label.0, lim_label.1, Color::Cyan),
    ]
    .into_iter()
    .flatten()
    .collect();
    let g2: Vec<Span> = vec![
        kv("correct", format!("{}", app.correct), Color::Green),
        sep(),
        kv("wrong", format!("{}", app.incorrect), Color::Red),
        sep(),
        kv("typed", format!("{}", app.keystrokes), Color::Gray),
    ]
    .into_iter()
    .flatten()
    .collect();
    f.render_widget(
        Paragraph::new(vec![Line::from(g1), Line::raw(""), Line::from(g2)])
            .alignment(Alignment::Center),
        rows[3],
    );

    f.render_widget(
        Paragraph::new("enter retry · tab next limit · ↑↓ source · esc exit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        rows[5],
    );
}

fn kv(label: &str, value: String, color: Color) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{label}: "), Style::default().fg(Color::DarkGray)),
        Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]
}

fn sep() -> Vec<Span<'static>> {
    vec![Span::styled(
        "   ·   ",
        Style::default().fg(Color::DarkGray),
    )]
}

// 5x5 block-glyph font; only digits and '.' supported.
fn big_number(s: &str) -> Vec<String> {
    let glyphs: Vec<Vec<&str>> = s.chars().map(big_glyph).collect();
    let mut rows = vec![String::new(); 5];
    for (i, row) in rows.iter_mut().enumerate() {
        for (j, g) in glyphs.iter().enumerate() {
            row.push_str(g[i]);
            if j + 1 < glyphs.len() {
                row.push(' ');
            }
        }
    }
    rows
}

fn big_glyph(c: char) -> Vec<&'static str> {
    match c {
        '0' => vec!["█████", "█   █", "█   █", "█   █", "█████"],
        '1' => vec!["  █  ", " ██  ", "  █  ", "  █  ", " ███ "],
        '2' => vec!["█████", "    █", "█████", "█    ", "█████"],
        '3' => vec!["█████", "    █", " ████", "    █", "█████"],
        '4' => vec!["█   █", "█   █", "█████", "    █", "    █"],
        '5' => vec!["█████", "█    ", "█████", "    █", "█████"],
        '6' => vec!["█████", "█    ", "█████", "█   █", "█████"],
        '7' => vec!["█████", "    █", "   █ ", "  █  ", "  █  "],
        '8' => vec!["█████", "█   █", "█████", "█   █", "█████"],
        '9' => vec!["█████", "█   █", "█████", "    █", "█████"],
        '.' => vec!["     ", "     ", "     ", "     ", "  █  "],
        _ => vec!["     ", "     ", "     ", "     ", "     "],
    }
}

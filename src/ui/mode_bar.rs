//! Top mode-selector bar: source row, limit row, secondary (lang/difficulty) row.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::{
    difficulty_label, lang_label, limit_label, limits_for, source_label, DIFFICULTIES, LANGS,
    SOURCES,
};

use super::chip_style;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let started = app.start.is_some();
    let is_code = app.is_code();

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" mode ", Style::default().fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let sep = Span::styled("  ·  ", Style::default().fg(Color::DarkGray));

    let mut row_src: Vec<Span> = vec![Span::styled(
        " source: ",
        Style::default().fg(Color::DarkGray),
    )];
    for (i, s) in SOURCES.iter().enumerate() {
        let selected = app.source == *s;
        row_src.push(Span::styled(
            format!(" {} ", source_label(*s)),
            chip_style(selected, started),
        ));
        if i + 1 < SOURCES.len() {
            row_src.push(sep.clone());
        }
    }
    let hint = match (started, is_code) {
        (true, true) => "    tab indent",
        (true, false) => "    esc restart",
        (false, true) => "    ↑↓ source · ←→ lang · tab limit",
        (false, false) => "    ↑↓ source · ←→ difficulty · tab limit",
    };
    row_src.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));
    f.render_widget(Paragraph::new(Line::from(row_src)), rows[0]);

    let limits = limits_for(app.source);
    let mut row_lim: Vec<Span> = vec![Span::styled(
        " limit:  ",
        Style::default().fg(Color::DarkGray),
    )];
    for (i, l) in limits.iter().enumerate() {
        let selected = app.limit == *l;
        row_lim.push(Span::styled(
            format!(" {} ", limit_label(*l)),
            chip_style(selected, started),
        ));
        if i + 1 < limits.len() {
            row_lim.push(sep.clone());
        }
    }
    f.render_widget(Paragraph::new(Line::from(row_lim)), rows[1]);

    let (label, items): (&str, Vec<(String, bool)>) = if is_code {
        (
            " lang:   ",
            LANGS
                .iter()
                .map(|l| (lang_label(*l).to_string(), app.lang == *l))
                .collect(),
        )
    } else {
        (
            " words:  ",
            DIFFICULTIES
                .iter()
                .map(|d| (difficulty_label(*d).to_string(), app.difficulty == *d))
                .collect(),
        )
    };
    let mut row3: Vec<Span> = vec![Span::styled(label, Style::default().fg(Color::DarkGray))];
    for (i, (text, selected)) in items.iter().enumerate() {
        row3.push(Span::styled(
            format!(" {text} "),
            chip_style(*selected, started),
        ));
        if i + 1 < items.len() {
            row3.push(sep.clone());
        }
    }
    f.render_widget(Paragraph::new(Line::from(row3)), rows[2]);
}

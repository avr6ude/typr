use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::{Mode, Preset, PRESETS};

const LINE_WIDTH: usize = 56;
const VISIBLE_LINES: usize = 3;
const BODY_HEIGHT: u16 = (VISIBLE_LINES as u16) * 2 + 1;

pub fn draw(f: &mut Frame, app: &App, preset_idx: usize) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(BODY_HEIGHT),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    draw_header(f, app, chunks[1]);
    draw_preset_bar(f, preset_idx, app.start.is_some(), chunks[2]);
    draw_body(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = match app.mode {
        Mode::Time => format!(
            " time  {:>3.0}s   wpm {:>5.1}   acc {:>5.1}%   [esc quit · tab restart] ",
            app.time_left(),
            app.wpm(),
            app.accuracy()
        ),
        Mode::Words => format!(
            " words {}/{}   wpm {:>5.1}   acc {:>5.1}%   [esc quit · tab restart] ",
            app.words_done(),
            app.target_words,
            app.wpm(),
            app.accuracy()
        ),
    };
    let p = Paragraph::new(header)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" typr "));
    f.render_widget(p, area);
}

fn draw_preset_bar(f: &mut Frame, preset_idx: usize, started: bool, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, p) in PRESETS.iter().enumerate() {
        let label = preset_label(p);
        let style = if i == preset_idx {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if started {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {label} "), style));
        spans.push(Span::raw(" "));
    }
    let hint = if started {
        " (locked while typing — backspace clears) "
    } else {
        " tab/shift+tab to cycle "
    };
    spans.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));

    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title(" mode "));
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let line_width = LINE_WIDTH.min(inner_width.max(20));
    let lines = wrap_lines(&app.target, line_width);
    let cursor = app.typed.len();
    let current_line = find_line(&lines, cursor);

    let half = VISIBLE_LINES / 2;
    let start = current_line.saturating_sub(half);
    let end = (start + VISIBLE_LINES).min(lines.len());
    let visible = &lines[start..end];

    let mut rendered: Vec<Line> = Vec::with_capacity(visible.len() * 2);
    for (i, (s, e)) in visible.iter().enumerate() {
        let is_current = start + i == current_line;
        rendered.push(line_for_range(app, *s, *e, is_current, line_width));
        if i + 1 < visible.len() {
            rendered.push(Line::raw(""));
        }
    }

    let p = Paragraph::new(rendered)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.finished {
        format!(
            " done · wpm {:.1} · raw {:.1} · acc {:.1}% · {}/{} correct chars · enter/esc to exit ",
            app.wpm(),
            app.raw_wpm(),
            app.accuracy(),
            app.correct,
            app.correct + app.incorrect
        )
    } else if app.start.is_none() {
        " pick mode with tab · start typing to begin ".to_string()
    } else {
        format!(" raw {:.1} wpm · {:.1}s elapsed ", app.raw_wpm(), app.elapsed())
    };
    let p = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}

fn preset_label(p: &Preset) -> String {
    match p.mode {
        Mode::Time => format!("time {}s", p.amount),
        Mode::Words => format!("words {}", p.amount),
    }
}

pub fn wrap_lines(target: &[char], width: usize) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let len = target.len();
    let mut start = 0;
    while start < len {
        let hard_end = (start + width).min(len);
        let mut end = hard_end;
        if end < len {
            if let Some(rel) = target[start..end].iter().rposition(|c| *c == ' ') {
                end = start + rel + 1;
            }
        }
        if end == start {
            end = hard_end;
        }
        lines.push((start, end));
        start = end;
    }
    if lines.is_empty() {
        lines.push((0, 0));
    }
    lines
}

fn find_line(lines: &[(usize, usize)], cursor: usize) -> usize {
    for (i, (_, e)) in lines.iter().enumerate() {
        if cursor < *e {
            return i;
        }
        if cursor == *e && i + 1 == lines.len() {
            return i;
        }
    }
    lines.len().saturating_sub(1)
}

fn line_for_range(app: &App, start: usize, end: usize, is_current: bool, width: usize) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::with_capacity(end - start);
    let pad = width.saturating_sub(end - start);
    for i in start..end {
        let ch = app.target[i];
        let typed = app.typed.get(i).copied();
        let mut style = if let Some(t) = typed {
            if t == ch {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red).add_modifier(Modifier::UNDERLINED)
            }
        } else if i == app.typed.len() {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        if is_current {
            style = style.add_modifier(Modifier::BOLD);
        }
        let display = if ch == ' ' && typed.is_some() && typed != Some(ch) {
            '_'
        } else {
            ch
        };
        spans.push(Span::styled(display.to_string(), style));
    }
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad)));
    }
    Line::from(spans)
}

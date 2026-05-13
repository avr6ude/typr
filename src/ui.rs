use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::cli::{Mode, Preset, PRESETS};

const VISIBLE_LINES: usize = 3;
const LINE_WIDTH_CAP: usize = 80;

pub fn draw(f: &mut Frame, app: &App, preset_idx: usize) {
    let area = f.area();
    let body_h = (VISIBLE_LINES * 2 - 1) as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(4)
        .vertical_margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(body_h),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header_line(f, app, chunks[0]);
    draw_preset_line(f, preset_idx, app.start.is_some(), chunks[1]);
    draw_body(f, app, chunks[3]);
    draw_footer_line(f, app, chunks[5]);
}

fn draw_header_line(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        Mode::Time => format!("time {:>3.0}s", app.time_left()),
        Mode::Words => format!("words {}/{}", app.words_done(), app.target_words),
    };
    let stats = format!("wpm {:.1}   acc {:.1}%", app.wpm(), app.accuracy());

    let spans = vec![
        Span::styled("typr ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(format!("· {mode}"), Style::default().fg(Color::Cyan)),
        Span::styled("   ", Style::default()),
        Span::styled(stats, Style::default().fg(Color::Gray)),
    ];
    f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Left), area);
}

fn draw_preset_line(f: &mut Frame, preset_idx: usize, started: bool, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, p) in PRESETS.iter().enumerate() {
        let label = preset_label(p);
        let selected = i == preset_idx;
        let style = if selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if started {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label, style));
        if i + 1 < PRESETS.len() {
            spans.push(Span::styled("  ·  ", Style::default().fg(Color::DarkGray)));
        }
    }
    f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), area);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let inner_width = area.width as usize;
    let line_width = inner_width.min(LINE_WIDTH_CAP).max(20);

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

    f.render_widget(Paragraph::new(rendered).alignment(Alignment::Center), area);
}

fn draw_footer_line(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.finished {
        format!(
            "done · wpm {:.1} · raw {:.1} · acc {:.1}% · {}/{} chars · enter to exit",
            app.wpm(),
            app.raw_wpm(),
            app.accuracy(),
            app.correct,
            app.correct + app.incorrect
        )
    } else if app.start.is_none() {
        "tab cycle mode · type to start · esc quit".to_string()
    } else {
        format!("raw {:.1} · {:.1}s · tab restart · esc quit", app.raw_wpm(), app.elapsed())
    };
    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        area,
    );
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
        } else if is_current {
            Style::default().fg(Color::Gray)
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

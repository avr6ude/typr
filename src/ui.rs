use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::{Mode, Preset, PRESETS};

const VISIBLE_LINES: usize = 3;
const SIDEBAR_WIDTH: u16 = 22;

pub fn draw(f: &mut Frame, app: &App, preset_idx: usize) {
    let area = f.area();
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " typr ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(2),
        ])
        .split(inner);

    draw_mode_bar(f, preset_idx, app.start.is_some(), rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(SIDEBAR_WIDTH),
            Constraint::Min(20),
        ])
        .split(rows[1]);

    draw_stats(f, app, cols[0]);
    draw_typing(f, app, cols[1]);
    draw_footer(f, app, rows[2]);
}

fn draw_mode_bar(f: &mut Frame, idx: usize, started: bool, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" mode ", Style::default().fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(" "));
    for (i, p) in PRESETS.iter().enumerate() {
        let label = preset_label(p);
        let style = if i == idx {
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if started {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {label} "), style));
        if i + 1 < PRESETS.len() {
            spans.push(Span::styled("·", Style::default().fg(Color::DarkGray)));
        }
    }
    let hint = if started {
        "   tab restart"
    } else {
        "   tab cycle · shift+tab back"
    };
    spans.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" stats ", Style::default().fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<(&str, String, Color)> = vec![
        match app.mode {
            Mode::Time => ("time", format!("{:.0}s", app.time_left()), Color::Cyan),
            Mode::Words => (
                "words",
                format!("{}/{}", app.words_done(), app.target_words),
                Color::Cyan,
            ),
        },
        ("wpm", format!("{:.1}", app.wpm()), Color::Green),
        ("raw", format!("{:.1}", app.raw_wpm()), Color::Gray),
        ("acc", format!("{:.1}%", app.accuracy()), Color::Yellow),
        (
            "chars",
            format!("{}/{}", app.correct, app.correct + app.incorrect),
            Color::Gray,
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::raw(""));
    for (k, v, c) in rows {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<7}", k), Style::default().fg(Color::DarkGray)),
            Span::styled(v, Style::default().fg(c).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::raw(""));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_typing(f: &mut Frame, app: &App, area: Rect) {
    let inner_w = area.width.saturating_sub(4) as usize;
    let inner_h = area.height as usize;
    let line_width = inner_w.max(20);

    let lines = wrap_lines(&app.target, line_width);
    let cursor = app.typed.len();
    let current = find_line(&lines, cursor);

    let half = VISIBLE_LINES / 2;
    let start = current.saturating_sub(half);
    let end = (start + VISIBLE_LINES).min(lines.len());
    let visible = &lines[start..end];

    let used = visible.len() * 2 - 1;
    let top_pad = inner_h.saturating_sub(used) / 2;

    let mut rendered: Vec<Line> = Vec::with_capacity(top_pad + visible.len() * 2);
    for _ in 0..top_pad {
        rendered.push(Line::raw(""));
    }
    for (i, (s, e)) in visible.iter().enumerate() {
        let is_current = start + i == current;
        rendered.push(line_for_range(app, *s, *e, is_current, line_width));
        if i + 1 < visible.len() {
            rendered.push(Line::raw(""));
        }
    }

    let inner = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: area.height,
    };
    f.render_widget(Paragraph::new(rendered).alignment(Alignment::Center), inner);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = if app.finished {
        format!(
            " done · wpm {:.1} · raw {:.1} · acc {:.1}% · enter to exit ",
            app.wpm(),
            app.raw_wpm(),
            app.accuracy()
        )
    } else if app.start.is_none() {
        " pick mode with tab · type to start · esc quit ".to_string()
    } else {
        format!(
            " {:.1}s · raw {:.1} · tab restart · esc quit ",
            app.elapsed(),
            app.raw_wpm()
        )
    };

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan)),
        inner,
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

fn line_for_range(
    app: &App,
    start: usize,
    end: usize,
    is_current: bool,
    width: usize,
) -> Line<'static> {
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

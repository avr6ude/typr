//! Stats sidebar, typing pane, and footer hint.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::Limit;

use super::dim_rgb;
use super::wrap::{find_line, wrap_lines};

const VISIBLE_LINES: usize = 3;

pub fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" stats ", Style::default().fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let primary: (&str, String, Color) = match app.limit {
        Limit::Time(_) => ("time", format!("{:.0}s", app.time_left()), Color::Cyan),
        Limit::Count(_) => (
            "words",
            format!("{}/{}", app.words_done(), app.target_words),
            Color::Cyan,
        ),
        Limit::Snippet => (
            "chars",
            format!("{}/{}", app.typed.len(), app.target.len()),
            Color::Cyan,
        ),
    };

    let rows: Vec<(&str, String, Color)> = vec![
        primary,
        ("wpm", format!("{:.1}", app.wpm()), Color::Green),
        ("raw", format!("{:.1}", app.raw_wpm()), Color::Gray),
        ("acc", format!("{:.1}%", app.accuracy()), Color::Yellow),
        ("typed", format!("{}", app.keystrokes), Color::Gray),
    ];

    let divider_w = inner.width.saturating_sub(2) as usize;
    let divider = Line::from(Span::styled(
        format!(" {} ", "─".repeat(divider_w)),
        Style::default().fg(Color::DarkGray),
    ));

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::raw(""));
    let last = rows.len();
    for (i, (k, v, c)) in rows.into_iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<7}"), Style::default().fg(Color::DarkGray)),
            Span::styled(v, Style::default().fg(c).add_modifier(Modifier::BOLD)),
        ]));
        if i + 1 < last {
            lines.push(Line::raw(""));
            lines.push(divider.clone());
            lines.push(Line::raw(""));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn draw_typing(f: &mut Frame, app: &App, area: Rect) {
    let inner_w = area.width.saturating_sub(4) as usize;
    let inner_h = area.height as usize;
    let is_code = app.is_code();
    let line_width = inner_w.max(20);

    let lines = wrap_lines(&app.target, line_width);
    let cursor = app.typed.len();
    let current = find_line(&lines, cursor);

    let (start, end, alignment) = if is_code {
        let visible_h = inner_h.max(3);
        let half = visible_h / 2;
        let s = current.saturating_sub(half);
        let e = (s + visible_h).min(lines.len());
        (s, e, Alignment::Left)
    } else {
        let half = VISIBLE_LINES / 2;
        let s = current.saturating_sub(half);
        let e = (s + VISIBLE_LINES).min(lines.len());
        (s, e, Alignment::Center)
    };
    let visible = &lines[start..end];

    let used = if is_code {
        visible.len()
    } else {
        visible.len().saturating_mul(2).saturating_sub(1)
    };
    let top_pad = inner_h.saturating_sub(used) / 2;

    let mut rendered: Vec<Line> = Vec::with_capacity(top_pad + visible.len() * 2);
    for _ in 0..top_pad {
        rendered.push(Line::raw(""));
    }
    for (i, (s, e)) in visible.iter().enumerate() {
        let is_current = start + i == current;
        rendered.push(line_for_range(app, *s, *e, is_current, line_width));
        if !is_code && i + 1 < visible.len() {
            rendered.push(Line::raw(""));
        }
    }

    let inner = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: area.height,
    };
    f.render_widget(Paragraph::new(rendered).alignment(alignment), inner);
}

pub fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = if app.start.is_none() {
        " pick mode with tab/arrows · type to start · esc quit ".to_string()
    } else if app.is_code() {
        format!(
            " {:.1}s · raw {:.1} · tab indent · esc restart · ctrl+c quit ",
            app.elapsed(),
            app.raw_wpm()
        )
    } else {
        format!(
            " {:.1}s · raw {:.1} · esc restart · ctrl+c quit ",
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

fn line_for_range(
    app: &App,
    start: usize,
    end: usize,
    is_current: bool,
    width: usize,
) -> Line<'static> {
    let is_code = app.is_code();
    let mut spans: Vec<Span> = Vec::with_capacity(end - start);
    let mut visible = 0usize;
    for i in start..end {
        let ch = app.target[i];
        let typed = app.typed.get(i).copied();
        let syntax_color = app.colors.get(i).copied().unwrap_or(Color::Gray);
        let mut style = if let Some(t) = typed {
            if t == ch {
                if is_code {
                    Style::default().fg(syntax_color)
                } else {
                    Style::default().fg(Color::Green)
                }
            } else {
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::UNDERLINED)
            }
        } else if i == app.typed.len() {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if is_code {
            Style::default().fg(dim_rgb(syntax_color))
        } else if is_current {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        if is_current && !is_code {
            style = style.add_modifier(Modifier::BOLD);
        }
        let display = if ch == '\n' {
            '↵'
        } else if ch == ' ' && typed.is_some() && typed != Some(ch) {
            '_'
        } else {
            ch
        };
        spans.push(Span::styled(display.to_string(), style));
        visible += 1;
    }
    let pad = width.saturating_sub(visible);
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad)));
    }
    Line::from(spans)
}

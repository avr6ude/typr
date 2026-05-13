use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::{
    difficulty_label, lang_label, limit_label, limits_for, source_label, Limit, Source,
    DIFFICULTIES, LANGS, SOURCES,
};

const VISIBLE_LINES: usize = 3;
const SIDEBAR_WIDTH: u16 = 22;

pub fn draw(f: &mut Frame, app: &App) {
    if app.finished {
        draw_results(f, app);
        return;
    }
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

    let mode_h = 5;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(mode_h),
            Constraint::Min(7),
            Constraint::Length(2),
        ])
        .split(inner);

    draw_mode_bar(f, app, rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(20)])
        .split(rows[1]);

    draw_stats(f, app, cols[0]);
    draw_typing(f, app, cols[1]);
    draw_footer(f, app, rows[2]);
}

fn draw_mode_bar(f: &mut Frame, app: &App, area: Rect) {
    let started = app.start.is_some();
    let is_code = matches!(app.source, Source::Code);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" mode ", Style::default().fg(Color::Gray)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let sep = Span::styled("  ·  ", Style::default().fg(Color::DarkGray));

    let mut row_src: Vec<Span> = vec![Span::styled(" source: ", Style::default().fg(Color::DarkGray))];
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
    let hint = if started && is_code {
        "    tab indent"
    } else if started {
        "    tab restart"
    } else if is_code {
        "    ↑↓ source · ←→ lang · tab limit"
    } else {
        "    ↑↓ source · ←→ difficulty · tab limit"
    };
    row_src.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));
    f.render_widget(Paragraph::new(Line::from(row_src)), inner_rows[0]);

    let limits = limits_for(app.source);
    let mut row_lim: Vec<Span> = vec![Span::styled(" limit:  ", Style::default().fg(Color::DarkGray))];
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
    f.render_widget(Paragraph::new(Line::from(row_lim)), inner_rows[1]);

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
            format!(" {} ", text),
            chip_style(*selected, started),
        ));
        if i + 1 < items.len() {
            row3.push(sep.clone());
        }
    }
    f.render_widget(Paragraph::new(Line::from(row3)), inner_rows[2]);
}

fn dim_rgb(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        _ => Color::DarkGray,
    }
}

fn chip_style(selected: bool, started: bool) -> Style {
    if selected {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else if started {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
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
            Span::styled(format!("  {:<7}", k), Style::default().fg(Color::DarkGray)),
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

fn draw_typing(f: &mut Frame, app: &App, area: Rect) {
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

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
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

fn draw_results(f: &mut Frame, app: &App) {
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

    let wpm_big = big_number(&format!("{:.0}", app.wpm()));
    let wpm_lines: Vec<Line> = wpm_big
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

    let g1: Vec<Span> = vec![
        kv("raw", format!("{:.1}", app.raw_wpm()), Color::Gray),
        sep(),
        kv("accuracy", format!("{:.1}%", app.accuracy()), Color::Green),
        sep(),
        kv("time", format!("{:.1}s", app.elapsed()), Color::Cyan),
    ]
    .into_iter()
    .flatten()
    .collect();
    let g2: Vec<Span> = vec![
        kv("correct", format!("{}", app.correct), Color::Green),
        sep(),
        kv("wrong", format!("{}", app.incorrect), Color::Red),
        sep(),
        kv(
            "chars",
            format!("{}", app.correct + app.incorrect),
            Color::Gray,
        ),
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
        Span::styled(
            format!("{}: ", label),
            Style::default().fg(Color::DarkGray),
        ),
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

pub fn wrap_lines(target: &[char], width: usize) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let len = target.len();
    let mut start = 0;
    while start < len {
        let hard_end = (start + width).min(len);
        let newline = target[start..hard_end].iter().position(|c| *c == '\n');
        let end = if let Some(p) = newline {
            start + p + 1
        } else if hard_end < len {
            if let Some(p) = target[start..hard_end].iter().rposition(|c| *c == ' ') {
                start + p + 1
            } else {
                hard_end
            }
        } else {
            hard_end
        };
        let end = if end == start { hard_end.max(start + 1) } else { end };
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
                Style::default().fg(Color::Red).add_modifier(Modifier::UNDERLINED)
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


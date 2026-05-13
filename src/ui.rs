use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cli::{amount_label, lang_label, type_label, Mode, PRESETS};

const VISIBLE_LINES: usize = 3;
const SIDEBAR_WIDTH: u16 = 22;

pub fn draw(f: &mut Frame, app: &App, preset_idx: usize) {
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

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
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

    let inner_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let current_mode = PRESETS[idx].mode;
    let sep = Span::styled("  ·  ", Style::default().fg(Color::DarkGray));
    let divider = Span::styled("   │   ", Style::default().fg(Color::DarkGray));

    let time_presets: Vec<(usize, &crate::cli::Preset)> = PRESETS
        .iter()
        .enumerate()
        .filter(|(_, p)| p.mode == Mode::Time)
        .collect();

    let mut row1: Vec<Span> = vec![Span::raw(" ")];
    for (j, (gi, p)) in time_presets.iter().enumerate() {
        let selected = *gi == idx;
        row1.push(Span::styled(format!(" {} ", amount_label(p)), chip_style(selected, started)));
        if j + 1 < time_presets.len() {
            row1.push(sep.clone());
        }
    }
    row1.push(divider.clone());
    for (j, t) in [Mode::Words, Mode::Code].iter().enumerate() {
        let selected = current_mode == *t;
        row1.push(Span::styled(format!(" {} ", type_label(*t)), chip_style(selected, started)));
        if j + 1 < 2 {
            row1.push(sep.clone());
        }
    }
    let hint1 = if started {
        "    tab restart"
    } else {
        "    ↑↓ switch · tab cycle"
    };
    row1.push(Span::styled(hint1, Style::default().fg(Color::DarkGray)));
    f.render_widget(Paragraph::new(Line::from(row1)), inner_rows[0]);

    if current_mode != Mode::Time {
        let group: Vec<(usize, &crate::cli::Preset)> = PRESETS
            .iter()
            .enumerate()
            .filter(|(_, p)| p.mode == current_mode)
            .collect();
        let label_prefix = match current_mode {
            Mode::Words => " words: ",
            Mode::Code => " code:  ",
            _ => " ",
        };
        let mut row2: Vec<Span> =
            vec![Span::styled(label_prefix, Style::default().fg(Color::DarkGray))];
        for (j, (gi, p)) in group.iter().enumerate() {
            let selected = *gi == idx;
            row2.push(Span::styled(format!(" {} ", amount_label(p)), chip_style(selected, started)));
            if j + 1 < group.len() {
                row2.push(sep.clone());
            }
        }
        f.render_widget(Paragraph::new(Line::from(row2)), inner_rows[1]);
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

    let rows: Vec<(&str, String, Color)> = vec![
        match app.mode {
            Mode::Time => ("time", format!("{:.0}s", app.time_left()), Color::Cyan),
            Mode::Words => (
                "words",
                format!("{}/{}", app.words_done(), app.target_words),
                Color::Cyan,
            ),
            Mode::Code => (
                "lang",
                app.lang
                    .map(lang_label)
                    .unwrap_or("plain")
                    .to_string(),
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
    let is_code = matches!(app.mode, Mode::Code);
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
        visible.len() * 2 - 1
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
        kv(
            match app.mode {
                Mode::Time => "time",
                Mode::Words => "words",
                Mode::Code => "lang",
            },
            match app.mode {
                Mode::Time => format!("{:.1}s", app.elapsed()),
                Mode::Words => format!("{}/{}", app.words_done(), app.target_words),
                Mode::Code => app.lang.map(lang_label).unwrap_or("plain").to_string(),
            },
            Color::Cyan,
        ),
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
        Paragraph::new(vec![
            Line::from(g1),
            Line::raw(""),
            Line::from(g2),
        ])
        .alignment(Alignment::Center),
        rows[3],
    );

    f.render_widget(
        Paragraph::new("enter retry · tab next mode · shift+tab prev · esc exit")
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
    let is_code = matches!(app.mode, Mode::Code);
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
            Style::default().fg(syntax_color).add_modifier(Modifier::DIM)
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

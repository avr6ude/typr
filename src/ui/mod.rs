//! TUI rendering. Submodules are pure renderers; this module owns the
//! top-level layout, shared styling, and the public `draw` entry point.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;

mod mode_bar;
mod results;
mod typing;
mod wrap;

const SIDEBAR_WIDTH: u16 = 22;

pub fn draw(f: &mut Frame, app: &App) {
    if app.finished {
        results::draw(f, app);
        return;
    }
    let area = f.area();
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " typr ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(7),
            Constraint::Length(2),
        ])
        .split(inner);

    mode_bar::draw(f, app, rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(20)])
        .split(rows[1]);

    typing::draw_stats(f, app, cols[0]);
    typing::draw_typing(f, app, cols[1]);
    typing::draw_footer(f, app, rows[2]);
}

pub(super) fn chip_style(selected: bool, started: bool) -> Style {
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

pub(super) fn dim_rgb(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        _ => Color::DarkGray,
    }
}

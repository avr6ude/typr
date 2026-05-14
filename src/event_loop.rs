//! Input dispatch loop. Reads keys, mutates `App`, ticks the clock, and
//! redraws on every iteration.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Terminal;

use crate::app::App;
use crate::ui::draw;

const TICK: Duration = Duration::from_millis(100);

pub fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, app))?;

        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c'))
                {
                    return Ok(());
                }
                if matches!(key.code, KeyCode::Char(_))
                    && key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
                {
                    continue;
                }
                if handle_key(app, key.code) {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= TICK {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

/// Returns `true` if the loop should exit.
fn handle_key(app: &mut App, code: KeyCode) -> bool {
    let can_cycle = app.start.is_none() || app.finished;
    match code {
        KeyCode::Esc => {
            if app.start.is_some() && !app.finished {
                app.restart();
                false
            } else {
                true
            }
        }
        KeyCode::Tab => {
            if can_cycle {
                app.cycle_limit_next();
            } else if app.is_code() {
                advance_indent(app);
            } else {
                app.restart();
            }
            false
        }
        KeyCode::BackTab => {
            if can_cycle {
                app.cycle_limit_prev();
            }
            false
        }
        KeyCode::Up => {
            if can_cycle {
                app.cycle_source_prev();
            }
            false
        }
        KeyCode::Down => {
            if can_cycle {
                app.cycle_source_next();
            }
            false
        }
        KeyCode::Left => {
            if can_cycle {
                app.cycle_secondary_prev();
            }
            false
        }
        KeyCode::Right => {
            if can_cycle {
                app.cycle_secondary_next();
            }
            false
        }
        KeyCode::Enter => {
            if app.finished {
                app.restart();
            } else if app.is_code() && app.start.is_some() {
                app.push('\n');
            }
            false
        }
        KeyCode::Backspace => {
            if !app.finished {
                app.backspace();
            }
            false
        }
        KeyCode::Char(c) => {
            if !app.finished {
                app.push(c);
            }
            false
        }
        _ => false,
    }
}

fn advance_indent(app: &mut App) {
    while app.typed.len() < app.target.len() {
        let next = app.target[app.typed.len()];
        if next == ' ' || next == '\t' {
            app.push(next);
        } else {
            break;
        }
    }
}

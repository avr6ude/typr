use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Terminal;

use crate::app::App;
use crate::cli::{preset_index, PRESETS};
use crate::ui::draw;

pub fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let tick = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    let mut preset_idx = preset_index(app.mode, app.amount);

    loop {
        terminal.draw(|f| draw(f, app, preset_idx))?;

        let timeout = tick.saturating_sub(last_tick.elapsed());
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
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        if app.start.is_none() {
                            preset_idx = (preset_idx + 1) % PRESETS.len();
                        } else {
                            preset_idx = preset_index(app.mode, app.amount);
                        }
                        let p = PRESETS[preset_idx];
                        *app = App::new(p.mode, p.amount);
                    }
                    KeyCode::BackTab => {
                        if app.start.is_none() {
                            preset_idx = (preset_idx + PRESETS.len() - 1) % PRESETS.len();
                            let p = PRESETS[preset_idx];
                            *app = App::new(p.mode, p.amount);
                        }
                    }
                    KeyCode::Backspace => app.backspace(),
                    KeyCode::Char(c) => app.push(c),
                    KeyCode::Enter => {
                        if app.finished {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick {
            app.tick();
            last_tick = Instant::now();
        }

        if app.finished {
            terminal.draw(|f| draw(f, app, preset_idx))?;
            wait_exit()?;
            return Ok(());
        }
    }
}

fn wait_exit() -> io::Result<()> {
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(())
                }
                _ => {}
            }
        }
    }
}

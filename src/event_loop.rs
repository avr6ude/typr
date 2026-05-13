use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Terminal;

use crate::app::App;
use crate::cli::{
    limits_for, next_in, prev_in, Limit, Source, DIFFICULTIES, LANGS, SOURCES,
};
use crate::ui::draw;

pub fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let tick = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, app))?;

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
                let can_cycle = app.start.is_none() || app.finished;
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        if can_cycle {
                            let limits = limits_for(app.source);
                            app.limit = next_in(limits, app.limit);
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        } else if app.is_code() {
                            while app.typed.len() < app.target.len() {
                                let next = app.target[app.typed.len()];
                                if next == ' ' || next == '\t' {
                                    app.push(next);
                                } else {
                                    break;
                                }
                            }
                        } else {
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::BackTab => {
                        if can_cycle {
                            let limits = limits_for(app.source);
                            app.limit = prev_in(limits, app.limit);
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::Up => {
                        if can_cycle {
                            let new_src = prev_in(SOURCES, app.source);
                            let new_limit = adjust_limit(app.limit, new_src);
                            *app = App::new(new_src, new_limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::Down => {
                        if can_cycle {
                            let new_src = next_in(SOURCES, app.source);
                            let new_limit = adjust_limit(app.limit, new_src);
                            *app = App::new(new_src, new_limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::Left => {
                        if can_cycle {
                            match app.source {
                                Source::Code => app.lang = prev_in(LANGS, app.lang),
                                Source::Text => {
                                    app.difficulty = prev_in(DIFFICULTIES, app.difficulty)
                                }
                            }
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::Right => {
                        if can_cycle {
                            match app.source {
                                Source::Code => app.lang = next_in(LANGS, app.lang),
                                Source::Text => {
                                    app.difficulty = next_in(DIFFICULTIES, app.difficulty)
                                }
                            }
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        }
                    }
                    KeyCode::Enter => {
                        if app.finished {
                            *app = App::new(app.source, app.limit, app.lang, app.difficulty);
                        } else if app.is_code() {
                            app.push('\n');
                        }
                    }
                    KeyCode::Backspace => {
                        if !app.finished {
                            app.backspace();
                        }
                    }
                    KeyCode::Char(c) => {
                        if !app.finished {
                            app.push(c);
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
    }
}

fn adjust_limit(current: Limit, new_source: Source) -> Limit {
    let allowed = limits_for(new_source);
    if allowed.contains(&current) {
        current
    } else {
        allowed[0]
    }
}

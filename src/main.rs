//! Binary entry point: parses CLI, sets up terminal, runs the event loop,
//! and tears down on exit.

mod app;
mod cli;
mod code;
mod event_loop;
mod ui;
mod words;

use std::io;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::cli::Cli;
use crate::event_loop::event_loop;

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let mut app = App::new(cli.source, cli.limit, cli.lang, cli.difficulty);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    print_results(&app);
    res
}

fn print_results(app: &App) {
    if app.finished {
        println!(
            "wpm {:.1} | raw {:.1} | acc {:.1}% | {:.1}s | {}/{} chars",
            app.wpm(),
            app.raw_wpm(),
            app.accuracy(),
            app.elapsed(),
            app.correct,
            app.correct + app.incorrect
        );
    }
}

fn main() -> io::Result<()> {
    run()
}

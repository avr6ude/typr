mod app;
mod cli;
mod code;
mod event_loop;
mod ui;

use std::io;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::cli::Cli;
use crate::event_loop::event_loop;

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let mut app = App::new(cli.mode, cli.amount, cli.lang);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    print_results(&app);
    res
}

fn print_results(app: &App) {
    if app.start.is_some() {
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

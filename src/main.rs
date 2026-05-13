use std::io;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::seq::SliceRandom;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

const WORDS: &str = include_str!("words.txt");

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Mode {
    Time,
    Words,
}

#[derive(Parser, Debug)]
#[command(name = "typr", about = "CLI typing test")]
struct Cli {
    #[arg(short, long, value_enum, default_value_t = Mode::Time)]
    mode: Mode,

    #[arg(short, long, default_value_t = 30)]
    amount: u32,
}

struct App {
    target: Vec<char>,
    typed: Vec<char>,
    correct: usize,
    incorrect: usize,
    start: Option<Instant>,
    finished: bool,
    mode: Mode,
    amount: u32,
    target_words: usize,
}

impl App {
    fn new(mode: Mode, amount: u32) -> Self {
        let pool: Vec<&str> = WORDS.split_whitespace().collect();
        let mut rng = rand::thread_rng();
        let count = match mode {
            Mode::Time => 500,
            Mode::Words => amount as usize,
        };
        let picked: Vec<String> = (0..count)
            .map(|_| pool.choose(&mut rng).unwrap().to_string())
            .collect();
        let target: Vec<char> = picked.join(" ").chars().collect();
        Self {
            target,
            typed: Vec::new(),
            correct: 0,
            incorrect: 0,
            start: None,
            finished: false,
            mode,
            amount,
            target_words: count,
        }
    }

    fn elapsed(&self) -> f64 {
        self.start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0)
    }

    fn time_left(&self) -> f64 {
        match self.mode {
            Mode::Time => (self.amount as f64 - self.elapsed()).max(0.0),
            Mode::Words => 0.0,
        }
    }

    fn wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < 0.1 {
            return 0.0;
        }
        (self.correct as f64 / 5.0) / (secs / 60.0)
    }

    fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed();
        if secs < 0.1 {
            return 0.0;
        }
        (self.typed.len() as f64 / 5.0) / (secs / 60.0)
    }

    fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            return 100.0;
        }
        (self.correct as f64 / total as f64) * 100.0
    }

    fn words_done(&self) -> usize {
        if self.typed.is_empty() {
            return 0;
        }
        let upto: String = self.target[..self.typed.len().min(self.target.len())]
            .iter()
            .collect();
        let raw = upto.split_whitespace().count();
        if self.typed.last() == Some(&' ') {
            raw
        } else {
            raw.saturating_sub(1)
        }
    }

    fn push(&mut self, c: char) {
        if self.finished {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        let idx = self.typed.len();
        if idx >= self.target.len() {
            return;
        }
        if self.target[idx] == c {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }
        self.typed.push(c);
        self.check_done();
    }

    fn backspace(&mut self) {
        if let Some(c) = self.typed.pop() {
            let idx = self.typed.len();
            if self.target[idx] == c {
                self.correct = self.correct.saturating_sub(1);
            } else {
                self.incorrect = self.incorrect.saturating_sub(1);
            }
        }
    }

    fn check_done(&mut self) {
        match self.mode {
            Mode::Time => {
                if self.elapsed() >= self.amount as f64 {
                    self.finished = true;
                }
            }
            Mode::Words => {
                if self.words_done() >= self.target_words {
                    self.finished = true;
                }
            }
        }
    }

    fn tick(&mut self) {
        if matches!(self.mode, Mode::Time)
            && self.start.is_some()
            && self.elapsed() >= self.amount as f64
        {
            self.finished = true;
        }
    }
}

fn render_target(app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::with_capacity(app.target.len());
    for (i, ch) in app.target.iter().enumerate() {
        let style = if i < app.typed.len() {
            if app.typed[i] == *ch {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red).add_modifier(Modifier::UNDERLINED)
            }
        } else if i == app.typed.len() {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let display = if *ch == ' ' && i < app.typed.len() && app.typed[i] != *ch {
            '_'
        } else {
            *ch
        };
        spans.push(Span::styled(display.to_string(), style));
    }
    spans
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let mut app = App::new(cli.mode, cli.amount);

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

fn event_loop<B: ratatui::backend::Backend>(
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
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c'))
                {
                    return Ok(());
                }
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        *app = App::new(app.mode, app.amount);
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
            terminal.draw(|f| draw(f, app))?;
            wait_exit()?;
            return Ok(());
        }
    }
}

fn wait_exit() -> io::Result<()> {
    loop {
        if let Event::Key(key) = event::read()? {
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

fn draw(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = match app.mode {
        Mode::Time => format!(
            " time  {:>3.0}s   wpm {:>5.1}   acc {:>5.1}%   [esc quit · tab restart] ",
            app.time_left(),
            app.wpm(),
            app.accuracy()
        ),
        Mode::Words => format!(
            " words {}/{}   wpm {:>5.1}   acc {:>5.1}%   [esc quit · tab restart] ",
            app.words_done(),
            app.target_words,
            app.wpm(),
            app.accuracy()
        ),
    };

    let header_p = Paragraph::new(header)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" typr "));
    f.render_widget(header_p, chunks[0]);

    let spans = render_target(app);
    let body = Paragraph::new(Line::from(spans))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(body, chunks[1]);

    let footer_text = if app.finished {
        format!(
            " done · wpm {:.1} · raw {:.1} · acc {:.1}% · {}/{} correct chars · enter/esc to exit ",
            app.wpm(),
            app.raw_wpm(),
            app.accuracy(),
            app.correct,
            app.correct + app.incorrect
        )
    } else if app.start.is_none() {
        " start typing... ".to_string()
    } else {
        format!(" raw {:.1} wpm · {:.1}s elapsed ", app.raw_wpm(), app.elapsed())
    };

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
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

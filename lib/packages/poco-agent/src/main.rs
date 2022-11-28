pub mod agent;
pub mod config;
pub mod trace;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex},
};
use trace::{AppCustomLayer, TracingEvent, TracingEvents};
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::trace::TracingCategory;

enum InputMode {
    Normal,
    Edit,
}

struct App {
    command: String,
    mode: InputMode,
    events: TracingEvents,
}

impl App {
    fn get_layer(&self) -> AppCustomLayer {
        AppCustomLayer::new(self.events.clone())
    }
}

impl Default for App {
    fn default() -> App {
        App {
            command: String::new(),
            mode: InputMode::Normal,
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

fn main() -> Result<(), io::Error> {
    let app = App::default();

    // Init Tracing
    tracing_subscriber::registry()
        // .with(tracing_subscriber::fmt::layer())
        // .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(app.get_layer())
        .init();

    tracing::event!(
        Level::INFO,
        message = "start initializing terminal ui",
        category = format!("{:?}", TracingCategory::Agent)
    );

    // Init Terminal UI State
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title(" PoCo ").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    let ret = run_app(&mut terminal, app);

    // Restore Terminal UI State
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = ret {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                InputMode::Normal => match key.code {
                    event::KeyCode::Char('i') => {
                        app.mode = InputMode::Edit;
                    }
                    event::KeyCode::Char('q') => return Ok(()),
                    _ => {}
                },
                InputMode::Edit => match key.code {
                    event::KeyCode::Enter => {
                        let command: String = app.command.drain(..).collect();

                        tracing::event!(
                            Level::INFO,
                            message = command,
                            category = format!("{:?}", TracingCategory::Agent)
                        );
                    }
                    event::KeyCode::Char(c) => {
                        app.command.push(c);
                    }
                    event::KeyCode::Backspace => {
                        app.command.pop();
                    }
                    event::KeyCode::Esc => {
                        app.mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let (msg, style) = match app.mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Edit => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to commit the command."),
            ],
            Style::default(),
        ),
    };

    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, chunks[2]);

    let input = Paragraph::new(app.command.as_ref())
        .style(match app.mode {
            InputMode::Normal => Style::default(),
            InputMode::Edit => Style::default().fg(Color::LightBlue),
        })
        .block(Block::default().borders(Borders::ALL).title(" Command "));
    frame.render_widget(input, chunks[1]);

    match app.mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Edit => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            frame.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.command.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let guard = app.events.lock().unwrap();

    let height = chunks[0].height as usize;

    let events: Vec<ListItem> = guard
        .iter()
        .rev()
        .take(height)
        .rev()
        .map(|m| ListItem::new(Spans(m.to_spans())))
        .collect();

    let events = List::new(events).block(Block::default().borders(Borders::ALL).title(" Events "));

    frame.render_widget(events, chunks[0]);
}

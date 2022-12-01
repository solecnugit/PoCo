pub mod action;
pub mod state;

use std::collections::VecDeque;
use std::io;
use std::time;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;

use self::action::UIAction;
use self::action::UIActionEvent;
use self::state::UIInputMode;
use self::state::UIState;

use super::CommandString;

pub struct UI {
    state: UIState,

    receiver: crossbeam_channel::Receiver<UIActionEvent>,
    sender: crossbeam_channel::Sender<String>,
}

impl UI {
    pub fn new(
        receiver: crossbeam_channel::Receiver<UIActionEvent>,
        sender: crossbeam_channel::Sender<String>,
    ) -> Self {
        let state = UIState {
            mode: UIInputMode::Normal,
            input: String::new(),
            ui_event_logs: VecDeque::new(),
        };

        UI {
            state,
            receiver,
            sender,
        }
    }

    pub fn receive_commands(&mut self) {
        while let Ok(command) = self.receiver.try_recv() {
            self.state.ui_event_logs.push_back(command);
        }
    }

    pub fn run_ui(&mut self) -> Result<(), io::Error> {
        // Init Terminal UI State
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.ui_loop(&mut terminal)?;

        // Restore Terminal UI State
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        terminal.show_cursor()?;

        Ok(())
    }

    pub fn ui_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            self.receive_commands();

            terminal.draw(|frame| self.draw_ui(frame))?;

            if event::poll(time::Duration::from_millis(24))? {
                if let Event::Key(key) = event::read()? {
                    match self.state.mode {
                        UIInputMode::Normal => match key.code {
                            event::KeyCode::Char('i') => {
                                self.state.mode = UIInputMode::Edit;
                            }
                            event::KeyCode::Char('q') => return Ok(()),
                            _ => {}
                        },
                        UIInputMode::Edit => match key.code {
                            event::KeyCode::Enter => {
                                self.state.mode = UIInputMode::Normal;

                                if !self.state.input.is_empty() {
                                    let command =
                                        self.state.input.drain(..).collect::<CommandString>();

                                    self.state
                                        .ui_event_logs
                                        .push_back(UIAction::LogCommand(command.clone()).into());

                                    self.sender.send(command).unwrap();
                                }
                                self.state.input.clear();
                            }
                            event::KeyCode::Char(c) => {
                                self.state.input.push(c);
                            }
                            event::KeyCode::Backspace => {
                                self.state.input.pop();
                            }
                            event::KeyCode::Esc => {
                                self.state.mode = UIInputMode::Normal;
                                self.state.input.clear();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    pub fn draw_ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
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

        let (msg, style) = match self.state.mode {
            UIInputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            UIInputMode::Edit => (
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

        let input = Paragraph::new(self.state.input.as_ref())
            .style(match self.state.mode {
                UIInputMode::Normal => Style::default(),
                UIInputMode::Edit => Style::default().fg(Color::LightBlue),
            })
            .block(Block::default().borders(Borders::ALL).title(" Command "));
        frame.render_widget(input, chunks[1]);

        match self.state.mode {
            UIInputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            UIInputMode::Edit => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                frame.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + self.state.input.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
        }

        // padding 2: Up and Bottom Border
        let height = (chunks[0].height - 2) as usize;

        let logs: Vec<ListItem> = self
            .state
            .ui_event_logs
            .iter()
            .rev()
            .take(height)
            .rev()
            .flat_map(|event| event.to_spans())
            .map(ListItem::new)
            .collect();

        let logs = List::new(logs).block(Block::default().borders(Borders::ALL).title(" Logs "));

        frame.render_widget(logs, chunks[0]);
    }
}

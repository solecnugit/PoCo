use std::{io, sync::Arc};
use std::time;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{Frame, Terminal};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, List, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::backend::command::CommandSource;
use crate::config::PocoAgentConfig;

use super::CommandString;

use self::event::UIAction;
use self::event::UIActionEvent;
use self::state::UIInputMode;
use self::state::UIState;

pub mod event;
pub mod state;
pub mod util;

pub struct UI {
    state: UIState,
    config: Arc<PocoAgentConfig>,

    receiver: crossbeam_channel::Receiver<UIActionEvent>,
    sender: crossbeam_channel::Sender<CommandSource>,

    command_counter: u64,
}

impl UI {
    pub fn new(
        receiver: crossbeam_channel::Receiver<UIActionEvent>,
        sender: crossbeam_channel::Sender<CommandSource>,
        config: Arc<PocoAgentConfig>,
    ) -> Self {
        let state = UIState::new(5001);

        UI {
            state,
            receiver,
            sender,
            config,
            command_counter: 0,
        }
    }

    pub fn retrieve_events(&mut self) -> bool {
        while let Ok(event) = self.receiver.try_recv() {
            if let UIAction::Panic(_) = event.1 {
                return false;
            } else {
                self.state.push_event(event);
            }
        }

        true
    }

    pub fn run_ui(&mut self) -> anyhow::Result<()> {
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

    pub fn ui_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        loop {
            if !self.retrieve_events() {
                return Err(anyhow::anyhow!("UI panicked"));
            }

            terminal.draw(|frame| self.draw_ui(frame))?;

            if crossterm::event::poll(time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    match self.state.mode {
                        UIInputMode::Normal => match key.code {
                            crossterm::event::KeyCode::Char('i') => {
                                self.state.mode = UIInputMode::Edit;
                            }
                            crossterm::event::KeyCode::Char('q') => return Ok(()),
                            crossterm::event::KeyCode::Up => {
                                self.state.offset += 1;
                            }
                            crossterm::event::KeyCode::Down => {
                                if self.state.offset > 0 {
                                    self.state.offset -= 1;
                                }
                            }
                            _ => {}
                        },
                        UIInputMode::Edit => match key.code {
                            crossterm::event::KeyCode::Enter => {
                                self.state.mode = UIInputMode::Normal;

                                if !self.state.input.is_empty() {
                                    let command =
                                        self.state.input.drain(..).collect::<CommandString>();

                                    let command_id = format!("#{}", self.command_counter);
                                    let source = CommandSource {
                                        source: command,
                                        id: command_id,
                                    };

                                    self.state
                                        .push_event(UIAction::LogCommand(source.clone()).into());

                                    self.command_counter += 1;

                                    self.sender.send(source).unwrap();
                                }
                                self.state.input.clear();
                            }
                            crossterm::event::KeyCode::Char(c) => {
                                self.state.input.push(c);
                            }
                            crossterm::event::KeyCode::Backspace => {
                                self.state.input.pop();
                            }
                            crossterm::event::KeyCode::Esc => {
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
        let height = match chunks[0].height.overflowing_sub(2) {
            (x, false) => x as usize,
            _ => 0,
        };

        let width = match chunks[0].width.overflowing_sub(2) {
            (x, false) => x as usize,
            _ => 0,
        };

        let event_list =
            self.state
                .render_event_list(self.config.ui.time_format.as_str(), width, height);
        let logs =
            List::new(event_list).block(Block::default().borders(Borders::ALL).title(" Logs "));

        frame.render_widget(logs, chunks[0]);
    }
}

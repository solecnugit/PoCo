use std::error::Error;

use chrono::{DateTime, Local};
use strum::Display;
use tracing::Level;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};
use unicode_width::UnicodeWidthStr;

use crate::app::{
    backend::command::CommandSource,
    trace::{TracingCategory, TracingEvent},
};

pub struct UIActionEvent(pub DateTime<Local>, pub UIAction);

impl UIActionEvent {
    pub fn new(action: UIAction) -> Self {
        UIActionEvent(Local::now(), action)
    }
}

impl From<UIAction> for UIActionEvent {
    fn from(action: UIAction) -> Self {
        UIActionEvent::new(action)
    }
}

#[derive(Debug, Clone, Display)]
pub enum CommandExecutionStage {
    Parsing,
    Executed,
}

#[derive(Debug, Clone, Display)]
pub enum CommandExecutionStatus {
    Succeed,
    Failed,
}

#[derive(Debug)]
pub enum UIAction {
    Panic(String),
    LogString(String),
    LogMultipleStrings(Vec<String>),
    LogTracingEvent(TracingEvent),
    LogCommand(CommandSource),
    LogCommandExecution(
        CommandSource,
        CommandExecutionStage,
        CommandExecutionStatus,
        Option<Box<anyhow::Error>>,
    ),
    QuitApp,
}

impl UIActionEvent {
    fn render_wrapped_string<'a>(
        &'a self,
        time_string: String,
        max_width: usize,
        string: &'a str,
    ) -> Vec<Spans<'a>> {
        let padding_width = time_string.width();

        let mut spans = vec![vec![]];

        spans[0].push(Span::styled(time_string, Style::default().fg(Color::White)));
        spans[0].push(Span::raw(" "));

        let max_width = max_width - padding_width - 1;
        let mut current_width = 0;

        for word in string.split_whitespace() {
            if current_width + word.width() > max_width {
                spans.push(vec![Span::styled(
                    " ".repeat(padding_width + 1),
                    Style::default().fg(Color::White),
                )]);

                current_width = 0;
            }

            let idx = spans.len() - 1;

            spans[idx].push(Span::raw(word));
            spans[idx].push(Span::raw(" "));

            current_width += word.width() + 1;
        }

        spans.into_iter().map(Spans::from).collect()
    }

    pub fn render_spans(&self, max_width: usize, time_format: &str) -> Vec<Spans> {
        let time_string = self.0.format(time_format).to_string();
        let max_width = match max_width.overflowing_sub(time_string.width() + 1) {
            (v, false) => v,
            _ => 0,
        };

        let time_span = Span::styled(time_string.clone(), Style::default().fg(Color::White));

        match &self.1 {
            UIAction::LogString(string) => {
                self.render_wrapped_string(time_string, max_width, string.as_str())
            }

            UIAction::LogMultipleStrings(strings) => strings
                .iter()
                .flat_map(|e| {
                    self.render_wrapped_string(time_string.clone(), max_width, e.as_str())
                })
                .collect(),

            UIAction::LogTracingEvent(event) => {
                let level_color = match event.level {
                    Level::TRACE => Color::White,
                    Level::DEBUG => Color::Cyan,
                    Level::INFO => Color::Green,
                    Level::WARN => Color::Yellow,
                    Level::ERROR => Color::Red,
                };

                let level_span =
                    Span::styled(event.level.as_str(), Style::default().fg(level_color));

                let category_color = match event.category {
                    TracingCategory::Contract => Color::LightYellow,
                    TracingCategory::Agent => Color::LightBlue,
                    TracingCategory::Config => Color::LightMagenta,
                    TracingCategory::Ipfs => Color::LightCyan,
                    TracingCategory::Backend => Color::LightBlue,
                };

                let category_span = Span::styled(
                    format!("[{}]", event.category),
                    Style::default().fg(category_color),
                );

                let message_span = Span::styled(
                    event.message.clone().unwrap_or_default(),
                    Style::default().fg(Color::White),
                );

                let ignore_fields = vec!["message", "category", "level"];

                let fields_span = event
                    .fields
                    .iter()
                    .filter(|e| ignore_fields.contains(&e.0.as_str()))
                    .flat_map(|(k, v)| {
                        vec![
                            Span::styled(k, Style::default().fg(Color::LightBlue)),
                            Span::raw("="),
                            Span::styled(v, Style::default().fg(Color::Green)),
                        ]
                    })
                    .collect::<Vec<Span>>();

                if fields_span.is_empty() {
                    vec![Spans::from(vec![
                        time_span,
                        Span::raw(" "),
                        level_span,
                        Span::raw(" "),
                        category_span,
                        Span::raw(" "),
                        message_span,
                    ])]
                } else {
                    vec![
                        Spans::from(vec![
                            time_span,
                            Span::raw(" "),
                            level_span,
                            Span::raw(" "),
                            category_span,
                            Span::raw(" "),
                            message_span,
                        ]),
                        Spans::from(fields_span),
                    ]
                }
            }

            UIAction::LogCommand(command_source) => {
                vec![Spans::from(vec![
                    time_span,
                    Span::raw(" "),
                    Span::styled(">>", Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(&command_source.id, Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled(&command_source.source, Style::default().fg(Color::White)),
                ])]
            }
            UIAction::QuitApp => vec![Spans::from(Span::styled(
                "Quitting app",
                Style::default().fg(Color::White),
            ))],
            UIAction::LogCommandExecution(source, stage, status, error) => vec![
                Spans::from(vec![
                    time_span,
                    Span::raw(" "),
                    Span::styled(
                        match status {
                            CommandExecutionStatus::Succeed => {
                                format!("Command {source} executed successfully")
                            }
                            CommandExecutionStatus::Failed => {
                                format!("Command {source} failed (Stage: {stage})", )
                            }
                        },
                        Style::default().fg(match status {
                            CommandExecutionStatus::Succeed => Color::Green,
                            CommandExecutionStatus::Failed => Color::Red,
                        }),
                    ),
                ]),
                match error {
                    Some(error) => Spans::from(vec![
                        Span::raw(" ".repeat(time_string.width() + 1)),
                        Span::styled(format!("Error: {error:?}"), Style::default().fg(Color::Red)),
                    ]),
                    None => Spans::from(vec![]),
                },
            ],
            UIAction::Panic(_) => unreachable!(),
        }
    }
}

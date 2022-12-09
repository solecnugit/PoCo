use chrono::{DateTime, Local};
use tracing::Level;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::app::trace::{TracingCategory, TracingEvent};

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

pub enum UIAction {
    Panic(String),
    LogString(String),
    LogMultipleString(Vec<String>),
    LogTracingEvent(TracingEvent),
    LogCommand(String),
    QuitApp,
}

impl UIActionEvent {
    pub fn render_spans(&self, time_format: &str) -> Vec<Spans> {
        let time_span = Span::styled(
            self.0.format(time_format).to_string(),
            Style::default().fg(Color::White),
        );

        match &self.1 {
            UIAction::LogString(string) => vec![Spans::from(vec![
                time_span,
                Span::raw(" "),
                Span::styled(string, Style::default().fg(Color::White)),
            ])],

            UIAction::LogMultipleString(strings) => strings
                .iter()
                .map(|e| {
                    Spans::from(vec![
                        time_span.clone(),
                        Span::raw(" "),
                        Span::styled(e, Style::default().fg(Color::White)),
                    ])
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

            UIAction::LogCommand(command) => {
                vec![Spans::from(vec![
                    time_span,
                    Span::raw(" "),
                    Span::styled(">>", Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(command, Style::default().fg(Color::White)),
                ])]
            }
            UIAction::QuitApp => vec![Spans::from(Span::styled(
                "Quitting app",
                Style::default().fg(Color::White),
            ))],

            UIAction::Panic(_) => unreachable!(),
        }
    }
}

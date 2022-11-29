use chrono::{DateTime, Local};
use tracing::Level;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::app::trace::TracingEvent;

pub enum UIAction {
    LogString(DateTime<Local>, String),
    LogTracingEvent(TracingEvent),
    LogCommandEvent(DateTime<Local>, String, Option<String>),
}

impl UIAction {
    pub fn to_ui_span(&self) -> Spans {
        match self {
            UIAction::LogString(timestamp, string) => Spans(vec![
                Span::styled(
                    timestamp.format("%H:%M:%S%.3f").to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" "),
                Span::styled("STR", Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::raw(string),
            ]),
            UIAction::LogTracingEvent(event) => {
                let mut spans = vec![];

                spans.push(Span::styled(
                    event.timestamp.format("%H:%M:%S%.3f").to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                spans.push(Span::raw(" "));

                spans.push(Span::styled(
                    event.level.as_str(),
                    match event.level {
                        Level::TRACE => Style::default().fg(Color::White),
                        Level::DEBUG => Style::default().fg(Color::White),
                        Level::INFO => Style::default().fg(Color::Green),
                        Level::WARN => Style::default().fg(Color::Yellow),
                        Level::ERROR => Style::default().fg(Color::Red),
                    },
                ));
                spans.push(Span::raw(" "));

                let message = event.message.clone().unwrap_or_default();

                spans.push(Span::raw(message));
                spans.push(Span::raw(" "));

                for (key, value) in &event.fields {
                    spans.push(Span::raw(format!(" {}={}", key, value)));
                }

                Spans(spans)
            }
            UIAction::LogCommandEvent(timestamp, command, err) => {
                if let Some(err) = err {
                    Spans(vec![
                        Span::styled(
                            timestamp.format("%H:%M:%S%.3f").to_string(),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(" "),
                        Span::styled("CMD", Style::default().fg(Color::LightBlue)),
                        Span::raw(" "),
                        Span::styled(
                            format!("{} {}", command, err),
                            Style::default().fg(Color::Red),
                        ),
                    ])
                } else {
                    Spans(vec![
                        Span::styled(
                            chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(" "),
                        Span::styled("CMD", Style::default().fg(Color::LightBlue)),
                        Span::raw(" "),
                        Span::styled(command, Style::default().fg(Color::Green)),
                    ])
                }
            }
        }
    }
}

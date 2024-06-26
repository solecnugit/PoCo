use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::{DateTime, Local};
use strum::Display;
use tracing::{field::Visit, Level, Subscriber};
use tracing_subscriber::Layer;

use super::ui::event::{UIAction, UIActionEvent};

#[derive(Debug, Clone)]
pub struct TracingEvent {
    pub category: TracingCategory,
    pub timestamp: DateTime<Local>,
    pub level: Level,
    pub message: Option<String>,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Display, Clone)]
pub enum TracingCategory {
    Contract,
    Agent,
    Ipfs,
    Config,
    Backend,
}

pub struct UITracingLayer {
    sender: crossbeam_channel::Sender<UIActionEvent>,
}

impl UITracingLayer {
    pub fn new(sender: crossbeam_channel::Sender<UIActionEvent>) -> Self {
        UITracingLayer { sender }
    }
}

impl<S: Subscriber> Layer<S> for UITracingLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let timestamp = Local::now();
        let level = metadata.level();
        let mut fields = HashMap::new();

        struct CustomVisitor<'a> {
            fields: &'a mut HashMap<String, String>,
        }

        impl<'a> Visit for CustomVisitor<'a> {
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                self.fields
                    .insert(field.name().to_string(), value.to_string());
            }

            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                let name = field.name();
                let value = format!("{value:?}");

                self.fields.insert(name.to_string(), value);
            }
        }

        event.record(&mut CustomVisitor {
            fields: &mut fields,
        });

        if let Some(category) = fields.get("category") {
            let category = match category.as_str() {
                "Contract" => TracingCategory::Contract,
                "Agent" => TracingCategory::Agent,
                "Config" => TracingCategory::Config,
                _ => return,
            };

            let message = fields.get("message").cloned();
            let fields = fields
                .into_iter()
                .filter(|e| e.0 != "message" && e.0 != "category")
                .map(|(k, v)| (k, v))
                .collect();

            let event = TracingEvent::new(category, timestamp, *level, message, fields);

            match self.sender.send(UIAction::LogTracingEvent(event).into()) {
                Ok(_) => {}
                Err(err) => {
                    tracing::error!(
                        message = "failed to send tracing event to ui",
                        error = format!("{err:?}")
                    );
                }
            }
        }
    }
}

impl TracingEvent {
    pub fn new(
        category: TracingCategory,
        timestamp: DateTime<Local>,
        level: Level,
        message: Option<String>,
        fields: Vec<(String, String)>,
    ) -> Self {
        Self {
            category,
            timestamp,
            level,
            message,
            fields,
        }
    }
}

impl Display for TracingEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();

        if let Some(msg) = &self.message {
            message.push_str(msg);
        }

        for (key, value) in &self.fields {
            message.push_str(format!("{key}: {value}, ").as_str());
        }

        write!(
            f,
            "{} {} {}",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.level,
            message
        )
    }
}

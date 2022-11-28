use std::sync::{Arc, Mutex};

use tracing::{field::Visit, Subscriber, Level};
use tracing_subscriber::Layer;

pub struct AppCustomLayer {
    events: Arc<Mutex<Vec<String>>>,
}

impl AppCustomLayer {
    pub fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
        Self { events }
    }
}

impl<S: Subscriber> Layer<S> for AppCustomLayer {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level();

        struct CustomVisitor<'a> {
            level: &'a Level,
            events: &'a Arc<Mutex<Vec<String>>>,
        }

        impl<'a> Visit for CustomVisitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    let prefix = format!("{} {:?}", self.level.to_string(), value);

                    let mut guard = self.events.lock().unwrap();

                    guard.push(prefix);
                }
            }
        }

        event.record(&mut CustomVisitor { level, events: &self.events });
    }
}

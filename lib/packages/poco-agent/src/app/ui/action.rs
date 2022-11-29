use crate::app::trace::TracingEvent;

pub enum UIAction {
    LogString(String),
    LogTracingEvent(TracingEvent),
}

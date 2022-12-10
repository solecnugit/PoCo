use std::collections::VecDeque;

use strum::Display;
use tui::widgets::ListItem;

use super::action::UIActionEvent;

#[derive(Display)]
pub(crate) enum UIInputMode {
    Normal,
    Edit,
}

pub(crate) struct UIState {
    pub mode: UIInputMode,
    pub input: String,
    pub offset: usize,
    pub buffer_size: usize,

    internal_event: VecDeque<UIActionEvent>,
}

impl UIState {
    pub fn new(buffer_size: usize) -> Self {
        UIState {
            mode: UIInputMode::Normal,
            input: String::new(),
            buffer_size,
            internal_event: VecDeque::new(),
            offset: 0,
        }
    }

    pub fn push_event(&mut self, event: UIActionEvent) {
        self.internal_event.push_back(event);

        if self.internal_event.len() > self.buffer_size {
            self.internal_event.pop_front();
        }
    }

    pub fn render_event_list(&mut self, time_format: &str, width: usize, height: usize) -> Vec<ListItem> {
        let items = self
            .internal_event
            .iter()
            .flat_map(|e| e.render_spans(width, time_format))
            .map(|e| ListItem::new(e))
            .collect::<Vec<ListItem>>();

        self.offset = self.offset.min(items.len().saturating_sub(height));

        items
            .into_iter()
            .rev()
            .skip(self.offset)
            .take(height)
            .rev()
            .collect()
    }
}

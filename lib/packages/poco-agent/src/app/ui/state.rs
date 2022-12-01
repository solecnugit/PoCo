use std::collections::VecDeque;

use strum::Display;

use super::action::UIActionEvent;

#[derive(Display)]
pub(crate) enum UIInputMode {
    Normal,
    Edit,
}

pub(crate) struct UIState {
    pub mode: UIInputMode,
    pub input: String,
    pub ui_event_logs: VecDeque<UIActionEvent>,
}

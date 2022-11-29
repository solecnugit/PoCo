use std::collections::VecDeque;

use super::action::UIAction;

pub(crate) enum UIInputMode {
    Normal,
    Edit,
}

pub(crate) struct UIState {
    pub mode: UIInputMode,
    pub input: String,
    pub ui_commands: VecDeque<UIAction>,
}

pub mod command;

use super::ui::action::UIAction;

pub struct Backend {
    receiver: crossbeam_channel::Receiver<String>,
    sender: crossbeam_channel::Sender<UIAction>,
}

impl Backend {
    pub fn new(
        receiver: crossbeam_channel::Receiver<String>,
        sender: crossbeam_channel::Sender<UIAction>,
    ) -> Self {
        Backend { receiver, sender }
    }

    pub fn run_backend(&mut self) {}
}

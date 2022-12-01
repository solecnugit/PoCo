pub mod command;

use crate::agent::agent::PocoAgent;

use super::ui::action::UIActionEvent;

pub struct Backend {
    receiver: crossbeam_channel::Receiver<String>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    agent: Box<PocoAgent>,
}

impl Backend {
    pub fn new(
        receiver: crossbeam_channel::Receiver<String>,
        sender: crossbeam_channel::Sender<UIActionEvent>,
    ) -> Self {
        Backend {
            receiver,
            sender,
            agent: Box::new(PocoAgent::new()),
        }
    }

    pub fn run_backend_thread(self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            match self.receiver.try_recv() {
                Ok(_command) => {
                    // let ui_action = self.agent.execute_command(command);
                    // self.sender.send(ui_action).unwrap();
                }
                Err(_) => {}
            }
        })
    }
}

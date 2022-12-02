pub mod command;

use crate::agent::agent::PocoAgent;
use crate::app::backend::command::parse_command;

use super::ui::action::UIActionEvent;

pub struct Backend {
    receiver: crossbeam_channel::Receiver<String>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    agent: Box<PocoAgent>,
    async_runtime: Box<tokio::runtime::Runtime>,
}

impl Backend {
    pub fn new(
        receiver: crossbeam_channel::Receiver<String>,
        sender: crossbeam_channel::Sender<UIActionEvent>,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Backend {
            receiver,
            sender,
            agent: Box::new(PocoAgent::new()),
            async_runtime: Box::new(runtime),
        }
    }

    pub fn run_backend_thread(self) -> std::thread::JoinHandle<()> {
        let builder = std::thread::Builder::new().name("backend".to_string());

        builder.spawn(move || loop {
            match self.receiver.try_recv() {
                Ok(command) => {
                    let command = command.trim();

                    parse_command(command);
                }
                Err(_) => {}
            }
        }).unwrap()
    }
}

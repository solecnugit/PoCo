use std::sync::Arc;

use crate::agent::agent::PocoAgent;
use crate::app::backend::command::{parse_command, BackendCommand, ParseBackendCommandError};
use crate::app::trace::TracingCategory;
use crate::config::PocoAgentConfig;
use crossbeam_channel::TryRecvError;
use thread_local::ThreadLocal;
use tracing::Level;

use super::ui::action::{UIAction, UIActionEvent};

pub mod command;

pub struct Backend {
    config: Arc<PocoAgentConfig>,
    receiver: crossbeam_channel::Receiver<String>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    async_runtime: Box<tokio::runtime::Runtime>,
    agent: Arc<ThreadLocal<PocoAgent>>,
}

impl Backend {
    pub fn new(
        config: Arc<PocoAgentConfig>,
        receiver: crossbeam_channel::Receiver<String>,
        sender: crossbeam_channel::Sender<UIActionEvent>,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Backend {
            config,
            receiver,
            sender,
            async_runtime: Box::new(runtime),
            agent: Arc::new(ThreadLocal::new()),
        }
    }

    fn execute_command(&mut self, command: BackendCommand) {
        match command {
            BackendCommand::HelpCommand(help) => {
                self.sender
                    .send(UIAction::LogMultipleString(help).into())
                    .unwrap();
            }
            BackendCommand::GasPriceCommand => {
                let sender = self.sender.clone();
                let agent = self.agent.clone();
                let config = self.config.clone();

                self.async_runtime.spawn(async move {
                    let agent =
                        agent.get_or(|| PocoAgent::connect(config.near.rpc_endpoint.as_str()));
                    let gas_price = agent.gas_price().await;

                    if let Ok(gas_price) = gas_price {
                        sender
                            .send(UIAction::LogString(format!("Gas Price: {}", gas_price)).into())
                            .unwrap();
                    } else {
                        sender
                            .send(UIAction::LogString("Failed to get gas price".to_string()).into())
                            .unwrap();
                    }
                });
            }
            BackendCommand::NetworkStatusCommand => {}
        }
    }

    pub fn run_backend_thread(mut self) -> std::thread::JoinHandle<()> {
        let builder = std::thread::Builder::new().name("backend".to_string());

        builder
            .spawn(move || 'outer: loop {
                loop {
                    match self.receiver.try_recv() {
                        Ok(command) => match parse_command(command.trim()) {
                            Ok(command) => self.execute_command(command),
                            Err(error) => match error {
                                ParseBackendCommandError::UnknownCommand(command) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message = format!("unknown command: {}", command),
                                        category = format!("{:?}", TracingCategory::Agent)
                                    );
                                }
                            },
                        },
                        Err(error) => match error {
                            TryRecvError::Empty => break,
                            TryRecvError::Disconnected => {
                                tracing::event!(
                                    Level::ERROR,
                                    message = "backend channel disconnected",
                                    category = format!("{:?}", TracingCategory::Agent)
                                );

                                break 'outer;
                            }
                        },
                    }
                }
            })
            .unwrap()
    }
}

use std::{sync::Arc, thread::JoinHandle};

use tracing::Level;

use crate::app::backend::command::CommandSource;
use crate::app::trace::TracingCategory;
use crate::app::ui::event::{CommandExecutionStatus, UIAction};
use crate::config::{AppRunningMode, PocoClientConfig};

use self::{
    backend::Backend,
    trace::UITracingLayer,
    ui::{event::UIActionEvent, UI},
};

pub mod backend;
pub mod trace;
pub mod ui;

type CommandString = String;

pub struct App {
    config: Arc<PocoClientConfig>,
    ui: UI,
    ui_channel: (
        crossbeam_channel::Sender<UIActionEvent>,
        crossbeam_channel::Receiver<UIActionEvent>,
    ),
    backend_channel: (
        crossbeam_channel::Sender<CommandSource>,
        crossbeam_channel::Receiver<CommandSource>,
    ),
    join_handles: Vec<JoinHandle<()>>,
}

impl App {
    pub fn new(config: PocoClientConfig) -> App {
        let (ui_action_sender, ui_action_receiver) = crossbeam_channel::unbounded();
        let (ui_command_sender, ui_command_receiver) = crossbeam_channel::unbounded();
        let config = Arc::new(config);

        App {
            ui: UI::new(
                ui_action_receiver.clone(),
                ui_command_sender.clone(),
                config.clone(),
            ),
            ui_channel: (ui_action_sender, ui_action_receiver),
            backend_channel: (ui_command_sender, ui_command_receiver),
            join_handles: Vec::new(),
            config,
        }
    }

    pub fn run(mut self, mode: AppRunningMode) -> anyhow::Result<()> {
        let backend = Backend::new(
            mode,
            self.config.clone(),
            self.backend_channel.1.clone(),
            self.ui_channel.0.clone(),
        );

        backend.run_backend_thread();

        let result = if mode != AppRunningMode::UI {
            let command = std::env::args()
                .skip_while(|arg| arg != "--")
                .skip(1)
                .reduce(|a, b| a + " " + &b)
                .unwrap_or("help".to_string());

            let command_source = CommandSource {
                source: command,
                id: "#1".to_string(),
            };

            self.backend_channel.0.send(command_source).unwrap();

            self.run_simple_ui_action_loop()
        } else {
            tracing::event!(
                Level::INFO,
                message = "start initializing terminal ui",
                category = format!("{:?}", TracingCategory::Agent)
            );

            self.ui.run_ui()
        };

        for handle in self.join_handles.drain(..) {
            handle.join().unwrap();
        }

        result
    }

    fn run_simple_ui_action_loop(&mut self) -> anyhow::Result<()> {
        loop {
            match self.ui_channel.1.recv() {
                Ok(event) => match event.1 {
                    UIAction::LogCommand(command_source) => {
                        println!("{} {}", &command_source.id, &command_source.source);
                    }
                    UIAction::LogString(string) => {
                        println!("{string}");
                    }
                    UIAction::LogMultipleStrings(strings) => {
                        for string in strings {
                            println!("{string}");
                        }
                    }
                    UIAction::LogTracingEvent(event) => {
                        println!("{event}");
                    }
                    UIAction::Panic(error) => {
                        println!("{error:?}");
                        return Err(anyhow::anyhow!(error));
                    }
                    UIAction::LogCommandExecution(command_source, stage, status, error) => {
                        match status {
                            CommandExecutionStatus::Succeed => {
                                println!("Command {} executed successfully", command_source.id);
                            }
                            CommandExecutionStatus::Failed => {
                                println!("Command {} failed(Stage: {stage})", command_source.id);
                                println!("Error: {error:?}");
                            }
                        }

                        return Ok(());
                    }
                    UIAction::QuitApp => {
                        return Ok(());
                    }
                },
                Err(error) => {
                    println!("error: {error}");

                    return Err(error.into());
                }
            }
        }
    }

    pub fn get_tracing_layer(&self) -> UITracingLayer {
        UITracingLayer::new(self.ui_channel.0.clone())
    }
}

pub mod backend;
pub mod trace;
pub mod ui;

use std::{io, sync::Arc, thread::JoinHandle};
use std::error::Error;
use futures::future::err;

use tracing::Level;

use crate::app::backend::command::CommandSource;
use crate::app::ui::action::UIAction;
use crate::{app::trace::TracingCategory, config::PocoAgentConfig};

use self::{
    backend::Backend,
    trace::UITracingLayer,
    ui::{action::UIActionEvent, UI},
};

type CommandString = String;

pub struct App {
    config: Arc<PocoAgentConfig>,
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
    pub fn new(config: PocoAgentConfig) -> App {
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

    pub fn run(mut self, direct_command_flag: bool) -> Result<(), Box<dyn Error>> {
        tracing::event!(
            Level::INFO,
            message = "start initializing terminal ui",
            category = format!("{:?}", TracingCategory::Agent)
        );

        let backend = Backend::new(
            self.config.clone(),
            self.backend_channel.1.clone(),
            self.ui_channel.0.clone(),
        );

        backend.run_backend_thread();

        if direct_command_flag {
            let command = std::env::args()
                .skip(1)
                .reduce(|a, b| a + " " + &b)
                .unwrap();
            let command_source = CommandSource {
                source: command,
                id: "#1".to_string(),
            };

            self.backend_channel.0.send(command_source).unwrap();

            loop {
                match self.ui_channel.1.recv() {
                    Ok(event) => {
                        match event.1 {
                        UIAction::LogCommand(command_id, command) => {
                            println!("{} {}", command_id, command);
                        }
                        UIAction::LogString(string) => {
                            println!("{}", string);
                        }
                        UIAction::LogMultipleString(strings) => {
                            for string in strings {
                                println!("{}", string);
                            }
                        }
                        UIAction::LogTracingEvent(event) => {
                            println!("{}", event);
                        }
                        UIAction::Panic(string) => {
                            println!("{}", string);
                            return Ok(());
                        }
                        UIAction::CommandExecutionDone(command_id, stage) => {
                            println!("{} done", command_id);
                            return Ok(());
                        }
                        UIAction::QuitApp => {
                            return Ok(());
                        }
                    }
                    }
                    Err(error) => {
                        println!("error: {}", error);
                        return Err(Box::new(error));
                    }
                }
            }
        } else {
            self.ui.run_ui()?;
        }

        for handle in self.join_handles.drain(..) {
            handle.join().unwrap();
        }

        Ok(())
    }

    pub fn get_tracing_layer(&self) -> UITracingLayer {
        UITracingLayer::new(self.ui_channel.0.clone())
    }
}

pub mod backend;
pub mod trace;
pub mod ui;

use std::{io, sync::Arc, thread::JoinHandle};

use tracing::Level;

use crate::{app::trace::TracingCategory, config::PocoAgentConfig};
use crate::app::ui::action::UIAction;

use self::{
    backend::Backend,
    trace::UITracingLayer,
    ui::{action::UIActionEvent, UI},
};

type CommandString = String;

pub struct App {
    ui: UI,
    ui_channel: (
        crossbeam_channel::Sender<UIActionEvent>,
        crossbeam_channel::Receiver<UIActionEvent>,
    ),
    backend_channel: (
        crossbeam_channel::Sender<CommandString>,
        crossbeam_channel::Receiver<CommandString>,
    ),
    join_handles: Vec<JoinHandle<()>>,
}

impl App {
    pub fn new() -> App {
        let (ui_action_sender, ui_action_receiver) = crossbeam_channel::unbounded();
        let (ui_command_sender, ui_command_receiver) = crossbeam_channel::unbounded();
        App {
            ui: UI::new(ui_action_receiver.clone(), ui_command_sender.clone()),
            ui_channel: (ui_action_sender, ui_action_receiver),
            backend_channel: (ui_command_sender, ui_command_receiver),
            join_handles: Vec::new(),
        }
    }

    fn setup_panic_hook(&self) {
        let old_hook = std::panic::take_hook();
        let old_hook = Box::leak(old_hook);
        let sender = self.ui_channel.0.clone();

        std::panic::set_hook(Box::new(move |panic_info| {
            sender
                .send(UIAction::Panic(panic_info.to_string()).into())
                .unwrap();

            old_hook.call((panic_info,));
        }));
    }

    pub fn run(&mut self, config: Arc<PocoAgentConfig>) -> Result<(), io::Error> {
        self.setup_panic_hook();

        tracing::event!(
            Level::INFO,
            message = "start connecting to near node",
            category = format!("{:?}", TracingCategory::Agent)
        );

        tracing::event!(
            Level::INFO,
            message = "finish connecting to near node",
            category = format!("{:?}", TracingCategory::Agent)
        );

        tracing::event!(
            Level::INFO,
            message = "start initializing terminal ui",
            category = format!("{:?}", TracingCategory::Agent)
        );

        let backend = Backend::new(
            config,
            self.backend_channel.1.clone(),
            self.ui_channel.0.clone(),
        );

        backend.run_backend_thread();

        self.ui.run_ui()
    }

    pub fn join(&mut self) {
        for handle in self.join_handles.drain(..) {
            handle.join().unwrap();
        }
    }

    pub fn get_tracing_layer(&self) -> UITracingLayer {
        UITracingLayer::new(self.ui_channel.0.clone())
    }
}

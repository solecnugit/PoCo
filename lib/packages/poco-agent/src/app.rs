pub mod backend;
pub mod trace;
pub mod ui;

use std::{io, thread::JoinHandle};

use tracing::Level;

use crate::app::trace::TracingCategory;

use self::{
    backend::Backend,
    trace::UITracingLayer,
    ui::{action::UIAction, UI},
};

pub struct App {
    ui: UI,
    backend: Backend,
    ui_channel: (
        crossbeam_channel::Sender<UIAction>,
        crossbeam_channel::Receiver<UIAction>,
    ),
    backend_channel: (
        crossbeam_channel::Sender<String>,
        crossbeam_channel::Receiver<String>,
    ),
    join_handles: Vec<JoinHandle<()>>,
}

impl App {
    pub fn new() -> App {
        let (ui_action_sender, ui_action_receiver) = crossbeam_channel::unbounded();
        let (ui_command_sender, ui_command_receiver) = crossbeam_channel::unbounded();
        App {
            ui: UI::new(ui_action_receiver.clone(), ui_command_sender.clone()),
            backend: Backend::new(ui_command_receiver.clone(), ui_action_sender.clone()),

            ui_channel: (ui_action_sender, ui_action_receiver),
            backend_channel: (ui_command_sender, ui_command_receiver),
            join_handles: Vec::new(),
        }
    }

    pub fn run(&mut self, _rpc_endpoint: String) -> Result<(), io::Error> {
        tracing::event!(
            Level::INFO,
            message = "start connecting to near node",
            category = format!("{:?}", TracingCategory::Agent)
        );

        // RefCell::borrow_mut(&self.agent).connect(rpc_endpoint);

        tracing::event!(
            Level::INFO,
            message = "finish connecting to near node",
            category = format!("{:?}", TracingCategory::Agent)
        );

        // self.agent.get_runtime().spawn(async {
        //     self.agent.gas_price();
        // });

        tracing::event!(
            Level::INFO,
            message = "start initializing terminal ui",
            category = format!("{:?}", TracingCategory::Agent)
        );

        self.backend.run_backend();
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

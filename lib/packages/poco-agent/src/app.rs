pub mod agent;
pub mod trace;
pub mod command;
pub mod ui;

use std::{io, thread::JoinHandle, sync::Arc, cell::RefCell};

use tracing::Level;

use crate::app::trace::TracingCategory;

use self::{
    agent::agent::PocoAgent,
    trace::UITracingLayer,
    ui::{action::UIAction, UI},
};

pub struct App {
    ui: UI,
    agent: Arc<RefCell<PocoAgent>>,

    join_handles: Vec<JoinHandle<()>>,
}

impl App {
    pub fn new() -> App {
        App {
            ui: UI::new(),
            agent: Arc::new(RefCell::new(PocoAgent::new())),
            join_handles: Vec::new(),
        }
    }

    pub fn create_ui_command_sender(&self) -> crossbeam_channel::Sender<UIAction> {
        self.ui.create_sender()
    }

    pub fn run(&mut self, rpc_endpoint: String) -> Result<(), io::Error> {
        tracing::event!(
            Level::INFO,
            message = "start connecting to near node",
            category = format!("{:?}", TracingCategory::Agent)
        );

        RefCell::borrow_mut(&self.agent).connect(rpc_endpoint);

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

        self.ui.run_ui()
    }

    pub fn join(&mut self) {
        for handle in self.join_handles.drain(..) {
            handle.join().unwrap();
        }
    }

    pub fn get_tracing_layer(&self) -> UITracingLayer {
        UITracingLayer::new(self.ui.create_sender())
    }
}

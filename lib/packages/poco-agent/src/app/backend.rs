use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task;

use anyhow::Error;
use clap::error::ErrorKind;
use futures::FutureExt;
use tracing::Level;

use poco_agent::agent::PocoAgent;
use poco_db::PocoDB;
use poco_ipfs::client::IpfsClient;

use crate::app::backend::command::CommandSource;
use crate::app::backend::executor::CommandExecutor;
use crate::app::backend::parser::CommandParser;
use crate::app::trace::{TracingCategory, TracingEvent};
use crate::app::ui::event::{
    CommandExecutionStage, CommandExecutionStatus, UIAction, UIActionEvent, UIActionSender,
};
use crate::app::ui::util::log_command_execution;
use crate::config::{AppRunningMode, PocoClientConfig};

pub mod command;

pub(crate) mod event;
pub(crate) mod executor;
pub(crate) mod microtask;
pub(crate) mod parser;
pub(crate) mod ui;

#[derive(Clone)]
pub struct InnerBackend {
    running_mode: AppRunningMode,
    config: Arc<PocoClientConfig>,
    ui_receiver: crossbeam_channel::Receiver<CommandSource>,
    ui_sender: crossbeam_channel::Sender<UIActionEvent>,
    runtime: Arc<tokio::runtime::Runtime>,
    db: PocoDB,
    agent: Arc<PocoAgent>,
    ipfs_client: Arc<IpfsClient>,
}

impl InnerBackend {
    fn new(
        running_mode: AppRunningMode,
        config: Arc<PocoClientConfig>,
        ui_receiver: crossbeam_channel::Receiver<CommandSource>,
        ui_sender: crossbeam_channel::Sender<UIActionEvent>,
        runtime: Arc<tokio::runtime::Runtime>,
        db: PocoDB,
        agent: Arc<PocoAgent>,
        ipfs_client: Arc<IpfsClient>,
    ) -> Self {
        Self {
            running_mode,
            config,
            ui_receiver,
            ui_sender,
            runtime,
            db,
            agent,
            ipfs_client,
        }
    }
}

#[derive(Clone)]
pub struct Backend {
    inner: Arc<InnerBackend>,
}

impl Deref for Backend {
    type Target = InnerBackend;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Backend {
    pub fn new(
        running_mode: AppRunningMode,
        config: Arc<PocoClientConfig>,
        ui_receiver: crossbeam_channel::Receiver<CommandSource>,
        ui_sender: crossbeam_channel::Sender<UIActionEvent>,
    ) -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime"),
        );

        let db = PocoDB::new(Arc::new(config.db.clone())).expect("Failed to initialize database");

        let ipfs_client = Arc::new(
            IpfsClient::create_ipfs_client(Arc::new(config.ipfs.clone()))
                .expect("Failed to create ipfs client"),
        );

        let agent = Arc::new(
            PocoAgent::build(Arc::new(config.agent.clone()))
                .expect("Failed to initialize Poco Agent"),
        );

        let inner = Arc::new(InnerBackend::new(
            running_mode,
            config,
            ui_receiver,
            ui_sender,
            runtime,
            db,
            agent,
            ipfs_client,
        ));

        Backend { inner }
    }

    fn execute_command_block<F, R, I>(&self, command_source: CommandSource, f: F)
        where
            F: FnOnce(Backend) -> R,
            R: Future<Output=anyhow::Result<I>> + Send + 'static,
            I: Send + 'static,
    {
        let backend = self.clone();
        let sender = self.ui_sender.clone();

        self.runtime.spawn(f(backend).then(async move |r| {
            match r {
                Ok(_) => {
                    log_command_execution(
                        &sender,
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Succeed,
                        None,
                    );
                }
                Err(e) => {
                    tracing::error!(
                        message = "failed to execute command",
                        command = format!("{command_source:?}"),
                        category = format!("{:?}", TracingCategory::Backend),
                        error = format!("{e:?}")
                    );

                    log_command_execution(
                        &sender,
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Failed,
                        Some(Box::new(e)),
                    );
                }
            };
        }));
    }

    fn start_backend_microtasks(&self) {
        {
            // event microtask

            // let config = self.config.clone();
            // let ui_sender = self.ui_sender.clone();
            // let agent = self.agent.clone();
            // let db = self.db.clone();
            // let runtime = self.runtime.clone();

            self.runtime.spawn(microtask::event_microtask(self.clone()));
        }
    }

    pub async fn run_backend_thread(self) -> task::JoinHandle<()> {
    // pub async fn run_backend_thread(self) -> std::thread::JoinHandle<()> {

        tracing::event!(
            Level::INFO,
            message = "start initializing backend thread",
            category = format!("{:?}", TracingCategory::Agent)
        );

        let handle = task::spawn(async move {
            'outer: loop {
                if self.running_mode != AppRunningMode::DIRECT {
                    self.start_backend_microtasks();
                }
    
                loop {
                    match self.ui_receiver.recv() {
                        Ok(command_source) => {
                            match self.parse_command(command_source.source.trim()) {
                                Ok(command) => {
                                    // self.log_string(format!("Received command: {}", command_source.source.trim()));
                                    self.dispatch_command(command_source, command).await
                                },
                                Err(error) => {
                                    match &error.kind() {
                                        ErrorKind::DisplayHelp => {}
                                        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {}
                                        ErrorKind::DisplayVersion => {}
                                        _ => tracing::error!(error = ?error, "Failed to parse command"),
                                    }
    
                                    log_command_execution(
                                        &self.ui_sender,
                                        command_source,
                                        CommandExecutionStage::Parsing,
                                        CommandExecutionStatus::Failed,
                                        Some(Box::new(anyhow::anyhow!(error))),
                                    );
                                }
                            }
                        }
                        Err(_) => {
                            tracing::event!(
                                Level::ERROR,
                                message = "backend channel disconnected",
                                category = format!("{:?}", TracingCategory::Backend)
                            );
    
                            break 'outer;
                        }
                    }
                }
            }
        });
    
        handle


        // 创建线程构建器
        // let builder = std::thread::Builder::new().name("backend".to_string());

        // builder
        //     .spawn(move || 'outer: loop {
        //         if self.running_mode != AppRunningMode::DIRECT {
        //             self.start_backend_microtasks();
        //         }

        //         loop {
        //             match self.ui_receiver.recv() {
        //                 Ok(command_source) => {
        //                     match self.parse_command(command_source.source.trim()) {
        //                         Ok(command) => self.dispatch_command(command_source, command),
        //                         Err(error) => {
        //                             match &error.kind() {
        //                                 ErrorKind::DisplayHelp => {}
        //                                 ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {}
        //                                 ErrorKind::DisplayVersion => {}
        //                                 _ => tracing::error!(error = ?error, "Failed to parse command"),
        //                             }

        //                             log_command_execution(
        //                                 &self.ui_sender,
        //                                 command_source,
        //                                 CommandExecutionStage::Parsing,
        //                                 CommandExecutionStatus::Failed,
        //                                 Some(Box::new(anyhow::anyhow!(error))),
        //                             );
        //                         }
        //                     }
        //                 }
        //                 Err(_) => {
        //                     tracing::event!(
        //                         Level::ERROR,
        //                         message = "backend channel disconnected",
        //                         category = format!("{:?}", TracingCategory::Backend)
        //                     );

        //                     break 'outer;
        //                 }
        //             }
        //         }
        //     })
        //     .unwrap()
    }
}
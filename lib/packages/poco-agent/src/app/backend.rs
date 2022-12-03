use std::sync::Arc;

use crate::agent::agent::PocoAgent;
use crate::app::backend::command::{
    BackendCommand::{
        GasPriceCommand, HelpCommand, NetworkStatusCommand, StatusCommand, ViewAccountCommand,
    },
    ParseBackendCommandError::{MissingCommandParameter, UnknownCommand},
};
use crate::app::trace::TracingCategory;
use crate::config::PocoAgentConfig;
use crossbeam_channel::{RecvError, TryRecvError};
use near_primitives::hash::CryptoHash;
use near_primitives::types::EpochReference::EpochId;
use thread_local::ThreadLocal;
use tracing::Level;

use self::command::{get_internal_command, BackendCommand, ParseBackendCommandError};

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
            BackendCommand::NetworkStatusCommand => {
                let sender = self.sender.clone();
                let agent = self.agent.clone();
                let config = self.config.clone();

                self.async_runtime.spawn(async move {
                    let agent =
                        agent.get_or(|| PocoAgent::connect(config.near.rpc_endpoint.as_str()));
                    let network_status = agent.network_status().await;

                    if let Ok(network_status) = network_status {
                        sender
                            .send(
                                UIAction::LogMultipleString(vec![
                                    format!(
                                        "Num Active Peers: {}",
                                        network_status.num_active_peers
                                    ),
                                    format!(
                                        "Sent Bytes Per Sec: {}",
                                        network_status.sent_bytes_per_sec
                                    ),
                                    format!(
                                        "Received Bytes Per Sec: {}",
                                        network_status.received_bytes_per_sec
                                    ),
                                ])
                                .into(),
                            )
                            .unwrap();
                    } else {
                        sender
                            .send(
                                UIAction::LogString("Failed to get network status".to_string())
                                    .into(),
                            )
                            .unwrap();
                    }
                });
            }
            BackendCommand::StatusCommand => {
                let sender = self.sender.clone();
                let agent = self.agent.clone();
                let config = self.config.clone();

                self.async_runtime.spawn(async move {
                    let agent =
                        agent.get_or(|| PocoAgent::connect(config.near.rpc_endpoint.as_str()));
                    let status = agent.status().await;

                    if let Ok(status) = status {
                        sender
                            .send(
                                UIAction::LogMultipleString(vec![
                                    format!("Version: {}", status.version.version),
                                    format!("Build: {}", status.version.build),
                                    format!("Protocol Version: {}", status.protocol_version),
                                    format!(
                                        "Latest Protocol Version: {}",
                                        status.latest_protocol_version
                                    ),
                                    format!("Rpc Address: {}", status.rpc_addr.unwrap_or_default()),
                                    format!("Sync Info: "),
                                    format!(
                                        "  Latest Block Hash: {}",
                                        status.sync_info.latest_block_hash
                                    ),
                                    format!(
                                        "  Latest Block Height: {}",
                                        status.sync_info.latest_block_height
                                    ),
                                    format!(
                                        "  Latest State Root: {}",
                                        status.sync_info.latest_state_root
                                    ),
                                    format!(
                                        "  Latest Block Time: {}",
                                        status.sync_info.latest_block_time
                                    ),
                                    format!("  Syncing: {}", status.sync_info.syncing),
                                    format!("Uptime Sec: {}", status.uptime_sec),
                                ])
                                .into(),
                            )
                            .unwrap();
                    } else {
                        sender
                            .send(UIAction::LogString("Failed to get status".to_string()).into())
                            .unwrap();
                    }
                });
            }
            BackendCommand::ViewAccountCommand(account_id) => {
                let sender = self.sender.clone();
                let agent = self.agent.clone();
                let config = self.config.clone();

                self.async_runtime.spawn(async move {
                    let agent =
                        agent.get_or(|| PocoAgent::connect(config.near.rpc_endpoint.as_str()));
                    if let Ok(account) = account_id.parse() {
                        let account = agent.view_account(account).await;

                        if let Ok(account) = account {
                            sender
                                .send(
                                    UIAction::LogMultipleString(vec![
                                        format!("Account ID: {}", account_id),
                                        format!("Amount: {}", account.amount),
                                        format!("Locked: {}", account.locked),
                                        format!("Code Hash: {}", account.code_hash),
                                        format!("Storage Usage: {}", account.storage_usage),
                                        format!("Storage Paid At: {}", account.storage_paid_at),
                                    ])
                                    .into(),
                                )
                                .unwrap();
                        } else {
                            sender
                                .send(
                                    UIAction::LogString("Failed to get account".to_string()).into(),
                                )
                                .unwrap();
                        }
                    } else {
                        sender
                            .send(UIAction::LogString("Invalid account ID".to_string()).into())
                            .unwrap();
                    }
                });
            }
        }
    }

    fn parse_command(&self, command: &str) -> Result<BackendCommand, ParseBackendCommandError> {
        let arg_matches = get_internal_command().get_matches_from(command.split_whitespace());

        match arg_matches.subcommand() {
            Some(("help", args)) => {
                if let Some(command) = args.get_one::<String>("command") {
                    Ok(HelpCommand(
                        get_internal_command()
                            .get_subcommands_mut()
                            .find(|subcommand| subcommand.get_name() == command)
                            .unwrap()
                            .render_help()
                            .to_string()
                            .lines()
                            .map(|e| e.to_string())
                            .collect(),
                    ))
                } else {
                    Ok(HelpCommand(
                        get_internal_command()
                            .render_help()
                            .to_string()
                            .lines()
                            .map(|e| e.to_string())
                            .collect(),
                    ))
                }
            }
            Some(("gas-price", _)) => Ok(GasPriceCommand),
            Some(("network-status", _)) => Ok(NetworkStatusCommand),
            Some(("status", _)) => Ok(StatusCommand),
            Some(("view-account", args)) => {
                if let Some(account) = args.get_one::<String>("account-id") {
                    Ok(ViewAccountCommand(account.to_string()))
                } else {
                    Err(MissingCommandParameter("account-id".to_string()))
                }
            }
            _ => Err(UnknownCommand(command.to_string())),
        }
    }

    pub fn run_backend_thread(mut self) -> std::thread::JoinHandle<()> {
        let builder = std::thread::Builder::new().name("backend".to_string());

        builder
            .spawn(move || 'outer: loop {
                loop {
                    match self.receiver.recv() {
                        Ok(command) => match self.parse_command(command.trim()) {
                            Ok(command) => self.execute_command(command),
                            Err(error) => match error {
                                ParseBackendCommandError::UnknownCommand(command) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message = format!("unknown command: {}", command),
                                        category = format!("{:?}", TracingCategory::Agent)
                                    );
                                }
                                ParseBackendCommandError::MissingCommandParameter(parameter) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message =
                                            format!("missing command parameter: {}", parameter),
                                        category = format!("{:?}", TracingCategory::Agent)
                                    );
                                }
                            },
                        },
                        Err(error) => {
                            tracing::event!(
                                Level::ERROR,
                                message = "backend channel disconnected",
                                category = format!("{:?}", TracingCategory::Agent)
                            );

                            break 'outer;
                        }
                    }
                }
            })
            .unwrap()
    }
}

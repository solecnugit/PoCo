use std::future::Future;
use std::sync::Arc;

use thread_local::ThreadLocal;
use tracing::Level;

use crate::agent::agent::PocoAgent;
use crate::app::backend::command::BackendCommand::RoundStatusCommand;
use crate::app::backend::command::{
    BackendCommand::{
        GasPriceCommand, HelpCommand, NetworkStatusCommand, StatusCommand, ViewAccountCommand,
    },
    ParseBackendCommandError::{MissingCommandParameter, UnknownCommand},
};
use crate::app::trace::TracingCategory;
use crate::config::PocoAgentConfig;

use super::ui::action::{UIAction, UIActionEvent};

use self::command::{get_internal_command, BackendCommand, ParseBackendCommandError};

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

    fn execute_command_block<F, R>(&mut self, f: F)
    where
        F: FnOnce(
            crossbeam_channel::Sender<UIActionEvent>,
            Arc<ThreadLocal<PocoAgent>>,
            Arc<PocoAgentConfig>,
        ) -> R,
        R: Future<Output = ()> + Send + 'static,
    {
        let sender = self.sender.clone();
        let agent = self.agent.clone();
        let config = self.config.clone();

        self.async_runtime.spawn(f(sender, agent, config));
    }

    fn execute_command(&mut self, command: BackendCommand) {
        match command {
            HelpCommand(help) => {
                self.sender
                    .send(UIAction::LogMultipleString(help).into())
                    .unwrap();
            }
            GasPriceCommand => self.execute_command_block(async move |sender, agent, config| {
                let agent = agent.get_or(|| PocoAgent::new(config.clone()));
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
            }),
            NetworkStatusCommand => {
                self.execute_command_block(async move |sender, agent, config| {
                    let agent = agent.get_or(|| PocoAgent::new(config));
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
                })
            }
            StatusCommand => self.execute_command_block(async move |sender, agent, config| {
                let agent = agent.get_or(|| PocoAgent::new(config));
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
            }),
            ViewAccountCommand(account_id) => {
                self.execute_command_block(async move |sender, agent, config| {
                    let agent = agent.get_or(|| PocoAgent::new(config));
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
                })
            }
            RoundStatusCommand => self.execute_command_block(async move |sender, agent, config| {
                let agent = agent.get_or(|| PocoAgent::new(config));

                let round_status = agent.get_round_status().await;

                if let Ok(round_status) = round_status {
                    sender
                        .send(UIAction::LogString(format!("Round Status: {}", round_status)).into())
                        .unwrap();
                } else {
                    sender
                        .send(UIAction::LogString("Failed to get round status".to_string()).into())
                        .unwrap();
                }
            }),
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
            Some(("round-status", _)) => Ok(RoundStatusCommand),
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
                                UnknownCommand(command) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message = format!("unknown command: {}", command),
                                        category = format!("{:?}", TracingCategory::Agent)
                                    );
                                }
                                MissingCommandParameter(parameter) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message =
                                            format!("missing command parameter: {}", parameter),
                                        category = format!("{:?}", TracingCategory::Agent)
                                    );
                                }
                            },
                        },
                        Err(_error) => {
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

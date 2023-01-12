use clap::error::ErrorKind;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use futures::lock::Mutex;
use futures::FutureExt;
use near_primitives::types::AccountId;
use poco_types::types::round::RoundStatus;
use tracing::Level;

use crate::agent::agent::PocoAgent;
use crate::agent::task::config::{RawTaskConfig, RawTaskInputSource};
use crate::app::backend::command::CommandSource;
use crate::app::backend::executor::CommandExecutor;
use crate::app::backend::parser::CommandParser;
use crate::app::trace::TracingCategory;
use crate::app::ui::action::{CommandExecutionStage, CommandExecutionStatus};
use crate::app::ui::util::log_command_execution;
use crate::config::PocoAgentConfig;
use crate::ipfs::client::IpfsClient;
use crate::util::{pretty_bytes, pretty_gas};

use super::ui::action::UIActionEvent;
use super::ui::util::{log_multiple_strings, log_string};

pub mod command;
pub(crate) mod executor;
pub(crate) mod parser;

pub struct Backend {
    config: Arc<PocoAgentConfig>,
    receiver: crossbeam_channel::Receiver<CommandSource>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    runtime: Box<tokio::runtime::Runtime>,
    db_connection: Arc<Mutex<rusqlite::Connection>>,
    agent: Arc<PocoAgent>,
    ipfs_client: Arc<IpfsClient>,
}

impl Backend {
    pub fn new(
        config: Arc<PocoAgentConfig>,
        receiver: crossbeam_channel::Receiver<CommandSource>,
        sender: crossbeam_channel::Sender<UIActionEvent>,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let db_connection = Arc::new(Mutex::new(
            rusqlite::Connection::open(&config.app.database_path)
                .expect("Failed to open database connection"),
        ));

        let ipfs_client = Arc::new(
            IpfsClient::create_ipfs_client(&config.ipfs.ipfs_endpoint)
                .expect("Failed to create ipfs client"),
        );

        Backend {
            receiver,
            sender,
            runtime: Box::new(runtime),
            db_connection,
            agent: Arc::new(PocoAgent::new(config.clone())),
            ipfs_client,
            config,
        }
    }

    fn execute_command_block<F, R, I>(&mut self, command_source: CommandSource, f: F)
    where
        F: FnOnce(
            crossbeam_channel::Sender<UIActionEvent>,
            Arc<PocoAgent>,
            Arc<IpfsClient>,
            Arc<PocoAgentConfig>,
        ) -> R,
        R: Future<Output = anyhow::Result<I>> + Send + 'static,
        I: Send + 'static,
    {
        let sender1 = self.sender.clone();
        let sender2 = self.sender.clone();

        let agent = self.agent.clone();
        let ipfs_client = self.ipfs_client.clone();
        let config = self.config.clone();

        self.runtime
            .spawn(f(sender1, agent, ipfs_client, config).then(async move |r| {
                if let Err(e) = r {
                    tracing::error!(
                        message = "failed to execute command",
                        command = format!("{command_source:?}"),
                        category = format!("{:?}", TracingCategory::Backend),
                        error = format!("{e:?}")
                    );

                    log_command_execution(
                        &sender2,
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Failed,
                        Some(Box::new(e)),
                    );
                } else {
                    log_command_execution(
                        &sender2,
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Succeed,
                        None,
                    );
                };
            }));
    }

    fn execute_publish_task_command(
        &mut self,
        command_source: CommandSource,
        task_config_path: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, ipfs_client, _config| {
                let task_config_path = Path::new(&task_config_path);

                if let Ok(true) = task_config_path.try_exists() {
                } else {
                    anyhow::bail!("Task config file does not exist");
                }

                let task_config = tokio::fs::read_to_string(task_config_path).await?;
                let task_config = serde_json::from_str::<RawTaskConfig>(&task_config)?;
                let task_config =
                    if let RawTaskInputSource::Ipfs { hash, file } = &task_config.input {
                        if hash.is_some() && file.is_some() {
                            anyhow::bail!(
                                "Both hash and file are specified in task config {}",
                                task_config_path.display()
                            );
                        }

                        if let Some(file) = file {
                            let file_path = Path::new(file.as_str());
                            let file_path = if file_path.is_absolute() {
                                file_path.to_path_buf()
                            } else {
                                task_config_path.parent().unwrap().join(file_path)
                            };

                            if let Ok(true) = file_path.try_exists() {
                                log_string(
                                    &sender,
                                    format!("Uploading file to ipfs: {}", file_path.display()),
                                );

                                let file_cid = ipfs_client.add_file(file_path.as_path()).await?;

                                log_string(&sender, format!("File uploaded to ipfs: {file_cid}"));

                                task_config.to_task_config(Some(file_cid))?
                            } else {
                                anyhow::bail!(
                                    "Task input file does not exist, {}",
                                    file_path.display()
                                );
                            }
                        } else {
                            task_config.to_task_config(None)?
                        }
                    } else {
                        task_config.to_task_config(None)?
                    };

                let round_status = agent.get_round_status().await?;

                if let RoundStatus::Pending = round_status {
                    anyhow::bail!("Round is not started yet. Please wait for the round to start.");
                }

                let (gas, task_id) = agent.publish_task(task_config).await?;

                log_string(
                    &sender,
                    format!(
                        "Task published. Gas used: {}, Task ID: {task_id}",
                        pretty_gas(gas)
                    ),
                );

                Ok(())
            },
        );
    }

    fn execute_ipfs_cat_file_command(&mut self, command_source: CommandSource, file_hash: String) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let buffer = ipfs_client.cat_file(file_hash.as_str()).await?;
                let buffer = String::from_utf8(buffer)?;

                log_multiple_strings(&sender, buffer.lines().map(|s| s.to_string()).collect());

                Ok(())
            },
        );
    }

    fn execute_ipfs_file_status_command(
        &mut self,
        command_source: CommandSource,
        file_hash: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let status = ipfs_client.file_status(file_hash.as_str()).await?;

                log_multiple_strings(
                    &sender,
                    vec![
                        format!("File hash: {}", status.hash),
                        format!("File size: {}", pretty_bytes(status.cumulative_size)),
                        format!("File block size: {}", pretty_bytes(status.block_size)),
                        format!("File links size: {}", pretty_bytes(status.links_size)),
                        format!("File data size: {}", pretty_bytes(status.data_size)),
                        format!("File num links: {}", status.num_links),
                    ],
                );

                Ok(())
            },
        );
    }

    fn execute_ipfs_get_file_command(
        &mut self,
        command_source: CommandSource,
        file_hash: String,
        file_path: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let mut interval = tokio::time::interval(Duration::from_millis(1000));
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);

                log_string(&sender, format!("Downloading file from ipfs: {file_hash}"));

                tokio::spawn(async move {
                    let mut last_progress = (0, 0);

                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                let (downloaded, total) = last_progress;

                                if downloaded != 0 {
                                    let percent = (downloaded as f64 / total as f64) * 100.0;
                                    let downloaded = pretty_bytes(downloaded);
                                    let total = pretty_bytes(total);

                                    log_string(&sender, format!(
                                        "Downloading file from ipfs: {downloaded}/{total}({percent:.2}%)"
                                    ));
                                }
                            }
                            progress = rx.recv() => {
                                if let Some(progress) = progress {
                                    last_progress = progress;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                });

                ipfs_client
                    .get_file(
                        file_hash.as_str(),
                        file_path.as_str(),
                        Some(tx),
                    )
                    .await?;

                Ok(())
            },
        );
    }

    fn execute_ipfs_add_file_command(&mut self, command_source: CommandSource, file_path: String) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let file_hash = ipfs_client.add_file(file_path.as_str()).await?;

                log_string(&sender, format!("File uploaded to ipfs: {file_hash}"));

                Ok(())
            },
        );
    }

    fn execute_start_new_round_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let round_status = agent.get_round_status().await?;

                let (gas, round_id) = if let RoundStatus::Pending = round_status {
                    agent.start_new_round().await?
                } else {
                    anyhow::bail!("Round is already started");
                };

                log_string(
                    &sender,
                    format!(
                        "New round started: {round_id}, gas used: {}",
                        pretty_gas(gas),
                    ),
                );

                Ok(())
            },
        );
    }

    fn execute_set_user_endpoint_command(
        &mut self,
        command_source: CommandSource,
        endpoint: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let gas = agent.set_user_endpoint(endpoint.as_str()).await?;

                log_string(
                    &sender,
                    format!(
                        "User endpoint set successfully. Gas used: {}",
                        pretty_gas(gas),
                    ),
                );

                Ok(())
            },
        )
    }

    fn execute_get_user_endpoint_command(
        &mut self,
        command_source: CommandSource,
        account_id: Option<AccountId>,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let endpoint = agent.get_user_endpoint(account_id).await?;

                if let Some(endpoint) = endpoint {
                    log_string(&sender, format!("User endpoint: {endpoint}"));
                } else {
                    log_string(&sender, "User endpoint is not set".to_string());
                }

                Ok(())
            },
        )
    }

    fn execute_query_events_command(
        &mut self,
        command_source: CommandSource,
        from: u32,
        count: u32,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let events = agent.query_events(from, count).await?;

                if events.is_empty() {
                    log_string(&sender, "No events found".to_string());
                } else {
                    for event in events {
                        log_string(&sender, format!("{event}"));
                    }
                }

                Ok(())
            },
        )
    }

    fn execute_count_events_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let count = agent.count_events().await?;

                log_string(&sender, format!("Events count: {count}"));

                Ok(())
            },
        )
    }

    fn execute_round_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let round_status = agent.get_round_status().await?;

                log_string(&sender, format!("Round status: {round_status}"));

                Ok(())
            },
        )
    }

    fn execute_view_account_command(
        &mut self,
        command_source: CommandSource,
        account_id: AccountId,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let account_id_in_string = account_id.to_string();

                let account = agent.view_account(account_id).await?;

                log_multiple_strings(
                    &sender,
                    vec![
                        format!("Account ID: {account_id_in_string}"),
                        format!("Amount: {}", account.amount),
                        format!("Locked: {}", account.locked),
                        format!("Code Hash: {}", account.code_hash),
                        format!("Storage Usage: {}", account.storage_usage),
                        format!("Storage Paid At: {}", account.storage_paid_at),
                    ],
                );

                Ok(())
            },
        )
    }

    fn execute_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let status = agent.status().await?;

                log_multiple_strings(
                    &sender,
                    vec![
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
                        format!("  Syncing: {}", status.sync_info.syncing),
                    ],
                );

                Ok(())
            },
        )
    }

    fn execute_network_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let status = agent.network_status().await?;

                log_multiple_strings(
                    &sender,
                    vec![
                        format!("Num Active Peers: {}", status.num_active_peers),
                        format!("Sent Bytes Per Sec: {}", status.sent_bytes_per_sec),
                        format!("Received Bytes Per Sec: {}", status.received_bytes_per_sec),
                    ],
                );

                Ok(())
            },
        );
    }

    fn execute_gas_price_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let gas_price = agent.gas_price().await?;

                log_string(&sender, format!("Gas price: {gas_price}"));

                Ok(())
            },
        )
    }

    pub fn run_backend_thread(mut self) -> std::thread::JoinHandle<()> {
        let builder = std::thread::Builder::new().name("backend".to_string());

        builder
            .spawn(move || 'outer: loop {
                loop {
                    match self.receiver.recv() {
                        Ok(command_source) => {
                            match self.parse_command(command_source.source.trim()) {
                                Ok(command) => self.execute_command(command_source, command),
                                Err(error) => {
                                    match &error.kind() {
                                        ErrorKind::DisplayHelp => {}
                                        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {}
                                        ErrorKind::DisplayVersion => {}
                                        _ => tracing::error!(error = ?error, "Failed to parse command"),
                                    }

                                    log_command_execution(
                                        &self.sender,
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
            })
            .unwrap()
    }
}

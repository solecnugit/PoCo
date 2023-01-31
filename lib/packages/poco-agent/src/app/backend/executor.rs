use std::path::Path;

use poco_types::types::round::RoundStatus;

use poco_actuator::config::{RawTaskConfigFile, RawTaskInputSource};
use poco_actuator::get_actuator;
use poco_agent::types::AccountId;
use poco_ipfs::client::GetFileProgress;

use crate::app::backend::Backend;
use crate::app::backend::command::{BackendCommand, CommandSource};
use crate::app::backend::command::BackendCommand::{
    CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand, IpfsAddFileCommand,
    IpfsCatFileCommand, IpfsFileStatusCommand, IpfsGetFileCommand, NetworkStatusCommand,
    PublishTaskCommand, QueryEventsCommand, RoundInfoCommand, RoundStatusCommand,
    SetUserEndpointCommand, StartRoundCommand, StatusCommand, ViewAccountCommand,
};
use crate::app::ui::event::{CommandExecutionStage, CommandExecutionStatus};
use crate::app::ui::util::{log_command_execution, log_multiple_strings, log_string};
use crate::util::{pretty_bytes, pretty_gas};

pub trait CommandExecutor {
    fn dispatch_command(&mut self, command_source: CommandSource, command: BackendCommand) {
        match command {
            HelpCommand(help) => self.execute_help_command(command_source, help),
            GasPriceCommand => self.execute_gas_price_command(command_source),
            NetworkStatusCommand => self.execute_network_status_command(command_source),
            StatusCommand => self.execute_status_command(command_source),
            ViewAccountCommand { account_id } => {
                self.execute_view_account_command(command_source, account_id)
            }
            RoundStatusCommand => self.execute_round_status_command(command_source),
            RoundInfoCommand => self.execute_round_info_command(command_source),
            CountEventsCommand => self.execute_count_events_command(command_source),
            QueryEventsCommand { from, count } => {
                self.execute_query_events_command(command_source, from, count)
            }
            GetUserEndpointCommand { account_id } => {
                self.execute_get_user_endpoint_command(command_source, account_id)
            }
            SetUserEndpointCommand { endpoint } => {
                self.execute_set_user_endpoint_command(command_source, endpoint)
            }
            IpfsAddFileCommand { file_path } => {
                self.execute_ipfs_add_file_command(command_source, file_path)
            }
            IpfsCatFileCommand { file_hash } => {
                self.execute_ipfs_cat_file_command(command_source, file_hash)
            }
            IpfsGetFileCommand {
                file_hash,
                file_path,
            } => self.execute_ipfs_get_file_command(command_source, file_hash, file_path),
            IpfsFileStatusCommand { file_hash } => {
                self.execute_ipfs_file_status_command(command_source, file_hash)
            }
            StartRoundCommand => self.execute_start_round_command(command_source),
            PublishTaskCommand { task_config_path } => {
                self.execute_publish_task_command(command_source, task_config_path)
            }
        }
    }

    fn execute_publish_task_command(
        &mut self,
        command_source: CommandSource,
        task_config_path: String,
    );
    fn execute_ipfs_cat_file_command(&mut self, command_source: CommandSource, file_hash: String);
    fn execute_ipfs_file_status_command(
        &mut self,
        command_source: CommandSource,
        file_hash: String,
    );
    fn execute_ipfs_get_file_command(
        &mut self,
        command_source: CommandSource,
        file_hash: String,
        file_path: String,
    );
    fn execute_ipfs_add_file_command(&mut self, command_source: CommandSource, file_path: String);
    fn execute_start_round_command(&mut self, command_source: CommandSource);
    fn execute_set_user_endpoint_command(
        &mut self,
        command_source: CommandSource,
        endpoint: String,
    );
    fn execute_gas_price_command(&mut self, command_source: CommandSource);
    fn execute_get_user_endpoint_command(
        &mut self,
        command_source: CommandSource,
        account_id: Option<AccountId>,
    );
    fn execute_query_events_command(
        &mut self,
        command_source: CommandSource,
        from: u32,
        count: u32,
    );
    fn execute_count_events_command(&mut self, command_source: CommandSource);
    fn execute_round_info_command(&mut self, command_source: CommandSource);
    fn execute_round_status_command(&mut self, command_source: CommandSource);
    fn execute_view_account_command(
        &mut self,
        command_source: CommandSource,
        account_id: AccountId,
    );
    fn execute_status_command(&mut self, command_source: CommandSource);
    fn execute_network_status_command(&mut self, command_source: CommandSource);
    fn execute_help_command(&mut self, command_source: CommandSource, help: Vec<String>);
}

impl CommandExecutor for Backend {
    fn execute_publish_task_command(
        &mut self,
        command_source: CommandSource,
        task_config_path: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, ipfs_client, _config| {
                let task_config_path = Path::new(&task_config_path);

                // Check if task config file exists
                match task_config_path.try_exists() {
                    Ok(true) => {}
                    Ok(false) => anyhow::bail!("Task config file does not exist"),
                    Err(e) => anyhow::bail!("Failed to check if task config file exists: {}", e),
                }

                // Read task config file
                let task_config = tokio::fs::read_to_string(task_config_path).await?;
                let task_config = serde_json::from_str::<RawTaskConfigFile>(&task_config)?;

                let actuator = if let Some(actuator) = get_actuator(&task_config.r#type) {
                    actuator
                } else {
                    anyhow::bail!("Unsupported task type: {}", task_config.r#type);
                };

                // Encode task config
                let task_config = match &task_config.input {
                    RawTaskInputSource::Ipfs { hash, file } => match (hash, file) {
                        (None, None) => unreachable!(),
                        (Some(_), Some(_)) => {
                            anyhow::bail!(
                                "Both hash and file are specified in task config {}",
                                task_config_path.display()
                            );
                        }
                        (Some(_hash), None) => task_config.build_task_config(None, &actuator)?,
                        (None, Some(file)) => {
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

                                task_config.build_task_config(Some(file_cid), &actuator)?
                            } else {
                                anyhow::bail!(
                                    "Task input file does not exist, {}",
                                    file_path.display()
                                );
                            }
                        }
                    },
                    RawTaskInputSource::Link { .. } => {
                        task_config.build_task_config(None, &actuator)?
                    }
                };

                // Check if round is started
                let round_status = agent.get_round_status().await?;

                if let RoundStatus::Pending = round_status {
                    anyhow::bail!("Round is not started yet. Please wait for the round to start.");
                }

                // Publish task
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
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);

                log_string(&sender, format!("Downloading file from ipfs: {file_hash}"));

                tokio::spawn(async move {
                    let mut last_progress: GetFileProgress = Default::default();

                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                let downloaded = last_progress.downloaded_size_in_bytes();
                                let total = last_progress.total_size_in_bytes();

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

    fn execute_start_round_command(&mut self, command_source: CommandSource) {
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

    fn execute_round_info_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let round_info = agent.get_round_info().await?;

                log_string(&sender, format!("Round info: {round_info}"));

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

    fn execute_help_command(&mut self, command_source: CommandSource, help: Vec<String>) {
        log_multiple_strings(&self.ui_sender, help);
        log_command_execution(
            &self.ui_sender,
            command_source,
            CommandExecutionStage::Executed,
            CommandExecutionStatus::Succeed,
            None,
        );
    }
}

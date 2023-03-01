use std::path::Path;

use poco_types::types::round::RoundStatus;

// use poco_actuator::config::{RawTaskConfigFile, RawTaskInputSource};
// use poco_actuator::get_actuator;
use poco_agent::types::AccountId;
use poco_ipfs::client::GetFileProgress;

use crate::app::backend::command::BackendCommand::{
    CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand, IpfsAddFileCommand,
    IpfsCatFileCommand, IpfsFileStatusCommand, IpfsGetFileCommand, NetworkStatusCommand,
    PublishTaskCommand, QueryEventsCommand, RoundInfoCommand, RoundStatusCommand,
    SetUserEndpointCommand, StartRoundCommand, StatusCommand, ViewAccountCommand,
};
use crate::app::backend::command::{BackendCommand, CommandSource};
use crate::app::backend::Backend;
use crate::app::ui::event::UIActionSender;
use crate::app::ui::event::{CommandExecutionStage, CommandExecutionStatus};
use crate::util::{pretty_bytes, pretty_gas};

pub trait CommandExecutor {
    fn dispatch_command(&self, command_source: CommandSource, command: BackendCommand) {
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

    fn execute_publish_task_command(&self, command_source: CommandSource, task_config_path: String);
    fn execute_ipfs_cat_file_command(&self, command_source: CommandSource, file_hash: String);
    fn execute_ipfs_file_status_command(&self, command_source: CommandSource, file_hash: String);
    fn execute_ipfs_get_file_command(
        &self,
        command_source: CommandSource,
        file_hash: String,
        file_path: String,
    );
    fn execute_ipfs_add_file_command(&self, command_source: CommandSource, file_path: String);
    fn execute_start_round_command(&self, command_source: CommandSource);
    fn execute_set_user_endpoint_command(&self, command_source: CommandSource, endpoint: String);
    fn execute_gas_price_command(&self, command_source: CommandSource);
    fn execute_get_user_endpoint_command(
        &self,
        command_source: CommandSource,
        account_id: Option<AccountId>,
    );
    fn execute_query_events_command(&self, command_source: CommandSource, from: u32, count: u32);
    fn execute_count_events_command(&self, command_source: CommandSource);
    fn execute_round_info_command(&self, command_source: CommandSource);
    fn execute_round_status_command(&self, command_source: CommandSource);
    fn execute_view_account_command(&self, command_source: CommandSource, account_id: AccountId);
    fn execute_status_command(&self, command_source: CommandSource);
    fn execute_network_status_command(&self, command_source: CommandSource);
    fn execute_help_command(&self, command_source: CommandSource, help: Vec<String>);
}

impl CommandExecutor for Backend {
    fn execute_publish_task_command(
        &self,
        command_source: CommandSource,
        task_config_path: String,
    ) {
        self.execute_command_block(command_source, async move |it| {
            // let task_config_path = Path::new(&task_config_path);
            //
            // // Check if task config file exists
            // match task_config_path.try_exists() {
            //     Ok(true) => {}
            //     Ok(false) => anyhow::bail!("Task config file does not exist"),
            //     Err(e) => anyhow::bail!("Failed to check if task config file exists: {}", e),
            // }
            //
            // // Read task config file
            // let task_config = tokio::fs::read_to_string(task_config_path).await?;
            // let task_config = serde_json::from_str::<RawTaskConfigFile>(&task_config)?;
            //
            // let actuator = if let Some(actuator) = get_actuator(&task_config.r#type) {
            //     actuator
            // } else {
            //     anyhow::bail!("Unsupported task type: {}", task_config.r#type);
            // };
            //
            // // Encode task config
            // let task_config = match &task_config.input {
            //     RawTaskInputSource::Ipfs { hash, file } => match (hash, file) {
            //         (None, None) => unreachable!(),
            //         (Some(_), Some(_)) => {
            //             anyhow::bail!(
            //                 "Both hash and file are specified in task config {}",
            //                 task_config_path.display()
            //             );
            //         }
            //         (Some(_hash), None) => task_config.build_task_config(None, &actuator)?,
            //         (None, Some(file)) => {
            //             let file_path = Path::new(file.as_str());
            //             let file_path = if file_path.is_absolute() {
            //                 file_path.to_path_buf()
            //             } else {
            //                 task_config_path.parent().unwrap().join(file_path)
            //             };
            //
            //             if let Ok(true) = file_path.try_exists() {
            //                 it.log_string(format!(
            //                     "Uploading file to ipfs: {}",
            //                     file_path.display()
            //                 ))?;
            //
            //                 let file_cid = it.ipfs_client.add_file(file_path.as_path()).await?;
            //
            //                 it.log_string(format!("File uploaded to ipfs: {file_cid}"))?;
            //
            //                 task_config.build_task_config(Some(file_cid), &actuator)?
            //             } else {
            //                 anyhow::bail!(
            //                     "Task input file does not exist, {}",
            //                     file_path.display()
            //                 );
            //             }
            //         }
            //     },
            //     RawTaskInputSource::Link { .. } => {
            //         task_config.build_task_config(None, &actuator)?
            //     }
            // };
            //
            // // Check if round is started
            // let round_status = it.agent.get_round_status().await?;
            //
            // if let RoundStatus::Pending = round_status {
            //     anyhow::bail!("Round is not started yet. Please wait for the round to start.");
            // }
            //
            // // Publish task
            // let (gas, task_id) = it.agent.publish_task(task_config).await?;
            //
            // it.log_string(format!(
            //     "Task published. Gas used: {}, Task ID: {task_id}",
            //     pretty_gas(gas)
            // ))?;
            //
            Ok(())
        });
    }

    fn execute_ipfs_cat_file_command(&self, command_source: CommandSource, file_hash: String) {
        self.execute_command_block(command_source, async move |it| {
            let buffer = it.ipfs_client.cat_file(file_hash.as_str()).await?;
            let buffer = String::from_utf8(buffer)?;

            it.log_multiple_strings(buffer.lines().map(|s| s.to_string()).collect())?;

            Ok(())
        });
    }

    fn execute_ipfs_file_status_command(&self, command_source: CommandSource, file_hash: String) {
        self.execute_command_block(command_source, async move |it| {
            let status = it.ipfs_client.file_status(file_hash.as_str()).await?;

            it.log_multiple_strings(vec![
                format!("File hash: {}", status.hash),
                format!("File size: {}", pretty_bytes(status.cumulative_size)),
                format!("File block size: {}", pretty_bytes(status.block_size)),
                format!("File links size: {}", pretty_bytes(status.links_size)),
                format!("File data size: {}", pretty_bytes(status.data_size)),
                format!("File num links: {}", status.num_links),
            ])?;

            Ok(())
        });
    }

    fn execute_ipfs_get_file_command(
        &self,
        command_source: CommandSource,
        file_hash: String,
        file_path: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |it| {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);

                it.log_string(format!("Downloading file from ipfs: {file_hash}"))?;

                let it2 = it.clone();

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

                                    if let Err(_) = it.log_string(format!(
                                        "Downloading file from ipfs: {downloaded}/{total}({percent:.2}%)"
                                    )) {
                                        tracing::error!("Failed to log get file progress");
                                    }
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

                it2.ipfs_client
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

    fn execute_ipfs_add_file_command(&self, command_source: CommandSource, file_path: String) {
        self.execute_command_block(command_source, async move |it| {
            let file_hash = it.ipfs_client.add_file(file_path.as_str()).await?;

            it.log_string(format!("File uploaded to ipfs: {file_hash}"))?;

            Ok(())
        });
    }

    fn execute_start_round_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let round_status = it.agent.get_round_status().await?;

            let (gas, round_id) = if let RoundStatus::Pending = round_status {
                it.agent.start_new_round().await?
            } else {
                anyhow::bail!("Round is already started");
            };

            it.log_string(format!(
                "New round started: {round_id}, gas used: {}",
                pretty_gas(gas),
            ))?;

            Ok(())
        });
    }

    fn execute_set_user_endpoint_command(&self, command_source: CommandSource, endpoint: String) {
        self.execute_command_block(command_source, async move |it| {
            let gas = it.agent.set_user_endpoint(endpoint.as_str()).await?;

            it.log_string(format!(
                "User endpoint set successfully. Gas used: {}",
                pretty_gas(gas),
            ))?;

            Ok(())
        })
    }

    fn execute_gas_price_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let gas_price = it.agent.gas_price().await?;

            it.log_string(format!("Gas price: {gas_price}"))?;

            Ok(())
        })
    }

    fn execute_get_user_endpoint_command(
        &self,
        command_source: CommandSource,
        account_id: Option<AccountId>,
    ) {
        self.execute_command_block(command_source, async move |it| {
            let endpoint = it.agent.get_user_endpoint(account_id).await?;

            if let Some(endpoint) = endpoint {
                it.log_string(format!("User endpoint: {endpoint}"))?;
            } else {
                it.log_string("User endpoint is not set".to_string())?;
            }

            Ok(())
        })
    }

    fn execute_query_events_command(&self, command_source: CommandSource, from: u32, count: u32) {
        self.execute_command_block(command_source, async move |it| {
            let events = it.agent.query_events(from, count).await?;

            if events.is_empty() {
                it.log_string("No events found".to_string())?;
            } else {
                for event in events {
                    it.log_string(format!("{event}"))?;
                }
            }

            Ok(())
        })
    }

    fn execute_count_events_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let count = it.agent.count_events().await?;

            it.log_string(format!("Events count: {count}"))?;

            Ok(())
        })
    }

    fn execute_round_info_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let round_info = it.agent.get_round_info().await?;

            it.log_string(format!("Round info: {round_info}"))?;

            Ok(())
        })
    }

    fn execute_round_status_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let round_status = it.agent.get_round_status().await?;

            it.log_string(format!("Round status: {round_status}"))?;

            Ok(())
        })
    }

    fn execute_view_account_command(&self, command_source: CommandSource, account_id: AccountId) {
        self.execute_command_block(command_source, async move |it| {
            let account_id_in_string = account_id.to_string();

            let account = it.agent.view_account(account_id).await?;

            it.log_multiple_strings(vec![
                format!("Account ID: {account_id_in_string}"),
                format!("Amount: {}", account.amount),
                format!("Locked: {}", account.locked),
                format!("Code Hash: {}", account.code_hash),
                format!("Storage Usage: {}", account.storage_usage),
                format!("Storage Paid At: {}", account.storage_paid_at),
            ])?;

            Ok(())
        })
    }

    fn execute_status_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let status = it.agent.status().await?;

            it.log_multiple_strings(vec![
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
            ])?;

            Ok(())
        })
    }

    fn execute_network_status_command(&self, command_source: CommandSource) {
        self.execute_command_block(command_source, async move |it| {
            let status = it.agent.network_status().await?;

            it.log_multiple_strings(vec![
                format!("Num Active Peers: {}", status.num_active_peers),
                format!("Sent Bytes Per Sec: {}", status.sent_bytes_per_sec),
                format!("Received Bytes Per Sec: {}", status.received_bytes_per_sec),
            ])?;

            Ok(())
        });
    }

    fn execute_help_command(&self, command_source: CommandSource, help: Vec<String>) {
        let c1 = command_source.clone();

        self.execute_command_block(command_source, async move |it| {
            it.log_multiple_strings(help)?;
            it.log_command_execution(
                c1,
                CommandExecutionStage::Executed,
                CommandExecutionStatus::Succeed,
                None,
            )?;

            Ok(())
        });
    }
}

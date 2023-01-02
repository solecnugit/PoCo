use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use futures::lock::Mutex;
use futures::FutureExt;
use near_primitives::types::AccountId;
use thiserror::__private::DisplayAsDisplay;
use tracing::Level;

use crate::agent::agent::PocoAgent;
use crate::agent::task::config::RawTaskConfig;
use crate::app::backend::command::{
    BackendCommand::{
        CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand,
        IpfsAddFileCommand, IpfsCatFileCommand, NetworkStatusCommand, PublishTaskCommand,
        QueryEventsCommand, RoundStatusCommand, SetUserEndpointCommand, StatusCommand,
        ViewAccountCommand,
    },
    CommandSource,
    ParseBackendCommandError::{InvalidCommandParameter, MissingCommandParameter, UnknownCommand},
};
use crate::app::trace::TracingCategory;
use crate::app::ui::action::UIAction::CommandExecutionDone;
use crate::app::ui::action::{CommandExecutionStage, CommandExecutionStatus};
use crate::config::PocoAgentConfig;
use crate::ipfs::client::IpfsClient;

use super::ui::action::{UIAction, UIActionEvent};

use self::command::{get_internal_command, BackendCommand, ParseBackendCommandError};

pub mod command;

pub struct Backend {
    config: Arc<PocoAgentConfig>,
    receiver: crossbeam_channel::Receiver<CommandSource>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    async_runtime: Box<tokio::runtime::Runtime>,
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
            rusqlite::Connection::open(config.app.database_path.clone())
                .expect("Failed to open database connection"),
        ));

        let ipfs_client = Arc::new(
            IpfsClient::create_ipfs_client(&config.ipfs.ipfs_endpoint)
                .expect("Failed to create ipfs client"),
        );

        Backend {
            receiver,
            sender,
            async_runtime: Box::new(runtime),
            db_connection,
            agent: Arc::new(PocoAgent::new(config.clone())),
            ipfs_client,
            config,
        }
    }

    fn execute_command_block<F, R>(&mut self, command_source: CommandSource, f: F)
    where
        F: FnOnce(
            crossbeam_channel::Sender<UIActionEvent>,
            Arc<PocoAgent>,
            Arc<IpfsClient>,
            Arc<PocoAgentConfig>,
        ) -> R,
        R: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let sender1 = self.sender.clone();
        let sender2 = self.sender.clone();

        let agent = self.agent.clone();
        let ipfs_client = self.ipfs_client.clone();
        let config = self.config.clone();

        self.async_runtime
            .spawn(f(sender1, agent, ipfs_client, config).inspect(move |r| {
                let msg = if let Err(e) = r {
                    tracing::error!(
                        message = "failed to execute command",
                        command = format!("{:?}", command_source),
                        category = format!("{:?}", TracingCategory::Backend),
                        error = format!("{:?}", e)
                    );

                    sender2
                        .send(UIAction::LogString(format!("{}", e.as_display())).into())
                        .unwrap();

                    CommandExecutionDone(
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Failure,
                    )
                    .into()
                } else {
                    CommandExecutionDone(
                        command_source,
                        CommandExecutionStage::Executed,
                        CommandExecutionStatus::Success,
                    )
                    .into()
                };

                sender2.send(msg).unwrap();

                ()
            }));
    }

    fn execute_command(&mut self, command_source: CommandSource, command: BackendCommand) {
        match command {
            HelpCommand(help) => {
                self.sender
                    .send(UIAction::LogMultipleString(help).into())
                    .unwrap();

                self.sender
                    .send(
                        UIAction::CommandExecutionDone(
                            command_source,
                            CommandExecutionStage::Executed,
                            CommandExecutionStatus::Success,
                        )
                        .into(),
                    )
                    .unwrap();
            }
            GasPriceCommand => self.execute_gas_price_command(command_source),
            NetworkStatusCommand => self.execute_network_status_command(command_source),
            StatusCommand => self.execute_status_command(command_source),
            ViewAccountCommand { account_id } => {
                self.execute_view_account_command(command_source, account_id)
            }
            RoundStatusCommand => self.execute_round_status_command(command_source),
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
            PublishTaskCommand { task_config_path } => {
                self.execute_publish_task_command(command_source, task_config_path)
            }
        }
    }

    fn execute_publish_task_command(
        &mut self,
        command_source: CommandSource,
        task_config_path: String,
    ) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| {
                let task_config_path = Path::new(&task_config_path);

                match task_config_path.try_exists() {
                    Ok(true) => {}
                    _ => {
                        sender
                            .send(
                                UIAction::LogString(format!(
                                    "Task config file not found: {}",
                                    task_config_path.display()
                                ))
                                .into(),
                            )
                            .unwrap();
                        return Ok(());
                    }
                }

                let task_config = tokio::fs::read_to_string(task_config_path).await?;
                let task_config = serde_json::from_str::<RawTaskConfig>(&task_config)?;

                task_config;

                Ok(())
            },
        );
    }

    fn execute_ipfs_cat_file_command(&mut self, command_source: CommandSource, file_hash: String) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let buffer = ipfs_client.cat_file(file_hash.as_str()).await.unwrap();
                let buffer = String::from_utf8(buffer).unwrap();

                sender
                    .send(
                        UIAction::LogMultipleString(
                            buffer.lines().into_iter().map(|e| e.to_string()).collect(),
                        )
                        .into(),
                    )
                    .unwrap();

                Ok(())
            },
        );
    }

    fn execute_ipfs_add_file_command(&mut self, command_source: CommandSource, file_path: String) {
        self.execute_command_block(
            command_source,
            async move |sender, _agent, ipfs_client, _config| {
                let file_hash = ipfs_client.add_file(file_path.as_str()).await.unwrap();

                sender
                    .send(UIAction::LogString(format!("File hash: {}", file_hash)).into())
                    .unwrap();

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
            async move |sender, agent, _ipfs_client, _config| match agent
                .set_user_endpoint(endpoint.as_str())
                .await
            {
                Ok(gas) => {
                    sender
                        .send(
                            UIAction::LogString(format!(
                                "Set user endpoint to {} ({} Gas Burnt)",
                                endpoint, gas
                            ))
                            .into(),
                        )
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(
                            UIAction::LogString(format!("Failed to set user endpoint: {:?}", e))
                                .into(),
                        )
                        .unwrap();

                    Err(e)
                }
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
            async move |sender, agent, _ipfs_client, _config| match agent
                .get_user_endpoint(account_id)
                .await
            {
                Ok(Some(endpoint)) => {
                    sender
                        .send(UIAction::LogString(format!("Endpoint: {}", endpoint)).into())
                        .unwrap();

                    Ok(())
                }
                Ok(None) => {
                    sender
                        .send(UIAction::LogString("No endpoint found".to_string()).into())
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get user endpoint".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
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
            async move |sender, agent, _ipfs_client, _config| match agent
                .query_events(from, count)
                .await
            {
                Ok(events) => {
                    if events.is_empty() {
                        sender
                            .send(UIAction::LogString("No events found".to_string()).into())
                            .unwrap();
                    } else {
                        for event in events {
                            sender
                                .send(UIAction::LogString(format!("Event: {}", event)).into())
                                .unwrap();
                        }
                    }

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to query events".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
            },
        )
    }

    fn execute_count_events_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| match agent.count_events().await {
                Ok(event_count) => {
                    sender
                        .send(UIAction::LogString(format!("Total Events: {}", event_count)).into())
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get count events".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
            },
        )
    }

    fn execute_round_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| match agent.get_round_status().await {
                Ok(round_status) => {
                    sender
                        .send(UIAction::LogString(format!("Round Status: {}", round_status)).into())
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get round status".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
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

                match agent.view_account(account_id).await {
                    Ok(account) => {
                        sender
                            .send(
                                UIAction::LogMultipleString(vec![
                                    format!("Account ID: {}", account_id_in_string),
                                    format!("Amount: {}", account.amount),
                                    format!("Locked: {}", account.locked),
                                    format!("Code Hash: {}", account.code_hash),
                                    format!("Storage Usage: {}", account.storage_usage),
                                    format!("Storage Paid At: {}", account.storage_paid_at),
                                ])
                                .into(),
                            )
                            .unwrap();

                        Ok(())
                    }
                    Err(e) => {
                        sender
                            .send(UIAction::LogString("Failed to get account".to_string()).into())
                            .unwrap();

                        sender
                            .send(UIAction::LogString(format!("Error: {}", e)).into())
                            .unwrap();

                        Err(e)
                    }
                }
            },
        )
    }

    fn execute_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| match agent.status().await {
                Ok(status) => {
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
                                format!("  Syncing: {}", status.sync_info.syncing),
                            ])
                            .into(),
                        )
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get status".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
            },
        )
    }

    fn execute_network_status_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| match agent.network_status().await {
                Ok(network_status) => {
                    sender
                        .send(
                            UIAction::LogMultipleString(vec![
                                format!("Num Active Peers: {}", network_status.num_active_peers),
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

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(
                            UIAction::LogString("Failed to get network status".to_string()).into(),
                        )
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
            },
        );
    }

    fn execute_gas_price_command(&mut self, command_source: CommandSource) {
        self.execute_command_block(
            command_source,
            async move |sender, agent, _ipfs_client, _config| match agent.gas_price().await {
                Ok(gas_price) => {
                    sender
                        .send(UIAction::LogString(format!("Gas Price: {}", gas_price)).into())
                        .unwrap();

                    Ok(())
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get gas price".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();

                    Err(e)
                }
            },
        )
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
                if let Some(account_id) = args.get_one::<String>("account-id") {
                    if let Ok(account_id) = account_id.parse() {
                        Ok(ViewAccountCommand { account_id })
                    } else {
                        Err(InvalidCommandParameter("account-id".to_string()))
                    }
                } else {
                    Err(MissingCommandParameter("account-id".to_string()))
                }
            }
            Some(("round-status", _)) => Ok(RoundStatusCommand),
            Some(("count-events", _)) => Ok(CountEventsCommand),
            Some(("query-events", args)) => {
                if let Some(Ok(from)) = args.get_one::<String>("from").map(|e| e.parse()) {
                    let count = args
                        .get_one::<String>("count")
                        .map(|e| e.parse())
                        .unwrap()
                        .unwrap();

                    Ok(QueryEventsCommand { from, count })
                } else {
                    Err(MissingCommandParameter("from".to_string()))
                }
            }
            Some(("get-user-endpoint", args)) => {
                let account_id = args.get_one::<String>("account-id");

                if let Some(account_id) = account_id {
                    let account_id = account_id.parse();

                    if let Ok(account_id) = account_id {
                        Ok(GetUserEndpointCommand {
                            account_id: Some(account_id),
                        })
                    } else {
                        Err(InvalidCommandParameter("account-id".to_string()))
                    }
                } else {
                    Ok(GetUserEndpointCommand { account_id: None })
                }
            }
            Some(("set-user-endpoint", args)) => {
                let endpoint = args.get_one::<String>("endpoint").cloned();

                if let Some(endpoint) = endpoint {
                    Ok(SetUserEndpointCommand { endpoint })
                } else {
                    Err(MissingCommandParameter("endpoint".to_string()))
                }
            }
            Some(("ipfs", args)) => match args.subcommand() {
                Some(("add", args)) => {
                    if let Some(file_path) = args.get_one::<String>("file") {
                        Ok(IpfsAddFileCommand {
                            file_path: file_path.to_string(),
                        })
                    } else {
                        Err(MissingCommandParameter("file".to_string()))
                    }
                }
                Some(("cat", args)) => {
                    if let Some(hash) = args.get_one::<String>("hash") {
                        Ok(IpfsCatFileCommand {
                            file_hash: hash.to_string(),
                        })
                    } else {
                        Err(MissingCommandParameter("hash".to_string()))
                    }
                }
                Some((command, _)) => Err(UnknownCommand(format!("ipfs {}", command))),
                None => Err(UnknownCommand("ipfs".to_string())),
            },
            Some(("publish-task", args)) => {
                let task_config_path = args.get_one::<String>("task-config-path");

                if let Some(task_config_path) = task_config_path {
                    Ok(PublishTaskCommand {
                        task_config_path: task_config_path.to_string(),
                    })
                } else {
                    Err(MissingCommandParameter("task-config-path".to_string()))
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
                        Ok(command_source) => {
                            match self.parse_command(command_source.source.trim()) {
                                Ok(command) => self.execute_command(command_source, command),
                                Err(error) => {
                                    match error {
                                        UnknownCommand(command) => {
                                            tracing::event!(
                                                Level::ERROR,
                                                message = format!("unknown command: {}", command),
                                                category =
                                                    format!("{:?}", TracingCategory::Backend)
                                            );
                                        }
                                        MissingCommandParameter(parameter) => {
                                            tracing::event!(
                                                Level::ERROR,
                                                message = format!(
                                                    "missing command parameter: {}",
                                                    parameter
                                                ),
                                                category =
                                                    format!("{:?}", TracingCategory::Backend)
                                            );
                                        }
                                        InvalidCommandParameter(parameter) => {
                                            tracing::event!(
                                                Level::ERROR,
                                                message = format!(
                                                    "invalid command parameter: {}",
                                                    parameter
                                                ),
                                                category =
                                                    format!("{:?}", TracingCategory::Backend)
                                            );
                                        }
                                    }

                                    self.sender
                                        .send(
                                            CommandExecutionDone(
                                                command_source,
                                                CommandExecutionStage::Parsing,
                                                CommandExecutionStatus::Failure,
                                            )
                                            .into(),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                        Err(_error) => {
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

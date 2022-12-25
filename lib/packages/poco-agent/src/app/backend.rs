use std::future::Future;
use std::sync::Arc;

use futures::lock::Mutex;
use near_primitives::types::AccountId;
use thread_local::ThreadLocal;
use tracing::Level;

use crate::agent::agent::PocoAgent;
use crate::app::backend::command::BackendCommand::{
    CountEventsCommand, QueryEventsCommand, RoundStatusCommand,
};
use crate::app::backend::command::{
    BackendCommand::{
        GasPriceCommand, GetUserEndpointCommand, HelpCommand, IpfsAddFileCommand,
        IpfsCatFileCommand, NetworkStatusCommand, SetUserEndpointCommand, StatusCommand,
        ViewAccountCommand,
    },
    ParseBackendCommandError::{InvalidCommandParameter, MissingCommandParameter, UnknownCommand},
};
use crate::app::trace::TracingCategory;
use crate::config::PocoAgentConfig;
use crate::ipfs::client::IpfsClient;

use super::ui::action::{UIAction, UIActionEvent};

use self::command::{get_internal_command, BackendCommand, ParseBackendCommandError};

pub mod command;

pub struct Backend {
    config: Arc<PocoAgentConfig>,
    receiver: crossbeam_channel::Receiver<String>,
    sender: crossbeam_channel::Sender<UIActionEvent>,
    async_runtime: Box<tokio::runtime::Runtime>,
    db_connection: Arc<Mutex<rusqlite::Connection>>,
    agent: Arc<ThreadLocal<PocoAgent>>,
    ipfs_client: Arc<ThreadLocal<IpfsClient>>,
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

        let db_connection = Arc::new(Mutex::new(
            rusqlite::Connection::open(config.app.database_path.clone())
                .expect("Failed to open database connection"),
        ));

        Backend {
            config,
            receiver,
            sender,
            async_runtime: Box::new(runtime),
            db_connection,
            agent: Arc::new(ThreadLocal::new()),
            ipfs_client: Arc::new(ThreadLocal::new()),
        }
    }

    fn execute_command_block<F, R>(&mut self, f: F)
    where
        F: FnOnce(
            crossbeam_channel::Sender<UIActionEvent>,
            Arc<ThreadLocal<PocoAgent>>,
            Arc<ThreadLocal<IpfsClient>>,
            Arc<PocoAgentConfig>,
        ) -> R,
        R: Future<Output = ()> + Send + 'static,
    {
        let sender = self.sender.clone();
        let agent = self.agent.clone();
        let ipfs_client = self.ipfs_client.clone();
        let config = self.config.clone();

        self.async_runtime
            .spawn(f(sender, agent, ipfs_client, config));
    }

    fn execute_command(&mut self, command: BackendCommand) {
        match command {
            HelpCommand(help) => {
                self.sender
                    .send(UIAction::LogMultipleString(help).into())
                    .unwrap();
            }
            GasPriceCommand => self.execute_gas_price_command(),
            NetworkStatusCommand => self.execute_network_status_command(),
            StatusCommand => self.execute_status_command(),
            ViewAccountCommand { account_id } => self.execute_view_account_command(account_id),
            RoundStatusCommand => self.execute_round_status_command(),
            CountEventsCommand => self.execute_count_events_command(),
            QueryEventsCommand { from, count } => self.execute_query_events_command(from, count),
            GetUserEndpointCommand { account_id } => {
                self.execute_get_user_endpoint_command(account_id)
            }
            SetUserEndpointCommand { endpoint } => self.execute_set_user_endpoint_command(endpoint),
            IpfsAddFileCommand { file_path } => self.execute_ipfs_add_file_command(file_path),
            IpfsCatFileCommand { file_hash } => self.execute_ipfs_cat_file_command(file_hash),
        }
    }

    fn execute_ipfs_cat_file_command(&mut self, file_hash: String) {
        self.execute_command_block(async move |sender, _agent, ipfs_client, config| {
            let ipfs_client = ipfs_client.get_or(|| {
                IpfsClient::create_ipfs_client(config.ipfs.ipfs_endpoint.as_str())
                    .map_err(|e| {
                        tracing::error!(
                            category = TracingCategory::Ipfs.to_string(),
                            message = format!("Failed to create IPFS client: {:?}", e)
                        );

                        panic!("Failed to create IPFS client: {:?}", e);
                    })
                    .unwrap()
            });

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
        });
    }

    fn execute_ipfs_add_file_command(&mut self, file_path: String) {
        self.execute_command_block(async move |sender, _agent, ipfs_client, config| {
            let ipfs_client = ipfs_client.get_or(|| {
                IpfsClient::create_ipfs_client(config.ipfs.ipfs_endpoint.as_str())
                    .map_err(|e| {
                        tracing::error!(
                            category = TracingCategory::Ipfs.to_string(),
                            message = format!("Failed to create IPFS client: {:?}", e)
                        );

                        panic!("Failed to create IPFS client: {:?}", e);
                    })
                    .unwrap()
            });

            let file_hash = ipfs_client.add_file(file_path.as_str()).await.unwrap();

            tracing::info!(
                category = TracingCategory::Ipfs.to_string(),
                message = format!("File Path: {} File hash: {}", file_path, file_hash)
            );

            sender
                .send(UIAction::LogString(format!("File hash: {}", file_hash)).into())
                .unwrap();
        });
    }

    fn execute_set_user_endpoint_command(&mut self, endpoint: String) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.set_user_endpoint(endpoint.as_str()).await {
                Ok(_) => {
                    sender
                        .send(
                            UIAction::LogString(format!("Set user endpoint to {}", endpoint))
                                .into(),
                        )
                        .unwrap();
                }
                Err(e) => {
                    sender
                        .send(
                            UIAction::LogString(format!("Failed to set user endpoint: {:?}", e))
                                .into(),
                        )
                        .unwrap();
                }
            }
        })
    }

    fn execute_get_user_endpoint_command(&mut self, account_id: Option<AccountId>) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.get_user_endpoint(account_id).await {
                Ok(Some(endpoint)) => {
                    sender
                        .send(UIAction::LogString(format!("Endpoint: {}", endpoint)).into())
                        .unwrap();
                }
                Ok(None) => {
                    sender
                        .send(UIAction::LogString("No endpoint found".to_string()).into())
                        .unwrap();
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get user endpoint".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_query_events_command(&mut self, from: u32, count: u32) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.query_events(from, count).await {
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
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to query events".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_count_events_command(&mut self) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.count_events().await {
                Ok(event_count) => {
                    sender
                        .send(UIAction::LogString(format!("Total Events: {}", event_count)).into())
                        .unwrap();
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get count events".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_round_status_command(&mut self) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.get_round_status().await {
                Ok(round_status) => {
                    sender
                        .send(UIAction::LogString(format!("Round Status: {}", round_status)).into())
                        .unwrap();
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get round status".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_view_account_command(&mut self, account_id: AccountId) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));
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
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get account".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_status_command(&mut self) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.status().await {
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
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get status".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
    }

    fn execute_network_status_command(&mut self) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config));

            match agent.network_status().await {
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
                }
            }
        });
    }

    fn execute_gas_price_command(&mut self) {
        self.execute_command_block(async move |sender, agent, _ipfs_client, config| {
            let agent = agent.get_or(|| PocoAgent::new(config.clone()));

            match agent.gas_price().await {
                Ok(gas_price) => {
                    sender
                        .send(UIAction::LogString(format!("Gas Price: {}", gas_price)).into())
                        .unwrap();
                }
                Err(e) => {
                    sender
                        .send(UIAction::LogString("Failed to get gas price".to_string()).into())
                        .unwrap();

                    sender
                        .send(UIAction::LogString(format!("Error: {}", e)).into())
                        .unwrap();
                }
            }
        })
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
                                InvalidCommandParameter(parameter) => {
                                    tracing::event!(
                                        Level::ERROR,
                                        message =
                                            format!("invalid command parameter: {}", parameter),
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

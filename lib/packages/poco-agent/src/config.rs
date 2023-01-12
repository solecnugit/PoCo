use std::path::Path;

use clap::{Arg, Command};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub verbose: bool,
    pub database_path: String,
    pub connection_timeout: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogConfig {
    pub directory: String,
    pub prefix: String,
    pub time_format: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UIConfig {
    pub time_format: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NearConfig {
    pub rpc_endpoint: String,
    pub signer_account_id: String,
    pub signer_secret_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IpfsConfig {
    pub ipfs_endpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PocoTaskPolicy {
    AlwaysTaken,
    AlwaysIgnore
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PocoConfig {
    pub contract_account: String,
    pub event_cycle_in_ms: u64,
    pub task_policy: PocoTaskPolicy
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PocoAgentConfig {
    pub app: AppConfig,
    pub log: LogConfig,
    pub ui: UIConfig,
    pub ipfs: IpfsConfig,
    pub near: NearConfig,
    pub poco: PocoConfig,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AppRunningMode {
    UI,
    DIRECT,
    DAEMON,
}

pub struct AppRunConfig {
    pub mode: AppRunningMode,
    pub config_path: String,
}

pub(crate) fn get_app_command_instance() -> Command {
    Command::new("poco-agent")
        .version("0.1.0")
        .arg(
            Arg::new("config")
                .short('f')
                .long("config")
                .value_name("CONFIG_FILE")
                .default_value("config.toml")
                .required(false),
        )
        .subcommands([
            Command::new("ui").about("Run poco-agent in UI mode"),
            Command::new("daemon").about("Run poco-agent in daemon mode")
        ])
}

pub(crate) fn parse() -> AppRunConfig {
    let commands = get_app_command_instance();
    let arg_matches = commands.get_matches_from(std::env::args().take_while(|arg| arg != "--"));

    let config_path = arg_matches
        .get_one::<String>("config")
        .map(|s| s.to_string())
        .unwrap_or("config.toml".to_string());

    match arg_matches.subcommand() {
        Some(("ui", _)) => AppRunConfig {
            mode: AppRunningMode::UI,
            config_path,
        },
        Some(("daemon", _)) => AppRunConfig {
            mode: AppRunningMode::DAEMON,
            config_path,
        },
        _ => AppRunConfig {
            mode: AppRunningMode::DIRECT,
            config_path,
        },
    }
}

impl AppRunConfig {
    pub(crate) fn get_config(&self) -> Result<PocoAgentConfig, config::ConfigError> {
        let config_path = Path::new(self.config_path.as_str());

        let config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .add_source(config::Environment::with_prefix("POCO"))
            .build()?;

        config.try_deserialize::<PocoAgentConfig>()
    }
}

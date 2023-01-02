use std::path::Path;

use crate::app::backend::command::get_command_instance;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub verbose: bool,
    pub database_path: String,
    pub connection_timeout: u64,
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
pub struct PocoConfig {
    pub poco_contract_account: String,
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

// #[derive(Parser)]
// #[command(name = "poco-agent")]
// #[command(author = "Twiliness")]
// #[command(version = "0.0.1")]
// pub struct CLI {
//     #[arg(short = 'f')]
//     #[arg(long = "config")]
//     #[arg(value_name = "CONFIG_FILE")]
//     #[arg(default_value = "config.toml")]
//     pub config_path: Option<PathBuf>,
// }

pub struct AppRunConfig {
    pub in_ui_mode: bool,
    pub config_path: String,
}

pub(crate) fn parse() -> AppRunConfig {
    let commands = get_command_instance(false);

    let arg_matches = commands.get_matches_from(std::env::args());
    let config_path = arg_matches
        .get_one::<String>("config")
        .map(|s| s.to_string())
        .unwrap_or("config.toml".to_string());

    match arg_matches.subcommand() {
        Some(("ui", _)) => AppRunConfig {
            in_ui_mode: true,
            config_path,
        },
        _ => AppRunConfig {
            in_ui_mode: false,
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

use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub log_dir: String,
    pub log_prefix: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NearConfig {
    pub rpc_endpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub app: AppConfig,
    pub near: NearConfig,
}

#[derive(Parser)]
#[command(name = "poco-agent")]
#[command(author = "Twiliness")]
#[command(version = "0.0.1")]
pub struct CLI {
    #[arg(short = 'f')]
    #[arg(long = "config")]
    #[arg(value_name = "CONFIG_FILE")]
    #[arg(default_value = "config.toml")]
    pub config_path: Option<PathBuf>,
}

pub(crate) fn parse() -> CLI {
    CLI::parse()
}

impl CLI {
    pub(crate) fn get_config(&self) -> Result<Config, config::ConfigError> {
        let config_path = self.config_path.as_ref().unwrap().as_path();

        let config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .add_source(config::Environment::with_prefix("POCO"))
            .build()?;

        config.try_deserialize::<Config>()
    }
}

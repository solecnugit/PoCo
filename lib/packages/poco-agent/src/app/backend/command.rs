use crate::app::backend::command::BackendCommand::{
    GasPriceCommand, HelpCommand, NetworkStatusCommand,
};
use crate::app::backend::command::ParseBackendCommandError::UnknownCommand;

use clap::Command;
use strum::Display;

fn get_internal_command() -> Command {
    Command::new("poco")
        .about("Poco Agent")
        .version("0.0.1")
        .ignore_errors(true)
        .no_binary_name(true)
        .subcommand_required(true)
        .arg_required_else_help(false)
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_colored_help(true)
        .disable_version_flag(true)
        .help_template(
            "
{name} {version}
{subcommands}
        ",
        )
        .subcommands([
            Command::new("help").about("Get help for poco-agent"),
            Command::new("gas-price").about("Get gas price"),
            Command::new("network-status").about("Get network status"),
            Command::new("view-account").about("View account"),
        ])
}

#[derive(Debug, Display)]
pub enum BackendCommand {
    HelpCommand(Vec<String>),
    GasPriceCommand,
    NetworkStatusCommand,
}

#[derive(Debug, Display)]
pub enum ParseBackendCommandError {
    UnknownCommand(String),
}

pub fn parse_command(command: &str) -> Result<BackendCommand, ParseBackendCommandError> {
    let arg_matches = get_internal_command().get_matches_from(command.split_whitespace());

    match arg_matches.subcommand() {
        Some(("help", _)) => Ok(HelpCommand(
            get_internal_command()
                .render_help()
                .to_string()
                .lines()
                .map(|e| e.to_string())
                .collect(),
        )),
        Some(("gas-price", _)) => Ok(GasPriceCommand),
        Some(("network-status", _)) => Ok(NetworkStatusCommand),
        _ => Err(UnknownCommand(command.to_string())),
    }
}

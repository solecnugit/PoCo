use clap::error::ErrorKind;

use crate::app::backend::Backend;
use crate::app::backend::command::{BackendCommand, commands};
use crate::app::backend::command::BackendCommand::{
    CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand, IpfsAddFileCommand,
    IpfsCatFileCommand, IpfsFileStatusCommand, IpfsGetFileCommand, NetworkStatusCommand,
    PublishTaskCommand, QueryEventsCommand, RoundInfoCommand, RoundStatusCommand,
    SetUserEndpointCommand, StartNewRoundCommand, StatusCommand, ViewAccountCommand,
};

pub type ParseBackendCommandError = clap::Error;

pub trait CommandParser {
    fn parse_command(&self, command: &str) -> Result<BackendCommand, ParseBackendCommandError>;
}

impl CommandParser for Backend {
    fn parse_command(&self, command: &str) -> Result<BackendCommand, ParseBackendCommandError> {
        let mut command_instance = commands();
        let arg_matches = command_instance.try_get_matches_from_mut(command.split_whitespace())?;

        match arg_matches.subcommand() {
            Some(("help", args)) => {
                if let Some(command) = args.get_one::<String>("command") {
                    Ok(HelpCommand(
                        command_instance
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
                        command_instance
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
                let account_id = args
                    .get_one::<String>("account-id")
                    .and_then(|e| e.parse().ok())
                    .unwrap();

                Ok(ViewAccountCommand { account_id })
            }
            Some(("round-status", _)) => Ok(RoundStatusCommand),
            Some(("round-info", _)) => Ok(RoundInfoCommand),
            Some(("count-events", _)) => Ok(CountEventsCommand),
            Some(("query-events", args)) => {
                let from = args
                    .get_one::<String>("from")
                    .and_then(|e| e.parse().ok())
                    .unwrap();
                let count = args
                    .get_one::<String>("count")
                    .and_then(|e| e.parse().ok())
                    .unwrap();

                Ok(QueryEventsCommand { from, count })
            }
            Some(("get-user-endpoint", args)) => {
                let account_id = args.get_one::<String>("account-id").cloned();

                if let Some(account_id) = account_id {
                    let parsed_account_id = account_id.parse().ok();

                    if let Some(account_id) = parsed_account_id {
                        Ok(GetUserEndpointCommand {
                            account_id: Some(account_id),
                        })
                    } else {
                        Err(clap::error::Error::raw(
                            ErrorKind::InvalidValue,
                            format!("Invalid account id: {account_id}"),
                        ))
                    }
                } else {
                    Ok(GetUserEndpointCommand { account_id: None })
                }
            }
            Some(("set-user-endpoint", args)) => {
                let endpoint = args.get_one::<String>("endpoint").cloned().unwrap();

                Ok(SetUserEndpointCommand { endpoint })
            }
            Some(("ipfs", args)) => match args.subcommand() {
                Some(("add", args)) => {
                    let file_path = args.get_one::<String>("file-path").cloned().unwrap();

                    Ok(IpfsAddFileCommand { file_path })
                }
                Some(("cat", args)) => {
                    let file_hash = args.get_one::<String>("hash").cloned().unwrap();

                    Ok(IpfsCatFileCommand { file_hash })
                }
                Some(("get", args)) => {
                    let file_hash = args.get_one::<String>("hash").cloned().unwrap();
                    let file_path = args.get_one::<String>("file-path").cloned().unwrap();

                    Ok(IpfsGetFileCommand {
                        file_hash,
                        file_path,
                    })
                }
                Some(("status", args)) => {
                    let file_hash = args.get_one::<String>("hash").cloned().unwrap();

                    Ok(IpfsFileStatusCommand { file_hash })
                }
                _ => unreachable!("clap should have handled this"),
            },
            Some(("start-new-round", _)) => Ok(StartNewRoundCommand),
            Some(("publish-task", args)) => {
                let task_config_path = args.get_one::<String>("task-config-path").cloned().unwrap();

                Ok(PublishTaskCommand {
                    task_config_path: task_config_path.to_string(),
                })
            }
            _ => unreachable!("clap should have handled this"),
        }
    }
}

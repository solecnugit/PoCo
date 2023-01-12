use strum::Display;
use crate::app::backend::Backend;

use crate::app::backend::command::{BackendCommand, commands};
use crate::app::backend::command::{
    BackendCommand::{
        CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand,
        IpfsAddFileCommand, IpfsCatFileCommand, IpfsFileStatusCommand, IpfsGetFileCommand,
        NetworkStatusCommand, PublishTaskCommand, QueryEventsCommand, RoundStatusCommand,
        SetUserEndpointCommand, StartNewRoundCommand, StatusCommand, ViewAccountCommand,
    },
};
use crate::app::backend::parser::ParseBackendCommandError::{InvalidCommandParameter, MissingCommandParameter, UnknownCommand};

#[derive(Debug, Display)]
pub enum ParseBackendCommandError {
    UnknownCommand(String),
    MissingCommandParameter(String),
    InvalidCommandParameter(String),
}

pub trait CommandParser {
    fn parse_command(&self, command: &str) -> Result<BackendCommand, ParseBackendCommandError>;
}

impl CommandParser for Backend {
    fn parse_command(&self, command: &str) -> Result<BackendCommand, ParseBackendCommandError> {
        let arg_matches = commands().get_matches_from(command.split_whitespace());

        match arg_matches.subcommand() {
            Some(("help", args)) => {
                if let Some(command) = args.get_one::<String>("command") {
                    Ok(HelpCommand(
                        commands()
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
                        commands()
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
                Some(("get", args)) => {
                    if let Some(hash) = args.get_one::<String>("hash") {
                        if let Some(file_path) = args.get_one::<String>("file-path") {
                            Ok(IpfsGetFileCommand {
                                file_hash: hash.to_string(),
                                file_path: file_path.to_string(),
                            })
                        } else {
                            Err(MissingCommandParameter("file".to_string()))
                        }
                    } else {
                        Err(MissingCommandParameter("hash".to_string()))
                    }
                }
                Some(("status", args)) => {
                    if let Some(hash) = args.get_one::<String>("hash") {
                        Ok(IpfsFileStatusCommand {
                            file_hash: hash.to_string(),
                        })
                    } else {
                        Err(MissingCommandParameter("hash".to_string()))
                    }
                }
                Some((command, _)) => Err(UnknownCommand(format!("ipfs {command}"))),
                None => Err(UnknownCommand("ipfs".to_string())),
            },
            Some(("start-new-round", _)) => Ok(StartNewRoundCommand),
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
}
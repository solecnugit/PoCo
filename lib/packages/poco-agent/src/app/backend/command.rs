use std::fmt::{Display, Formatter};

use clap::{Arg, Command};
use near_primitives::types::AccountId;
use strum::Display;

#[derive(Debug, Display)]
pub enum BackendCommand {
    HelpCommand(Vec<String>),
    // Ipfs Commands
    IpfsAddFileCommand {
        file_path: String,
    },
    IpfsCatFileCommand {
        file_hash: String,
    },
    IpfsGetFileCommand {
        file_hash: String,
        file_path: String,
    },
    IpfsFileStatusCommand {
        file_hash: String,
    },
    // Near Network
    GasPriceCommand,
    NetworkStatusCommand,
    StatusCommand,
    ViewAccountCommand {
        account_id: AccountId,
    },
    // PoCo Contract Commands
    RoundStatusCommand,
    RoundInfoCommand,
    CountEventsCommand,
    QueryEventsCommand {
        from: u32,
        count: u32,
    },
    GetUserEndpointCommand {
        account_id: Option<AccountId>,
    },
    SetUserEndpointCommand {
        endpoint: String,
    },
    StartNewRoundCommand,
    // Task Related Commands
    PublishTaskCommand {
        task_config_path: String,
    },
}

#[derive(Debug, Clone)]
pub struct CommandSource {
    pub id: String,
    pub source: String,
}

impl Display for CommandSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.id, self.source)
    }
}

pub fn get_command_instance(in_ui_mode: bool) -> Command {
    let subcommand = if in_ui_mode {
        |name: &'static str| {
            Command::new(name)
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true)
        }
    } else {
        |name: &'static str| {
            Command::new(name)
                .disable_help_subcommand(false)
                .disable_help_flag(false)
                .disable_colored_help(false)
                .disable_version_flag(false)
        }
    };

    let subcommands = [
        subcommand("help")
            .about("Get help for poco-agent")
            .arg(Arg::new("command").required(false).index(1)),
        subcommand("gas-price").about("Get gas price"),
        subcommand("network-status").about("Get network status"),
        subcommand("status").about("Get Blockchain status"),
        subcommand("view-account")
            .about("View account")
            .arg(Arg::new("account-id").required(true).index(1)),
        subcommand("round-status").about("Get round status"),
        subcommand("round-info").about("Get round info"),
        subcommand("count-events").about("Count events"),
        subcommand("query-events")
            .about("Query events")
            .arg(
                Arg::new("from")
                    .required(true)
                    .index(1)
                    .allow_negative_numbers(false),
            )
            .arg(
                Arg::new("count")
                    .required(false)
                    .index(2)
                    .allow_negative_numbers(false)
                    .default_value("10"),
            ),
        subcommand("get-user-endpoint")
            .about("Get User Endpoint")
            .arg(Arg::new("account-id").required(false).index(1)),
        subcommand("set-user-endpoint")
            .about("Set User Endpoint")
            .arg(Arg::new("endpoint").required(true).index(1)),
        subcommand("ipfs")
            .about("IPFS")
            .subcommand_required(true)
            .subcommands(vec![
                subcommand("add")
                    .about("Add file to IPFS")
                    .arg(Arg::new("file").required(true).index(1)),
                subcommand("cat")
                    .about("Cat file from IPFS")
                    .arg(Arg::new("hash").required(true).index(1)),
                subcommand("get")
                    .about("Get file from IPFS")
                    .arg(Arg::new("hash").required(true).index(1))
                    .arg(Arg::new("file-path").required(true).index(2)),
                subcommand("status")
                    .about("Get file status from IPFS")
                    .arg(Arg::new("hash").required(false).index(1)),
            ]),
        subcommand("start-new-round").about("Start new round"),
        subcommand("publish-task")
            .about("Publish task")
            .arg(Arg::new("task-config-path").required(true).index(1)),
    ];

    let command = if in_ui_mode {
        Command::new("poco")
            .about("Poco Agent")
            .version("0.1.0")
            .ignore_errors(false)
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
    } else {
        Command::new("poco")
            .about("Poco Agent")
            .version("0.1.0")
            .ignore_errors(false)
            .no_binary_name(true)
            .subcommand_required(false)
            .arg_required_else_help(true)
            .disable_help_subcommand(false)
            .disable_help_flag(false)
            .disable_colored_help(false)
            .disable_version_flag(false)
    };

    command.subcommands(subcommands)
}

pub(crate) fn commands() -> Command {
    get_command_instance(true)
}

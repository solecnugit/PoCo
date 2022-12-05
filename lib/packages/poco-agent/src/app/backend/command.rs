use clap::Command;
use strum::Display;

#[inline]
fn subcommand(name: &'static str) -> Command {
    Command::new(name)
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_colored_help(true)
        .disable_version_flag(true)
}

pub(crate) fn get_internal_command() -> Command {
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
            subcommand("help")
                .about("Get help for poco-agent")
                .arg(clap::Arg::new("command").required(false).index(1)),
            subcommand("gas-price")
                .about("Get gas price"),
            subcommand("network-status")
                .about("Get network status"),
            subcommand("status")
                .about("Get Blockchain status"),
            subcommand("view-account")
                .about("View account")
                .arg(clap::Arg::new("account-id").required(true).index(1)),
            subcommand("round-status")
                .about("Get round status"),
            subcommand("count-events")
                .about("Count events"),
            subcommand("query-events")
                .about("Query events")
                .arg(clap::Arg::new("from").required(true).index(1).allow_negative_numbers(false))
                .arg(clap::Arg::new("count").required(false).index(2).allow_negative_numbers(false).default_value("10")),
        ])
}

#[derive(Debug, Display)]
pub enum BackendCommand {
    HelpCommand(Vec<String>),
    GasPriceCommand,
    NetworkStatusCommand,
    StatusCommand,
    ViewAccountCommand { account_id: String },
    RoundStatusCommand,
    CountEventsCommand,
    QueryEventsCommand { from: u32, count: u32 },
}

#[derive(Debug, Display)]
pub enum ParseBackendCommandError {
    UnknownCommand(String),
    MissingCommandParameter(String),
}

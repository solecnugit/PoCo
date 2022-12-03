use clap::Command;
use strum::Display;

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
            Command::new("help")
                .about("Get help for poco-agent")
                .arg(clap::Arg::new("command").required(false).index(1))
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true),
            Command::new("gas-price")
                .about("Get gas price")
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true),
            Command::new("network-status")
                .about("Get network status")
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true),
            Command::new("status")
                .about("Get Blockchain status")
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true),
            Command::new("view-account")
                .about("View account")
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true)
                .arg(clap::Arg::new("account-id").required(true).index(1)),
            Command::new("round-status")
                .about("Get round status")
                .disable_help_subcommand(true)
                .disable_help_flag(true)
                .disable_colored_help(true)
                .disable_version_flag(true)
        ])
}

#[derive(Debug, Display)]
pub enum BackendCommand {
    HelpCommand(Vec<String>),
    GasPriceCommand,
    NetworkStatusCommand,
    StatusCommand,
    ViewAccountCommand(String),
    RoundStatusCommand,
}

#[derive(Debug, Display)]
pub enum ParseBackendCommandError {
    UnknownCommand(String),
    MissingCommandParameter(String),
}

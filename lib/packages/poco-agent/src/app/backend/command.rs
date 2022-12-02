use clap::Command;

fn get_internal_command() -> Command {
    Command::new("poco")
        .about("Poco Agent")
        .no_binary_name(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("gas-price")
                .about("Get gas price"),
            Command::new("network-status")
                .about("Get network status")
        ])
}

pub fn parse_command(command: &str) {
    let arg_matches = get_internal_command()
        .get_matches_from(command.split_whitespace());

    match arg_matches.subcommand() {
        Some(("gas-price", _)) => {
            println!("gas-price");
        }
        Some(("network-status", _)) => {
            println!("network-status");
        }
        _ => {
            println!("Unknown command");
        }
    }
}

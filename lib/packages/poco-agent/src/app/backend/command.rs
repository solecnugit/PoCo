pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(command: String) -> Result<(), String> {
        if !command.starts_with("/") {
            return Err("command must start with /".to_string());
        }

        let command = command.trim_start_matches("/");
        let parts = command.split_whitespace().collect::<Vec<&str>>();

        match parts[0] {
            "help" => Ok(()),
            &_ => Err(format!("command {} not found", parts[0])),
        }
    }
}

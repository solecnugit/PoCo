use crate::app::backend::command::BackendCommand::{
    CountEventsCommand, GasPriceCommand, GetUserEndpointCommand, HelpCommand, IpfsAddFileCommand,
    IpfsCatFileCommand, IpfsFileStatusCommand, IpfsGetFileCommand, NetworkStatusCommand,
    PublishTaskCommand, QueryEventsCommand, RoundStatusCommand, SetUserEndpointCommand,
    StartNewRoundCommand, StatusCommand, ViewAccountCommand,
};
use crate::app::backend::command::{BackendCommand, CommandSource};
use crate::app::backend::Backend;
use crate::app::ui::action::{CommandExecutionStage, CommandExecutionStatus};
use crate::app::ui::util::{log_command_execution, log_multiple_strings};

pub trait CommandExecutor {
    fn execute_command(&mut self, command_source: CommandSource, command: BackendCommand);
}

impl CommandExecutor for Backend {
    fn execute_command(&mut self, command_source: CommandSource, command: BackendCommand) {
        match command {
            HelpCommand(help) => {
                log_multiple_strings(&self.sender, help);
                log_command_execution(
                    &self.sender,
                    command_source,
                    CommandExecutionStage::Executed,
                    CommandExecutionStatus::Succeed,
                    None,
                );
            }
            GasPriceCommand => self.execute_gas_price_command(command_source),
            NetworkStatusCommand => self.execute_network_status_command(command_source),
            StatusCommand => self.execute_status_command(command_source),
            ViewAccountCommand { account_id } => {
                self.execute_view_account_command(command_source, account_id)
            }
            RoundStatusCommand => self.execute_round_status_command(command_source),
            CountEventsCommand => self.execute_count_events_command(command_source),
            QueryEventsCommand { from, count } => {
                self.execute_query_events_command(command_source, from, count)
            }
            GetUserEndpointCommand { account_id } => {
                self.execute_get_user_endpoint_command(command_source, account_id)
            }
            SetUserEndpointCommand { endpoint } => {
                self.execute_set_user_endpoint_command(command_source, endpoint)
            }
            IpfsAddFileCommand { file_path } => {
                self.execute_ipfs_add_file_command(command_source, file_path)
            }
            IpfsCatFileCommand { file_hash } => {
                self.execute_ipfs_cat_file_command(command_source, file_hash)
            }
            IpfsGetFileCommand {
                file_hash,
                file_path,
            } => self.execute_ipfs_get_file_command(command_source, file_hash, file_path),
            IpfsFileStatusCommand { file_hash } => {
                self.execute_ipfs_file_status_command(command_source, file_hash)
            }
            StartNewRoundCommand => self.execute_start_new_round_command(command_source),
            PublishTaskCommand { task_config_path } => {
                self.execute_publish_task_command(command_source, task_config_path)
            }
        }
    }
}

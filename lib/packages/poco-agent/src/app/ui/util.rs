use crate::app::backend::command::CommandSource;
use crate::app::ui::action::{
    CommandExecutionStage, CommandExecutionStatus, UIAction, UIActionEvent,
};

#[inline]
pub fn log_string(sender: &crossbeam_channel::Sender<UIActionEvent>, message: String) {
    sender.send(UIAction::LogString(message).into()).unwrap();
}

#[inline]
pub fn log_multiple_strings(
    sender: &crossbeam_channel::Sender<UIActionEvent>,
    messages: Vec<String>,
) {
    sender
        .send(UIAction::LogMultipleStrings(messages).into())
        .unwrap();
}

#[inline]
pub fn log_command(sender: &crossbeam_channel::Sender<UIActionEvent>, command: CommandSource) {
    sender.send(UIAction::LogCommand(command).into()).unwrap();
}

#[inline]
pub fn log_command_execution_done(
    sender: &crossbeam_channel::Sender<UIActionEvent>,
    command: CommandSource,
    stage: CommandExecutionStage,
    status: CommandExecutionStatus,
) {
    sender
        .send(UIAction::LogCommandExecutionDone(command, stage, status).into())
        .unwrap();
}

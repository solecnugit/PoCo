use crate::app::backend::command::CommandSource;
use crate::app::ui::event::{
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
pub fn log_command_execution(
    sender: &crossbeam_channel::Sender<UIActionEvent>,
    command: CommandSource,
    stage: CommandExecutionStage,
    status: CommandExecutionStatus,
    error: Option<Box<anyhow::Error>>,
) {
    sender
        .send(UIAction::LogCommandExecution(command, stage, status, error).into())
        .unwrap();
}

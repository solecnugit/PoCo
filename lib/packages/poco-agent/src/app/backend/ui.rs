use crate::app::backend::command::CommandSource;
use crate::app::backend::Backend;
use crate::app::trace::TracingEvent;
use crate::app::ui::event::{
    CommandExecutionStage, CommandExecutionStatus, UIAction, UIActionEvent, UIActionSender,
};

impl UIActionSender for Backend {
    type Error = crossbeam_channel::SendError<UIActionEvent>;

    fn panic(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.ui_sender.send(UIAction::Panic(error).into())
    }

    fn log_string(&self, message: String) -> Result<(), Self::Error> {
        self.ui_sender.send(UIAction::LogString(message).into())
    }

    fn log_multiple_strings(&self, messages: Vec<String>) -> Result<(), Self::Error> {
        self.ui_sender
            .send(UIAction::LogMultipleStrings(messages).into())
    }

    fn log_tracing_event(&self, event: TracingEvent) -> Result<(), Self::Error> {
        self.ui_sender.send(UIAction::LogTracingEvent(event).into())
    }

    fn log_command(&self, command: CommandSource) -> Result<(), Self::Error> {
        self.ui_sender.send(UIAction::LogCommand(command).into())
    }

    fn log_command_execution(
        &self,
        command: CommandSource,
        stage: CommandExecutionStage,
        status: CommandExecutionStatus,
        error: Option<Box<anyhow::Error>>,
    ) -> Result<(), Self::Error> {
        self.ui_sender
            .send(UIAction::LogCommandExecution(command, stage, status, error).into())
    }

    fn quit_app(&self) -> Result<(), Self::Error> {
        self.ui_sender.send(UIAction::QuitApp.into())
    }
}

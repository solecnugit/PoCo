use async_trait::async_trait;
use poco_types::types::convert_account_id_from_sdk_to_primitives;
use poco_types::types::round::RoundId;
use poco_types::types::task::id::TaskId;
use poco_types::types::task::OnChainTaskConfig;
use std::fmt::Display;
use poco_types::types::task::service::TaskService;

use poco_agent::types::AccountId;

use crate::app::backend::Backend;
use crate::app::ui::event::{UIActionEvent, UIActionSender};
use crate::config::PocoTaskPolicy;

pub type ContractEvent = poco_types::types::event::IndexedEvent;
pub type ContractEventPayload = poco_types::types::event::Events;

#[async_trait]
pub trait ContractEventHandler {
    type Output;
    type Error;

    fn handle_event(&self, event: ContractEvent);

    async fn handle_new_round_event(&self, round_id: &RoundId)
        -> Result<Self::Output, Self::Error>;

    async fn handle_new_task_event(
        &self,
        task_id: &TaskId,
        task_config: &TaskService,
    ) -> Result<Self::Output, Self::Error>;

    async fn handle_user_profile_field_update_event(
        &self,
        user_id: &AccountId,
        field: &str,
        value: &str,
    ) -> Result<Self::Output, Self::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum ContractBackendError {
    ChannelError(#[from] crossbeam_channel::SendError<UIActionEvent>),
}

impl Display for ContractBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractBackendError::ChannelError(e) => write!(f, "Channel error: {:?}", e),
        }
    }
}

#[async_trait]
impl ContractEventHandler for Backend {
    type Output = ();
    type Error = ContractBackendError;

    fn handle_event(&self, event: ContractEvent) {
        let it = self.clone();

        self.runtime.spawn(async move {
            let ret = match &event.payload {
                ContractEventPayload::NewRoundEvent { round_id } => {
                    it.handle_new_round_event(round_id).await
                }
                ContractEventPayload::NewTaskEvent {
                    task_id,
                    task_service,
                } => it.handle_new_task_event(task_id, task_service).await,
                ContractEventPayload::UserProfileFieldUpdateEvent {
                    user_id,
                    field,
                    value,
                } => {
                    let user_id = convert_account_id_from_sdk_to_primitives(user_id);

                    it.handle_user_profile_field_update_event(&user_id, field, value)
                        .await
                }
            };

            if let Err(e) = ret {
                it.log_string(format!(
                    "Error happened during handle event {event:?}, error {e:?}"
                ))
                .unwrap();
            }
        });
    }

    async fn handle_new_round_event(
        &self,
        round_id: &RoundId,
    ) -> Result<Self::Output, Self::Error> {
        self.log_string(format!("New round: {round_id}"))?;

        self.db.set_last_round_id(*round_id).unwrap();

        Ok(())
    }

    async fn handle_new_task_event(
        &self,
        task_id: &TaskId,
        task_service: &TaskService,
    ) -> Result<Self::Output, Self::Error> {
        self.log_string(format!("New task: {task_id}"))?;

        // self.db.cache_task_config(task_id, &task_config).unwrap();

        match self.config.app.task_policy {
            PocoTaskPolicy::AlwaysTaken => {
                // let _ = agent.take_task(task_id).await;
            }
            PocoTaskPolicy::AlwaysIgnore => {}
        }

        Ok(())
    }

    async fn handle_user_profile_field_update_event(
        &self,
        user_id: &AccountId,
        field: &str,
        value: &str,
    ) -> Result<Self::Output, Self::Error> {
        self.log_string(format!("User {user_id} update field {field} to {value}"))?;

        Ok(())
    }
}

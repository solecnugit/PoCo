use async_trait::async_trait;
use poco_types::types::round::RoundId;
use poco_types::types::task::id::TaskId;
use poco_types::types::task::OnChainTaskConfig;
use super::super::super::util::convert_account_id_from_sdk_to_primitives;
// use poco_types::types::convert_account_id_from_sdk_to_primitives;

use poco_agent::types::AccountId;

use crate::app::backend::Backend;
use crate::app::ui::util::log_string;
use crate::config::PocoTaskPolicy;

pub type ContractEvent = poco_types::types::event::IndexedEvent;
// 修改原来的type
pub type ContractEventPayload = poco_types::types::event::Events;

#[async_trait]
pub trait ContractEventHandler {
    type Output;
    type Error;

    fn handle_event(&self, event: ContractEvent);

    async fn handle_new_round_event(&self, round_id: &RoundId) -> Result<Self::Output, Self::Error>;

    async fn handle_new_task_event(
        &self,
        task_id: &TaskId,
        task_config: &OnChainTaskConfig,
    ) -> Result<Self::Output, Self::Error>;

    async fn handle_user_profile_field_update_event(
        &self,
        user_id: &AccountId,
        field: &str,
        value: &str,
    ) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug)]
pub enum ContractBackendError {}

#[async_trait]
impl ContractEventHandler for Backend {
    type Output = ();
    type Error = ContractBackendError;

    fn handle_event(&self, event: ContractEvent) {
        let backend = self.clone();

        self.runtime.spawn(async move {
            let _ret = match &event.payload {
                ContractEventPayload::NewRoundEvent { round_id } => {
                    backend.handle_new_round_event(round_id).await
                }
                ContractEventPayload::NewTaskEvent {
                    task_id,
                    task_config,
                } => backend.handle_new_task_event(task_id, task_config).await,
                ContractEventPayload::UserProfileFieldUpdateEvent {
                    user_id,
                    field,
                    value,
                } => {
                    let user_id = convert_account_id_from_sdk_to_primitives(user_id);

                    backend.handle_user_profile_field_update_event(&user_id, field, value).await
                }
            };

            // if let Err(e) = ret {
            //     log_string(&self.ui_sender, format!("Error happened during handle event {event:?}, error {e:?}"));
            // }
        });
    }

    async fn handle_new_round_event(&self, round_id: &RoundId) -> Result<Self::Output, Self::Error> {
        log_string(&self.ui_sender, format!("New round: {round_id}"));

        Ok(())
    }

    async fn handle_new_task_event(
        &self,
        task_id: &TaskId,
        task_config: &OnChainTaskConfig,
    ) -> Result<Self::Output, Self::Error> {
        log_string(&self.ui_sender, format!("New task: {:?}", task_id));

        let task_id_u64: u64 = task_id.into();

        self.db
            .cache_task_config(task_id_u64, &task_config)
            .unwrap();

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
        log_string(
            &self.ui_sender,
            format!("User {user_id} update field {field} to {value}"),
        );

        Ok(())
    }
}

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Deserialize;
use near_sdk::{near_bindgen, AccountId};
use schemars::JsonSchema;

use crate::r#type::{RoundId, TaskNonce};
use crate::task::TaskConfig;

#[near_bindgen(event_json(standard = "nep297"))]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, JsonSchema, Clone)]
pub enum Events {
    #[event_version("0.0.1")]
    NewRoundEvent { round_id: RoundId },

    #[event_version("0.0.1")]
    NewTaskEvent {
        round_id: RoundId,
        task_nonce: TaskNonce,
        task_config: TaskConfig,
    },

    #[event_version("0.0.1")]
    UserProfileFieldUpdateEvent {
        user_id: AccountId,
        field: String,
        value: String,
    },
}

impl Events {
    #[inline]
    pub fn log_event(&self) {
        self.emit()
    }
}

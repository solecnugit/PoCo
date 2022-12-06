use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize, Serializer};
use near_sdk::AccountId;
use schemars::JsonSchema;
use std::fmt::{Display, Formatter};
use strum::Display;

use crate::types::round::RoundId;
use crate::types::task::config::TaskConfig;
use crate::types::task::TaskNonce;

pub type EventNonce = u32;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexedEvent {
    pub event_id: u32,
    pub payload: Events,
}

impl Display for IndexedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Event {{ event_id: {}, payload: {} }}",
            self.event_id, self.payload
        )
    }
}

#[near_bindgen(event_json(standard = "nep297"))]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, JsonSchema, Clone, Debug, Display)]
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

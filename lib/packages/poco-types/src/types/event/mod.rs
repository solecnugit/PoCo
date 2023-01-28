#[cfg(feature = "all")]
use std::fmt::{Display, Formatter};

use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::round::RoundId;
use crate::types::task::id::TaskId;
use crate::types::task::OnChainTaskConfig;

pub type EventNonce = u32;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexedEvent {
    pub event_id: u32,
    pub payload: Events,
}

#[cfg(feature = "all")]
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
#[derive(BorshDeserialize, BorshSerialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum Events {
    #[event_version("0.0.1")]
    NewRoundEvent { round_id: RoundId },

    #[event_version("0.0.1")]
    NewTaskEvent {
        task_id: TaskId,
        task_config: OnChainTaskConfig,
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

#[cfg(feature = "all")]
impl Display for Events {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Events::NewRoundEvent { round_id } => {
                write!(f, "NewRoundEvent {{ round_id: {round_id} }}")
            }
            Events::NewTaskEvent {
                task_id,
                task_config,
            } => write!(
                f,
                "NewTaskEvent {{ task_id: {}, task_config: {} }}",
                task_id, task_config
            ),
            Events::UserProfileFieldUpdateEvent {
                user_id,
                field,
                value,
            } => write!(
                f,
                "UserProfileFieldUpdateEvent {{ user_id: {}, field: {}, value: {} }}",
                user_id, field, value
            ),
        }
    }
}

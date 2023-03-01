#[cfg(feature = "native")]
use std::fmt::{Display, Formatter};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use schemars::JsonSchema;

use crate::types::round::RoundId;
use crate::types::task::id::TaskId;
use crate::types::task::OnChainTaskConfig;
use crate::types::task::service::TaskService;

pub type EventNonce = u32;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexedEvent {
    pub event_id: EventNonce,
    pub payload: Events,
}

#[cfg(feature = "native")]
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
        task_service: TaskService,
    },

    #[event_version("0.0.1")]
    UserProfileFieldUpdateEvent {
        user_id: AccountId,
        field: String,
        value: String,
    },
}

impl Events {
    pub fn emit_event(&self) {
        self.emit();
    }
}

#[cfg(feature = "native")]
impl Display for Events {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Events::NewRoundEvent { round_id } => {
                write!(f, "NewRoundEvent {{ round_id: {round_id} }}")
            }
            Events::NewTaskEvent {
                task_id,
                task_service,
            } => write!(
                f,
                "NewTaskEvent {{ task_id: {}, task_service: {} }}",
                task_id, task_service
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

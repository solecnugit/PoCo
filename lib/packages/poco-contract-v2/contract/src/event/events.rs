use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Deserialize;
use near_sdk::{near_bindgen, AccountId};

use crate::r#type::RoundId;

#[near_bindgen(event_json(standard = "nep297"))]
#[derive(BorshDeserialize, BorshSerialize, Deserialize)]
pub enum Events {
    #[event_version("0.0.1")]
    NewRoundEvent { round_id: RoundId },
    
    #[event_version("0.0.1")]
    NewTaskEvent { task_id: String },

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

mod event_bus;

use near_sdk::near_bindgen;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::r#type::RoundId;

#[near_bindgen(event_json(standard = "nep297"))]
#[derive(BorshDeserialize, BorshSerialize, Deserialize)]
pub enum ContractEvent {
    #[event_version("0.0.1")]
    NewRoundEvent { round_id: RoundId },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractEventData {
    id: u64,
    payload: ContractEvent,
}

pub use event_bus::EventBus;
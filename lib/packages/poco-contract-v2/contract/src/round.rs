use std::cmp::Ordering;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};

pub type RoundId = u64;
pub type RoundDuration = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RoundManager {
    round_id: RoundId,
    round_start_time: u64,
    round_duration: RoundDuration,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RoundStatus {
    Running,
    Pending,
}

#[near_bindgen]
impl RoundManager {
    #[private]
    pub fn new(initial_round_id: RoundId, initial_round_duration: u64) -> RoundManager {
        RoundManager {
            round_id: initial_round_id,
            round_start_time: 0,
            round_duration: initial_round_duration,
        }
    }

    pub fn get_round_status(&self) -> RoundStatus {
        let block_time = near_sdk::env::block_timestamp_ms();

        match (block_time - self.round_start_time).cmp(&self.round_duration) {
            Ordering::Less => RoundStatus::Running,
            _ => RoundStatus::Pending,
        }
    }

    pub fn get_round_id(&self) -> RoundId {
        self.round_id
    }
}

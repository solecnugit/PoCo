pub mod config;
pub mod id;

use crate::types::task::config::TaskConfig;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

pub type TaskNonce = u32;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    owner: AccountId,
    config: TaskConfig,
}

impl Task {
    pub fn new(owner: AccountId, config: TaskConfig) -> Self {
        Task { owner, config }
    }
}

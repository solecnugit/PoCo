mod round_manager;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum RoundStatus {
    Running,
    Pending,
}

pub use round_manager::RoundManager;

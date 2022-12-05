use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use strum::Display;

pub type BlockTimestamp = u64;
pub type RoundId = u32;
pub type RoundDuration = u64;

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Display
)]
#[serde(crate = "near_sdk::serde")]
pub enum RoundStatus {
    Running,
    Pending,
}

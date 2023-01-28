use std::ops::{Add, Sub};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use strum::Display;

pub type RoundId = u32;

#[derive(
BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Eq, Ord, PartialOrd, PartialEq, Copy, Clone
)]
#[serde(crate = "near_sdk::serde")]
pub struct BlockTimestamp { timestamp_in_ms: u64 }

impl BlockTimestamp {
    pub fn new(timestamp_in_ms: u64) -> Self {
        Self { timestamp_in_ms }
    }

    pub fn between(&self, start: BlockTimestamp, end: BlockTimestamp) -> bool {
        self.timestamp_in_ms >= start.timestamp_in_ms && self.timestamp_in_ms <= end.timestamp_in_ms
    }
}

impl From<u64> for BlockTimestamp {
    fn from(timestamp_in_ms: u64) -> Self {
        Self { timestamp_in_ms }
    }
}

impl Add for BlockTimestamp {
    type Output = BlockTimestamp;

    fn add(self, rhs: BlockTimestamp) -> Self::Output {
        BlockTimestamp {
            timestamp_in_ms: self.timestamp_in_ms + rhs.timestamp_in_ms,
        }
    }
}

impl Add<RoundDuration> for BlockTimestamp {
    type Output = BlockTimestamp;

    fn add(self, rhs: RoundDuration) -> Self::Output {
        BlockTimestamp {
            timestamp_in_ms: self.timestamp_in_ms + rhs.duration_in_ms,
        }
    }
}

impl Sub for BlockTimestamp {
    type Output = BlockTimestamp;

    fn sub(self, rhs: BlockTimestamp) -> Self::Output {
        BlockTimestamp {
            timestamp_in_ms: self.timestamp_in_ms - rhs.timestamp_in_ms,
        }
    }
}

#[derive(
BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Copy, Clone
)]
#[serde(crate = "near_sdk::serde")]
pub struct RoundDuration { duration_in_ms: u64 }

impl RoundDuration {
    pub fn new(duration_in_ms: u64) -> Self {
        Self { duration_in_ms }
    }
}

impl From<u64> for RoundDuration {
    fn from(duration_in_ms: u64) -> Self {
        Self { duration_in_ms }
    }
}

#[derive(
BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Display,
)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "UPPERCASE")]
pub enum RoundStatus {
    Running,
    Pending,
}

#[derive(
Serialize, Deserialize, JsonSchema, Debug, PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct RoundInfo {
    pub id: RoundId,
    pub status: RoundStatus,
    pub start_time: BlockTimestamp,
    pub duration: RoundDuration,
    pub task_count: u32,
    pub event_count: u32,
}
use std::{cmp::Ordering, ops::Add};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::{round::RoundStatus, r#type::{RoundId, RoundDuration, BlockTimestamp}};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RoundManager {
    round_id: RoundId,
    round_start_time: BlockTimestamp,
    round_duration: RoundDuration,
}


impl RoundManager {
    pub fn new(initial_round_id: RoundId, initial_round_duration: u64) -> RoundManager {
        RoundManager {
            round_id: initial_round_id,
            round_start_time: 0,
            round_duration: initial_round_duration,
        }
    }

    #[inline]
    pub fn start_new_round(&mut self) -> RoundId {
        assert_eq!(self.get_round_status(), RoundStatus::Pending, "current round has not ended yet.");

        self.round_id += 1u64;
        self.round_start_time = near_sdk::env::block_timestamp_ms();
        
        self.round_id
    }

    #[inline]
    pub fn get_round_status(&self) -> RoundStatus {
        let block_time = near_sdk::env::block_timestamp_ms();

        match (block_time - self.round_start_time).cmp(&self.round_duration) {
            Ordering::Less => RoundStatus::Running,
            _ => RoundStatus::Pending,
        }
    }

    #[inline]
    pub fn get_round_id(&self) -> RoundId {
        self.round_id
    }
}

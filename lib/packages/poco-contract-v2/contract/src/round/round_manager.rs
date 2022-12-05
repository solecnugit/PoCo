use std::cmp::Ordering;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use poco_types::types::event::EventNonce;
use poco_types::types::round::{BlockTimestamp, RoundDuration, RoundId, RoundStatus};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RoundManager {
    round_id: RoundId,
    round_start_time: BlockTimestamp,
    round_duration: RoundDuration,
    round_event_offset: EventNonce,
}

impl RoundManager {
    pub fn new(initial_round_id: RoundId, initial_round_duration: u64) -> RoundManager {
        RoundManager {
            round_id: initial_round_id,
            round_start_time: 0,
            round_duration: initial_round_duration,
            round_event_offset: 0,
        }
    }

    #[inline]
    pub fn start_new_round(&mut self, event_offset: u32) -> RoundId {
        assert_eq!(
            self.get_round_status(),
            RoundStatus::Pending,
            "current round has not ended yet."
        );

        self.round_id += 1;
        self.round_start_time = near_sdk::env::block_timestamp_ms();

        self.round_event_offset = event_offset;

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

    #[inline]
    pub fn get_round_event_offset(&self) -> EventNonce {
        self.round_event_offset
    }
}

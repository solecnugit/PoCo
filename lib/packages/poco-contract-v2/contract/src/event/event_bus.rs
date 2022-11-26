use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};

use crate::r#type::RoundId;
use crate::Events;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EventBus {
    // Events happens in previous rounds
    prev_round_events: UnorderedMap<RoundId, Vector<Events>>,
    // Events happens in current round
    events: Vector<Events>,
    // Preserve last `preserve_round` events of round
    preserve_round: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EventData {
    id: u64,
    payload: Events,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EventQuery {
    round_id: u64,
    events: Vec<EventData>,
}

impl EventBus {
    pub fn new(initial_preserve_round: u64) -> Self {
        EventBus {
            prev_round_events: UnorderedMap::new(b"p"),
            events: Vector::new(b"c"),
            preserve_round: initial_preserve_round,
        }
    }

    #[inline]
    pub fn emit(&mut self, event: Events) {
        event.log_event();

        self.events.push(&event);
    }

    #[inline]
    pub fn switch_to_next_round(&mut self, current_round_id: RoundId) {
        if self.prev_round_events.len() > self.preserve_round {
            let round_id_to_remove = current_round_id - self.preserve_round;
            self.prev_round_events.remove(&round_id_to_remove);
        }

        self.prev_round_events
            .insert(&current_round_id, &self.events);
        self.events.clear();
    }

    #[inline]
    pub fn count_round_events(&self) -> u64 {
        self.events.len()
    }

    #[inline]
    pub fn get_preserve_round_parameter(&self) -> u64 {
        self.preserve_round
    }

    #[inline]
    pub fn query_round_events(&self, round_id: RoundId, from: usize, count: usize) -> EventQuery {
        let events = self
            .events
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| EventData {
                id: e.0 as u64,
                payload: e.1,
            })
            .collect();

        EventQuery { round_id, events }
    }

    #[inline]
    pub fn query_round_events_at(
        &self,
        current_round_id: &RoundId,
        round_id_to_fetch: &RoundId,
        from: usize,
        count: usize,
    ) -> EventQuery {
        assert!(
            current_round_id - self.preserve_round > *round_id_to_fetch,
            "Round is too old to retrieve events."
        );

        let events = self.prev_round_events.get(round_id_to_fetch);

        assert!(events.is_some(), "Invalid round.");

        let events = events
            .unwrap()
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| EventData {
                id: e.0 as u64,
                payload: e.1,
            })
            .collect();

        EventQuery {
            round_id: *round_id_to_fetch,
            events,
        }
    }
}

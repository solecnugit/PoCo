use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};

use crate::r#type::RoundId;
use crate::ContractEvent;
use crate::ContractEventData;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EventBus {
    // Events happens in previous rounds
    prev_round_events: UnorderedMap<RoundId, Vector<ContractEvent>>,
    // Events happens in current round
    events: Vector<ContractEvent>,
    // Preserve last `preserve_round` events of round
    preserve_round: u64,
}

impl EventBus {
    pub fn new(initial_preserve_round: u64) -> Self {
        EventBus {
            prev_round_events: UnorderedMap::new(b"p"),
            events: Vector::new(b"c"),
            preserve_round: initial_preserve_round
        }
    }

    #[inline]
    pub fn emit(&mut self, event: ContractEvent) {
        event.emit();

        self.events.push(&event);
    }

    #[inline]
    pub fn switch_to_next_round(&mut self, current_round_id: RoundId) {
        if self.prev_round_events.len() > self.preserve_round {
            let round_id_to_remove = current_round_id - self.preserve_round;
            self.prev_round_events.remove(&round_id_to_remove);
        }

        self.prev_round_events.insert(&current_round_id, &self.events);
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
    pub fn fetch_round_events(&self, from: usize, count: usize) -> Vec<ContractEventData> {
        self.events
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| ContractEventData {
                id: e.0 as u64,
                payload: e.1,
            })
            .collect()
    }

    #[inline]
    pub fn fetch_round_events_at(
        &self,
        current_round_id: &RoundId,
        round_id_to_fetch: &RoundId,
        from: usize,
        count: usize,
    ) -> Vec<ContractEventData> {
        assert!(
            current_round_id - self.preserve_round > *round_id_to_fetch,
            "Round is too old to retrieve events."
        );

        let events = self.prev_round_events.get(round_id_to_fetch);

        assert!(events.is_some(), "Invalid round.");

        events
            .unwrap()
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| ContractEventData {
                id: e.0 as u64,
                payload: e.1,
            })
            .collect()
    }
}

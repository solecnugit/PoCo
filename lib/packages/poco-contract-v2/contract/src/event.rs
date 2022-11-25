use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};

use crate::round::RoundId;

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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EventBus {
    // Events happens in previous rounds
    prev_round_events: UnorderedMap<RoundId, Vector<ContractEvent>>,
    // Events happens in current round
    events: Vector<ContractEvent>,
    // Preserve last `preserve_round` events of round
    preserve_round: u64,
    // Current Round id
    round_id: RoundId,
}

#[near_bindgen]
impl EventBus {
    #[private]
    pub fn new(initial_round_id: RoundId, initial_preserve_round: u64) -> Self {
        EventBus {
            prev_round_events: UnorderedMap::new(b"p"),
            events: Vector::new(b"c"),
            preserve_round: initial_preserve_round,
            round_id: initial_round_id,
        }
    }

    #[private]
    pub fn emit(&mut self, event: &ContractEvent) {
        event.emit();

        self.events.push(event);
    }

    #[private]
    pub fn switch_to_next_round(&mut self) {
        if self.prev_round_events.len() > self.preserve_round {
            let round_id_to_remove = self.round_id - self.preserve_round;
            self.prev_round_events.remove(&round_id_to_remove);
        }

        self.prev_round_events.insert(&self.round_id, &self.events);
        self.events.clear();
        self.round_id += 1;
    }

    pub fn count_round_events(&self) -> u64 {
        self.events.len()
    }

    pub fn fetch_round_events(&self, from: usize, count: usize) -> Vec<ContractEventData> {
        self.events
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| ContractEventData {
                id: (e.0 + 1) as u64,
                payload: e.1,
            })
            .collect()
    }

    pub fn fetch_round_events_at(
        &self,
        round_id: &RoundId,
        from: usize,
        count: usize,
    ) -> Vec<ContractEventData> {
        assert!(
            self.round_id - self.preserve_round > *round_id,
            "Round is too old to retrieve events."
        );

        let events = self.prev_round_events.get(round_id);

        assert!(events.is_some(), "Invalid round.");

        events
            .unwrap()
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| ContractEventData {
                id: (e.0 + 1) as u64,
                payload: e.1,
            })
            .collect()
    }
}

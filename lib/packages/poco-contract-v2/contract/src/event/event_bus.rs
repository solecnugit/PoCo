use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::store::Vector;
use schemars::JsonSchema;

use crate::Events;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EventBus {
    events: Vector<Events>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct EventData {
    event_id: u32,
    payload: Events,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            events: Vector::new(b"events".to_vec()),
        }
    }

    #[inline]
    pub fn emit(&mut self, event: Events) {
        event.log_event();

        self.events.push(event);
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.events.len()
    }

    #[inline]
    pub fn query_event(&self, from: u32, count: u32) -> Vec<EventData> {
        let from = from as usize;
        let count = count as usize;

        self.events
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| EventData {
                event_id: e.0 as u32,
                payload: e.1.clone(),
            })
            .collect()
    }
}

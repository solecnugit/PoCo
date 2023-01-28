use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::Vector;
use poco_types::types::event::{Events, IndexedEvent};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct EventBus {
    events: Vector<Events>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            events: Vector::new(b"event-bus:events".to_vec()),
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
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    #[inline]
    pub fn query_event(&self, from: u32, count: u32) -> Vec<IndexedEvent> {
        let from = from as usize;
        let count = count as usize;

        self.events
            .iter()
            .enumerate()
            .skip(from)
            .take(count)
            .map(|e| IndexedEvent {
                event_id: e.0 as u32,
                payload: e.1.clone(),
            })
            .collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub mod event;
pub mod round;
pub mod r#type;

use event::{EventBus, ContractEventData, ContractEvent};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen};
use r#type::RoundId;
use round::{RoundManager, RoundStatus};

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    round_manager: RoundManager,
    event_bus: EventBus
}

// Define the default, which automatically initializes the contract
impl Default for Contract{
    fn default() -> Self{
        let initial_round_id = 0u64;
        let initial_round_duration = 1000 * 60 * 30;
        let initial_preserve_round = 3;

        Self{
            round_manager: RoundManager::new(initial_round_id, initial_round_duration),
            event_bus: EventBus::new(initial_preserve_round),
        }
    }
}


#[near_bindgen]
impl Contract {
    pub fn start_new_round(&mut self) -> RoundId {
        let old_round_id = self.round_manager.get_round_id();
        let new_round_id = self.round_manager.start_new_round();

        self.event_bus.switch_to_next_round(old_round_id);
        self.event_bus.emit(ContractEvent::NewRoundEvent{ round_id: new_round_id });
        
        new_round_id
    }

    pub fn get_round_id(&self) -> RoundId {
        self.round_manager.get_round_id()
    }

    pub fn get_round_status(&self) -> RoundStatus {
        self.round_manager.get_round_status()
    }

    pub fn get_round_id_and_status(&self) -> (RoundId, RoundStatus) {
        (self.round_manager.get_round_id(), self.round_manager.get_round_status())
    }

    pub fn count_round_events(&self) -> u64 {
        self.event_bus.count_round_events()
    }

    pub fn fetch_round_events(&self, from: usize, count: usize) -> Vec<ContractEventData> {
        self.event_bus.fetch_round_events(from, count)
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn get_default_greeting() {
    //     let contract = Contract::default();
    //     // this test did not call set_greeting so should return the default "Hello" greeting
    //     assert_eq!(
    //         contract.get_greeting(),
    //         "Hello".to_string()
    //     );
    // }

    // #[test]
    // fn set_then_get_greeting() {
    //     let mut contract = Contract::default();
    //     contract.set_greeting("howdy".to_string());
    //     assert_eq!(
    //         contract.get_greeting(),
    //         "howdy".to_string()
    //     );
    // }
}

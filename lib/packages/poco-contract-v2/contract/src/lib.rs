pub mod event;
pub mod round;

use event::EventBus;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen};
use round::RoundManager;

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
        let initial_round_id = 1;
        let initial_round_duration = 1000 * 60 * 30;
        let initial_preserve_round = 3;

        Self{
            round_manager: RoundManager::new(initial_round_id, initial_round_duration),
            event_bus: EventBus::new(initial_round_id, initial_preserve_round),
        }
    }
}


#[near_bindgen]
impl Contract {

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

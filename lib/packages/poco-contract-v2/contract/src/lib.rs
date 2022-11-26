pub mod event;
pub mod round;
pub mod r#type;
pub mod user;

use event::{EventBus, ContractEventData, ContractEvent};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId};
use r#type::RoundId;
use round::{RoundManager, RoundStatus};
use user::{UserManager, UserProfile};

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    user_manager: UserManager,
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
            user_manager: UserManager::new(),
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

    pub fn get_user_profile(&self, account: AccountId) -> UserProfile {
        self.user_manager.get_user_profile(&account)
    }

    pub fn get_own_profile(&self) -> UserProfile {
        let account = near_sdk::env::signer_account_id();

        self.user_manager.get_user_profile(&account)
    }

    pub fn set_user_endpoint(&mut self, endpoint: String) {
        let account = near_sdk::env::signer_account_id();

        self.user_manager.set_user_endpoint(&account, endpoint);
    }
}

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

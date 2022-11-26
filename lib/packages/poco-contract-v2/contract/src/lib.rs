pub mod event;
pub mod round;
pub mod user;
pub mod task;
pub mod r#type;

use event::{EventBus, EventQuery, Events};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId};
use r#type::RoundId;
use round::{RoundManager, RoundStatus};
use task::{TaskManager, TaskId};
use user::{UserManager, UserProfile};

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    user_manager: UserManager,
    round_manager: RoundManager,
    task_manager: TaskManager,
    event_bus: EventBus,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        let initial_round_id = 0u64;
        let initial_round_duration = 1000 * 60 * 30;
        let initial_preserve_round = 3;

        Self {
            user_manager: UserManager::new(),
            round_manager: RoundManager::new(initial_round_id, initial_round_duration),
            task_manager: TaskManager::new(initial_preserve_round),
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
        self.event_bus.emit(Events::NewRoundEvent {
            round_id: new_round_id,
        });

        self.task_manager.switch_to_next_round(old_round_id);

        new_round_id
    }

    pub fn get_round_id(&self) -> RoundId {
        self.round_manager.get_round_id()
    }

    pub fn get_round_status(&self) -> RoundStatus {
        self.round_manager.get_round_status()
    }

    pub fn get_round_id_and_status(&self) -> (RoundId, RoundStatus) {
        (
            self.round_manager.get_round_id(),
            self.round_manager.get_round_status(),
        )
    }

    pub fn count_round_events(&self) -> u64 {
        self.event_bus.count_round_events()
    }

    pub fn query_round_events(&self, from: usize, count: usize) -> EventQuery {
        self.event_bus
            .query_round_events(self.get_round_id(), from, count)
    }

    pub fn query_round_events_at(
        &self,
        round_id: RoundId,
        from: usize,
        count: usize,
    ) -> EventQuery {
        self.event_bus
            .query_round_events_at(&self.get_round_id(), &round_id, from, count)
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

        self.user_manager.set_user_endpoint(&account, &endpoint);
        self.event_bus.emit(Events::UserProfileFieldUpdateEvent {
            user_id: account,
            field: "endpoint".to_string(),
            value: endpoint,
        })
    }

    pub fn get_user_endpoint(&self, account: AccountId) -> Option<String> {
        self.user_manager.get_user_endpoint(&account)
    }

    pub fn publish_task(&mut self) -> TaskId {
        assert_eq!(self.get_round_status(), RoundStatus::Running, "Round has not been started yet.");

        let owner = near_sdk::env::signer_account_id();
        let task_id = self.task_manager.publish_task(self.get_round_id(), owner);

        self.event_bus.emit(Events::NewTaskEvent { task_id: (&task_id).into() });

        task_id
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

pub mod event;
pub mod round;
pub mod task;
pub mod user;
pub mod util;
pub mod r#type;

use event::{EventBus, EventData, Events};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId};
use r#type::RoundId;
use round::{RoundManager, RoundStatus};
use task::{TaskId, TaskManager};
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
        let initial_round_id = 0;
        let initial_round_duration = 1000 * 60 * 30;

        Self {
            user_manager: UserManager::new(),
            round_manager: RoundManager::new(initial_round_id, initial_round_duration),
            task_manager: TaskManager::new(),
            event_bus: EventBus::new(),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn start_new_round(&mut self) -> RoundId {
        let new_round_id = self.round_manager.start_new_round(self.event_bus.len());

        self.event_bus.emit(Events::NewRoundEvent {
            round_id: new_round_id,
        });

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

    pub fn count_events(&self) -> u32 {
        self.event_bus.len()
    }

    pub fn query_events(&self, from: u32, count: u32) -> Vec<EventData> {
        self.event_bus.query_event(from, count)
    }

    pub fn query_round_events(&self, round_offset: u32, count: u32) -> Vec<EventData> {
        let from = round_offset - self.round_manager.get_round_event_offset();

        self.event_bus.query_event(from, count)
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
        assert_eq!(
            self.get_round_status(),
            RoundStatus::Running,
            "Round has not been started yet."
        );

        let owner = near_sdk::env::signer_account_id();
        let task_id = self.task_manager.publish_task(self.get_round_id(), owner);

        self.event_bus.emit(Events::NewTaskEvent {
            round_id: self.round_manager.get_round_id(),
            task_nonce: task_id.get_task_nonce(),
        });

        task_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

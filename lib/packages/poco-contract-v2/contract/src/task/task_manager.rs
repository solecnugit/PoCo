use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::Vector;
use near_sdk::AccountId;

use crate::r#type::RoundId;

use super::task::Task;
use super::TaskId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    tasks: Vector<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: Vector::new(b"v"),
        }
    }

    #[inline]
    pub fn publish_task(&mut self, current_round_id: RoundId, owner: AccountId) -> TaskId {
        self.tasks.push(Task::new(owner));

        TaskId::new(current_round_id, self.tasks.len() - 1)
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.tasks.len()
    }
}

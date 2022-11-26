use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};

use crate::r#type::RoundId;

use super::TaskId;
use super::task::Task;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    // Tasks published in the previous round
    prev_round_tasks: UnorderedMap<RoundId, Vector<Task>>,
    // Tasks published in the current round
    tasks: Vector<Task>,
    // Preserve last `preserve_round` tasks of round
    preserve_round: u64,
}

impl TaskManager {
    pub fn new(initial_preserve_round: u64) -> Self {
        TaskManager {
            prev_round_tasks: UnorderedMap::new(b"u"),
            tasks: Vector::new(b"v"),
            preserve_round: initial_preserve_round,
        }
    }

    #[inline]
    pub fn switch_to_next_round(&mut self, current_round_id: RoundId) {
        if self.prev_round_tasks.len() > self.preserve_round {
            let round_id_to_remove = current_round_id - self.preserve_round;
            self.prev_round_tasks.remove(&round_id_to_remove);
        }

        self.prev_round_tasks
            .insert(&current_round_id, &self.tasks);
        self.tasks.clear();
    }

    #[inline]
    pub fn publish_task(&mut self, current_round_id: RoundId, owner: AccountId) -> TaskId {
        self.tasks.push(&Task::new(owner));

        TaskId::new(current_round_id, self.tasks.len() - 1)
    }
}

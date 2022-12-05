use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::Vector;
use near_sdk::AccountId;
use poco_types::types::round::RoundId;
use poco_types::types::task::config::TaskConfig;
use poco_types::types::task::id::TaskId;
use poco_types::types::task::Task;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    tasks: Vector<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: Vector::new(b"taskmanager".to_vec()),
        }
    }

    #[inline]
    pub fn publish_task(
        &mut self,
        current_round_id: RoundId,
        owner: AccountId,
        config: TaskConfig,
    ) -> TaskId {
        self.tasks.push(Task::new(owner, config));

        TaskId::new(current_round_id, self.tasks.len() - 1)
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.tasks.len()
    }
}

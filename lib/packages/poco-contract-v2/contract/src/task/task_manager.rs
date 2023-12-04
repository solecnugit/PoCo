use near_sdk::{AccountId, env};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::{LookupMap, Vector};
use poco_types::types::round::RoundId;
use poco_types::types::task::{OnChainTaskConfig, TaskConfig};
use poco_types::types::task::id::TaskId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    tasks: LookupMap<RoundId, Vector<OnChainTaskConfig>>,
    count: u64,
}

impl TaskManager {
    pub fn new() -> Self {
        let tasks = LookupMap::new(b"task-manager:tasks".to_vec());

        // 这里要多做一个数据结构，用于保存每一个task的完成状态（可能有多次尝试）

        TaskManager { tasks, count: 0 }
    }

    // #[inline]
    // pub fn query_specific_task(
    //     &self,
    //     task_id: TaskId,
    // ) -> OnChainTaskConfig {
    //     self.tasks
    //         .get(&task_id.get_round_id())
    //         .and_then(|tasks| tasks.get(*(&task_id.get_task_nonce())).cloned()).unwrap()
    // }

    #[inline]
    pub fn query_specific_task(
        &self,
        round_id: u32,
        task_nounce: u32
    ) -> OnChainTaskConfig {
        self.tasks
            .get(&round_id)
            .and_then(|tasks| tasks.get(task_nounce).cloned()).unwrap()
    }

    #[inline]
    pub fn show_tasks(
        &self,
        round_id: u32,
    ) -> Option<&Vector<OnChainTaskConfig>> {
        self.tasks.get(&round_id)
    }

    // #[inline]
    // pub fn commit_task(
    //     &mut self,

    // )

    #[inline]
    pub fn publish_task(
        &mut self,
        round_id: RoundId,
        owner: AccountId,
        config: TaskConfig,
    ) -> (TaskId, OnChainTaskConfig) {
        let tasks_for_round = self.tasks.entry(round_id).or_insert_with(|| {
            Vector::new(
                format!("task-manager:tasks:{round_id}")
                    .as_bytes()
                    .to_vec(),
            )
        });

        let task_id = TaskId::new(round_id, tasks_for_round.len());
        let config = config.to_on_chain_task_config(owner, task_id.clone());

        if let Ok(config) = config {
            tasks_for_round.push(config.clone());

            self.count += 1;

            (task_id, config)
        } else {
            env::panic_str("Failed to publish task");
        }
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn round_count(&self, round_id: RoundId) -> u32 {
        self.tasks
            .get(&round_id)
            .map(|tasks| tasks.len())
            .unwrap_or(0)
    }

    #[inline]
    pub fn is_round_empty(&self, round_id: RoundId) -> bool {
        self.round_count(round_id) == 0
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
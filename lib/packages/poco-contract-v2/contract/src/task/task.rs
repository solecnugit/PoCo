use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::AccountId;
use near_sdk::serde::Serialize;

use crate::r#type::RoundId;

#[derive(BorshDeserialize, BorshSerialize, PartialEq, PartialOrd, Hash, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskId(RoundId, u64);

impl TryFrom<&str> for TaskId {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let index = value.find("/");

        if let Some(index) = index {
            let (round_id, task_id) = value.split_at(index);
            let round_id = round_id.parse();
            let task_id = task_id.parse();

            if let (Ok(round_id), Ok(task_id)) = (round_id, task_id) {
                Ok(TaskId(round_id, task_id))
            } else {
                Err("invalid task id format")
            }
        } else {
            Err("invalid task id format")
        }
    }
}

impl TryFrom<String> for TaskId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TryFrom::<&str>::try_from(value.as_str())
    }
}

impl From<&TaskId> for String {
    fn from(task_id: &TaskId) -> Self {
        format!("{}/{}", task_id.0, task_id.1)
    }
}

impl From<TaskId> for String {
    fn from(task_id: TaskId) -> Self {
        From::from(&task_id)
    }
}

impl TaskId {
    #[inline]
    pub fn new(round_id: RoundId, task_nonce: u64) -> Self {
        TaskId(round_id, task_nonce)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Task {
    owner: AccountId
}

impl Task {
    pub fn new(owner: AccountId) -> Self {
        Task {
            owner
        }
    }
}
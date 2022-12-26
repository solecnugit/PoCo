use std::fmt::Display;
use crate::types::round::RoundId;
use crate::types::task::TaskNonce;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::de::{Unexpected, Visitor};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(
    BorshDeserialize, BorshSerialize, JsonSchema, PartialEq, PartialOrd, Hash, Clone, Debug,
)]
pub struct TaskId(RoundId, TaskNonce);

impl Serialize for TaskId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: near_sdk::serde::Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(s.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: near_sdk::serde::Deserializer<'de>,
    {
        struct TaskIdVisitor;
        impl<'de> Visitor<'de> for TaskIdVisitor {
            type Value = TaskId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "task id format: round_id/task_nonce")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: near_sdk::serde::de::Error,
            {
                let r: Result<TaskId, &'static str> = v.try_into();

                match r {
                    Ok(r) => Ok(r),
                    Err(_) => Err(E::invalid_value(Unexpected::Str(v), &self)),
                }
            }
        }

        deserializer.deserialize_str(TaskIdVisitor)
    }
}

impl TryFrom<&str> for TaskId {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let index = value.find('/');

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
    pub fn new(round_id: RoundId, task_nonce: TaskNonce) -> Self {
        TaskId(round_id, task_nonce)
    }

    #[inline]
    pub fn get_round_id(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn get_task_nonce(&self) -> u32 {
        self.1
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}
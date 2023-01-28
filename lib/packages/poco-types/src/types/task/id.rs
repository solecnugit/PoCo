#[cfg(feature = "all")]
use std::fmt::Display;

use impl_serde::serde::de::Error;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde::de::Visitor;
use schemars::JsonSchema;

use crate::types::round::RoundId;
use crate::types::task::TaskNonce;

#[derive(
BorshDeserialize, BorshSerialize, JsonSchema, PartialEq, PartialOrd, Hash, Clone, Debug,
)]
pub struct TaskId(RoundId, TaskNonce);

impl Serialize for TaskId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: near_sdk::serde::Serializer,
    {
        serializer.serialize_u64(self.into())
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
                write!(formatter, "task id should be u64")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(TaskId::from(v))
            }
        }

        deserializer.deserialize_u64(TaskIdVisitor)
    }
}

impl TryFrom<&str> for TaskId {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.parse::<u64>()?;

        Ok(TaskId::from(value))
    }
}

impl TryFrom<String> for TaskId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TryFrom::<&str>::try_from(value.as_str())
    }
}

impl From<u64> for TaskId {
    fn from(value: u64) -> Self {
        let round_id = (value >> 32) as u32;
        let task_nonce = (value & 0x00000000FFFFFFFF) as u32;

        TaskId(round_id, task_nonce)
    }
}

impl From<&TaskId> for u64 {
    fn from(value: &TaskId) -> Self {
        let TaskId(round_id, task_id) = value;

        ((*round_id as u64) << 32) | (*task_id as u64)
    }
}

impl From<TaskId> for u64 {
    fn from(value: TaskId) -> Self {
        From::from(&value)
    }
}

impl From<&TaskId> for String {
    fn from(task_id: &TaskId) -> Self {
        let task_id: u64 = task_id.into();

        format!("{:x}", task_id)
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

#[cfg(feature = "all")]
impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", u64::from(self))
    }
}

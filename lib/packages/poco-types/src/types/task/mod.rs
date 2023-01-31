#[cfg(feature = "all")]
use std::fmt::{Debug, Display};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use schemars::JsonSchema;

use crate::types::task::id::TaskId;
use crate::types::uint::U256;

pub mod id;

pub type TaskNonce = u32;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskInputSource {
    Ipfs { hash: String },
    Link { url: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskOutputSource {
    Ipfs,
    Link { url: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskRequirementOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskRequirement {
    pub property: String,
    pub operator: TaskRequirementOperator,
    pub value: U256,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskOffer {
    pub bounty: U256,
    pub requirements: Option<Vec<TaskRequirement>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OnChainTaskConfig {
    pub owner: AccountId,
    pub id: TaskId,
    pub input: TaskInputSource,
    pub output: TaskOutputSource,
    pub requirements: Vec<TaskRequirement>,
    pub offer: Vec<TaskOffer>,
    pub config: Vec<u8>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskConfig {
    pub input: TaskInputSource,
    pub output: TaskOutputSource,
    pub requirements: Vec<TaskRequirement>,
    pub offer: Vec<TaskOffer>,
    pub config: Vec<u8>,
    pub r#type: String,
}

impl TaskConfig {
    pub fn to_on_chain_task_config(
        self,
        owner: AccountId,
        id: TaskId,
    ) -> anyhow::Result<OnChainTaskConfig> {
        Ok(OnChainTaskConfig {
            owner,
            id,
            input: self.input,
            output: self.output,
            requirements: self.requirements,
            offer: self.offer,
            config: self.config,
            r#type: self.r#type,
        })
    }
}

impl TaskRequirement {
    pub fn is_ok(&self, rhs: &U256) -> bool {
        match self.operator {
            TaskRequirementOperator::Equal => self.value == *rhs,
            TaskRequirementOperator::NotEqual => self.value != *rhs,
            TaskRequirementOperator::GreaterThan => self.value > *rhs,
            TaskRequirementOperator::GreaterThanOrEqual => self.value >= *rhs,
            TaskRequirementOperator::LessThan => self.value < *rhs,
            TaskRequirementOperator::LessThanOrEqual => self.value <= *rhs,
        }
    }
}

#[cfg(feature = "all")]
impl Display for OnChainTaskConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TaskConfig {{ owner: {}, id: {}, input: {:?}, output: {:?}, requirements: {:?}, offer: {:?}, config: {:?}, type: {:?} }}",
            self.owner, self.id, self.input, self.output, self.requirements, self.offer, self.config, self.r#type
        )
    }
}

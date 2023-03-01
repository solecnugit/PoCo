#[cfg(feature = "native")]
use std::fmt::{Debug, Display};
use std::fmt::Formatter;
use std::mem;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use schemars::JsonSchema;
use crate::types::region::RegionId;

use crate::types::task::id::TaskId;
use crate::types::task::service::{TaskService, TaskServiceType};
use crate::types::uint::U256;

pub mod id;
pub mod service;

pub type TaskNonce = u32;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum TaskInputSource {
    Ipfs { hash: String },
    Link { url: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
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
    pub service: TaskService,
    pub config: Vec<u8>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskConfigRequest {
    pub service: TaskService,
    pub config: Vec<u8>
}

impl TaskConfigRequest {
    #[cfg(feature = "protocol")]
    pub fn to_on_chain_task_config(mut self, owner: AccountId, task_id: TaskId) -> anyhow::Result<OnChainTaskConfig> {
        Ok(OnChainTaskConfig {
            owner,
            id: task_id,
            service: mem::take(&mut self.service),
            config: mem::take(&mut self.config),
        })
    }
}

#[derive(BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub struct RawTaskConfig<S : TaskServiceType> {
    pub task_input: TaskInputSource,
    pub task_output: TaskOutputSource,
    pub task_region: RegionId,
    pub task_requirements: Vec<S::TaskRequirement>,
    pub task_terminations: Vec<S::TaskTermination>,
    pub task_prices: Vec<S::TaskPrice>,
    pub task_service: TaskService
}

impl <S: TaskServiceType> RawTaskConfig<S> {
    #[cfg(feature = "native")]
    pub fn to_task_config_buffer(
        self,
    ) -> anyhow::Result<TaskConfigRequest> {
        let config = self.try_to_vec()?;


        Ok(TaskConfigRequest {
            service: self.task_service,
            config,
        })
    }
}

impl TaskRequirement {
    #[cfg(feature = "native")]
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

#[cfg(feature = "native")]
impl Display for OnChainTaskConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnChainTaskConfig {{ owner: {}, id: {}, service: {} }}", self.owner, self.id, self.service)
    }
}
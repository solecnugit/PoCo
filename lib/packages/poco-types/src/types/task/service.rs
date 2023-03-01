use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde::de::DeserializeOwned;
use schemars::JsonSchema;
use crate::types::uint::U256;

pub trait Serializable : BorshSerialize + BorshDeserialize + Serialize + DeserializeOwned + JsonSchema {}

impl <T> Serializable for T
where
    T: BorshSerialize + BorshDeserialize + Serialize + DeserializeOwned + JsonSchema {}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Default, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub struct TaskService {
    pub category: String,
    pub r#type: String,
}

#[cfg(feature = "native")]
impl Display for TaskService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.category, self.r#type)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operators {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub struct DefaultTaskRequirementEntry<T> {
    pub property: T,
    pub operator: Operators,
    pub value: U256,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub struct DefaultTaskPrice<T> {
    pub value: U256,
    pub conditions: Vec<T>,
}

pub trait TaskServiceType {
    type TaskRequirementProperties: Serializable;
    type TaskTerminationProperties: Serializable;
    type TaskPriceConditionProperties: Serializable;

    type TaskRequirement : Serializable = DefaultTaskRequirementEntry<Self::TaskRequirementProperties>;
    type TaskTermination : Serializable = DefaultTaskRequirementEntry<Self::TaskTerminationProperties>;
    type TaskPriceCondition : Serializable = DefaultTaskRequirementEntry<Self::TaskPriceConditionProperties>;
    type TaskPrice : Serializable = DefaultTaskPrice<Self::TaskPriceCondition>;

    fn get_task_service_description(&self) -> TaskService;
}

#[derive(Debug, Clone)]
pub struct MediaTranscodingTaskService;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub enum MediaTranscodingTaskRequirementProperties {
    ServiceSLAFactor,
    UserStakeAmount
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub enum MediaTranscodingTaskTerminationProperties {
    BlockHeight,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "camelCase")]
pub enum MediaTranscodingTaskPriceConditionProperties {
    BlockHeight,
}


impl TaskServiceType for MediaTranscodingTaskService {
    type TaskRequirementProperties = MediaTranscodingTaskRequirementProperties;
    type TaskTerminationProperties = MediaTranscodingTaskTerminationProperties;
    type TaskPriceConditionProperties = MediaTranscodingTaskPriceConditionProperties;

    fn get_task_service_description(&self) -> TaskService {
        TaskService {
            category: "MediaTranscoding".to_string(),
            r#type: "ffmpeg-30s-mp4-from-h264-1080p-to-h265-1080p".to_string(),
        }
    }
}
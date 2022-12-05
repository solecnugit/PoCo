pub mod media;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use self::media::{MediaTranscodingSourceConfig, MediaTranscodingTargetConfig};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskInputSource {
    Link { url: String },
    Ipfs { cid: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskInput {
    hash: String,
    source: TaskInputSource,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskOutput {
    Link,
    Ipfs,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskConfig {
    MediaTranscodingTask {
        r#type: String,
        input: TaskInput,
        output: TaskOutput,
        source: MediaTranscodingSourceConfig,
        target: MediaTranscodingTargetConfig,
    },
}

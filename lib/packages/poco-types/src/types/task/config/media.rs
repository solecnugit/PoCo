use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VideoConfig {
    encoding: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AudioConfig {
    encoding: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MediaTranscodingSourceConfig {
    video: VideoConfig,
    audio: AudioConfig,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MediaTranscodingTargetConfig {
    video: VideoConfig,
    audio: AudioConfig,
}

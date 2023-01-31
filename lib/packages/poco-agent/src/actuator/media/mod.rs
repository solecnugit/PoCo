use async_trait::async_trait;
use borsh::{BorshDeserialize, BorshSerialize};
use poco_types::types::task::{OnChainTaskConfig, TaskConfig};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::actuator::{TaskActuator, TaskConfigFactory};
use crate::agent::task::config::DomainTaskConfig;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub struct MediaTranscodingTaskConfig {
    source: MediaTranscodingSourceConfig,
    target: MediaTranscodingTargetConfig,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
struct VideoConfig {
    codec: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
struct AudioConfig {
    codec: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
struct MediaTranscodingConfig {
    video: VideoConfig,
    audio: AudioConfig,
}

type MediaTranscodingSourceConfig = MediaTranscodingConfig;
type MediaTranscodingTargetConfig = MediaTranscodingConfig;

pub const MEDIA_TRANSCODING_TASK_TYPE: &str = "MEDIA_TRANSCODING";

impl DomainTaskConfig for MediaTranscodingTaskConfig {
    fn r#type(&self) -> &'static str {
        MEDIA_TRANSCODING_TASK_TYPE
    }
}

pub struct MediaTranscodingActuator {}

impl TaskConfigFactory for MediaTranscodingActuator {
    type Config = MediaTranscodingTaskConfig;
}

#[async_trait]
impl TaskActuator for MediaTranscodingActuator {
    async fn execute(&mut self, _config: &OnChainTaskConfig) -> anyhow::Result<()> {
        todo!()
    }

    fn encode_task_config(&self, config: Value) -> anyhow::Result<Vec<u8>> {
        let config: <Self as TaskConfigFactory>::Config = serde_json::from_value(config)?;

        config.to_bytes()
    }

    fn r#type(&self) -> &'static str {
        MEDIA_TRANSCODING_TASK_TYPE
    }
}

impl MediaTranscodingActuator {
    pub fn new() -> Self {
        Self {}
    }
}

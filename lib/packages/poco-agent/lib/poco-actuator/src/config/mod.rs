use borsh::{BorshDeserialize, BorshSerialize};
use poco_types::types::task::{
    TaskConfig, TaskInputSource, TaskOffer, TaskOutputSource, TaskRequirement,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::BoxedTaskActuator;

pub trait DomainTaskConfig:
    Serialize + DeserializeOwned + BorshDeserialize + BorshSerialize
{
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        self.try_to_vec().map_err(|e| anyhow::anyhow!(e))
    }

    fn r#type(&self) -> &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "UPPERCASE")]
pub enum RawTaskInputSource {
    Ipfs {
        hash: Option<String>,
        file: Option<String>,
    },
    Link {
        url: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawTaskConfigFile {
    pub input: RawTaskInputSource,
    pub output: TaskOutputSource,
    pub requirements: Vec<TaskRequirement>,
    pub offer: Vec<TaskOffer>,
    pub config: serde_json::Value,
    pub r#type: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TaskInputSourceBuildError {
    #[error("Both hash and file are set")]
    BothHashAndFileAreSet,
    #[error("Neither hash nor file are set")]
    NeitherHashNorFileAreSet,
    #[error("IPFS CID is not set")]
    IpfsCidIsNotSet,
}

impl RawTaskInputSource {
    pub(crate) fn build_task_input_source(
        self,
        ipfs_cid: Option<String>,
    ) -> Result<TaskInputSource, TaskInputSourceBuildError> {
        match self {
            RawTaskInputSource::Ipfs { hash, file } => {
                return match (hash, file) {
                    (Some(_), Some(_)) => Err(TaskInputSourceBuildError::BothHashAndFileAreSet),
                    (None, None) => Err(TaskInputSourceBuildError::NeitherHashNorFileAreSet),
                    (Some(hash), None) => Ok(TaskInputSource::Ipfs { hash }),
                    (None, Some(_)) => {
                        if let Some(ipfs_cid) = ipfs_cid {
                            Ok(TaskInputSource::Ipfs { hash: ipfs_cid })
                        } else {
                            Err(TaskInputSourceBuildError::IpfsCidIsNotSet)
                        }
                    }
                };
            }
            RawTaskInputSource::Link { url } => Ok(TaskInputSource::Link { url }),
        }
    }
}

impl RawTaskConfigFile {
    pub fn build_task_config(
        self,
        ipfs_cid: Option<String>,
        actuator: &BoxedTaskActuator,
    ) -> anyhow::Result<TaskConfig> {
        let input = self.input.build_task_input_source(ipfs_cid)?;
        let config = actuator.encode_task_config(self.config)?;
        let r#type = actuator.r#type().to_string();

        Ok(TaskConfig {
            input,
            output: self.output.clone(),
            requirements: self.requirements.clone(),
            offer: self.offer.clone(),
            config,
            r#type,
        })
    }
}

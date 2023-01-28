use crate::actuator::{BoxedTaskActuator, TaskActuator, TaskConfigFactory};
use anyhow::anyhow;
use borsh::{BorshDeserialize, BorshSerialize};
use poco_types::types::task::{
    TaskConfig, TaskInputSource, TaskOffer, TaskOutputSource, TaskRequirement,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::Any;

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

impl RawTaskInputSource {
    pub(crate) fn to_task_input_source(
        &self,
        ipfs_cid: Option<String>,
    ) -> anyhow::Result<TaskInputSource> {
        match self {
            RawTaskInputSource::Ipfs { hash, file } => {
                if hash.is_some() && file.is_some() {
                    anyhow::bail!("Both hash and file are set");
                } else if let Some(hash) = hash {
                    Ok(TaskInputSource::Ipfs { hash: hash.clone() })
                } else if file.is_some() {
                    let ipfs_cid = ipfs_cid.ok_or_else(|| anyhow!("IPFS CID is not set"))?;

                    Ok(TaskInputSource::Ipfs { hash: ipfs_cid })
                } else {
                    anyhow::bail!("Neither hash nor file are set");
                }
            }
            RawTaskInputSource::Link { url } => Ok(TaskInputSource::Link { url: url.clone() }),
        }
    }
}

pub fn build_task_config(
    raw_task_config: &RawTaskConfigFile,
    ipfs_cid: Option<String>,
    actuator: &BoxedTaskActuator,
) -> anyhow::Result<TaskConfig> {
    let input = raw_task_config.input.to_task_input_source(ipfs_cid)?;
    let config = actuator.encode_domain_config_json_value(&raw_task_config.config)?;
    let r#type = actuator.r#type().to_string();

    Ok(TaskConfig {
        input,
        output: raw_task_config.output.clone(),
        requirements: raw_task_config.requirements.clone(),
        offer: raw_task_config.offer.clone(),
        config,
        r#type,
    })
}

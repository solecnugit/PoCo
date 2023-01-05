use anyhow::anyhow;
use poco_types::types::task::{
    TaskConfig, TaskInputSource, TaskOffer, TaskOutputSource, TaskRequirement, WorkloadConfig,
};
use serde::{Deserialize, Serialize};

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
pub struct RawTaskConfig {
    pub input: RawTaskInputSource,
    pub output: TaskOutputSource,
    pub requirements: Vec<TaskRequirement>,
    pub offer: Vec<TaskOffer>,
    pub config: WorkloadConfig,
}

impl RawTaskInputSource {
    pub(crate) fn to_task_input_source(
        self,
        ipfs_cid: Option<String>,
    ) -> anyhow::Result<TaskInputSource> {
        match self {
            RawTaskInputSource::Ipfs { hash, file } => {
                if hash.is_some() && file.is_some() {
                    anyhow::bail!("Both hash and file are set");
                } else if let Some(hash) = hash {
                    Ok(TaskInputSource::Ipfs { hash })
                } else if file.is_some() {
                    let ipfs_cid = ipfs_cid.ok_or_else(|| anyhow!("IPFS CID is not set"))?;

                    Ok(TaskInputSource::Ipfs { hash: ipfs_cid })
                } else {
                    anyhow::bail!("Neither hash nor file are set");
                }
            }
            RawTaskInputSource::Link { url } => Ok(TaskInputSource::Link { url }),
        }
    }
}

impl RawTaskConfig {
    pub fn to_task_config(self, ipfs_cid: Option<String>) -> anyhow::Result<TaskConfig> {
        let input = self.input.to_task_input_source(ipfs_cid)?;

        Ok(TaskConfig {
            input,
            output: self.output,
            requirements: self.requirements,
            offer: self.offer,
            config: self.config,
        })
    }
}

use poco_types::types::task::{TaskOffer, TaskOutputSource, TaskRequirement, WorkloadConfig};
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

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use lazy_static::lazy_static;
use poco_types::types::task::{OnChainTaskConfig, TaskConfig};

use crate::agent::task::config::DomainTaskConfig;

pub mod media;

pub type ActuatorPredicate = fn(&TaskConfig) -> bool;

pub trait TaskConfigFactory {
    type Config: DomainTaskConfig;

    fn parse_from_json_value(&self, value: serde_json::Value) -> anyhow::Result<Self::Config> {
        let config = serde_json::from_value(value)?;

        Ok(config)
    }

    fn parse_from_json_str(&self, json: &str) -> anyhow::Result<Self::Config> {
        let config = serde_json::from_str(json)?;

        Ok(config)
    }

    fn parse_from_bytes(&self, bytes: &[u8]) -> anyhow::Result<Self::Config> {
        let config = borsh::BorshDeserialize::try_from_slice(bytes)?;

        Ok(config)
    }
}

pub trait TaskExecutor {
    type Output;

    fn execute(&mut self, config: &TaskConfig) -> anyhow::Result<Self::Output>;
}

pub trait TaskVerifier {
    type Input;
    type Output;

    fn verify(&mut self, input: Self::Input) -> anyhow::Result<Self::Output>;
}

pub trait TaskMeasurer {
    type Input;
    type Output;

    fn measure(&mut self, input: Self::Input) -> anyhow::Result<Self::Output>;
}

#[async_trait]
pub trait TaskActuator: Send + Sync {
    async fn execute(&mut self, config: &OnChainTaskConfig) -> anyhow::Result<()>;

    fn encode_task_config(&self, config: serde_json::Value) -> anyhow::Result<Vec<u8>>;

    fn r#type(&self) -> &'static str;
}

#[derive(Clone)]
pub struct BoxedTaskActuator {
    inner: Arc<dyn TaskActuator>,
}

impl BoxedTaskActuator {
    pub fn new<T: TaskActuator + 'static>(actuator: T) -> Self {
        Self {
            inner: Arc::new(actuator),
        }
    }
}

impl Deref for BoxedTaskActuator {
    type Target = dyn TaskActuator;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

lazy_static! {
    pub static ref ACTUATORS: RwLock<HashMap<String, BoxedTaskActuator>> =
        RwLock::new(HashMap::new());
}

pub fn register_actuator(r#type: &str, actuator: BoxedTaskActuator) -> anyhow::Result<()> {
    let mut mp = ACTUATORS.write().unwrap();

    if mp.contains_key(r#type) {
        anyhow::bail!("Actuator {} already registered", r#type);
    }

    mp.insert(r#type.to_string(), actuator);

    Ok(())
}

pub fn get_actuator(r#type: &str) -> Option<BoxedTaskActuator> {
    let mp = ACTUATORS.read().unwrap();

    mp.get(r#type).cloned()
}

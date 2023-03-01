// use std::collections::HashMap;
// use std::ops::Deref;
// use std::sync::{Arc, RwLock};
//
// use async_trait::async_trait;
// use lazy_static::lazy_static;
// use poco_types::types::task::{OnChainTaskConfig, RawTaskConfig};
//
// use crate::config::DomainTaskConfig;
//
// pub mod config;
// pub mod media;
//
// pub type ActuatorPredicate = fn(&TaskConfig) -> bool;
//
// pub trait TaskConfigFactory {
//     type Config: DomainTaskConfig;
//
//     fn parse_from_json_value(&self, value: serde_json::Value) -> anyhow::Result<Self::Config> {
//         let config = serde_json::from_value(value)?;
//
//         Ok(config)
//     }
//
//     fn parse_from_json_str(&self, json: &str) -> anyhow::Result<Self::Config> {
//         let config = serde_json::from_str(json)?;
//
//         Ok(config)
//     }
//
//     fn parse_from_bytes(&self, bytes: &[u8]) -> anyhow::Result<Self::Config> {
//         let config = borsh::BorshDeserialize::try_from_slice(bytes)?;
//
//         Ok(config)
//     }
// }
//
// pub trait TaskExecutor {
//     type Output;
//
//     fn execute(&mut self, config: &TaskConfig) -> anyhow::Result<Self::Output>;
// }
//
// pub trait TaskVerifier {
//     type Input;
//     type Output;
//
//     fn verify(&mut self, input: Self::Input) -> anyhow::Result<Self::Output>;
// }
//
// pub trait TaskMeasurer {
//     type Input;
//     type Output;
//
//     fn measure(&mut self, input: Self::Input) -> anyhow::Result<Self::Output>;
// }
//
// #[async_trait]
// pub trait InnerTaskService: Send + Sync {
//     async fn execute(&mut self, config: &OnChainTaskConfig) -> anyhow::Result<()>;
//
//     fn encode_task_config(&self, config: serde_json::Value) -> anyhow::Result<Vec<u8>>;
//
//     fn r#type(&self) -> &'static str;
// }
//
// #[derive(Clone)]
// pub struct TaskService {
//     inner: Arc<dyn InnerTaskService>,
// }
//
// impl TaskService {
//     pub fn new<T: InnerTaskService + 'static>(actuator: T) -> Self {
//         Self {
//             inner: Arc::new(actuator),
//         }
//     }
// }
//
// impl Deref for TaskService {
//     type Target = dyn InnerTaskService;
//
//     fn deref(&self) -> &Self::Target {
//         self.inner.deref()
//     }
// }
//
// lazy_static! {
//     pub static ref GLOBAL_SERVICES: RwLock<HashMap<String, TaskService>> =
//         RwLock::new(HashMap::new());
// }
//
// pub fn register_service(r#type: &str, service: Box<dyn InnerTaskService>) -> anyhow::Result<()> {
//     let mut mp = GLOBAL_SERVICES.write().unwrap();
//
//     if mp.contains_key(r#type) {
//         anyhow::bail!("Actuator {} already registered", r#type);
//     }
//
//     mp.insert(r#type.to_string(), service.into());
//
//     Ok(())
// }
//
// pub fn lookup_service(r#type: &str) -> Option<TaskService> {
//     let mp = GLOBAL_SERVICES.read().unwrap();
//
//     mp.get(r#type).cloned()
// }

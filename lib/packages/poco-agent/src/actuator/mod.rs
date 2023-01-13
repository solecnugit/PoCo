use poco_types::types::task::TaskConfig;
use serde::{Serialize};
use serde::de::DeserializeOwned;

#[marker] pub trait DomainTaskConfig: Serialize + DeserializeOwned {}

pub trait PocoTaskActuator<C>
where C : DomainTaskConfig {

}

pub trait TaskConfigParser {
    type Output: DomainTaskConfig;

    fn parse(&self) -> anyhow::Result<Self::Output>;
}

pub trait TaskExecutor {
    type Output;

    fn execute(&mut self, config: &(TaskConfig, impl DomainTaskConfig)) -> anyhow::Result<Self::Output>;
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
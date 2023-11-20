use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use near_crypto::{InMemorySigner, PublicKey};
use near_jsonrpc_client::methods::network_info::RpcNetworkInfoResponse;
use near_jsonrpc_client::methods::status::RpcStatusResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryResponse};
use near_primitives::transaction::FunctionCallAction;
use near_primitives::transaction::{Action, Transaction};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality, Gas};
use near_primitives::views::{AccessKeyView, AccountView, FinalExecutionStatus, QueryRequest};
use poco_types::types::event::IndexedEvent;
use poco_types::types::round::{RoundId, RoundInfo, RoundStatus};
use poco_types::types::task::id::TaskId;
use poco_types::types::task::TaskConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use strum::Display;

use crate::agent::PocoAgentError::{
    UnexpectedResponseData, UnexpectedResponseKind, UnexpectedTxExecutionStatus,
};
use crate::config::PocoAgentConfig;

pub struct PocoAgent {
    config: Arc<PocoAgentConfig>,
    inner: JsonRpcClient,
    signer: InMemorySigner,
    contract_id: AccountId,
}

#[derive(Debug, Display)]
pub enum ArgsType {
    JSON,
    TEXT,
    BASE64,
}

#[derive(thiserror::Error, Debug)]
pub enum PocoAgentBuildError {
    #[error("Failed to build reqwest client")]
    ReqwestBuildError(#[from] reqwest::Error),
    #[error("Failed to parse account id {0}")]
    AccountIdParseError(String),
    #[error("Failed to parse secret key {0}")]
    SecretKeyParseError(String),
    #[error("Failed to parse contract id {0}")]
    ContractIdParseError(String),
}

#[derive(Debug, Display)]
pub enum TxExecutionStatus {
    NotStarted,
    Started,
    Failure,
    Success,
}

impl From<FinalExecutionStatus> for TxExecutionStatus {
    fn from(value: FinalExecutionStatus) -> Self {
        match value {
            FinalExecutionStatus::NotStarted => TxExecutionStatus::NotStarted,
            FinalExecutionStatus::Started => TxExecutionStatus::Started,
            FinalExecutionStatus::Failure(_) => TxExecutionStatus::Failure,
            FinalExecutionStatus::SuccessValue(_) => TxExecutionStatus::Success,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PocoAgentError {
    #[error("JsonRpc error: {0}")]
    JsonRpcError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Unexpected response kind")]
    UnexpectedResponseKind,
    #[error("Unexpected response data: {0}")]
    UnexpectedResponseData(String),
    #[error("Unexpected transaction status: {0}")]
    UnexpectedTxExecutionStatus(TxExecutionStatus),
    #[error("Transaction execution error: {0}")]
    TxExecutionError(#[from] near_primitives::errors::TxExecutionError),
    #[error("Failed to parse json: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl<T: Debug + Display + Send + Sync + 'static> From<near_jsonrpc_client::errors::JsonRpcError<T>>
    for PocoAgentError
{
    fn from(e: near_jsonrpc_client::errors::JsonRpcError<T>) -> Self {
        PocoAgentError::JsonRpcError(Box::new(e))
    }
}

impl PocoAgent {
    pub fn build(config: Arc<PocoAgentConfig>) -> Result<Self, PocoAgentBuildError> {
        let mut headers = reqwest::header::HeaderMap::with_capacity(1);

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .connection_verbose(config.verbose)
            .connect_timeout(Duration::from_millis(config.connection_timeout_in_ms))
            .default_headers(headers)
            .build()?;

        let rpc_client = JsonRpcClient::with(client).connect(&config.near_rpc_endpoint);

        let account_id = config.near_signer_account_id.parse().map_err(|_| {
            PocoAgentBuildError::AccountIdParseError(config.near_signer_account_id.to_string())
        })?;

        let secret_key = config.near_signer_secret_key.parse().map_err(|_| {
            PocoAgentBuildError::SecretKeyParseError(config.near_signer_secret_key.to_string())
        })?;

        let signer = InMemorySigner::from_secret_key(account_id, secret_key);

        let contract_id = config.poco_contract_account.parse().map_err(|_| {
            PocoAgentBuildError::AccountIdParseError(config.poco_contract_account.to_string())
        })?;

        Ok(PocoAgent {
            config,
            inner: rpc_client,
            signer,
            contract_id,
        })
    }

    pub async fn gas_price(&self) -> Result<Balance, PocoAgentError> {
        let request = methods::gas_price::RpcGasPriceRequest { block_id: None };

        let response = self.inner.call(request).await?;
        let gas_price = response.gas_price;

        Ok(gas_price)
    }

    pub async fn network_status(&self) -> Result<RpcNetworkInfoResponse, PocoAgentError> {
        let request = methods::network_info::RpcNetworkInfoRequest;
        let response = self.inner.call(request).await?;

        Ok(response)
    }

    pub async fn status(&self) -> Result<RpcStatusResponse, PocoAgentError> {
        let request = methods::status::RpcStatusRequest;
        let response = self.inner.call(request).await?;

        Ok(response)
    }

    pub async fn view_account(&self, account_id: AccountId) -> Result<AccountView, PocoAgentError> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccount { account_id },
        };

        let response = self.inner.call(request).await?;

        if let QueryResponseKind::ViewAccount(account_view) = response.kind {
            Ok(account_view)
        } else {
            Err(PocoAgentError::UnexpectedResponseKind)
        }
    }

    pub async fn verify_account(
        &self,
        account_id: AccountId,
        public_key: PublicKey,
    ) -> Result<AccessKeyView, PocoAgentError> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key,
            },
        };

        let response = self.inner.call(request).await?;

        if let QueryResponseKind::AccessKey(access_key_view) = response.kind {
            Ok(access_key_view)
        } else {
            Err(PocoAgentError::UnexpectedResponseKind)
        }
    }

    fn encode_args(&self, args: &str, r#type: ArgsType) -> Vec<u8> {
        match r#type {
            ArgsType::JSON => serde_json::Value::from_str(args)
                .unwrap()
                .to_string()
                .into_bytes(),
            ArgsType::TEXT => args.to_string().into_bytes(),
            ArgsType::BASE64 => base64::engine::general_purpose::STANDARD
                .decode(args.as_bytes())
                .unwrap(),
        }
    }

    fn get_buffer_from_call_response(
        response: RpcQueryResponse,
    ) -> Result<Vec<u8>, PocoAgentError> {
        if let QueryResponseKind::CallResult(call_result) = response.kind {
            Ok(call_result.result)
        } else {
            Err(PocoAgentError::UnexpectedResponseKind)
        }
    }

    async fn call_view_function(
        &self,
        method_name: &str,
        args: &str,
        r#type: ArgsType,
    ) -> Result<Vec<u8>, PocoAgentError> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: self.contract_id.clone(),
                method_name: method_name.to_string(),
                args: self.encode_args(args, r#type).into(),
            },
        };

        let response = self.inner.call(request).await?;

        Self::get_buffer_from_call_response(response)
    }

    async fn call_change_function(
        &self,
        method_name: &str,
        args: &str,
        r#type: ArgsType,
        gas: u64,
        deposit: u128,
    ) -> Result<(Gas, Vec<u8>), PocoAgentError> {
        let access_key_response = self
            .inner
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            })
            .await?;

        let current_nonce = match access_key_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => Err(UnexpectedResponseKind)?,
        };

        let transaction = Transaction {
            signer_id: self.signer.account_id.clone(),
            public_key: self.signer.public_key.clone(),
            nonce: current_nonce + 1,
            receiver_id: self.contract_id.clone(),
            block_hash: access_key_response.block_hash,
            actions: vec![Action::FunctionCall(FunctionCallAction {
                method_name: method_name.to_string(),
                args: self.encode_args(args, r#type),
                gas,
                deposit,
            })],
        };

        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: transaction.sign(&self.signer),
        };

        let response = self.inner.call(request).await?;

        let mut gas_burnt = response
            .receipts_outcome
            .iter()
            .map(|e| e.outcome.gas_burnt)
            .sum();

        gas_burnt += response.transaction_outcome.outcome.gas_burnt;

        match response.status {
            FinalExecutionStatus::SuccessValue(buffer) => Ok((gas_burnt, buffer)),
            FinalExecutionStatus::Failure(error) => Err(error)?,
            _ => Err(UnexpectedTxExecutionStatus(response.status.into())),
        }
    }

    pub async fn call_view_function_json<T, R>(
        &self,
        method_name: &str,
        args: &T,
    ) -> Result<R, PocoAgentError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let args = serde_json::to_string(args).unwrap();

        let buffer = self
            .call_view_function(method_name, args.as_str(), ArgsType::JSON)
            .await?;

        let result = serde_json::from_slice(buffer.as_slice()).unwrap();

        Ok(result)
    }

    pub async fn call_change_function_json<T, R>(
        &self,
        method_name: &str,
        args: &T,
        gas: u64,
        deposit: u128,
    ) -> Result<(Gas, R), PocoAgentError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let args = serde_json::to_string(args)?;

        let (gas, buffer) = self
            .call_change_function(method_name, args.as_str(), ArgsType::JSON, gas, deposit)
            .await?;

        let json = serde_json::from_slice(buffer.as_slice());

        match json {
            Ok(result) => Ok((gas, result)),
            Err(_) => Err(UnexpectedResponseData(String::from_utf8(buffer).unwrap())),
        }
    }

    pub async fn call_change_function_json_no_response<T>(
        &self,
        method_name: &str,
        args: &T,
        gas: u64,
        deposit: u128,
    ) -> Result<Gas, PocoAgentError>
    where
        T: Serialize,
    {
        let args = serde_json::to_string(args)?;

        let (gas, _) = self
            .call_change_function(method_name, args.as_str(), ArgsType::JSON, gas, deposit)
            .await?;

        Ok(gas)
    }

    pub async fn get_round_status(&self) -> Result<RoundStatus, PocoAgentError> {
        let response = self
            .call_view_function_json("get_round_status", &json!({}))
            .await?;

        Ok(response)
    }

    pub async fn get_round_info(&self) -> Result<RoundInfo, PocoAgentError> {
        let response = self
            .call_view_function_json("get_round_info", &json!({}))
            .await?;

        Ok(response)
    }

    pub async fn count_events(&self) -> Result<u32, PocoAgentError> {
        let response = self
            .call_view_function_json("count_events", &json!({}))
            .await?;

        Ok(response)
    }
    
    pub async fn count_tasks(&self) -> Result<u32, PocoAgentError> {
        let response = self
            .call_view_function_json("count_tasks", &json!({}))
            .await?;

        Ok(response)
    }

    pub async fn query_events(
        &self,
        from: u32,
        count: u32,
    ) -> Result<Vec<IndexedEvent>, PocoAgentError> {
        let response = self
            .call_view_function_json(
                "query_events",
                &json!({
                    "from" : from,
                    "count" : count
                }),
            )
            .await?;

        Ok(response)
    }

    pub async fn get_user_endpoint(
        &self,
        account_id: Option<AccountId>,
    ) -> Result<Option<String>, PocoAgentError> {
        self.call_view_function_json("get_user_endpoint", &json!({ "account_id": account_id }))
            .await
    }

    pub async fn set_user_endpoint(&self, endpoint: &str) -> Result<Gas, PocoAgentError> {
        self.call_change_function_json_no_response(
            "set_user_endpoint",
            &json!({ "endpoint": endpoint }),
            10_000_000_000_000,
            0,
        )
        .await
    }

    pub async fn start_new_round(&self) -> Result<(Gas, RoundId), PocoAgentError> {
        self.call_change_function_json("start_new_round", &json!({}), 10_000_000_000_000, 0)
            .await
    }

    pub async fn publish_task(
        &self,
        task_config: TaskConfig,
    ) -> Result<(Gas, TaskId), PocoAgentError> {
        #[derive(Serialize)]
        struct WrappedTaskConfig {
            config: TaskConfig,
        }

        let task_config = WrappedTaskConfig {
            config: task_config,
        };

        let result = self
            .call_change_function_json::<WrappedTaskConfig, TaskId>(
                "publish_task",
                &task_config,
                10_000_000_000_000,
                0,
            )
            .await?;

        Ok(result)
    }
}

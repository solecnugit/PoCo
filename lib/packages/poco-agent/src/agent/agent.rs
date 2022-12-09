use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use map_macro::map;
use near_crypto::PublicKey;
use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::gas_price::RpcGasPriceError;
use near_jsonrpc_client::methods::network_info::{RpcNetworkInfoError, RpcNetworkInfoResponse};
use near_jsonrpc_client::methods::query::RpcQueryError;
use near_jsonrpc_client::methods::status::{RpcStatusError, RpcStatusResponse};
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryResponse};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality};
use near_primitives::views::{AccessKeyView, AccountView, QueryRequest};
use poco_types::types::event::IndexedEvent;
use poco_types::types::round::RoundStatus;
use serde::de::DeserializeOwned;
use serde::Serialize;
use strum::Display;

use crate::config::PocoAgentConfig;

pub struct PocoAgent {
    config: Arc<PocoAgentConfig>,
    inner: JsonRpcClient,
}

#[derive(Debug, Display)]
pub enum ArgsType {
    JSON,
    TEXT,
    BASE64,
}

impl PocoAgent {
    pub fn new(config: Arc<PocoAgentConfig>) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(2);

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .connection_verbose(config.app.verbose)
            .connect_timeout(Duration::from_millis(config.app.connection_timeout))
            .default_headers(headers)
            .build()
            .unwrap();

        let rpc_client = JsonRpcClient::with(client).connect(config.near.rpc_endpoint.as_str());

        PocoAgent { config, inner: rpc_client }
    }

    pub async fn gas_price(&self) -> Result<Balance, JsonRpcError<RpcGasPriceError>> {
        let request = methods::gas_price::RpcGasPriceRequest { block_id: None };

        let response = self.inner.call(request).await?;
        let gas_price = response.gas_price;

        Ok(gas_price)
    }

    pub async fn network_status(
        &self,
    ) -> Result<RpcNetworkInfoResponse, JsonRpcError<RpcNetworkInfoError>> {
        let request = methods::network_info::RpcNetworkInfoRequest;
        let response = self.inner.call(request).await?;

        Ok(response)
    }

    pub async fn status(&self) -> Result<RpcStatusResponse, JsonRpcError<RpcStatusError>> {
        let request = methods::status::RpcStatusRequest;
        let response = self.inner.call(request).await?;

        Ok(response)
    }

    pub async fn view_account(
        &self,
        account_id: AccountId,
    ) -> Result<AccountView, JsonRpcError<RpcQueryError>> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccount { account_id },
        };

        let response = self.inner.call(request).await?;

        if let QueryResponseKind::ViewAccount(account_view) = response.kind {
            Ok(account_view)
        } else {
            Err(JsonRpcError::ServerError(
                JsonRpcServerError::InternalError {
                    info: "Unexpected response".to_string().into(),
                },
            ))
        }
    }

    pub async fn verify_account(
        &self,
        account_id: AccountId,
        public_key: PublicKey,
    ) -> Result<AccessKeyView, JsonRpcError<RpcQueryError>> {
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
            Err(JsonRpcError::ServerError(
                JsonRpcServerError::InternalError {
                    info: "Unexpected response".to_string().into(),
                },
            ))
        }
    }

    fn encode_args(&self, args: String, r#type: ArgsType) -> Vec<u8> {
        match r#type {
            ArgsType::JSON => serde_json::Value::from_str(&args)
                .unwrap()
                .to_string()
                .into_bytes(),
            ArgsType::TEXT => args.into_bytes(),
            ArgsType::BASE64 => base64::decode(&args.as_bytes()).unwrap(),
        }
    }

    async fn call_view_function(
        &self,
        account_id: AccountId,
        method_name: String,
        args: String,
        r#type: ArgsType,
    ) -> Result<Vec<u8>, JsonRpcError<RpcQueryError>> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id,
                method_name,
                args: self.encode_args(args, r#type).into(),
            },
        };

        let response = self.inner.call(request).await?;

        Self::get_buffer_from_call_response(response)
    }

    fn get_buffer_from_call_response(
        response: RpcQueryResponse,
    ) -> Result<Vec<u8>, JsonRpcError<RpcQueryError>> {
        if let QueryResponseKind::CallResult(call_result) = response.kind {
            Ok(call_result.result)
        } else {
            Err(JsonRpcError::ServerError(
                JsonRpcServerError::InternalError {
                    info: "Unexpected response".to_string().into(),
                },
            ))
        }
    }

    async fn call_change_function(
        &self,
        account_id: AccountId,
        method_name: String,
        args: String,
        r#type: ArgsType,
    ) -> Result<Vec<u8>, JsonRpcError<RpcQueryError>> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id,
                method_name,
                args: self.encode_args(args, r#type).into(),
            },
        };

        let response = self.inner.call(request).await?;

        Self::get_buffer_from_call_response(response)
    }

    pub async fn call_view_function_json<T, R>(
        &self,
        account_id: AccountId,
        method_name: String,
        args: T,
    ) -> Result<R, JsonRpcError<RpcQueryError>>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let args = serde_json::to_string(&args).unwrap();

        let buffer = self
            .call_view_function(account_id, method_name, args, ArgsType::JSON)
            .await?;

        let result = serde_json::from_slice(buffer.as_slice()).unwrap();

        Ok(result)
    }

    pub async fn call_function_json<T, R>(
        &self,
        account_id: AccountId,
        method_name: String,
        args: T,
    ) -> Result<R, JsonRpcError<RpcQueryError>>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let args = serde_json::to_string(&args).unwrap();

        let buffer = self
            .call_change_function(account_id, method_name, args, ArgsType::JSON)
            .await?;

        let result = serde_json::from_slice(buffer.as_slice()).unwrap();

        Ok(result)
    }

    pub async fn get_round_status(&self) -> Result<RoundStatus, JsonRpcError<RpcQueryError>> {
        let response = self
            .call_view_function_json(
                self.config.poco.poco_contract_account.parse().unwrap(),
                "get_round_status".to_string(),
                (),
            )
            .await?;

        Ok(response)
    }

    pub async fn count_events(&self) -> Result<u32, JsonRpcError<RpcQueryError>> {
        let response = self
            .call_view_function_json(
                self.config.poco.poco_contract_account.parse().unwrap(),
                "count_events".to_string(),
                (),
            )
            .await?;

        Ok(response)
    }

    pub async fn query_events(
        &self,
        from: u32,
        count: u32,
    ) -> Result<Vec<IndexedEvent>, JsonRpcError<RpcQueryError>> {
        let response = self
            .call_view_function_json(
                self.config.poco.poco_contract_account.parse().unwrap(),
                "query_events".to_string(),
                map! {
                    "from" => from,
                    "count" => count,
                },
            )
            .await?;

        Ok(response)
    }
}

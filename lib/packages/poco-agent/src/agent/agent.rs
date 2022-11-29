use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::gas_price::RpcGasPriceError;
use near_jsonrpc_client::methods::network_info::{RpcNetworkInfoError, RpcNetworkInfoResponse};
use near_jsonrpc_client::methods::query::RpcQueryError;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{AccountId, Balance, BlockReference, Finality};
use near_primitives::views::{AccountView, QueryRequest};

pub struct PocoAgent {
    runtime: tokio::runtime::Runtime,
    near_client: Option<JsonRpcClient>,
}

impl PocoAgent {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        PocoAgent {
            runtime,
            near_client: None,
        }
    }

    pub fn connect(&mut self, rpc_endpoint: String) -> &Self {
        let near_rpc_client = JsonRpcClient::connect(rpc_endpoint);
        self.near_client = Some(near_rpc_client);
        self
    }

    pub fn get_runtime(&self) -> &tokio::runtime::Runtime {
        &self.runtime
    }

    pub fn get_near_rpc_client(&self) -> &JsonRpcClient {
        self.near_client.as_ref().unwrap()
    }

    pub async fn gas_price(&self) -> Result<Balance, JsonRpcError<RpcGasPriceError>> {
        let client = self.get_near_rpc_client();
        let request = methods::gas_price::RpcGasPriceRequest { block_id: None };

        let response = client.call(request).await?;
        let gas_price = response.gas_price;

        Ok(gas_price)
    }

    pub async fn network_status(
        &self,
    ) -> Result<RpcNetworkInfoResponse, JsonRpcError<RpcNetworkInfoError>> {
        let client = self.get_near_rpc_client();

        let request = methods::network_info::RpcNetworkInfoRequest;
        let response = client.call(request).await?;

        Ok(response)
    }

    pub async fn view_account(
        &self,
        account_id: AccountId,
    ) -> Result<AccountView, JsonRpcError<RpcQueryError>> {
        let client = self.get_near_rpc_client();

        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccount { account_id },
        };

        let response = client.call(request).await?;

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
}

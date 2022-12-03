use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::gas_price::RpcGasPriceError;
use near_jsonrpc_client::methods::network_info::{RpcNetworkInfoError, RpcNetworkInfoResponse};
use near_jsonrpc_client::methods::query::RpcQueryError;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{AccountId, Balance, BlockReference, Finality};
use near_primitives::views::{AccountView, QueryRequest};

pub struct PocoAgent {
    rpc_client: JsonRpcClient,
}

impl PocoAgent {
    pub fn connect(rpc_endpoint: &str) -> Self {
        let rpc_client = JsonRpcClient::connect(rpc_endpoint);

        PocoAgent { rpc_client }
    }

    pub async fn gas_price(&self) -> Result<Balance, JsonRpcError<RpcGasPriceError>> {
        let request = methods::gas_price::RpcGasPriceRequest { block_id: None };

        let response = self.rpc_client.call(request).await?;
        let gas_price = response.gas_price;

        Ok(gas_price)
    }

    pub async fn network_status(
        &self,
    ) -> Result<RpcNetworkInfoResponse, JsonRpcError<RpcNetworkInfoError>> {
        let request = methods::network_info::RpcNetworkInfoRequest;
        let response = self.rpc_client.call(request).await?;

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

        let response = self.rpc_client.call(request).await?;

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

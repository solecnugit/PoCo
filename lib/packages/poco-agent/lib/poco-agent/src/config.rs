use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PocoAgentConfig {
    pub verbose: bool,
    pub connection_timeout_in_ms: u64,
    pub near_rpc_endpoint: String,
    pub near_signer_account_id: String,
    pub near_signer_secret_key: String,
    pub poco_contract_account: String,
}

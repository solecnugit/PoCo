use near_jsonrpc_client::JsonRpcClient;
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
        let near_client = self.runtime.block_on(async {
            JsonRpcClient::connect(rpc_endpoint)
        });

        self.near_client = Some(near_client);
        self
    }

    pub fn get_runtime(&self) -> &tokio::runtime::Runtime {
        &self.runtime
    }

    pub fn get_near_client(&self) -> &JsonRpcClient {
        self.near_client.as_ref().unwrap()
    }
}

use futures::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, TryFromUri};

use std::sync::Arc;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct IpfsClient {
    inner: Arc<ipfs_api_backend_hyper::IpfsClient>,
}

#[derive(Debug)]
pub enum IpfsClientError {
    InvalidUrl(String),
    IoError(std::io::Error),
    InnerError(ipfs_api_backend_hyper::Error),
}

impl From<std::io::Error> for IpfsClientError {
    fn from(e: std::io::Error) -> Self {
        IpfsClientError::IoError(e)
    }
}

impl From<ipfs_api_backend_hyper::Error> for IpfsClientError {
    fn from(e: ipfs_api_backend_hyper::Error) -> Self {
        IpfsClientError::InnerError(e)
    }
}

impl IpfsClient {
    pub fn create_ipfs_client(ipfs_endpoint: &str) -> Result<Self, IpfsClientError> {
        let client = if let Ok(client) = ipfs_api_backend_hyper::IpfsClient::from_str(ipfs_endpoint)
        {
            client
        } else {
            return Err(IpfsClientError::InvalidUrl(ipfs_endpoint.to_string()));
        };

        let inner = Arc::new(client);

        Ok(Self { inner })
    }

    pub async fn add_file(&self, file_path: &str) -> Result<String, IpfsClientError> {
        let file = tokio::fs::File::open(file_path).await?;
        let file = file.compat();
        let file = self.inner.add_async(file).await?;

        Ok(file.hash)
    }

    pub async fn cat_file(&self, hash: &str) -> Result<Vec<u8>, IpfsClientError> {
        let buffer = self
            .inner
            .cat(hash)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        Ok(buffer)
    }

    pub async fn cat_file_range(
        &self,
        hash: &str,
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>, IpfsClientError> {
        let buffer = self
            .inner
            .cat_range(hash, offset, length)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        Ok(buffer)
    }
}

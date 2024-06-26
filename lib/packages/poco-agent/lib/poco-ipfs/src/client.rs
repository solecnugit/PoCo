use std::error::Error;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use futures::{TryFutureExt, TryStreamExt};
use ipfs_api_backend_hyper::response::ObjectStatResponse;
use ipfs_api_backend_hyper::{IpfsApi, TryFromUri};
use tokio::io::AsyncWriteExt;
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::config::PoCoIpfsConfig;

pub struct IpfsClient {
    inner: Arc<ipfs_api_backend_hyper::IpfsClient>,
}

pub struct GetFileProgress {
    downloaded_size_in_bytes: u64,
    total_size_in_bytes: u64,
}

impl Default for GetFileProgress {
    fn default() -> Self {
        GetFileProgress {
            downloaded_size_in_bytes: 0,
            total_size_in_bytes: 0,
        }
    }
}

impl GetFileProgress {
    pub fn new(downloaded_size_in_bytes: u64, total_size_in_bytes: u64) -> Self {
        GetFileProgress {
            downloaded_size_in_bytes,
            total_size_in_bytes,
        }
    }

    #[inline]
    pub fn downloaded_size_in_bytes(&self) -> u64 {
        self.downloaded_size_in_bytes
    }

    #[inline]
    pub fn total_size_in_bytes(&self) -> u64 {
        self.total_size_in_bytes
    }
}

#[derive(Debug)]
pub enum IpfsClientError {
    InvalidUrlError(String),
    InvalidHashError(String),
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

impl Display for IpfsClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpfsClientError::InvalidUrlError(url) => write!(f, "invalid url: {url}"),
            IpfsClientError::InvalidHashError(hash) => write!(f, "invalid hash: {hash}"),
            IpfsClientError::IoError(e) => write!(f, "io error: {e}"),
            IpfsClientError::InnerError(e) => write!(f, "inner error: {e}"),
        }
    }
}

impl Error for IpfsClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IpfsClientError::InvalidUrlError(_) => None,
            IpfsClientError::InvalidHashError(_) => None,
            IpfsClientError::IoError(e) => Some(e),
            IpfsClientError::InnerError(e) => Some(e),
        }
    }
}

type FileStatus = ObjectStatResponse;

impl IpfsClient {
    pub fn create_ipfs_client(
        config: Arc<PoCoIpfsConfig>,
    ) -> anyhow::Result<Self, IpfsClientError> {
        let ipfs_endpoint = &config.ipfs_endpoint;

        let client = if let Ok(client) = ipfs_api_backend_hyper::IpfsClient::from_str(ipfs_endpoint)
        {
            client
        } else {
            return Err(IpfsClientError::InvalidUrlError(ipfs_endpoint.to_string()));
        };

        let inner = Arc::new(client);

        Ok(Self { inner })
    }

    pub async fn add_file(&self, file_path: impl AsRef<Path>) -> Result<String, IpfsClientError> {
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

    pub async fn get_file<'a>(
        &'a self,
        hash: &'a str,
        path: impl AsRef<Path> + 'a,
        progress_callback_sender: Option<tokio::sync::mpsc::Sender<GetFileProgress>>,
    ) -> Result<(), IpfsClientError> {
        let file_status = self.file_status(hash).await?;
        let file_size = file_status.cumulative_size;
        let file = tokio::fs::File::create(path).await?;

        if let Some(sender) = progress_callback_sender {
            let sender2 = sender.clone();

            self.inner
                .cat(hash)
                .try_fold(
                    (file, 0, sender),
                    |(mut file, mut downloaded, sender), chunk| async move {
                        if let Err(e) = file.write_all(&chunk).await {
                            return Err(ipfs_api_backend_hyper::Error::IpfsClientError(
                                ipfs_api_prelude::Error::Io(e),
                            ));
                        }

                        downloaded += chunk.len() as u64;

                        if let Err(_) = sender
                            .send(GetFileProgress::new(downloaded, file_size))
                            .await
                        {
                            return Err(ipfs_api_backend_hyper::Error::IpfsClientError(
                                ipfs_api_prelude::Error::Io(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "failed to send progress",
                                )),
                            ));
                        }

                        Ok((file, downloaded, sender))
                    },
                )
                .map_err(IpfsClientError::InnerError)
                .await?;

            if let Err(_) = sender2
                .send(GetFileProgress::new(file_size, file_size))
                .await
            {
                return Err(IpfsClientError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to send progress",
                )));
            }
        } else {
            self.inner
                .cat(hash)
                .try_fold(file, |mut file, chunk| async move {
                    if let Err(e) = file.write_all(&chunk).await {
                        return Err(ipfs_api_backend_hyper::Error::IpfsClientError(
                            ipfs_api_prelude::Error::Io(e),
                        ));
                    }

                    Ok(file)
                })
                .map_err(IpfsClientError::InnerError)
                .await?;
        }

        Ok(())
    }

    pub async fn file_status(&self, hash: &str) -> Result<FileStatus, IpfsClientError> {
        let response = self.inner.object_stat(hash).await?;

        Ok(response)
    }
}

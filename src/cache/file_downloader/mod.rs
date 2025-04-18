use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;

mod impl_object_store;
mod local_downloader;

pub use impl_object_store::*;
pub use local_downloader::LocalDownloader;
use serde::{Deserialize, Serialize};

use super::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// The full path to the object
    pub location: String,
    /// The last modified time
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// The size in bytes of the object
    pub size: usize,
    /// The unique identifier for the object
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc9110#name-etag>
    pub e_tag: Option<String>,
}

#[async_trait]
pub trait FileDownloader: Send + Sync {
    /// Get a file from the file provider.
    /// Returns a stream of bytes if the file exists,
    /// otherwise an ResourceNotFound error.
    async fn get_file(
        &self,
        path: &str,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Error>> + Send>>, Error>;

    /// Get metadata for a file from the file provider.
    /// Returns ResourceNotFound error if the file does not exist.
    async fn get_meta(&self, path: &str) -> Result<FileMetadata, Error>;

    /// returns a hash of the credentials of the provider to be used as a cache key
    fn hash(&self) -> Bytes {
        Bytes::new()
    }

    /// Returns if the downloader is the local file system
    fn is_local(&self) -> bool {
        false
    }
}

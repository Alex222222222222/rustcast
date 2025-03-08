use async_trait::async_trait;
use futures::Stream;

use crate::Error;

use super::FileMetadata;

pub struct LocalDownloader {}

#[async_trait]
impl super::FileDownloader for LocalDownloader {
    /// Get a file from the file provider.
    /// Returns a stream of bytes if the file exists, otherwise None.
    async fn get_file(
        &self,
        _: &str,
    ) -> anyhow::Result<Box<dyn Stream<Item = Result<bytes::Bytes, Error>> + Unpin>, Error> {
        // This function should never be called for the local downloader.
        Err(Error::NotImplemented)
    }

    /// Get metadata for a file from the file provider.
    async fn get_meta(&self, _: &str) -> Result<FileMetadata, Error> {
        // This function should never be called for the local downloader.
        Err(Error::NotImplemented)
    }

    /// Returns if the downloader is the local file system
    fn is_local(&self) -> bool {
        true
    }
}

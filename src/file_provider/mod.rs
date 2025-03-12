use std::path::PathBuf;

use async_trait::async_trait;
use futures::Stream;

mod aws;
mod from_config;
mod local;

pub use aws::AwsS3FileProvider;
pub use from_config::build_file_provider;
pub use local::LocalFileProvider;

#[async_trait]
/// A trait for file providers.
/// File providers are responsible for fetching files from a remote source and caching them locally.
/// File providers should be able to provide a stream of bytes for a given file path.
///
/// Local file providers should not create a local cache,
/// but should be able to provide a stream of bytes for a given file path.
pub trait FileProvider: Send + Sync {
    /// Get a file local cache path from the file provider.
    /// return the local cache path if the file exists, otherwise None.
    async fn get_local_cache_path(&self, path: &str) -> anyhow::Result<Option<PathBuf>>;

    /// Get a file from the file provider.
    /// Returns a stream of bytes if the file exists, otherwise None.
    async fn get_file(
        &self,
        path: &str,
    ) -> anyhow::Result<Option<Box<dyn tokio::io::AsyncRead + Send + Sync + Unpin>>> {
        let path = self.get_local_cache_path(path).await?;
        let path = match path {
            Some(p) => p,
            None => return Ok(None),
        };
        let file = tokio::fs::File::open(path).await?;

        Ok(Some(Box::new(file)))
    }

    /// Get meta of a file from the file provider.
    /// Returns the meta if the file exists, otherwise None.
    async fn get_meta(&self, path: &str) -> anyhow::Result<Option<cache::FileMetadata>>;

    /// List files in a directory.
    /// Returns iterator of file paths.
    async fn list_files(
        &self,
        path: Option<String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = anyhow::Result<String>> + Unpin + Send>>;
}

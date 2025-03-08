use async_trait::async_trait;
use futures::Stream;

mod impl_object_store;

pub struct FileMetadata {
    pub location: String,
    pub size: usize,
}

#[async_trait]
pub trait FileProvider {
    /// Get a file from the file provider.
    /// Returns a stream of bytes if the file exists, otherwise None.
    async fn get_file(
        &self,
        path: &str,
    ) -> anyhow::Result<Option<Box<dyn Stream<Item = anyhow::Result<bytes::Bytes>>>>>;

    /// Get size of a file from the file provider.
    /// Returns the size if the file exists, otherwise None.
    async fn get_size(&self, path: &str) -> anyhow::Result<Option<usize>>;

    /// List files in a directory.
    /// Returns iterator of file paths.
    async fn list_files(
        &self,
        path: Option<String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = anyhow::Result<String>>>>;
}

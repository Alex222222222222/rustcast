use std::{path::PathBuf, sync::Arc};

use async_stream::stream;
use async_trait::async_trait;
use cache::AwsS3Downloader;
use futures::{Stream, StreamExt};
use log::debug;
use object_store::{ObjectStore, aws::AmazonS3Builder};

use super::FileProvider;

pub struct AwsS3FileProvider {
    cache: Arc<cache::Cache>,
    object_store: Arc<dyn ObjectStore>,
}

impl AwsS3FileProvider {
    pub async fn new(builder: AmazonS3Builder) -> anyhow::Result<Self> {
        let aws_downloader = AwsS3Downloader::new(builder)?;
        let object_store = aws_downloader.get_object_store();
        let cache = cache::Cache::builder()
            .file_downloader(Box::new(aws_downloader))
            .build()
            .await?;
        Ok(Self {
            cache: cache.into(),
            object_store,
        })
    }
}

#[async_trait]
impl FileProvider for AwsS3FileProvider {
    /// Get a file local cache path from the file provider.
    /// return the local cache path if the file exists, otherwise None.
    async fn get_local_cache_path(&self, path: &str) -> anyhow::Result<Option<PathBuf>> {
        match self.cache.cached_path(path).await {
            Ok(p) => Ok(Some(p)),
            Err(cache::Error::ResourceNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_meta(&self, path: &str) -> anyhow::Result<Option<cache::FileMetadata>> {
        let meta = match self.cache.get_file_meta(path).await {
            Ok(m) => m,
            Err(cache::Error::ResourceNotFound(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        Ok(Some(meta))
    }

    async fn list_files<'s, 'p>(
        &'s self,
        path: Option<&'p str>,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send + 'p>>>
    where
        's: 'p,
    {
        debug!("list files in {:?}", path);
        let s = stream! {
            let p = match path {
                Some(p) => Some(object_store::path::Path::parse(p)?),
                None => None,
            };
            let mut s = self.object_store.list(p.as_ref());

            while let Some(meta) = s.next().await {
                yield meta.map(|m| m.location.to_string()).map_err(|e| e.into());
            }
        };

        Ok(Box::pin(s))
    }
}

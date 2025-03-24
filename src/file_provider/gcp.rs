use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use cache::GcpDownloader;
use futures::{Stream, StreamExt};
use log::debug;
use object_store::ObjectStore;

use super::FileProvider;

pub struct GoogleCouldStorageFileProvider {
    cache: Arc<cache::Cache>,
    object_store: Arc<dyn ObjectStore>,
}

impl GoogleCouldStorageFileProvider {
    pub async fn new(
        cache_dir: Option<Arc<String>>,
        builder: object_store::gcp::GoogleCloudStorageBuilder,
    ) -> anyhow::Result<Self> {
        let aws_downloader = GcpDownloader::new(builder)?;
        let object_store = aws_downloader.get_object_store();
        let mut cache = cache::Cache::builder().file_downloader(Box::new(aws_downloader));
        if let Some(dir) = cache_dir {
            let path = PathBuf::from(dir.as_str());
            cache = cache.dir(path);
        };

        let cache = cache.build().await?;
        Ok(Self {
            cache: cache.into(),
            object_store,
        })
    }
}

#[async_trait]
impl FileProvider for GoogleCouldStorageFileProvider {
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
        recursive: bool,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send + 'p>>>
    where
        's: 'p,
    {
        debug!("list files in {:?}", path);
        let p = match path {
            Some(p) => Some(object_store::path::Path::parse(p)?),
            None => None,
        };
        let s = if recursive {
            // the list method is recursive
            self.object_store.list(p.as_ref())
        } else {
            // the list_with_delimiter method is not recursive
            let s = self
                .object_store
                .list_with_delimiter(p.as_ref())
                .await?
                .objects;
            let s = futures::stream::iter(s).map(Ok);
            Box::pin(s)
        };

        let s = s.map(|m| m.map(|r| r.location.to_string()).map_err(|e| e.into()));

        Ok(Box::pin(s))
    }
}

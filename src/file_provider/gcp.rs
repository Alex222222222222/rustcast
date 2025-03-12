use std::{collections::HashMap, path::PathBuf, pin::Pin, sync::Arc, task::Poll};

use async_trait::async_trait;
use cache::GcpDownloader;
use futures::{FutureExt, Stream, StreamExt};
use log::debug;
use object_store::ObjectStore;
use tokio::sync::Mutex;

use crate::CONTEXT;

use super::FileProvider;

pub struct GoogleCouldStorageFileProvider {
    cache: Arc<cache::Cache>,
    object_store: Arc<dyn ObjectStore>,
}

impl GoogleCouldStorageFileProvider {
    pub async fn new(
        builder: object_store::gcp::GoogleCloudStorageBuilder,
    ) -> anyhow::Result<Self> {
        let aws_downloader = GcpDownloader::new(builder)?;
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

    async fn list_files(
        &self,
        path: Option<String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = anyhow::Result<String>> + Unpin + Send>> {
        debug!("list files in {:?}", path);
        let s =
            ObjectStoreListStream2FileProviderStream::new(self.object_store.clone(), path).await?;
        Ok(Box::new(s))
    }
}

type StringOutPending =
    Pin<Box<dyn futures::Future<Output = Option<anyhow::Result<String>>> + std::marker::Send>>;

struct ObjectStoreListStream2FileProviderStream {
    res: usize,
    pending: Option<StringOutPending>,
}

impl ObjectStoreListStream2FileProviderStream {
    async fn new(s: Arc<dyn ObjectStore>, p: Option<String>) -> anyhow::Result<Self> {
        let p = match p {
            Some(p) => Some(object_store::path::Path::parse(&p)?),
            None => None,
        };
        let (sender, res) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            let mut s = s.list(p.as_ref());
            while let Some(meta) = s.next().await {
                if let Err(e) = sender
                    .send(meta.map(|m| m.location.to_string()).map_err(|e| e.into()))
                    .await
                {
                    log::error!("failed to send file path: {}", e);
                    break;
                }
            }
        });
        let id = CONTEXT.get_id().await;
        insert_res(id, res).await;
        Ok(Self {
            res: id,
            pending: None,
        })
    }
}

struct ResMap(Mutex<HashMap<usize, tokio::sync::mpsc::Receiver<anyhow::Result<String>>>>);

impl ResMap {
    fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

static RES_MAP: once_cell::sync::Lazy<ResMap> = once_cell::sync::Lazy::new(ResMap::new);

fn remove_id(id: usize) {
    tokio::spawn(async move {
        let mut map = RES_MAP.0.lock().await;
        map.remove(&id);
    });
}

async fn insert_res(id: usize, res: tokio::sync::mpsc::Receiver<anyhow::Result<String>>) {
    let mut map = RES_MAP.0.lock().await;
    map.insert(id, res);
}

async fn get_res(id: usize) -> Option<anyhow::Result<String>> {
    let mut map = RES_MAP.0.lock().await;
    let r = map.get_mut(&id);
    if let Some(r) = r {
        r.recv().await
    } else {
        None
    }
}

impl Drop for ObjectStoreListStream2FileProviderStream {
    fn drop(&mut self) {
        remove_id(self.res);
    }
}

impl Stream for ObjectStoreListStream2FileProviderStream {
    type Item = anyhow::Result<String>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if let Some(ref mut future) = self.pending {
            // Poll the future and check if it's ready
            match future.as_mut().poll(cx) {
                Poll::Ready(e) => {
                    self.pending = None; // Reset future
                    return Poll::Ready(e);
                }
                Poll::Pending => return Poll::Pending, // Still waiting
            }
        }

        let r = get_res(self.res);
        self.pending = Some(r.boxed());
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

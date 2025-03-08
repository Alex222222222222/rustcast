use std::{collections::HashMap, path, pin::Pin, sync::Arc, task::Poll};

use async_trait::async_trait;
use futures::{FutureExt, Stream, StreamExt};
use object_store::{ObjectMeta, ObjectStore};
use tokio::sync::Mutex;

use crate::CONTEXT;

use super::FileProvider;

#[async_trait]
impl FileProvider for Arc<dyn ObjectStore> {
    async fn get_file(
        &self,
        path: &str,
    ) -> anyhow::Result<Option<Box<dyn Stream<Item = anyhow::Result<bytes::Bytes>>>>> {
        // TODO impl cache
        let path = object_store::path::Path::from(path);
        match self.get(&path).await {
            Ok(file) => Ok(Some(Box::new(ObjectStoreFileStream2FileProviderStream(
                file.into_stream(),
            )))),
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => return Err(e.into()),
        }
    }

    async fn get_size(&self, path: &str) -> anyhow::Result<Option<usize>>
    {
        let path = object_store::path::Path::from(path);
        match self.head(&path).await {
            Ok(meta) => Ok(Some(meta.size)),
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => return Err(e.into()),
        }
    }

    async fn list_files(
        &self,
        path: Option<String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = anyhow::Result<String>>>>
    {
        let r = ObjectStoreListStream2FileProviderStream::new(self.clone(), path).await;
        let r: Box<dyn Stream<Item = anyhow::Result<String>>> = Box::new(r);
        Ok(r)
    }
}

struct ObjectStoreFileStream2FileProviderStream(
    Pin<
        Box<
            (
                dyn futures::Stream<Item = Result<bytes::Bytes, object_store::Error>>
                    + std::marker::Send
                    + 'static
            ),
        >,
    >,
);

struct ObjectStoreListStream2FileProviderStream {
    res: usize,
    pending: Option<
        Pin<Box<dyn futures::Future<Output = Option<anyhow::Result<String>>> + std::marker::Send>>,
    >,
}

impl Stream for ObjectStoreFileStream2FileProviderStream {
    type Item = anyhow::Result<bytes::Bytes>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.0.poll_next_unpin(cx) {
            std::task::Poll::Ready(Some(Ok(bytes))) => std::task::Poll::Ready(Some(Ok(bytes))),
            std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(e.into()))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl ObjectStoreListStream2FileProviderStream {
    async fn new(s: Arc<dyn ObjectStore>, p: Option<String>) -> Self {
        let p = p.map(|p| object_store::path::Path::from(p));
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
        Self {
            res: id,
            pending: None,
        }
    }
}

struct ResMap(Mutex<HashMap<usize, tokio::sync::mpsc::Receiver<anyhow::Result<String>>>>);

impl ResMap {
    fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

static RES_MAP: once_cell::sync::Lazy<ResMap> = once_cell::sync::Lazy::new(|| ResMap::new());

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

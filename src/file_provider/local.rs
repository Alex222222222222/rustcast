use std::{path::PathBuf, pin::Pin, sync::Arc, task::Poll};

use async_trait::async_trait;
use futures::Stream;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use super::FileProvider;

pub struct LocalFileProvider {}

impl Default for LocalFileProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalFileProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl FileProvider for LocalFileProvider {
    /// Get a file local cache path from the file provider.
    /// return the local cache path if the file exists, otherwise None.
    async fn get_local_cache_path(&self, path: &str) -> anyhow::Result<Option<PathBuf>> {
        // test if the file exists
        let path = PathBuf::from(path);
        if path.exists() {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    async fn get_meta(&self, path: &str) -> anyhow::Result<Option<cache::FileMetadata>> {
        // test if the file exists
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            let meta = path_buf.metadata()?;
            Ok(Some(cache::FileMetadata {
                size: meta.len() as usize,
                location: path.to_string(),
                last_modified: meta.modified()?.into(),
                e_tag: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_files(
        &self,
        path: Option<String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = anyhow::Result<String>> + Unpin + Send>> {
        let p = match path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from("."),
        };
        let entries = tokio::fs::read_dir(p).await?;
        let stream = tokio_stream::wrappers::ReadDirStream::new(entries);
        let stream = Box::new(MyReadDirStream {
            entries: Arc::new(Mutex::new(Some(Box::new(stream)))),
            pending: None,
        });

        Ok(stream)
    }
}

type Entries = Arc<
    Mutex<Option<Box<dyn Stream<Item = tokio::io::Result<tokio::fs::DirEntry>> + Unpin + Send>>>,
>;
type StringOutPending =
    Pin<Box<dyn futures::Future<Output = Option<anyhow::Result<String>>> + Send>>;

struct MyReadDirStream {
    entries: Entries,

    pending: Option<StringOutPending>,
}

async fn next(s: Entries) -> Option<anyhow::Result<String>> {
    loop {
        // take the data so we can replace it with the new stream
        let mut s_lock = s.lock().await;
        let mut s_d = match s_lock.take() {
            Some(s_d) => s_d,
            None => return None,
        };
        drop(s_lock);

        let e = match s_d.next().await {
            Some(Ok(e)) => e,
            Some(Err(e)) => return Some(Err(e.into())),
            None => return None,
        };

        let t = match e.file_type().await {
            Ok(t) => t,
            Err(e) => return Some(Err(e.into())),
        };
        if t.is_file() {
            return Some(Ok(e.path().to_string_lossy().to_string()));
        }

        let entries = match tokio::fs::read_dir(e.path()).await {
            Ok(e) => e,
            Err(_) => continue,
        };
        let stream = tokio_stream::wrappers::ReadDirStream::new(entries);

        let s_d = s_d.merge(stream);
        // replace the stream
        let mut s_lock = s.lock().await;
        s_lock.replace(Box::new(s_d));
    }
}

impl Stream for MyReadDirStream {
    type Item = anyhow::Result<String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(ref mut p) = this.pending {
            match p.as_mut().poll(cx) {
                Poll::Ready(e) => {
                    this.pending = None;
                    return Poll::Ready(e);
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        let s = this.entries.clone();
        let p = next(s);
        this.pending = Some(Box::pin(p));
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

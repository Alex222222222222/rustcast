use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use std::{collections::VecDeque, path::PathBuf};
use tokio_stream::{StreamExt, wrappers::ReadDirStream};

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

    async fn list_files<'s, 'p>(
        &'s self,
        path: Option<&'p str>,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send + 'p>>>
    where
        's: 'p,
    {
        let p = match path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from("."),
        };
        let e = tokio::fs::read_dir(p).await?;
        let mut e: std::pin::Pin<
            Box<dyn Stream<Item = Result<tokio::fs::DirEntry, tokio::io::Error>> + Send>,
        > = Box::pin(ReadDirStream::new(e));
        let mut dirs = VecDeque::new();
        let s = stream! {
            loop {
                let d = dirs.pop_front();
                if let Some(d) = d {
                    let new_e = match tokio::fs::read_dir(d).await {
                        Ok(e) => e,
                        Err(e) => {
                            yield Err(e.into());
                            continue;
                        },
                    };
                    let new_e = ReadDirStream::new(new_e);
                    e = Box::pin(e.merge(new_e));
                }

                let f = e.next().await;
                match f {
                    Some(Ok(f)) => {
                        let f = f;
                        let t = f.file_type().await?;
                        if t.is_dir() {
                            dirs.push_back(f.path());
                        } else {
                            yield Ok(f.path().to_string_lossy().to_string());
                        }
                    },
                    Some(Err(e)) => {
                        yield Err(e.into());
                    },
                    None => {
                        if dirs.is_empty() {
                            break;
                        }
                    },
                }
            }
        };

        Ok(Box::pin(s))
    }
}

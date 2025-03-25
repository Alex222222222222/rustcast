use std::default::Default;
use std::env;
use std::path::PathBuf;

use super::super::{FileDownloader, LocalDownloader};
use super::Error;

use super::Cache;

/// Builder to facilitate creating [`Cache`] objects.
pub struct CacheBuilder {
    config: Config,
}

struct Config {
    dir: Option<PathBuf>,
    file_downloader: Box<dyn FileDownloader>,
    freshness_lifetime: Option<u64>,
}

impl CacheBuilder {
    /// Construct a new `CacheBuilder`.
    pub fn new() -> CacheBuilder {
        CacheBuilder {
            config: Config {
                dir: None,
                file_downloader: Box::new(LocalDownloader {}),
                freshness_lifetime: None,
            },
        }
    }

    /// Set the cache location. This can be set through the environment
    /// variable `RUST_CACHED_PATH_ROOT`. Otherwise it will default to a subdirectory
    /// named 'cache' of the default system temp directory.
    pub fn dir(mut self, dir: PathBuf) -> CacheBuilder {
        self.config.dir = Some(dir);
        self
    }

    /// Set the `ClientBuilder`.
    pub fn file_downloader(mut self, file_downloader: Box<dyn FileDownloader>) -> CacheBuilder {
        self.config.file_downloader = file_downloader;
        self
    }

    /// Build the `Cache` object.
    pub async fn build(self) -> Result<Cache, Error> {
        let dir = self
            .config
            .dir
            .unwrap_or_else(|| env::temp_dir().join("cache/"));
        tokio::fs::create_dir_all(&dir).await?;
        Ok(Cache {
            dir,
            freshness_lifetime: self.config.freshness_lifetime,
            file_downloader: self.config.file_downloader,
        })
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

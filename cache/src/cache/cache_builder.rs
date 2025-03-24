use std::default::Default;
use std::env;
use std::path::PathBuf;

use crate::Error;
use crate::{FileDownloader, LocalDownloader};

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

    /// Construct a new `CacheBuilder` with a `ClientBuilder`.
    pub fn with_file_downloader(file_downloader: Box<dyn FileDownloader>) -> CacheBuilder {
        CacheBuilder::new().file_downloader(file_downloader)
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

    /// Set the default freshness lifetime, in seconds. The default is None, meaning
    /// the ETAG for an external resource will always be checked for a fresher value.
    pub fn freshness_lifetime(mut self, freshness_lifetime: u64) -> CacheBuilder {
        self.config.freshness_lifetime = Some(freshness_lifetime);
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

/// Options to use with [`Cache::cached_path_with_options`].
#[derive(Default)]
pub struct Options {
    /// An optional subdirectory (relative to the cache root) to cache the resource in.
    pub subdir: Option<String>,
}

impl Options {
    pub fn new(subdir: Option<&str>) -> Self {
        Self {
            subdir: subdir.map(String::from),
        }
    }

    /// The the cache subdirectory to use.
    pub fn subdir(mut self, subdir: &str) -> Self {
        self.subdir = Some(subdir.into());
        self
    }
}

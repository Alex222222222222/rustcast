use futures::StreamExt;
use glob::glob;
use log::{debug, info};
use sha2::Digest;
use std::default::Default;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::utils::hash_str;
use crate::{Error, meta::Meta};
use crate::{FileDownloader, FileMetadata};

mod cache_builder;
pub use cache_builder::*;

/// Fetches and manages resources in a local cache directory.
pub struct Cache {
    /// The root directory of the cache.
    pub dir: PathBuf,
    /// An optional freshness lifetime (in seconds).
    ///
    /// If set, resources that were cached within the past `freshness_lifetime` seconds
    /// will always be regarded as fresh, and so the ETag of the corresponding remote
    /// resource won't be checked.
    freshness_lifetime: Option<u64>,
    /// downloader for files
    file_downloader: Box<dyn FileDownloader>,
}

impl Cache {
    /// Create a new `Cache` instance.
    pub async fn new() -> Result<Self, Error> {
        Cache::builder().build().await
    }

    /// Create a `CacheBuilder`.
    pub fn builder() -> CacheBuilder {
        CacheBuilder::new()
    }

    /// Get the cached path to a resource.
    ///
    /// If the resource is local file, it's path is returned.
    /// Otherwise, we will call the downloader to fetch the resource.
    /// It will cached locally and the path to the cache file will be returned.
    pub async fn cached_path(&self, resource: &str) -> Result<PathBuf, Error> {
        self.cached_path_with_options(resource, &Options::default())
            .await
    }

    /// Get the cached path to a resource using the given options.
    ///
    /// # Examples
    ///
    /// Use a particular subdirectory of the cache root:
    ///
    /// ```rust,no_run
    /// # use cached_path::{Cache, Options};
    /// # let cache = Cache::new().unwrap();
    /// # let subdir = "target";
    /// # let resource = "README.md";
    /// let path = cache.cached_path_with_options(
    ///     resource,
    ///     &Options::default().subdir(subdir),
    /// ).unwrap();
    /// ```
    pub async fn cached_path_with_options(
        &self,
        resource: &str,
        options: &Options,
    ) -> Result<PathBuf, Error> {
        let cached_path: PathBuf;

        if self.file_downloader.is_local() {
            // If the downloader is local, we can just return the path to the resource.
            cached_path = PathBuf::from(resource);

            if !cached_path.is_file() {
                return Err(Error::ResourceNotFound(String::from(resource)));
            }
        } else {
            // This is a remote resource, so fetch it to the cache.
            let meta = self
                .fetch_remote_resource(resource, options.subdir.as_deref())
                .await?;

            cached_path = meta.resource_path;
        }

        Ok(cached_path)
    }

    async fn fetch_remote_resource(
        &self,
        resource: &str,
        subdir: Option<&str>,
    ) -> Result<Meta, Error> {
        // Ensure root directory exists in case it has changed or been removed.
        if let Some(subdir_path) = subdir {
            tokio::fs::create_dir_all(self.dir.join(subdir_path)).await?;
        } else {
            tokio::fs::create_dir_all(&self.dir).await?;
        };

        // Find any existing cached versions of resource and check if they are still
        // fresh according to the `freshness_lifetime` setting.
        let versions = self.find_existing(resource, subdir).await; // already sorted, latest is first.
        if !versions.is_empty() && versions[0].is_fresh(self.freshness_lifetime) {
            // Oh hey, the latest version is still fresh!
            info!("Latest cached version of {} is still fresh", resource);
            return Ok(versions[0].clone());
        }

        // No existing version or the existing versions are older than their freshness
        // lifetimes, so we'll query for the ETAG of the resource and then compare
        // that with any existing versions.
        let file_meta = self.get_file_meta(resource).await?;
        let path = self.resource_to_filepath(
            &file_meta.location,
            subdir,
            None,
            file_meta.e_tag.as_deref(),
        );
        debug!("Resource path: {:?}", path);

        // TODO do we need to lock the file here?
        // Before going further we need to obtain a lock on the file to provide
        // parallel downloads of the same resource.

        if path.exists() {
            // Oh cool! The cache is up-to-date according to the ETAG.
            // We'll return the up-to-date version and clean up any other
            // dangling ones.
            info!("Cached version of {} is up-to-date", resource);
            return Meta::from_cache(&path).await;
        }

        // No up-to-date version cached, so we have to try downloading it.
        let meta = self.download_resource(&path, &file_meta).await?;

        info!("New version of {} cached", resource);

        Ok(meta)
    }

    /// Find existing versions of a cached resource, sorted by most recent first.
    async fn find_existing(&self, resource: &str, subdir: Option<&str>) -> Vec<Meta> {
        let mut existing_meta: Vec<Meta> = vec![];
        let glob_string = format!(
            "{}.*.meta",
            self.resource_to_filepath(resource, subdir, None, None)
                .to_str()
                .unwrap(),
        );
        for meta_path in glob(&glob_string).unwrap().filter_map(Result::ok) {
            if let Ok(meta) = Meta::from_path(&meta_path).await {
                existing_meta.push(meta);
            }
        }
        existing_meta
            .sort_unstable_by(|a, b| b.creation_time.partial_cmp(&a.creation_time).unwrap());
        existing_meta
    }

    async fn download_resource(
        &self,
        path: &Path,
        file_meta: &FileMetadata,
    ) -> Result<Meta, Error> {
        let mut response = self.file_downloader.get_file(&file_meta.location).await?;

        // First we make a temporary file and download the contents of the resource into it.
        // Otherwise if we wrote directly to the cache file and the download got
        // interrupted we could be left with a corrupted cache file.
        let tempfile = NamedTempFile::new_in(path.parent().unwrap())?;
        let mut tempfile_write_handle =
            OpenOptions::new().write(true).open(tempfile.path()).await?;

        info!("Starting download of {}", file_meta.location);

        let mut bytes_downloaded = 0;
        while let Some(b) = response.next().await {
            let b = b?;
            bytes_downloaded += b.len();
            tempfile_write_handle.write_all(&b).await?;
        }

        info!("Downloaded {} bytes", bytes_downloaded);
        tempfile_write_handle.flush().await?;
        drop(tempfile_write_handle);

        debug!("Writing meta file");

        let meta = Meta::new(path.into(), file_meta.clone(), self.freshness_lifetime);
        meta.to_file().await?;

        debug!(
            "Renaming temp file to cache location for {}",
            file_meta.location
        );

        tokio::fs::rename(tempfile.path(), path).await?;

        Ok(meta)
    }

    pub async fn get_file_meta(&self, resource: &str) -> Result<FileMetadata, Error> {
        debug!("Fetching ETAG for {}", resource);
        self.file_downloader.get_meta(resource).await
    }

    fn resource_to_filepath(
        &self,
        resource: &str,
        subdir: Option<&str>,
        suffix: Option<&str>,
        e_tag: Option<&str>,
    ) -> PathBuf {
        let mut resource_hash = sha2::Sha256::new();
        resource_hash.update(self.file_downloader.hash());
        resource_hash.update(resource.as_bytes());
        let resource_hash = format!("{:x}", resource_hash.finalize());
        let mut filename = if let Some(tag) = e_tag {
            let etag_hash = hash_str(&tag[..]);
            format!("{}.{}", resource_hash, etag_hash)
        } else {
            resource_hash
        };

        if let Some(suf) = suffix {
            filename.push_str(suf);
        }

        let filepath = PathBuf::from(filename);

        if let Some(subdir_path) = subdir {
            self.dir.join(subdir_path).join(filepath)
        } else {
            self.dir.join(filepath)
        }
    }
}

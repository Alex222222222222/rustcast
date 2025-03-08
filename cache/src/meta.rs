use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::utils::now;
use crate::{Error, FileMetadata};

/// Holds information about a cached resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Meta {
    /// the meta data of the resource
    pub(crate) meta_data: FileMetadata,
    /// Path to the cached resource.
    pub(crate) resource_path: PathBuf,
    /// Path to the serialized meta.
    pub(crate) meta_path: PathBuf,
    /// Time that the freshness of this cached resource will expire.
    pub(crate) expires: Option<f64>,
    /// Time this version of the resource was cached.
    pub(crate) creation_time: f64,
}

impl Meta {
    pub(crate) fn new(
        resource_path: PathBuf,
        meta_data: FileMetadata,
        freshness_lifetime: Option<u64>,
    ) -> Meta {
        let mut expires: Option<f64> = None;
        let creation_time = now();
        if let Some(lifetime) = freshness_lifetime {
            expires = Some(creation_time + (lifetime as f64));
        }
        let meta_path = Meta::meta_path(&resource_path);
        Meta {
            meta_data,
            resource_path,
            meta_path,
            expires,
            creation_time,
        }
    }

    pub(crate) fn meta_path(resource_path: &Path) -> PathBuf {
        let mut meta_path = PathBuf::from(resource_path);
        let resource_file_name = meta_path.file_name().unwrap().to_str().unwrap();
        let meta_file_name = format!("{}.meta", resource_file_name);
        meta_path.set_file_name(&meta_file_name[..]);
        meta_path
    }

    pub(crate) async fn to_file(&self) -> Result<(), Error> {
        let serialized = serde_json::to_string(self).unwrap();
        tokio::fs::write(&self.meta_path, &serialized[..]).await?;
        Ok(())
    }

    pub(crate) async fn from_cache(resource_path: &Path) -> Result<Self, Error> {
        let meta_path = Meta::meta_path(resource_path);
        Meta::from_path(&meta_path).await
    }

    /// Read `Meta` from a path.
    pub(crate) async fn from_path(path: &Path) -> Result<Self, Error> {
        if !path.is_file() {
            return Err(Error::CacheCorrupted(format!("missing meta at {:?}", path)));
        }
        let serialized = tokio::fs::read_to_string(path).await?;
        let meta: Meta = serde_json::from_str(&serialized[..])
            .map_err(|e| Error::CacheCorrupted(format!("invalid meta at {:?}: {:?}", path, e)))?;
        Ok(meta)
    }

    /// Check if resource is still fresh. Passing a `Some` value for
    /// `freshness_lifetime` will override the expiration time (if there is one)
    /// of this resource.
    pub(crate) fn is_fresh(&self, freshness_lifetime: Option<u64>) -> bool {
        if let Some(lifetime) = freshness_lifetime {
            let expiration_time = self.creation_time + (lifetime as f64);
            expiration_time > now()
        } else if let Some(expiration_time) = self.expires {
            expiration_time > now()
        } else {
            false
        }
    }
}

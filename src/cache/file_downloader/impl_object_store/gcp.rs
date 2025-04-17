use std::sync::Arc;

use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use log::debug;
use object_store::{
    ObjectStore,
    gcp::{GoogleCloudStorage, GoogleCloudStorageBuilder, GoogleConfigKey},
};
use sha2::Digest;

use super::super::{Error, FileDownloader, FileMetadata};

use super::{CLIENT_CONFIG_KEYS, object_store_config_key_to_string};

static GCP_CONFIG_KEYS: &[GoogleConfigKey; 4] = &[
    GoogleConfigKey::ServiceAccount,
    GoogleConfigKey::ServiceAccountKey,
    GoogleConfigKey::Bucket,
    GoogleConfigKey::ApplicationCredentials,
    // And a list of GoogleConfigKey::Client(CLIENT_CONFIG_KEYS),
];

fn gcp_config_key_to_string(key: &GoogleConfigKey) -> &'static str {
    match key {
        GoogleConfigKey::ServiceAccount => "ServiceAccount",
        GoogleConfigKey::ServiceAccountKey => "ServiceAccountKey",
        GoogleConfigKey::Bucket => "Bucket",
        GoogleConfigKey::ApplicationCredentials => "ApplicationCredentials",
        GoogleConfigKey::Client(c) => object_store_config_key_to_string(c),
        &_ => "Unknown",
    }
}

pub struct GcpDownloader {
    object_store: Arc<GoogleCloudStorage>,
    hash: Bytes,
}

impl GcpDownloader {
    pub fn new(builder: GoogleCloudStorageBuilder) -> Result<Self, Error> {
        let mut hasher = sha2::Sha256::new();
        hasher.update("gcp:".as_bytes());
        // iter over AMAZON_S3_CONFIG_KEYS and set the value from the builder
        for key in GCP_CONFIG_KEYS.iter() {
            let v = builder.get_config_value(key);
            if let Some(value) = v {
                hasher.update(gcp_config_key_to_string(key).as_bytes());
                hasher.update(":".as_bytes());
                hasher.update(value.as_bytes());
                hasher.update(",".as_bytes());
            }
        }
        for key in CLIENT_CONFIG_KEYS.iter() {
            let v = builder.get_config_value(&GoogleConfigKey::Client(*key));
            if let Some(value) = v {
                hasher.update(object_store_config_key_to_string(key).as_bytes());
                hasher.update(":".as_bytes());
                hasher.update(value.as_bytes());
                hasher.update(",".as_bytes());
            }
        }

        // TODO optimize this
        let hash = hasher.finalize();
        let hash = Bytes::copy_from_slice(hash.as_slice());
        let object_store = builder.build()?;
        Ok(Self {
            object_store: Arc::new(object_store),
            hash,
        })
    }

    pub fn get_object_store(&self) -> Arc<GoogleCloudStorage> {
        self.object_store.clone()
    }
}

#[async_trait]
impl FileDownloader for GcpDownloader {
    async fn get_file(
        &self,
        path: &str,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Error>> + Send>>, Error>
    {
        let path = object_store::path::Path::parse(path)?;
        match self.object_store.get(&path).await {
            Ok(file) => {
                let s = stream! {
                    let mut stream = file.into_stream();
                    while let Some(bytes) = stream.next().await {
                        yield bytes.map_err(|e| e.into());
                    }
                };
                Ok(Box::pin(s))
            }
            Err(object_store::Error::NotFound { .. }) => {
                Err(Error::ResourceNotFound(path.to_string()))
            }
            Err(e) => return Err(e.into()),
        }
    }

    async fn get_meta(&self, path: &str) -> Result<FileMetadata, Error> {
        debug!("get meta of {}", path);
        let path = object_store::path::Path::parse(path)?;
        match self.object_store.head(&path).await {
            Ok(meta) => Ok(FileMetadata {
                size: meta.size as usize,
                location: meta.location.to_string(),
                last_modified: meta.last_modified,
                e_tag: meta.e_tag,
            }),
            Err(object_store::Error::NotFound { .. }) => {
                debug!("file not found: {}", path);
                Err(Error::ResourceNotFound(path.to_string()))
            }
            Err(e) => return Err(e.into()),
        }
    }

    fn hash(&self) -> Bytes {
        self.hash.clone()
    }
}

use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use log::debug;
use object_store::{
    ObjectStore,
    aws::{AmazonS3, AmazonS3Builder, AmazonS3ConfigKey},
};
use sha2::Digest;
use std::sync::Arc;

use super::super::{Error, FileDownloader, FileMetadata};

use super::{CLIENT_CONFIG_KEYS, object_store_config_key_to_string};

static AMAZON_S3_CONFIG_KEYS: &[AmazonS3ConfigKey; 19] = &[
    AmazonS3ConfigKey::AccessKeyId,
    AmazonS3ConfigKey::SecretAccessKey,
    AmazonS3ConfigKey::Region,
    AmazonS3ConfigKey::DefaultRegion,
    AmazonS3ConfigKey::Bucket,
    AmazonS3ConfigKey::Endpoint,
    AmazonS3ConfigKey::Token,
    AmazonS3ConfigKey::ImdsV1Fallback,
    AmazonS3ConfigKey::VirtualHostedStyleRequest,
    AmazonS3ConfigKey::UnsignedPayload,
    AmazonS3ConfigKey::Checksum,
    AmazonS3ConfigKey::MetadataEndpoint,
    AmazonS3ConfigKey::ContainerCredentialsRelativeUri,
    AmazonS3ConfigKey::CopyIfNotExists,
    AmazonS3ConfigKey::ConditionalPut,
    AmazonS3ConfigKey::SkipSignature,
    AmazonS3ConfigKey::DisableTagging,
    AmazonS3ConfigKey::S3Express,
    AmazonS3ConfigKey::RequestPayer,
    // And a list of AmazonS3ConfigKey::Client(CLIENT_CONFIG_KEYS),
];

fn amazon_s3_config_key_to_string(key: &AmazonS3ConfigKey) -> &'static str {
    match key {
        AmazonS3ConfigKey::AccessKeyId => "AccessKeyId",
        AmazonS3ConfigKey::SecretAccessKey => "SecretAccessKey",
        AmazonS3ConfigKey::Region => "Region",
        AmazonS3ConfigKey::DefaultRegion => "DefaultRegion",
        AmazonS3ConfigKey::Bucket => "Bucket",
        AmazonS3ConfigKey::Endpoint => "Endpoint",
        AmazonS3ConfigKey::Token => "Token",
        AmazonS3ConfigKey::ImdsV1Fallback => "ImdsV1Fallback",
        AmazonS3ConfigKey::VirtualHostedStyleRequest => "VirtualHostedStyleRequest",
        AmazonS3ConfigKey::UnsignedPayload => "UnsignedPayload",
        AmazonS3ConfigKey::Checksum => "Checksum",
        AmazonS3ConfigKey::MetadataEndpoint => "MetadataEndpoint",
        AmazonS3ConfigKey::ContainerCredentialsRelativeUri => "ContainerCredentialsRelativeUri",
        AmazonS3ConfigKey::CopyIfNotExists => "CopyIfNotExists",
        AmazonS3ConfigKey::ConditionalPut => "ConditionalPut",
        AmazonS3ConfigKey::SkipSignature => "SkipSignature",
        AmazonS3ConfigKey::DisableTagging => "DisableTagging",
        AmazonS3ConfigKey::S3Express => "S3Express",
        AmazonS3ConfigKey::RequestPayer => "RequestPayer",
        AmazonS3ConfigKey::Client(c) => object_store_config_key_to_string(c),
        &_ => "Unknown",
    }
}

pub struct AwsS3Downloader {
    object_store: Arc<AmazonS3>,
    hash: Bytes,
}

impl AwsS3Downloader {
    pub fn new(builder: AmazonS3Builder) -> Result<Self, Error> {
        let mut hasher = sha2::Sha256::new();
        hasher.update("aws:".as_bytes());
        // iter over AMAZON_S3_CONFIG_KEYS and set the value from the builder
        for key in AMAZON_S3_CONFIG_KEYS.iter() {
            let v = builder.get_config_value(key);
            if let Some(value) = v {
                hasher.update(amazon_s3_config_key_to_string(key).as_bytes());
                hasher.update(":".as_bytes());
                hasher.update(value.as_bytes());
                hasher.update(",".as_bytes());
            }
        }
        for key in CLIENT_CONFIG_KEYS.iter() {
            let v = builder.get_config_value(&AmazonS3ConfigKey::Client(*key));
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

    pub fn get_object_store(&self) -> Arc<AmazonS3> {
        self.object_store.clone()
    }
}

#[async_trait]
impl FileDownloader for AwsS3Downloader {
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
                size: meta.size,
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

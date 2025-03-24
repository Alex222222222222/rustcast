use std::{collections::HashMap, sync::Arc};

mod clap_args;
mod file_provider_config;
mod log_level;
mod playlist_config;

pub use clap_args::ClapArgs;
pub use file_provider_config::FileProviderConfig;
use log_level::LogLevel;
pub use playlist_config::{PlaylistChildConfig, PlaylistConfig};

#[derive(Debug, serde::Deserialize)]
pub struct GlobalConfig {
    pub playlists: HashMap<String, PlaylistConfig>,
    #[serde(default)]
    pub file_provider: HashMap<String, FileProviderConfig>,
    pub outputs: Vec<ShoutCastOutput>,
    #[serde(default)]
    pub log_level: Option<LogLevel>,
    #[serde(default)]
    pub log_file: Vec<String>,
    #[serde(default)]
    pub cache_dir: Option<Arc<String>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ShoutCastOutput {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub playlist: String,
    // TODO: Add authentication
}

impl GlobalConfig {
    fn from_json(json: &str) -> anyhow::Result<Self> {
        let config: GlobalConfig = serde_json::from_str(json)?;
        Ok(config)
    }

    async fn from_path(path: &str) -> anyhow::Result<Self> {
        let json = tokio::fs::read_to_string(path).await?;
        Self::from_json(&json)
    }

    pub async fn from_clap_args(clap_args: ClapArgs) -> anyhow::Result<Self> {
        // Initialize logging before parsing the log level from the configuration file
        let ClapArgs {
            config,
            log_level,
            log_file,
        } = clap_args;
        let mut config = GlobalConfig::from_path(&config).await?;
        if let Some(log_level) = log_level {
            config.log_level = Some(log_level);
        }
        if !log_file.is_empty() {
            config.log_file = log_file;
        }

        #[cfg(not(debug_assertions))]
        log_level::set_log_output(config.log_level, &config.log_file).await?;

        Ok(config)
    }
}
#[cfg(test)]
mod tests {
    use file_provider_config::AwsS3ConfigKeys;

    use super::*;

    #[tokio::test]
    async fn test_from_json() {
        let json = r#"
        {
            "playlists": {
                "main":{
                    "child": {
                        "LocalFolder": {
                            "folder": "/path/to/folder"
                        }
                    },
                    "name": "Test Playlist"
                }
            },
            "file_provider": {
                "local": {
                    "AwsS3": {
                        "bucket": "test-bucket",
                        "region": "us-west-2",
                        "access_key_id": "access-key",
                        "secret_access_key": "secret-key",
                        "default_region": null,
                        "endpoint": "https://s3.amazonaws.com",
                        "token": null,
                        "imds_v1_fallback": true,
                        "virtual_hosted_style_request": false,
                        "unsigned_payload": null,
                        "checksum": true,
                        "metadata_endpoint": null,
                        "container_credentials_relative_uri": null,
                        "skip_signature": false,
                        "s3_express": false,
                        "request_payer": "requester"
                    }
                }
            },
            "outputs": [
                {
                    "host": "127.0.0.1",
                    "port": 8000,
                    "path": "/stream",
                    "playlist": "main"
                }
            ]
        }
        "#;

        let config = GlobalConfig::from_json(json).unwrap();

        assert_eq!(config.playlists.len(), 1);
        assert!(config.playlists.contains_key("main"));

        let file_provider = config.file_provider;
        assert_eq!(file_provider.len(), 1);
        assert!(file_provider.contains_key("local"));
        if let FileProviderConfig::AwsS3(config) = file_provider["local"].clone() {
            assert_eq!(
                config.get(&AwsS3ConfigKeys::Bucket),
                Some(&serde_json::Value::String("test-bucket".to_string()))
            );
            assert_eq!(
                config.get(&AwsS3ConfigKeys::Region),
                Some(&serde_json::Value::String("us-west-2".to_string()))
            );
            assert_eq!(
                config.get(&AwsS3ConfigKeys::AccessKeyId),
                Some(&serde_json::Value::String("access-key".to_string()))
            );
            assert_eq!(
                config.get(&AwsS3ConfigKeys::SecretAccessKey),
                Some(&serde_json::Value::String("secret-key".to_string()))
            );
            assert_eq!(
                config.get(&AwsS3ConfigKeys::Endpoint),
                Some(&serde_json::Value::String(
                    "https://s3.amazonaws.com".to_string()
                ))
            );
            assert_eq!(
                config
                    .get(&AwsS3ConfigKeys::ImdsV1Fallback)
                    .and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(
                config
                    .get(&AwsS3ConfigKeys::VirtualHostedStyleRequest)
                    .and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                config
                    .get(&AwsS3ConfigKeys::Checksum)
                    .and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(
                config
                    .get(&AwsS3ConfigKeys::SkipSignature)
                    .and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                config
                    .get(&AwsS3ConfigKeys::S3Express)
                    .and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                config.get(&AwsS3ConfigKeys::RequestPayer),
                Some(&serde_json::Value::String("requester".to_string()))
            );
        } else {
            panic!("Expected AwsS3 variant");
        }

        assert_eq!(config.outputs.len(), 1);
        assert_eq!(config.outputs[0].host, "127.0.0.1");
        assert_eq!(config.outputs[0].port, 8000);
        assert_eq!(config.outputs[0].path, "/stream");
        assert_eq!(config.outputs[0].playlist, "main");
    }

    #[tokio::test]
    #[should_panic]
    async fn test_from_invalid_json() {
        let json = r#"
        {
            "playlists": {
                "main": {
                    "items": []
                }
            },
            "outputs": [
                {
                    "host": "127.0.0.1",
                    "port": 8000,
                    "playlist": "main"
                }
            ]
        }
        "#;

        // This should panic because the "file_provider" field is missing
        let _config = GlobalConfig::from_json(json).unwrap();
    }
}

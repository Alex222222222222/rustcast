use std::collections::HashMap;

use file_provider_config::FileProviderConfig;
use playlist_config::PlaylistConfig;

mod file_provider_config;
mod playlist_config;

#[derive(Debug, serde::Deserialize)]
pub struct GlobalConfig {
    pub playlists: HashMap<String, PlaylistConfig>,
    pub file_provider: HashMap<String, FileProviderConfig>,
    pub outputs: Vec<ShoutCastOutput>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ShoutCastOutput {
    pub host: String,
    pub port: u16,
    pub playlist: String,
    // TODO: Add authentication
}

impl GlobalConfig {
    pub async fn from_json(json: &str) -> anyhow::Result<Self> {
        let config: GlobalConfig = serde_json::from_str(json)?;
        Ok(config)
    }
}
#[cfg(test)]
mod tests {
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
                    "host": "localhost",
                    "port": 8000,
                    "playlist": "main"
                }
            ]
        }
        "#;

        let config = GlobalConfig::from_json(json).await.unwrap();

        assert_eq!(config.playlists.len(), 1);
        assert!(config.playlists.contains_key("main"));

        assert_eq!(config.file_provider.len(), 1);
        assert!(config.file_provider.contains_key("local"));

        if let FileProviderConfig::AwsS3 {
            bucket,
            region,
            access_key_id,
            secret_access_key,
            endpoint,
            imds_v1_fallback,
            virtual_hosted_style_request,
            checksum,
            skip_signature,
            s3_express,
            request_payer,
            ..
        } = config.file_provider["local"].clone()
        {
            assert_eq!(*bucket, Some("test-bucket".to_string()));
            assert_eq!(*region, Some("us-west-2".to_string()));
            assert_eq!(*access_key_id, Some("access-key".to_string()));
            assert_eq!(*secret_access_key, Some("secret-key".to_string()));
            assert_eq!(*endpoint, Some("https://s3.amazonaws.com".to_string()));
            assert_eq!(imds_v1_fallback, Some(true));
            assert_eq!(virtual_hosted_style_request, Some(false));
            assert_eq!(checksum, Some(true));
            assert_eq!(skip_signature, Some(false));
            assert_eq!(s3_express, Some(false));
            assert_eq!(*request_payer, Some("requester".to_string()));
        } else {
            panic!("Expected AwsS3 variant");
        }

        assert_eq!(config.outputs.len(), 1);
        assert_eq!(config.outputs[0].host, "localhost");
        assert_eq!(config.outputs[0].port, 8000);
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
                    "host": "localhost",
                    "port": 8000,
                    "playlist": "main"
                }
            ]
        }
        "#;

        // This should panic because the "file_provider" field is missing
        let _config = GlobalConfig::from_json(json).await.unwrap();
    }
}

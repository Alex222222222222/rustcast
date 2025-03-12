pub use aws::AwsS3ConfigKeys;
pub use gcp::GoogleCloudStorageConfigKeys;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

mod aws;
mod gcp;

#[derive(Debug, Deserialize, Clone)]
pub enum FileProviderConfig {
    AwsS3(BTreeMap<AwsS3ConfigKeys, Value>),
    GoogleCloudStorage(BTreeMap<GoogleCloudStorageConfigKeys, Value>),
}

impl FileProviderConfig {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config: FileProviderConfig = serde_json::from_str(json)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_aws_s3() {
        let json = r#"{
            "AwsS3": {
                "bucket": "test-bucket",
                "region": "us-west-2",
                "access_key_id": "access-key",
                "secret_access_key": "secret-key",
                "endpoint": "https://s3.amazonaws.com",
                "imds_v1_fallback": true,
                "virtual_hosted_style_request": false,
                "checksum": true,
                "skip_signature": false,
                "s3_express": false,
                "request_payer": "requester"
            }
        }"#;

        let config = FileProviderConfig::from_json(json).unwrap();
        if let FileProviderConfig::AwsS3(config) = config {
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
    }

    #[test]
    fn test_from_json_invalid() {
        let json = r#"{"InvalidType":null}"#;
        let result = FileProviderConfig::from_json(json);
        assert!(result.is_err());
    }
}

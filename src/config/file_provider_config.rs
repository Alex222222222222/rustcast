use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileProviderConfig {
    Local,
    AwsS3 {
        bucket: Box<Option<String>>,
        region: Box<Option<String>>,
        access_key_id: Box<Option<String>>,
        secret_access_key: Box<Option<String>>,
        default_region: Box<Option<String>>,
        endpoint: Box<Option<String>>,
        token: Box<Option<String>>,
        imds_v1_fallback: Option<bool>,
        virtual_hosted_style_request: Option<bool>,
        unsigned_payload: Option<bool>,
        checksum: Option<bool>,
        metadata_endpoint: Box<Option<String>>,
        container_credentials_relative_uri: Box<Option<String>>,
        skip_signature: Option<bool>,
        s3_express: Option<bool>,
        request_payer: Box<Option<String>>,
    },
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
    fn test_from_json_local() {
        let json = r#"{"Local":null}"#;
        let config = FileProviderConfig::from_json(json).unwrap();
        assert!(matches!(config, FileProviderConfig::Local));
    }

    #[test]
    fn test_from_json_aws_s3() {
        let json = r#"{
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
        }"#;

        let config = FileProviderConfig::from_json(json).unwrap();

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
        } = config
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
    }

    #[test]
    fn test_from_json_invalid() {
        let json = r#"{"InvalidType":null}"#;
        let result = FileProviderConfig::from_json(json);
        assert!(result.is_err());
    }
}

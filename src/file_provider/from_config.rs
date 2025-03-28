use std::{collections::HashMap, sync::Arc};

use anyhow::Ok;
use object_store::aws::AmazonS3Builder;

use super::{FileProvider, aws::AwsS3FileProvider, gcp::GoogleCouldStorageFileProvider};

// TODO add cache dir functionality

fn serde_json_value_to_string(value: serde_json::Value) -> anyhow::Result<String> {
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Null => Ok("".to_string()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        _ => Err(anyhow::anyhow!("Unsupported value type")),
    }
}

pub async fn build_file_provider(
    cache_dir: Option<Arc<String>>,
    config: HashMap<String, crate::config::FileProviderConfig>,
) -> anyhow::Result<HashMap<String, Arc<dyn FileProvider>>> {
    let mut res = HashMap::new();
    for (name, provider) in config {
        let provider: Arc<dyn FileProvider> = match provider {
            crate::config::FileProviderConfig::AwsS3(keys) => {
                let mut s3_builder = AmazonS3Builder::new();
                for (key, value) in keys {
                    s3_builder =
                        s3_builder.with_config(key.into(), serde_json_value_to_string(value)?)
                }

                Arc::new(AwsS3FileProvider::new(cache_dir.clone(), s3_builder).await?)
            }
            crate::config::FileProviderConfig::GoogleCloudStorage(keys) => {
                let mut gcp_builder = object_store::gcp::GoogleCloudStorageBuilder::new();
                for (key, value) in keys {
                    gcp_builder =
                        gcp_builder.with_config(key.into(), serde_json_value_to_string(value)?)
                }

                Arc::new(GoogleCouldStorageFileProvider::new(cache_dir.clone(), gcp_builder).await?)
            }
        };
        res.insert(name, provider);
    }

    Ok(res)
}

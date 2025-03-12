use object_store::gcp::GoogleConfigKey;
use serde::Deserialize;

#[derive(
    strum::Display, strum::EnumIter, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
#[serde(rename_all = "snake_case")]
pub enum GoogleCloudStorageConfigKeys {
    ServiceAccount,
    ServiceAccountKey,
    Bucket,
    ApplicationCredentials,
}

impl From<GoogleCloudStorageConfigKeys> for GoogleConfigKey {
    fn from(value: GoogleCloudStorageConfigKeys) -> Self {
        match value {
            GoogleCloudStorageConfigKeys::ServiceAccount => Self::ServiceAccount,
            GoogleCloudStorageConfigKeys::ServiceAccountKey => Self::ServiceAccountKey,
            GoogleCloudStorageConfigKeys::Bucket => Self::Bucket,
            GoogleCloudStorageConfigKeys::ApplicationCredentials => Self::ApplicationCredentials,
        }
    }
}

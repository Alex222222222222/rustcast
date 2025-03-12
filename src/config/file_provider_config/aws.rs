use serde::Deserialize;

#[derive(
    strum::Display, strum::EnumIter, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
#[serde(rename_all = "snake_case")]
pub enum AwsS3ConfigKeys {
    Bucket,
    Region,
    AccessKeyId,
    SecretAccessKey,
    DefaultRegion,
    Endpoint,
    Token,
    ImdsV1Fallback,
    VirtualHostedStyleRequest,
    UnsignedPayload,
    Checksum,
    MetadataEndpoint,
    ContainerCredentialsRelativeUri,
    SkipSignature,
    S3Express,
    RequestPayer,
}

impl From<AwsS3ConfigKeys> for object_store::aws::AmazonS3ConfigKey {
    fn from(value: AwsS3ConfigKeys) -> Self {
        match value {
            AwsS3ConfigKeys::Bucket => Self::Bucket,
            AwsS3ConfigKeys::Region => Self::Region,
            AwsS3ConfigKeys::AccessKeyId => Self::AccessKeyId,
            AwsS3ConfigKeys::SecretAccessKey => Self::SecretAccessKey,
            AwsS3ConfigKeys::DefaultRegion => Self::DefaultRegion,
            AwsS3ConfigKeys::Endpoint => Self::Endpoint,
            AwsS3ConfigKeys::Token => Self::Token,
            AwsS3ConfigKeys::ImdsV1Fallback => Self::ImdsV1Fallback,
            AwsS3ConfigKeys::VirtualHostedStyleRequest => Self::VirtualHostedStyleRequest,
            AwsS3ConfigKeys::UnsignedPayload => Self::UnsignedPayload,
            AwsS3ConfigKeys::Checksum => Self::Checksum,
            AwsS3ConfigKeys::MetadataEndpoint => Self::MetadataEndpoint,
            AwsS3ConfigKeys::ContainerCredentialsRelativeUri => {
                Self::ContainerCredentialsRelativeUri
            }
            AwsS3ConfigKeys::SkipSignature => Self::SkipSignature,
            AwsS3ConfigKeys::S3Express => Self::S3Express,
            AwsS3ConfigKeys::RequestPayer => Self::RequestPayer,
        }
    }
}

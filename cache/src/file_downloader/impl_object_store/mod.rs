use object_store::ClientConfigKey;

mod aws;

pub use aws::AwsS3Downloader;

static CLIENT_CONFIG_KEYS: &[ClientConfigKey; 17] = &[
    ClientConfigKey::AllowHttp,
    ClientConfigKey::AllowInvalidCertificates,
    ClientConfigKey::ConnectTimeout,
    ClientConfigKey::DefaultContentType,
    ClientConfigKey::Http1Only,
    ClientConfigKey::Http2KeepAliveInterval,
    ClientConfigKey::Http2KeepAliveTimeout,
    ClientConfigKey::Http2KeepAliveWhileIdle,
    ClientConfigKey::Http2MaxFrameSize,
    ClientConfigKey::Http2Only,
    ClientConfigKey::PoolIdleTimeout,
    ClientConfigKey::PoolMaxIdlePerHost,
    ClientConfigKey::ProxyUrl,
    ClientConfigKey::ProxyCaCertificate,
    ClientConfigKey::ProxyExcludes,
    ClientConfigKey::Timeout,
    ClientConfigKey::UserAgent,
];

fn object_store_config_key_to_string(key: &ClientConfigKey) -> &'static str {
    match key {
        ClientConfigKey::AllowHttp => "AllowHttp",
        ClientConfigKey::AllowInvalidCertificates => "AllowInvalidCertificates",
        ClientConfigKey::ConnectTimeout => "ConnectTimeout",
        ClientConfigKey::DefaultContentType => "DefaultContentType",
        ClientConfigKey::Http1Only => "Http1Only",
        ClientConfigKey::Http2KeepAliveInterval => "Http2KeepAliveInterval",
        ClientConfigKey::Http2KeepAliveTimeout => "Http2KeepAliveTimeout",
        ClientConfigKey::Http2KeepAliveWhileIdle => "Http2KeepAliveWhileIdle",
        ClientConfigKey::Http2MaxFrameSize => "Http2MaxFrameSize",
        ClientConfigKey::Http2Only => "Http2Only",
        ClientConfigKey::PoolIdleTimeout => "PoolIdleTimeout",
        ClientConfigKey::PoolMaxIdlePerHost => "PoolMaxIdlePerHost",
        ClientConfigKey::ProxyUrl => "ProxyUrl",
        ClientConfigKey::ProxyCaCertificate => "ProxyCaCertificate",
        ClientConfigKey::ProxyExcludes => "ProxyExcludes",
        ClientConfigKey::Timeout => "Timeout",
        ClientConfigKey::UserAgent => "UserAgent",
        &_ => "Unknown",
    }
}

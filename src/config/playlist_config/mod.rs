pub use playlist_child::PlaylistChildConfig;

mod playlist_child;

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistConfig {
    pub child: PlaylistChildConfig,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PlaylistConfig {
        pub async fn from_json(json: &str) -> anyhow::Result<Self> {
            let config: PlaylistConfig = serde_json::from_str(json)?;
            Ok(config)
        }
    }

    #[tokio::test]
    async fn test_from_json() {
        let json = r#"{
            "child": {
                "LocalFolder": {
                    "folder": "/path/to/folder"
                }
            },
            "name": "Test Playlist"
        }"#;

        let config = PlaylistConfig::from_json(json).await.unwrap();

        assert_eq!(config.name, "Test Playlist");

        // Verify the enum variant
        match config.child {
            PlaylistChildConfig::LocalFolder { folder, .. } => {
                assert_eq!(folder, "/path/to/folder");
            }
            _ => panic!("Expected LocalFolder variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_invalid() {
        let json = r#"{
            "invalid": "json"
        }"#;

        let result = PlaylistConfig::from_json(json).await;
        assert!(result.is_err());
    }
}

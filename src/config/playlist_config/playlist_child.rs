use std::sync::Arc;

#[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum PlaylistChildConfig {
    Silent,
    LocalFolder {
        folder: Arc<String>,
        #[serde(default)]
        repeat: Option<bool>,
        #[serde(default)]
        shuffle: Option<bool>,
        #[serde(default)]
        recursive: Option<bool>,
        #[serde(default)]
        fail_over: Option<Arc<PlaylistChildConfig>>,
    },
    LocalFiles {
        files: Arc<Vec<Arc<String>>>,
        #[serde(default)]
        repeat: Option<bool>,
        #[serde(default)]
        shuffle: Option<bool>,
        #[serde(default)]
        fail_over: Option<Arc<PlaylistChildConfig>>,
    },
    RemoteFolder {
        folder: Arc<String>,
        remote_client: Arc<String>,
        #[serde(default)]
        repeat: Option<bool>,
        #[serde(default)]
        shuffle: Option<bool>,
        #[serde(default)]
        recursive: Option<bool>,
        #[serde(default)]
        fail_over: Option<Arc<PlaylistChildConfig>>,
    },
    RemoteFiles {
        files: Arc<Vec<Arc<String>>>,
        remote_client: String,
        #[serde(default)]
        repeat: Option<bool>,
        #[serde(default)]
        shuffle: Option<bool>,
        #[serde(default)]
        fail_over: Option<Arc<PlaylistChildConfig>>,
    },
    Playlists {
        children: Arc<Vec<Arc<PlaylistChildConfig>>>,
        #[serde(default)]
        repeat: Option<bool>,
        #[serde(default)]
        shuffle: Option<bool>,
        #[serde(default)]
        fail_over: Option<Arc<PlaylistChildConfig>>,
    },
}
#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    impl PlaylistChildConfig {
        pub async fn from_json(json: &str) -> anyhow::Result<Self> {
            let config: PlaylistChildConfig = serde_json::from_str(json)?;
            Ok(config)
        }
    }

    #[tokio::test]
    async fn test_playlist_child_config_sync_send() {
        assert_impl_all!(PlaylistChildConfig: Send, Sync);
    }

    #[tokio::test]
    async fn test_playlist_child_fail_over_silent() {
        let json = r#"{"LocalFolder":{"folder": "/app/music","fail_over": "Silent"}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFolder { fail_over, .. } => {
                assert_eq!(*fail_over.unwrap(), PlaylistChildConfig::Silent)
            }
            _ => panic!("Expected LocalFolder variant"),
        }
    }

    #[tokio::test]
    async fn test_playlist_child_fail_over_folder() {
        let json = r#"{"LocalFolder":{"folder":"/app/music","fail_over":{"LocalFolder":{"folder":"/app/music","fail_over":"Silent"}}}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFolder { fail_over, .. } => {
                let target = PlaylistChildConfig::LocalFolder {
                    folder: Arc::new("/app/music".to_string()),
                    repeat: None,
                    shuffle: None,
                    recursive: None,
                    fail_over: Some(Arc::new(PlaylistChildConfig::Silent)),
                };
                assert_eq!(*fail_over.unwrap(), target)
            }
            _ => panic!("Expected LocalFolder variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_local_folder() {
        let json = r#"{"LocalFolder":{"folder":"/path/to/folder"}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFolder { folder, .. } => {
                assert_eq!(*folder, "/path/to/folder")
            }
            _ => panic!("Expected LocalFolder variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_local_files() {
        let json = r#"{"LocalFiles":{"files":["/path/to/file1.mp3","/path/to/file2.mp3"]}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFiles { files, .. } => {
                assert_eq!(files.len(), 2);
                assert_eq!(*files[0], "/path/to/file1.mp3");
                assert_eq!(*files[1], "/path/to/file2.mp3");
            }
            _ => panic!("Expected LocalFiles variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_s3_folder() {
        let json = r#"{"RemoteFolder":{"folder":"bucket/folder","remote_client":"default"}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::RemoteFolder {
                folder,
                remote_client,
                ..
            } => {
                assert_eq!(*folder, "bucket/folder");
                assert_eq!(*remote_client, "default");
            }
            _ => panic!("Expected S3Folder variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_remote_files() {
        let json = r#"{"RemoteFiles":{"files":["bucket/file1.mp3","bucket/file2.mp3"],"remote_client":"default"}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::RemoteFiles {
                files,
                remote_client,
                ..
            } => {
                assert_eq!(files.len(), 2);
                assert_eq!(*files[0], "bucket/file1.mp3");
                assert_eq!(*files[1], "bucket/file2.mp3");
                assert_eq!(remote_client, "default");
            }
            _ => panic!("Expected S3Files variant"),
        }
    }

    #[tokio::test]
    async fn test_from_json_invalid() {
        let json = r#"{"InvalidType":"some_value"}"#;
        assert!(PlaylistChildConfig::from_json(json).await.is_err());
    }

    #[tokio::test]
    async fn test_repeat_option() {
        let json = r#"{"LocalFolder":{"folder":"/path/to/folder","repeat":true}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFolder { folder, repeat, .. } => {
                assert_eq!(*folder, "/path/to/folder");
                assert_eq!(repeat, Some(true));
            }
            _ => panic!("Expected LocalFolder variant"),
        }
    }

    #[tokio::test]
    async fn test_shuffle_option() {
        let json = r#"{"LocalFiles":{"files":["/path/to/file1.mp3"],"shuffle":true}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::LocalFiles { shuffle, .. } => {
                assert_eq!(shuffle, Some(true));
            }
            _ => panic!("Expected LocalFiles variant"),
        }
    }

    #[tokio::test]
    async fn test_fail_over_option() {
        let json = r#"{"RemoteFolder":{"folder":"bucket/folder","remote_client":"default","fail_over":{"Silent":null}}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::RemoteFolder { fail_over, .. } => {
                assert!(fail_over.is_some());
                match *fail_over.unwrap() {
                    PlaylistChildConfig::Silent => {}
                    _ => panic!("Expected Silent fail_over variant"),
                }
            }
            _ => panic!("Expected S3Folder variant"),
        }
    }

    #[tokio::test]
    async fn test_all_options() {
        let json = r#"{"Playlists":{"children":[{"LocalFolder":{"folder":"/path/to/folder"}}],"repeat":true,"shuffle":false,"fail_over":{"Silent":null}}}"#;
        let config = PlaylistChildConfig::from_json(json).await.unwrap();
        match config {
            PlaylistChildConfig::Playlists {
                repeat,
                shuffle,
                fail_over,
                ..
            } => {
                assert_eq!(repeat, Some(true));
                assert_eq!(shuffle, Some(false));
                assert!(fail_over.is_some());
                match *fail_over.unwrap() {
                    PlaylistChildConfig::Silent => {}
                    _ => panic!("Expected Silent fail_over variant"),
                }
            }
            _ => panic!("Expected Playlists variant"),
        }
    }
}

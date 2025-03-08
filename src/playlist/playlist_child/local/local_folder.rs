use std::sync::Arc;

use crate::{FileProvider, playlist::PlaylistChild};
use async_trait::async_trait;
use bytes::Bytes;
use derive_lazy_playlist_child::LazyPlaylistChild;

use super::LocalFileTrackList;
use crate::LocalFileProvider;

#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "String"))]
#[custom_input_type(additional_input(name = "repeat", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(name = "shuffle", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(
    name = "file_provider",
    input_type = "Arc<dyn FileProvider>",
    default = "Arc::new(LocalFileProvider::new())",
    optional = true
))]
struct LocalFolderInner {
    /// list of local file tracks
    tracks: LocalFileTrackList,
}

impl LocalFolderInner {
    async fn new(
        tracks: String,
        repeat: bool,
        shuffle: bool,
        file_provider: Arc<dyn FileProvider>,
    ) -> anyhow::Result<Self> {
        // read the folder and get all the tracks
        let mut new_tracks = Vec::new();
        let mut dir = tokio::fs::read_dir(tracks).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                new_tracks.push((path.to_string_lossy().to_string(), file_provider.clone()));
            }
        }

        Ok(Self {
            tracks: LocalFileTrackList::new(new_tracks, Some(repeat), Some(shuffle)).await?,
        })
    }
}

impl_playlist_child_by_redirect_to_self_variable!(LocalFolderInner, tracks);

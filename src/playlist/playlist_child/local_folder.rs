use std::sync::Arc;

use crate::playlist::PlaylistChild;
use async_trait::async_trait;
use bytes::Bytes;
use derive_lazy_playlist_child::LazyPlaylistChild;

use super::LocalFileTrackList;

#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "String"))]
#[custom_input_type(additional_input(name = "repeat", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(name = "shuffle", input_type = "bool", default = "false"))]
struct LocalFolderListInner {
    /// list of local file tracks
    tracks: LocalFileTrackList,
}

impl LocalFolderListInner {
    async fn new(tracks: String, repeat: bool, shuffle: bool) -> anyhow::Result<Self> {
        // read the folder and get all the tracks
        let mut new_tracks = Vec::new();
        let mut dir = tokio::fs::read_dir(tracks).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                new_tracks.push(path.to_string_lossy().to_string());
            }
        }

        Ok(Self {
            tracks: LocalFileTrackList::new(new_tracks, Some(repeat), Some(shuffle)).await?,
        })
    }
}
#[async_trait]
impl PlaylistChild for LocalFolderListInner {
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>> {
        self.tracks.current_title().await
    }

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>> {
        self.tracks.current_artist().await
    }

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Arc<String>> {
        self.tracks.content_type().await
    }

    /// return the current byte_per_millisecond
    async fn byte_per_millisecond(&mut self) -> anyhow::Result<u128> {
        self.tracks.byte_per_millisecond().await
    }

    /// return a stream representing the current track, and the byte_per_millisecond
    /// the stream should be closed when the track is finished
    /// return none if the playlist is finished
    async fn next_frame(&mut self) -> anyhow::Result<Option<Bytes>> {
        self.tracks.next_frame().await
    }

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        self.tracks.is_finished().await
    }

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()> {
        self.tracks.reset().await
    }
}

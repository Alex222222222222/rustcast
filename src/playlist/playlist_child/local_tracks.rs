extern crate derive_lazy_playlist_child;

use std::sync::Arc;

use crate::playlist::PlaylistChild;
use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;

#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "Vec<String>"))]
struct LocalFileTrackListInner {
    /// list of local file tracks
    tracks: Vec<super::LocalFileTrack>,
    /// current track index
    current_index: usize,
    /// whether repeat the playlist
    repeat: bool,
    /// whether shuffle the playlist before playing
    shuffle: bool,
}

impl LocalFileTrackListInner {
    async fn new(tracks: Vec<String>, repeat: bool, shuffle: bool) -> anyhow::Result<Self> {
        todo!()
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrackListInner {
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>> {
        todo!()
    }

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>> {
        todo!()
    }

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Arc<String>> {
        todo!()
    }

    /// return a stream representing the current track, and the byte_per_millisecond
    /// the stream should be closed when the track is finished
    /// return none if the playlist is finished
    async fn next_stream(
        &mut self,
    ) -> anyhow::Result<
        Option<(
            Box<dyn tokio::io::AsyncRead + Unpin + Sync + std::marker::Send>,
            u128,
        )>,
    > {
        todo!()
    }

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        todo!()
    }

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

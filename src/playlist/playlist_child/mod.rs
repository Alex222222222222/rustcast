use std::sync::Arc;

use async_trait::async_trait;

mod local_track;
mod local_tracks;

pub use local_track::LocalFileTrack;
pub use local_tracks::LocalFileTrackList;
use tokio::io::AsyncRead;

#[async_trait]
pub trait PlaylistChild: Sync + Send {
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>>;

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>>;

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Arc<String>>;

    /// return a stream representing the current track, and the byte_per_millisecond
    /// the stream should be closed when the track is finished
    /// return none if the playlist is finished
    async fn next_stream(
        &mut self,
    ) -> anyhow::Result<Option<(Box<dyn AsyncRead + Unpin + Sync + std::marker::Send>, u128)>>;

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool>;

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()>;
}

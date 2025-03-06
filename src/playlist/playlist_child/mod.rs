use std::sync::Arc;

use async_trait::async_trait;

mod local_track;
mod local_tracks;
mod local_folder;

use bytes::Bytes;
pub use local_track::LocalFileTrack;
pub use local_tracks::LocalFileTrackList;

#[async_trait]
pub trait PlaylistChild: Sync + Send {
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>>;

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>>;

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Arc<String>>;

    /// return the current byte_per_millisecond
    async fn byte_per_millisecond(&mut self) -> anyhow::Result<u128>;

    /// return a stream representing the current track, and the byte_per_millisecond
    /// the stream should be closed when the track is finished
    /// return none if the playlist is finished
    async fn next_frame(&mut self) -> anyhow::Result<Option<Bytes>>;

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool>;

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()>;
}

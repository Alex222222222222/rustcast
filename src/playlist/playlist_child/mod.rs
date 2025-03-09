use std::sync::Arc;

use async_trait::async_trait;

#[macro_use]
mod impl_playlist_child_by_redirect_to_self_variable;

mod local;
mod playlist_child_list;

use bytes::Bytes;
// re-export the local module
pub use local::*;
pub use playlist_child_list::PlaylistChildList;

#[async_trait]
pub trait PlaylistChild: Sync + Send {
    /// current_title returns the title of current playing song
    /// return none if the playlist is finished
    async fn current_title(&mut self) -> anyhow::Result<Option<Arc<String>>>;

    /// Artist returns the artist which is currently playing.
    /// return none if the playlist is finished
    async fn current_artist(&mut self) -> anyhow::Result<Option<Arc<String>>>;

    /// return the current content type of the playlist
    /// return none if the playlist is finished
    async fn content_type(&mut self) -> anyhow::Result<Option<Arc<String>>>;

    /// return the current byte_per_millisecond
    /// return none if the playlist is finished
    async fn byte_per_millisecond(&mut self) -> anyhow::Result<Option<f64>>;

    /// return a stream representing the current track, and the byte_per_millisecond
    /// the stream should be closed when the track is finished
    /// return none if the playlist is finished
    async fn next_frame(&mut self) -> anyhow::Result<Option<Bytes>>;

    async fn next_frame_with_meta(
        &mut self,
    ) -> anyhow::Result<Option<(Bytes, Arc<String>, Arc<String>)>> {
        let frame = match self.next_frame().await? {
            Some(frame) => frame,
            None => return Ok(None),
        };
        let title = match self.current_title().await? {
            Some(title) => title,
            None => return Ok(None),
        };
        let artist = match self.current_artist().await? {
            Some(artist) => artist,
            None => return Ok(None),
        };
        Ok(Some((frame, title, artist)))
    }

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool>;

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()>;
}

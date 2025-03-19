use std::sync::Arc;

use async_trait::async_trait;

#[macro_use]
mod impl_playlist_child_by_redirect_to_self_variable;

mod infinite_shuffle_stream;
mod local;
mod playlist_child_list;
mod silent;

use bytes::Bytes;
// re-export the local module
pub use local::*;
pub use playlist_child_list::PlaylistChildList;
pub use silent::Silent;

#[derive(Clone)]
pub struct FrameWithMeta {
    pub frame: Bytes,
    pub title: Arc<String>,
    pub artist: Arc<String>,
    pub content_type: Arc<String>,
    /// duration of the frame in milliseconds
    pub duration: f64,
}

#[async_trait]
pub trait PlaylistChild: Sync + Send {
    async fn stream_frame_with_meta(
        &'_ mut self,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>>,
    >;

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool>;
}

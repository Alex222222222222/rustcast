extern crate derive_lazy_playlist_child;

use crate::playlist::PlaylistChild;
use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;

/*
pub struct LocalFileTrackList {
    inner: Option<LocalFileTrackListInner>,
    tracks_path: Option<Vec<String>>,
    repeat: bool,
    shuffle: bool,
}
*/

#[derive(LazyPlaylistChild)]
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
    async fn new(
        tracks: Vec<super::LocalFileTrack>,
        current_index: usize,
        repeat: bool,
        shuffle: bool,
    ) -> anyhow::Result<Self> {
        todo!()
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrackListInner {
    #[doc = " current_title returns the title of current playing song"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn current_title<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<Arc<String>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    #[doc = " Artist returns the artist which is currently playing."]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn current_artist<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<Arc<String>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    #[doc = " return the current content type of the playlist"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn content_type<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<Arc<String>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    #[doc = " return a stream representing the current track, and the byte_per_millisecond"]
    #[doc = " the stream should be closed when the track is finished"]
    #[doc = " return none if the playlist is finished"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn next_stream<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                    Output = anyhow::Result<
                        Option<(
                            Box<dyn tokio::io::AsyncRead + Unpin + Sync + std::marker::Send>,
                            u128,
                        )>,
                    >,
                > + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    #[doc = " check if the Playlist is finished"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn is_finished<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<bool>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    #[doc = " reset the played status of the child"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn reset<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}
extern crate derive_lazy_playlist_child;

use std::pin::Pin;

use crate::playlist::{LocalFileTrack, PlaylistChild, PlaylistChildList};
use async_trait::async_trait;
use bytes::Bytes;

pub struct LocalFileTrackList {
    t: PlaylistChildList<String>,
}

impl LocalFileTrackList {
    pub async fn new(
        tracks: Vec<String>,
        repeat: Option<bool>,
        shuffle: Option<bool>,
    ) -> anyhow::Result<Self> {
        fn init_fn(
            t: String,
        ) -> Pin<
            Box<
                dyn futures::Future<Output = anyhow::Result<Box<dyn PlaylistChild>>>
                    + std::marker::Send,
            >,
        > {
            Box::pin(async move {
                Ok(Box::new(LocalFileTrack::new(t, Some(false)).await?) as Box<dyn PlaylistChild>)
            })
        }
        let t = PlaylistChildList::new(tracks, repeat, shuffle, Some(init_fn)).await?;
        Ok(Self { t })
    }
}

impl_playlist_child_by_redirect_to_self_variable!(LocalFileTrackList, t);

extern crate derive_lazy_playlist_child;

use std::{pin::Pin, sync::Arc};

use crate::{
    FileProvider,
    playlist::{LocalFileTrack, PlaylistChild, PlaylistChildList},
};
use async_trait::async_trait;
use bytes::Bytes;

pub struct LocalFileTrackList {
    t: PlaylistChildList<(String, Arc<dyn FileProvider>)>,
}

impl LocalFileTrackList {
    pub async fn new(
        tracks: Vec<(String, Arc<dyn FileProvider>)>,
        repeat: Option<bool>,
        shuffle: Option<bool>,
    ) -> anyhow::Result<Self> {
        type PlaylistChildOutPin = Pin<
            Box<
                dyn futures::Future<Output = anyhow::Result<Box<dyn PlaylistChild>>>
                    + std::marker::Send,
            >,
        >;

        fn init_fn(t: (String, Arc<dyn FileProvider>)) -> PlaylistChildOutPin {
            Box::pin(async move {
                Ok(Box::new(LocalFileTrack::new(t.0, t.1, Some(false)).await?)
                    as Box<dyn PlaylistChild>)
            })
        }
        let t: PlaylistChildList<(String, Arc<dyn FileProvider>)> =
            PlaylistChildList::<(String, Arc<dyn FileProvider>)>::new(
                tracks,
                repeat,
                shuffle,
                Some(init_fn),
            )
            .await?;
        Ok(Self { t })
    }
}

impl_playlist_child_by_redirect_to_self_variable!(LocalFileTrackList, t);

extern crate derive_lazy_playlist_child;

use super::super::FrameWithMeta;
use std::{pin::Pin, sync::Arc};

use crate::{
    FileProvider,
    playlist::{LocalFileTrack, PlaylistChild, PlaylistChildList},
};
use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;

pub struct LocalFileTrackList {
    t: PlaylistChildList<Vec<Arc<String>>, Arc<dyn FileProvider>>,
}

type ReturnStream = Pin<Box<dyn Stream<Item = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>;

fn original_data2_stream(
    t: Arc<Vec<Arc<String>>>,
    fp: Arc<dyn FileProvider>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<ReturnStream>> + Send>> {
    let s = stream! {
        for i in t.iter() {
            yield Ok(Box::new(LocalFileTrack::new(i.clone(), fp.clone(), Some(false))?)
                as Box<dyn PlaylistChild>
            );
        }
    };
    let s: ReturnStream = Box::pin(s);

    Box::pin(async { Ok(s) })
}

impl LocalFileTrackList {
    pub fn new(
        tracks: Arc<Vec<Arc<String>>>,
        repeat: Option<bool>,
        shuffle: Option<bool>,
        file_provider: Arc<dyn FileProvider>,
    ) -> anyhow::Result<Self> {
        let t = PlaylistChildList::<Vec<Arc<String>>, Arc<dyn FileProvider>>::new(
            tracks,
            repeat,
            shuffle,
            original_data2_stream,
            file_provider,
        )?;
        Ok(Self { t })
    }
}

impl_playlist_child_by_redirect_to_self_variable!(LocalFileTrackList, t);

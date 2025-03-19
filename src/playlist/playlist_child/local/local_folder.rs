use std::{pin::Pin, sync::Arc};

use super::super::FrameWithMeta;
use crate::{
    FileProvider,
    playlist::{LocalFileTrack, PlaylistChild, PlaylistChildList},
};
use async_stream::stream;
use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;
use futures::Stream;
use tokio_stream::StreamExt;

#[allow(clippy::duplicated_attributes)]
#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "Arc<String>"))]
#[custom_input_type(additional_input(name = "repeat", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(name = "shuffle", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(name = "recursive", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(
    name = "file_provider",
    input_type = "Arc<dyn FileProvider>",
    default = "Arc::new(crate::LocalFileProvider::new())",
    optional = true
))]
struct LocalFolderInner {
    /// list of local file tracks
    tracks: PlaylistChildList<(Arc<String>, bool), Arc<dyn FileProvider>>,
}

async fn folder_to_stream(
    p: Arc<(Arc<String>, bool)>,
    file_provider: Arc<dyn FileProvider>,
) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>> {
    let recursive = p.1;
    let p = p.0.clone();
    let s = stream! {
        let file_provider1 = file_provider.clone();
        let mut o_s = file_provider.list_files(Some(p.as_ref()), recursive).await?;
        while let Some(i) = o_s.next().await {
            match i {
                Ok(i) => {
                    let res = LocalFileTrack::new(
                        Arc::new(i),
                        file_provider1.clone(),
                        Some(false),
                    );
                    if let Err(e) = res {
                        yield Err(e);
                        continue;
                    }
                    yield Ok(Box::new(res.unwrap()) as Box<dyn PlaylistChild>)
                },
                Err(e) => yield Err(e),
            }
        }
    };

    Ok(Box::pin(s))
}

type ReturnStream = Pin<Box<dyn Stream<Item = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>;

fn original_data2_stream_default(
    p: Arc<(Arc<String>, bool)>,
    fp: Arc<dyn FileProvider>,
) -> Pin<Box<dyn Future<Output = anyhow::Result<ReturnStream>> + Send>> {
    Box::pin(folder_to_stream(p, fp))
}

impl LocalFolderInner {
    async fn new(
        tracks: Arc<String>,
        repeat: bool,
        shuffle: bool,
        recursive: bool,
        file_provider: Arc<dyn FileProvider>,
    ) -> anyhow::Result<Self> {
        let tracks = Arc::new((tracks, recursive));
        Ok(Self {
            tracks: PlaylistChildList::new(
                tracks,
                Some(repeat),
                Some(shuffle),
                original_data2_stream_default,
                file_provider,
            )?,
        })
    }
}

impl_playlist_child_by_redirect_to_self_variable!(LocalFolderInner, tracks);

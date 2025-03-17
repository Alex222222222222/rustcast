use async_stream::stream;
use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;
use futures::{StreamExt, pin_mut};
use std::sync::Arc;

use super::{
    FrameWithMeta, PlaylistChild,
    infinite_shuffle_stream::{InfiniteShuffleStream, OriginalData2Stream},
};

/// PlaylistChildList is a struct that contains a list of
/// playlist children, the current index, whether to repeat
/// the playlist, whether to shuffle the playlist, and whether
/// the playlist is played.
#[allow(clippy::duplicated_attributes)]
#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "Arc<O>"))]
#[custom_input_type(additional_input(name = "repeat", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(name = "shuffle", input_type = "bool", default = "false"))]
#[custom_input_type(additional_input(
    name = "original_data2_stream",
    input_type = "OriginalData2Stream<O, Box<dyn PlaylistChild>,FP>",
    default = "original_data2_stream_default",
    optional = true
))]
#[custom_input_type(additional_input(
    name = "file_provider",
    input_type = "FP",
    default = "FP::default()",
    optional = true
))]
struct PlaylistChildListInner<O, FP>
where
    O: Send + Sync + Unpin,
    FP: Send + Sync + Unpin + Clone,
{
    tracks: InfiniteShuffleStream<O, Box<dyn PlaylistChild>, FP>,
    /// whether the playlist is played
    played: bool,
}

impl<O, FP> PlaylistChildListInner<O, FP>
where
    O: Send + Sync + Unpin,
    FP: Send + Sync + Unpin + Clone,
{
    async fn new(
        tracks: Arc<O>,
        repeat: bool,
        shuffle: bool,
        original_data2_stream: OriginalData2Stream<O, Box<dyn PlaylistChild>, FP>,
        file_provider: FP,
    ) -> anyhow::Result<Self> {
        let tracks = InfiniteShuffleStream::new(
            file_provider,
            tracks,
            repeat,
            shuffle,
            original_data2_stream,
        );

        Ok(Self {
            tracks,
            played: false,
        })
    }
}

#[async_trait]
impl<O, FP> PlaylistChild for PlaylistChildListInner<O, FP>
where
    O: Send + Sync + Unpin,
    FP: Send + Sync + Unpin + Clone,
{
    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        Ok(self.played)
    }

    async fn stream_frame_with_meta(
        &'_ mut self,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>>,
    > {
        let s = stream! {
            let s = self.tracks.stream();
            let s = s.fuse();
            pin_mut!(s);

            loop {
                let data = s.next().await;
                if data.is_none() {
                    self.played = true;
                    break;
                }
                let data = data.unwrap();
                if let Err(e) = data {
                    yield Err(e);
                    continue;
                }
                let mut data = data.unwrap();
                let data_s = match data.stream_frame_with_meta().await {
                    Ok(s) => s,
                    Err(e) => {
                        yield Err(e);
                        continue;
                    }
                };
                let mut data_s = data_s.fuse();

                loop {
                    let frame = data_s.next().await;
                    match frame {
                        None => break,
                        Some(f) => {
                            yield f;
                        }
                    }
                }
            }
        };

        Ok(Box::pin(s))
    }
}

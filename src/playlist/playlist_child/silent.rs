use super::super::FrameWithMeta;
use async_stream::stream;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::playlist::PlaylistChild;

const SILENT_FILE: &'static [u8; 37206] = include_bytes!("../../../1-second-of-silence.mp3");

const SILENT_FILE_FRAME_WITH_META: once_cell::sync::Lazy<FrameWithMeta> =
    Lazy::new(|| FrameWithMeta {
        frame: bytes::Bytes::from_static(SILENT_FILE),
        title: Arc::new("Silent".to_string()),
        artist: Arc::new("Silent".to_string()),
        content_type: Arc::new("audio/mpeg".to_string()),
        duration: 1000.0,
    });

pub struct Silent {}

impl Silent {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl PlaylistChild for Silent {
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn stream_frame_with_meta(
        &'_ mut self,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>>,
    > {
        let s = stream! {
            loop {
                yield Ok((*SILENT_FILE_FRAME_WITH_META).clone())
            }
        };

        Ok(Box::pin(s))
    }
}

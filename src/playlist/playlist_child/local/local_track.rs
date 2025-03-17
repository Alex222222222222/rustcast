use super::super::FrameWithMeta;
use async_stream::stream;
use async_trait::async_trait;
use core::str;
use derive_lazy_playlist_child::LazyPlaylistChild;
use id3::TagLike;
use log::debug;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    FileProvider,
    playlist::{DEFAULT_FRAME_SIZE, PlaylistChild},
};

/// use Arc to share the same content between threads
static FILE_EXT_CONTENT_TYPES: Lazy<HashMap<String, Arc<String>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("mp3".to_string(), "audio/mpeg".to_string().into());
    m.insert("flac".to_string(), "audio/flac".to_string().into());
    m.insert("aac".to_string(), "audio/x-aac".to_string().into());
    m.insert("mp4a".to_string(), "audio/mp4".to_string().into());
    m.insert("mp4".to_string(), "video/mp4".to_string().into());
    m.insert("nsv".to_string(), "video/nsv".to_string().into());
    m.insert("ogg".to_string(), "audio/ogg".to_string().into());
    m.insert("spx".to_string(), "audio/ogg".to_string().into());
    m.insert("opus".to_string(), "audio/ogg".to_string().into());
    m.insert("oga".to_string(), "audio/ogg".to_string().into());
    m.insert("ogv".to_string(), "video/ogg".to_string().into());
    m.insert("weba".to_string(), "audio/webm".to_string().into());
    m.insert("webm".to_string(), "video/webm".to_string().into());
    m.insert("axa".to_string(), "audio/annodex".to_string().into());
    m.insert("axv".to_string(), "video/annodex".to_string().into());
    m
});

fn get_content_type_from_path(path: &str) -> Option<Arc<String>> {
    let ext = get_ext(path)?;
    FILE_EXT_CONTENT_TYPES.get(ext.as_str()).cloned()
}

fn get_ext(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|ext| ext.to_lowercase())
}

struct MetaData {
    content_type: Arc<String>,
    title: Option<String>,
    artist: Option<String>,
    byte_per_millisecond: f64,
}

async fn get_meta_data_from_file(
    path: &str,
    file_provider: Arc<dyn FileProvider>,
) -> anyhow::Result<MetaData> {
    let cache_path = match file_provider.get_local_cache_path(path).await? {
        Some(cache_path) => cache_path,
        None => return Err(anyhow::anyhow!("file not found")),
    };
    let content_type = match get_content_type_from_path(path) {
        Some(content_type) => content_type,
        None => return Err(anyhow::anyhow!("unsupported file type")),
    };

    let mut title = None;
    let mut artist = None;
    let mut duration = Option::None;

    let tag = id3::Tag::read_from_path(&cache_path);
    let tag = id3::partial_tag_ok(tag);
    if let Ok(tag) = tag {
        title = tag.title().map(|t| t.to_string());
        artist = tag.artist().map(|t| t.to_string());
        duration = tag.duration();
    }

    if duration.is_none() {
        duration = match mp3_duration::from_path(&cache_path) {
            Ok(duration) => Some((duration.as_nanos() / 1_000_000) as u32),
            Err(_) => None,
        };
    }
    let duration = match duration {
        Some(duration) => duration,
        None => return Err(anyhow::anyhow!("failed to get duration")),
    };

    // get size of the file
    let meta = match file_provider.get_meta(path).await? {
        Some(meta) => meta,
        None => return Err(anyhow::anyhow!("file not found")),
    };
    let size = meta.size;

    // calculate the bitrate
    let byte_per_millisecond = (size + 1) as f64 / duration as f64;

    Ok(MetaData {
        content_type,
        title,
        artist,
        byte_per_millisecond,
    })
}

#[derive(LazyPlaylistChild)]
pub struct LocalFileTrackInner {
    path: Arc<String>,
    file_provider: Arc<dyn FileProvider>,
    title: Arc<String>,
    artist: Arc<String>,
    content_type: Arc<String>,
    repeat: bool,
    played: bool,
    byte_per_millisecond: f64,
}

impl LocalFileTrackInner {
    pub async fn new(
        path: Arc<String>,
        file_provider: Arc<dyn FileProvider>,
        repeat: bool,
    ) -> anyhow::Result<Self> {
        let mut meta_data = get_meta_data_from_file(&path, file_provider.clone()).await?;

        if meta_data.title.is_none() || meta_data.artist.is_none() {
            debug!("failed to get title and artist from id3 tag, trying to get from file name");
            let file_name = match std::path::Path::new(path.as_str())
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
            {
                Some(file_name) => file_name,
                None => return Err(anyhow::anyhow!("invalid file name")),
            };

            let mut file_name = file_name.split('-');
            let first = file_name.next();
            if meta_data.title.is_none() {
                meta_data.title = first.map(|t| t.trim().to_string());
            }
            if meta_data.artist.is_none() {
                meta_data.artist = file_name.next().map(|t| t.trim().to_string())
            }
            debug!(
                "got title: {:?}, artist: {:?}",
                meta_data.title, meta_data.artist
            );
        }

        let title = meta_data
            .title
            .unwrap_or("Unknown Track".to_string())
            .into();
        let artist = meta_data
            .artist
            .unwrap_or("Unknown Artist".to_string())
            .into();

        Ok(Self {
            path,
            title,
            artist,
            file_provider,
            content_type: meta_data.content_type,
            repeat,
            played: false,
            byte_per_millisecond: meta_data.byte_per_millisecond,
        })
    }
}

impl LocalFileTrackInner {
    async fn new_stream(&self) -> anyhow::Result<Box<dyn AsyncRead + Send + Sync + Unpin>> {
        let stream = match self.file_provider.get_file(&self.path).await? {
            Some(stream) => stream,
            None => return Err(anyhow::anyhow!("file not found")),
        };
        Ok(Box::new(stream))
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrackInner {
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        Ok(self.played && !self.repeat)
    }

    async fn stream_frame_with_meta(
        &'_ mut self,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>>,
    > {
        let s = stream! {
            loop {
                let mut stream = self.new_stream().await?;
                let mut start = true;
                loop {
                    let mut buf = vec![0; DEFAULT_FRAME_SIZE];
                    let read = stream.read(&mut buf).await?;
                    if read == 0 && start {
                        Err(anyhow::anyhow!("file is empty"))?;
                    } else if read == 0 {
                        break;
                    }

                    let frame = bytes::Bytes::from(buf).slice(0..read);
                    let frame_with_meta = FrameWithMeta {
                        duration: frame.len() as f64 / self.byte_per_millisecond,
                        frame,
                        title: self.title.clone(),
                        artist: self.artist.clone(),
                        content_type: self.content_type.clone(),

                    };
                    start = false;
                    yield Ok(frame_with_meta);
                }
                if !self.repeat {
                    self.played = true;
                    break;
                }
            }
        };

        Ok(Box::pin(s))
    }
}

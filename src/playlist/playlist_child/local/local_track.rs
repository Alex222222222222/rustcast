use async_trait::async_trait;
use core::str;
use derive_lazy_playlist_child::LazyPlaylistChild;
use id3::TagLike;
use log::debug;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::playlist::{DEFAULT_FRAME_SIZE, PlaylistChild};

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
    debug!("get content type from path: {}", path);
    let ext = get_ext(path)?;
    debug!("got file extension: {}", ext);
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
    byte_per_millisecond: u128,
}

fn get_meta_data_from_file(path: &str) -> anyhow::Result<MetaData> {
    let content_type = match get_content_type_from_path(path) {
        Some(content_type) => content_type,
        None => return Err(anyhow::anyhow!("unsupported file type")),
    };
    debug!("got content type: {}", content_type);

    let mut title = None;
    let mut artist = None;
    let mut duration = Option::None;

    debug!("reading id3 tag from file: {}", path);
    let tag = id3::Tag::read_from_path(path);
    let tag = id3::partial_tag_ok(tag);
    if let Ok(tag) = tag {
        title = tag.title().map(|t| t.to_string());
        artist = tag.artist().map(|t| t.to_string());
        duration = tag.duration();
        debug!("got title: {:?}, artist: {:?}", title, artist);
    } else {
        debug!("failed to read id3 tag from file: {}", path);
    }

    if duration.is_none() {
        duration = match mp3_duration::from_path(path) {
            Ok(duration) => Some((duration.as_nanos() / 1_000_000) as u32),
            Err(_) => None,
        };
    }
    let duration = match duration {
        Some(duration) => duration,
        None => return Err(anyhow::anyhow!("failed to get duration")),
    };

    // get size of the file
    let file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    debug!("got file size: {}", size);

    // calculate the bitrate
    let byte_per_millisecond = size / duration as u64 + 1;
    debug!("byte per millisecond: {}", byte_per_millisecond);

    Ok(MetaData {
        content_type,
        title,
        artist,
        byte_per_millisecond: byte_per_millisecond as u128,
    })
}

#[derive(LazyPlaylistChild)]
pub struct LocalFileTrackInner {
    path: String,
    title: Arc<String>,
    artist: Arc<String>,
    content_type: Arc<String>,
    repeat: bool,
    played: bool,
    byte_per_millisecond: u128,
    current_stream: Option<Box<dyn AsyncRead + Send + Sync + Unpin>>,
}

impl LocalFileTrackInner {
    pub async fn new(path: String, repeat: bool) -> anyhow::Result<Self> {
        let mut meta_data = get_meta_data_from_file(&path)?;

        if meta_data.title.is_none() || meta_data.artist.is_none() {
            debug!("failed to get title and artist from id3 tag, trying to get from file name");
            let file_name = match std::path::Path::new(&path)
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
            content_type: meta_data.content_type,
            repeat,
            played: false,
            byte_per_millisecond: meta_data.byte_per_millisecond,
            current_stream: None,
        })
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrackInner {
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.title.clone())
    }

    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.artist.clone())
    }

    async fn content_type(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.content_type.clone())
    }

    async fn byte_per_millisecond(&mut self) -> anyhow::Result<u128> {
        Ok(self.byte_per_millisecond)
    }

    async fn next_frame(&mut self) -> anyhow::Result<Option<bytes::Bytes>> {
        if self.played && !self.repeat {
            self.current_stream = None;
            return Ok(None);
        }

        if self.current_stream.is_none() {
            let stream = tokio::fs::File::open(&self.path).await?;
            self.current_stream = Some(Box::new(stream));
        }

        let stream = self.current_stream.as_mut().unwrap();
        let mut buf = vec![0; DEFAULT_FRAME_SIZE];
        let mut read = stream.read(&mut buf).await?;
        if read == 0 {
            self.played = true;
            if !self.repeat {
                self.current_stream = None;
                return Ok(None);
            }

            let mut stream = tokio::fs::File::open(&self.path).await?;
            read = stream.read(&mut buf).await?;
            if read == 0 {
                self.current_stream = None;
                anyhow::bail!("failed to read from file: {}", self.path);
            }
            self.current_stream = Some(Box::new(stream));
        }

        let frame = bytes::Bytes::from(buf).slice(0..read);
        Ok(Some(frame))
    }

    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        Ok(self.played && !self.repeat)
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        self.played = false;
        self.current_stream = None;
        Ok(())
    }
}

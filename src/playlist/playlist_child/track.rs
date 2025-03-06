use async_trait::async_trait;
use id3::TagLike;
use log::debug;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

use tokio::io::AsyncRead;

use super::PlaylistChild;

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
    let ext = match get_ext(path) {
        Some(ext) => ext,
        None => return None,
    };
    debug!("got file extension: {}", ext);
    match FILE_EXT_CONTENT_TYPES.get(ext.as_str()) {
        Some(content_type) => Some(content_type.clone()),
        None => None,
    }
}

fn get_ext(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(|ext| Some(ext.to_lowercase()))
}

fn get_meta_data_from_file(
    path: &str,
) -> anyhow::Result<(Arc<String>, Option<String>, Option<String>, u128)> {
    let content_type = match get_content_type_from_path(&path) {
        Some(content_type) => content_type,
        None => return Err(anyhow::anyhow!("unsupported file type")),
    };
    debug!("got content type: {}", content_type);

    let mut title = None;
    let mut artist = None;
    let mut duration = Option::None;

    debug!("reading id3 tag from file: {}", path);
    let tag = id3::Tag::read_from_path(&path);
    let tag = id3::partial_tag_ok(tag);
    if let Ok(tag) = tag {
        title = tag.title().and_then(|t| Some(t.to_string()));
        artist = tag.artist().and_then(|t| Some(t.to_string()));
        duration = tag.duration();
        debug!("got title: {:?}, artist: {:?}", title, artist);
    } else {
        debug!("failed to read id3 tag from file: {}", path);
    }

    if duration.is_none() {
        duration = match mp3_duration::from_path(&path) {
            Ok(duration) => Some((duration.as_nanos() / 1_000_000) as u32),
            Err(_) => None,
        };
    }
    let duration = match duration {
        Some(duration) => duration,
        None => return Err(anyhow::anyhow!("failed to get duration")),
    };

    // get size of the file
    let file = std::fs::File::open(&path)?;
    let size = file.metadata()?.len();
    debug!("got file size: {}", size);

    // calculate the bitrate
    let byte_per_millisecond = size / duration as u64 + 1;
    debug!("byte per millisecond: {}", byte_per_millisecond);

    Ok((content_type, title, artist, byte_per_millisecond as u128))
}

pub struct LocalFileTrack {
    path: String,
    title: Arc<String>,
    artist: Arc<String>,
    content_type: Arc<String>,
    byte_per_millisecond: u128,
}

impl LocalFileTrack {
    pub fn new(path: String) -> anyhow::Result<Self> {
        let (content_type, mut title, mut artist, byte_per_millisecond) =
            get_meta_data_from_file(&path)?;

        if title.is_none() || artist.is_none() {
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
            if title.is_none() {
                title = first.and_then(|t| Some(t.trim().to_string()));
            }
            if artist.is_none() {
                artist = file_name.next().and_then(|t| Some(t.trim().to_string()));
            }
            debug!("got title: {:?}, artist: {:?}", title, artist);
        }

        let title = title.unwrap_or("Unknown Track".to_string()).into();
        let artist = artist.unwrap_or("Unknown Artist".to_string()).into();

        Ok(Self {
            path,
            title,
            artist,
            content_type,
            byte_per_millisecond,
        })
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrack {
    async fn current_title(&self) -> Arc<String> {
        self.title.clone()
    }

    async fn current_artist(&self) -> Arc<String> {
        self.artist.clone()
    }

    fn content_type(&self) -> Arc<String> {
        self.content_type.clone()
    }

    async fn next_stream(
        &mut self,
    ) -> anyhow::Result<Option<(Box<dyn AsyncRead + Unpin + Sync + std::marker::Send>, u128)>> {
        let stream = tokio::fs::File::open(&self.path).await?;
        Ok(Some((Box::new(stream), self.byte_per_millisecond)))
    }

    async fn is_finished(&self) -> bool {
        false
    }
}

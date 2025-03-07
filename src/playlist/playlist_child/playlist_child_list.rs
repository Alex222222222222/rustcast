use std::{marker::PhantomData, pin::Pin, sync::Arc};

use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;
use rand::seq::SliceRandom;

use log::{debug, error};

use super::PlaylistChild;

/// PlaylistChildList is a struct that contains a list of
/// playlist children, the current index, whether to repeat
/// the playlist, whether to shuffle the playlist, and whether
/// the playlist is played.
#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "Vec<O>"))]
#[custom_input_type(additional_input(
    name = "init",
    input_type = "Option<
            fn(O) -> Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>,
        >",
    default = "None",
    optional = true
))]
struct PlaylistChildListInner<O>
where
    O: Send + Sync,
{
    /// list of local file tracks
    tracks: Vec<Box<dyn PlaylistChild>>,
    /// current track index
    current_index: usize,
    /// whether repeat the playlist
    repeat: bool,
    /// whether shuffle the playlist before playing
    shuffle: bool,
    /// whether the playlist is played
    played: bool,
    /// PhantomData to hold the type of the playlist child
    p: PhantomData<O>,
}

impl<O> PlaylistChildListInner<O>
where
    O: Send + Sync,
{
    async fn new(
        tracks: Vec<O>,
        repeat: bool,
        shuffle: bool,
        init: Option<
            fn(O) -> Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>,
        >,
    ) -> anyhow::Result<Self> {
        let init = match init {
            Some(init) => init,
            None => {
                return Err(anyhow::anyhow!(
                    "init function is required for PlaylistChildListInner"
                ));
            }
        };
        let mut tracks = tracks;
        let mut new_tracks: Vec<Box<dyn PlaylistChild>> = Vec::with_capacity(tracks.len());
        while let Some(track) = tracks.pop() {
            debug!("creating playlist child");
            let track = match (init)(track).await {
                Ok(track) => track,
                Err(e) => {
                    log::error!("failed to create playlist child: {}", e);
                    continue;
                }
            };
            new_tracks.push(track);
        }
        if shuffle {
            new_tracks.shuffle(&mut rand::rng());
        } else {
            new_tracks.reverse();
        }
        Ok(Self {
            tracks: new_tracks,
            current_index: 0,
            repeat,
            shuffle,
            played: false,
            p: PhantomData,
        })
    }
}

impl<O> PlaylistChildListInner<O>
where
    O: Send + Sync,
{
    async fn remove_track(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.tracks.len() {
            return Err(anyhow::anyhow!("index out of range"));
        }
        self.tracks.remove(index);
        self.check_current_index_out_range().await?;

        Ok(())
    }

    async fn check_current_index_out_range(&mut self) -> anyhow::Result<()> {
        if self.current_index >= self.tracks.len() {
            self.played = true;
            self.current_index = 0;

            if self.shuffle {
                self.tracks.shuffle(&mut rand::rng());
            }
            for track in &mut self.tracks {
                track.reset().await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<O> PlaylistChild for PlaylistChildListInner<O>
where
    O: Send + Sync,
{
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Option<Arc<String>>> {
        loop {
            if self.tracks.is_empty() {
                return Ok(None);
            }
            if self.played && !self.repeat {
                return Ok(None);
            }
            let t = self.tracks[self.current_index].current_title().await;
            if t.is_ok() {
                return t;
            }
            error!("failed to get current title of track: {}", t.err().unwrap());
            self.remove_track(self.current_index).await?;
        }
    }

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Option<Arc<String>>> {
        loop {
            if self.tracks.is_empty() {
                return Ok(None);
            }
            if self.played && !self.repeat {
                return Ok(None);
            }
            let t = self.tracks[self.current_index].current_artist().await;
            if t.is_ok() {
                return t;
            }
            error!(
                "failed to get current artist of track: {}",
                t.err().unwrap()
            );
            self.remove_track(self.current_index).await?;
        }
    }

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Option<Arc<String>>> {
        loop {
            if self.tracks.is_empty() {
                return Ok(None);
            }
            if self.played && !self.repeat {
                return Ok(None);
            }
            let t = self.tracks[self.current_index].content_type().await;
            if t.is_ok() {
                return t;
            }
            error!("failed to get content type of track: {}", t.err().unwrap());
            self.remove_track(self.current_index).await?;
        }
    }

    /// return the current byte_per_millisecond
    async fn byte_per_millisecond(&mut self) -> anyhow::Result<Option<f64>> {
        loop {
            if self.tracks.is_empty() {
                return Ok(None);
            }
            if self.played && !self.repeat {
                return Ok(None);
            }
            let t = self.tracks[self.current_index].byte_per_millisecond().await;
            if t.is_ok() {
                return t;
            }
            error!(
                "failed to get byte per millisecond of track: {}",
                t.err().unwrap()
            );
            self.remove_track(self.current_index).await?;
        }
    }

    /// return a stream representing the current track, and the byte_per_millisecond
    async fn next_frame(&mut self) -> anyhow::Result<Option<bytes::Bytes>> {
        let mut start_of_track = false;
        loop {
            self.check_current_index_out_range().await?;

            if self.played && !self.repeat {
                return Ok(None);
            }
            if self.tracks.is_empty() {
                return Ok(None);
            }

            let frame = match self.tracks[self.current_index].next_frame().await {
                Ok(frame) => frame,
                Err(e) => {
                    error!("failed to get next frame: {}", e);
                    self.remove_track(self.current_index).await?;
                    continue;
                }
            };

            if let Some(frame) = frame {
                return Ok(Some(frame));
            }

            debug!("end of track");
            // read to the end of current stream move to the next track
            self.current_index += 1;
            // if the current index is the start of the track, and the frame is none
            // then the track may have error, remove it
            if start_of_track {
                self.remove_track(self.current_index - 1).await?;
            } else {
                start_of_track = true;
            }
        }
    }

    /// check if the Playlist is finished
    async fn is_finished(&mut self) -> anyhow::Result<bool> {
        Ok((self.played && !self.repeat) || self.tracks.is_empty())
    }

    /// reset the played status of the child
    async fn reset(&mut self) -> anyhow::Result<()> {
        self.current_index = 0;
        self.played = false;

        for track in &mut self.tracks {
            track.reset().await?;
        }
        if self.shuffle {
            self.tracks.shuffle(&mut rand::rng());
        }

        Ok(())
    }
}

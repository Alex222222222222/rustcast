extern crate derive_lazy_playlist_child;

use std::sync::Arc;

use crate::playlist::PlaylistChild;
use async_trait::async_trait;
use derive_lazy_playlist_child::LazyPlaylistChild;
use rand::seq::SliceRandom;

#[derive(LazyPlaylistChild)]
#[custom_input_type(input_type(name = "tracks", input_type = "Vec<String>"))]
struct LocalFileTrackListInner {
    /// list of local file tracks
    tracks: Vec<super::LocalFileTrack>,
    /// current track index
    current_index: usize,
    /// whether repeat the playlist
    repeat: bool,
    /// whether shuffle the playlist before playing
    shuffle: bool,
    /// whether the playlist is played
    played: bool,
}

impl LocalFileTrackListInner {
    async fn new(tracks: Vec<String>, repeat: bool, shuffle: bool) -> anyhow::Result<Self> {
        let mut tracks = tracks;
        let mut new_tracks = Vec::with_capacity(tracks.len());
        while let Some(track) = tracks.pop() {
            new_tracks.push(super::LocalFileTrack::new(track, Some(false)).await?);
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
        })
    }
}

#[async_trait]
impl PlaylistChild for LocalFileTrackListInner {
    /// current_title returns the title of current playing song
    async fn current_title(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.tracks[self.current_index].current_title().await?)
    }

    /// Artist returns the artist which is currently playing.
    async fn current_artist(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.tracks[self.current_index].current_artist().await?)
    }

    /// return the current content type of the playlist
    async fn content_type(&mut self) -> anyhow::Result<Arc<String>> {
        Ok(self.tracks[self.current_index].content_type().await?)
    }

    /// return the current byte_per_millisecond
    async fn byte_per_millisecond(&mut self) -> anyhow::Result<u128> {
        Ok(self.tracks[self.current_index]
            .byte_per_millisecond()
            .await?)
    }

    /// return a stream representing the current track, and the byte_per_millisecond
    async fn next_frame(&mut self) -> anyhow::Result<Option<bytes::Bytes>> {
        if self.played && !self.repeat {
            return Ok(None);
        }
        if self.tracks.is_empty() {
            return Ok(None);
        }
        loop {
            let frame = self.tracks[self.current_index].next_frame().await?;

            if let Some(frame) = frame {
                return Ok(Some(frame));
            }

            // move to the next track
            self.current_index += 1;
            if self.current_index >= self.tracks.len() {
                self.played = true;
                if self.repeat {
                    self.current_index = 0;
                } else {
                    return Ok(None);
                }
                if self.shuffle {
                    self.tracks.shuffle(&mut rand::rng());
                }
                for track in &mut self.tracks {
                    track.reset().await?;
                }
            }
            let frame = self.tracks[self.current_index].next_frame().await?;
            // if the frame is None, then this track is broken, remove it
            if let Some(frame) = frame {
                return Ok(Some(frame));
            }
            self.tracks.remove(self.current_index);
            if self.current_index >= self.tracks.len() {
                self.played = true;
                if self.repeat {
                    self.current_index = 0;
                } else {
                    return Ok(None);
                }
                if self.shuffle {
                    self.tracks.shuffle(&mut rand::rng());
                }
                for track in &mut self.tracks {
                    track.reset().await?;
                }
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

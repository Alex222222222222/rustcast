use std::{sync::Arc, vec};

use bytes::Bytes;
use log::debug;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::Mutex,
};

use crate::{CONTEXT, DB};

use super::{DEFAULT_FRAME_SIZE, PlaylistChild};

/// We keep a linked list of `PreparedFrame` to stream to the client,
///     each `prepared_frame` is wrapped in an `Arc<Mutex<PreparedFrame>>`
///     to allow multiple clients to point to the same list,
///     while allowing them to listen to different frames.
///
/// `oldest_prepared_frames` is the entry point of the linked list,
///     every new client will start from here.
/// `newest_prepared_frames` is the end of the linked list,
///     when a client reaches this point, it and tries to get the next frame,
///     the `prepare_frame` function will be called to prepare the next frame.
/// We use a in memory sqlite db to store the listener's current frame id
///     that have been written to the client.
///     When a listener request a new frame,
///     the function `log_current_frame` will be called
///     to update the listener's current frame id in `db`.
///     The function `update_oldest_frame_by_db_smallest_frame_id`
///     will be chain triggered by `log_current_frame` to update
///     `oldest_prepared_frames` to match the smallest frame id in the db.
/// When a listener disconnects, the function `delete_listener_data`
///     will be called to delete the listener's data in the db.
///     The function `update_oldest_frame_by_db_smallest_frame_id`
///     will be chain triggered by `delete_listener_data` to update
///     `oldest_prepared_frames` to match the smallest frame id in the db.
pub struct Playlist {
    pub name: Arc<String>,
    db: DB,
    child: Arc<Mutex<dyn PlaylistChild>>,
    newest_prepared_frames: Arc<Mutex<PreparedFrame>>,
    oldest_prepared_frames: Arc<Mutex<PreparedFrame>>,
    current_stream: Arc<Mutex<(Box<dyn AsyncRead + Unpin + Sync + std::marker::Send>, u128)>>,
}

impl Playlist {
    pub async fn new(name: String, child: Arc<Mutex<dyn PlaylistChild>>, db: DB) -> Self {
        let frame = Arc::new(Mutex::new(PreparedFrame {
            frame: Bytes::new(),
            duration: 0,
            id: CONTEXT.get_id().await,
            next: Arc::new(Mutex::new(None)),
        }));
        Self {
            name: name.into(),
            child,
            newest_prepared_frames: frame.clone(),
            oldest_prepared_frames: frame,
            current_stream: Arc::new(Mutex::new((Box::new(tokio::io::empty()), 0))),
            db,
        }
    }

    /// delete a listener from the playlist
    pub async fn delete_listener_data(&self, listener_id: usize) -> anyhow::Result<()> {
        self.db.delete_listener_data(listener_id).await?;

        self.update_oldest_frame_by_db_smallest_frame_id().await
    }

    /// log the listener current frame
    pub async fn log_current_frame(
        &self,
        listener_id: usize,
        frame_id: usize,
    ) -> anyhow::Result<()> {
        self.db
            .insert_listener_frame_data(listener_id, frame_id)
            .await?;

        self.update_oldest_frame_by_db_smallest_frame_id().await
    }

    /// get the smallest frame id in ListenerFrame
    async fn get_smallest_frame_id(&self) -> anyhow::Result<Option<i64>> {
        self.db.get_smallest_frame_id().await
    }

    async fn update_oldest_frame_by_db_smallest_frame_id(&self) -> anyhow::Result<()> {
        let db_smallest_frame_id = match self.get_smallest_frame_id().await? {
            Some(id) => id as usize,
            None => return Ok(()),
        };

        let mut self_oldest_frame_id = self.get_self_oldest_frame_id().await;
        while db_smallest_frame_id < self_oldest_frame_id {
            self.advance_oldest_frame().await;
            self_oldest_frame_id = self.get_self_oldest_frame_id().await;
        }

        Ok(())
    }

    async fn advance_oldest_frame(&self) {
        let mut oldest_prepared_frames = self.oldest_prepared_frames.lock().await;
        let next = oldest_prepared_frames.get_next().await;
        if let Some(next) = next {
            *oldest_prepared_frames = next;
        }
    }

    async fn get_self_oldest_frame_id(&self) -> usize {
        self.oldest_prepared_frames.lock().await.id
    }

    /// current_title returns the title of current playing song
    pub async fn current_title(&self) -> Arc<String> {
        self.child.lock().await.current_title().await
    }

    ///    Artist returns the artist which is currently playing.
    pub async fn current_artist(&self) -> Arc<String> {
        self.child.lock().await.current_artist().await
    }

    /// return the current content type of the playlist
    pub async fn content_type(&self) -> Arc<String> {
        self.child.lock().await.content_type().clone()
    }

    /// check if the Playlist is finished
    pub async fn is_finished(&self) -> bool {
        self.child.lock().await.is_finished().await
    }

    pub async fn get_oldest_prepared_frames(&self) -> PreparedFrame {
        self.oldest_prepared_frames.lock().await.clone()
    }

    /// prepare_frames prepares the frames for the playlist
    /// do nothing if the playlist is finished
    /// or the playlist already has the next frame
    pub async fn prepare_frame(&self) -> anyhow::Result<()> {
        debug!("prepare frame for playlist: {:?}", self.name);
        if self.is_finished().await {
            debug!("playlist is finished: {:?}", self.name);
            return Ok(());
        }

        let newest_prepared_frames = self.newest_prepared_frames.lock().await;
        if newest_prepared_frames.has_next().await {
            debug!("playlist already has next frame: {:?}", self.name);
            return Ok(());
        }
        drop(newest_prepared_frames);

        loop {
            let mut current_stream = self.current_stream.lock().await;
            let mut buf = vec![0; DEFAULT_FRAME_SIZE];
            let read = current_stream.0.read(&mut buf).await?;
            if read == 0 {
                let mut child = self.child.lock().await;
                if child.is_finished().await {
                    return Ok(());
                }

                let new_stream = child.next_stream().await?;
                if let Some(new_stream) = new_stream {
                    *current_stream = new_stream;
                    continue;
                } else {
                    return Ok(());
                }
            }

            let frame = Bytes::from(buf).slice(0..read);
            let duration = read as u128 / current_stream.1;
            let prepared_frame = PreparedFrame {
                id: CONTEXT.get_id().await,
                frame,
                duration,
                next: Arc::new(Mutex::new(None)),
            };

            let mut newest_prepared_frames = self.newest_prepared_frames.lock().await;
            newest_prepared_frames
                .set_next(prepared_frame.clone())
                .await;
            *newest_prepared_frames = prepared_frame.into();

            return Ok(());
        }
    }
}

/// A linked list of frames that is zero-copy and can be used to stream frames to a client.
#[derive(Clone)]
pub struct PreparedFrame {
    pub frame: Bytes,
    /// duration of the frame in milliseconds calculated from the frame size and bitrate
    pub duration: u128,
    /// id of the frame that is used to track the order of the frames
    /// should be monotonically increasing
    pub id: usize,
    next: Arc<Mutex<Option<PreparedFrame>>>,
}

impl PreparedFrame {
    pub async fn has_next(&self) -> bool {
        self.next.lock().await.is_some()
    }

    pub async fn get_next(&self) -> Option<PreparedFrame> {
        match &*self.next.lock().await {
            Some(next) => Some(next.clone()),
            None => None,
        }
    }

    pub async fn set_next(&self, next: PreparedFrame) {
        *self.next.lock().await = Some(next);
    }
}

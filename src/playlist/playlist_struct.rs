use std::sync::Arc;

use bytes::Bytes;
use log::debug;
use tokio::sync::Mutex;

use crate::{CONTEXT, shoutcast::ListenerID};

use super::{PlaylistChild, listener_frame_data::ListenerFrameData};

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
    child: Arc<Mutex<Box<dyn PlaylistChild>>>,
    newest_prepared_frames: Mutex<PreparedFrame>,
    listener_frame_data_db: ListenerFrameData,
}

impl Playlist {
    pub async fn new(name: String, child: Arc<Mutex<Box<dyn PlaylistChild>>>) -> Self {
        let frame = PreparedFrame {
            frame: Bytes::new(),
            duration: 0.0,
            id: CONTEXT.get_id().await,
            next: Arc::new(Mutex::new(None)),
            title: Arc::new("".to_string()),
            artist: Arc::new("".to_string()),
        };
        Self {
            name: name.into(),
            child,
            newest_prepared_frames: Mutex::new(frame.clone()),
            listener_frame_data_db: ListenerFrameData::new(Arc::new(Mutex::new(frame.clone()))),
        }
    }

    /// log the listener current frame
    pub async fn log_current_frame(&self, listener_id: &ListenerID, frame: PreparedFrame) {
        self.listener_frame_data_db
            .log_current_frame(listener_id, frame)
            .await;
    }

    /// return the current content type of the playlist
    pub async fn content_type(&self) -> anyhow::Result<Option<Arc<String>>> {
        self.child.lock().await.content_type().await
    }

    /// check if the Playlist is finished
    pub async fn is_finished(&self) -> anyhow::Result<bool> {
        self.child.lock().await.is_finished().await
    }

    pub async fn get_oldest_prepared_frames(&self) -> PreparedFrame {
        self.listener_frame_data_db.get_current_frame().await
    }

    /// prepare_frames prepares the frames for the playlist
    /// do nothing if the playlist is finished
    /// or the playlist already has the next frame
    /// prepare one frame each time
    pub async fn prepare_frame(&self) -> anyhow::Result<()> {
        if self.is_finished().await? {
            debug!("playlist is finished: {:?}", self.name);
            return Ok(());
        }

        let newest_prepared_frames = self.newest_prepared_frames.lock().await;
        if newest_prepared_frames.has_next().await {
            return Ok(());
        }
        drop(newest_prepared_frames);

        let mut child = self.child.lock().await;
        let (frame, title, artist) = match child.next_frame_with_meta().await? {
            Some(frame) => frame,
            None => {
                // the child is finished
                return Ok(());
            }
        };
        let byte_per_millisecond = match child.byte_per_millisecond().await? {
            Some(byte_per_millisecond) => byte_per_millisecond,
            None => {
                // the child is finished
                return Ok(());
            }
        };
        let duration = frame.len() as f64 / byte_per_millisecond;
        drop(child);

        let prepared_frame = PreparedFrame {
            id: CONTEXT.get_id().await,
            frame,
            duration,
            next: Arc::new(Mutex::new(None)),
            title,
            artist,
        };

        let mut newest_prepared_frames = self.newest_prepared_frames.lock().await;
        newest_prepared_frames
            .set_next(prepared_frame.clone())
            .await;
        *newest_prepared_frames = prepared_frame;

        Ok(())
    }

    pub async fn get_frame_with_id(&self, id: &ListenerID) -> Option<PreparedFrame> {
        self.listener_frame_data_db.get_frame_with_id(id).await
    }

    /// Get the listener_id from the session_id, if the session_id is not found, return None
    pub async fn get_listener_id_from_session_id(&self, session_id: &str) -> Option<usize> {
        self.listener_frame_data_db
            .get_listener_id_from_session_id(session_id)
            .await
    }

    /// log session_id to listener_id
    pub async fn log_session_id(&self, session_id: String, listener_id: usize) {
        self.listener_frame_data_db
            .log_session_id(session_id, listener_id)
            .await;
    }
}

/// A linked list of frames that is zero-copy and can be used to stream frames to a client.
#[derive(Clone)]
pub struct PreparedFrame {
    pub frame: Bytes,
    /// duration of the frame in milliseconds calculated from the frame size and bitrate
    pub duration: f64,
    /// id of the frame that is used to track the order of the frames
    /// should be monotonically increasing
    pub id: usize,
    pub title: Arc<String>,
    pub artist: Arc<String>,
    next: Arc<Mutex<Option<PreparedFrame>>>,
}

impl PreparedFrame {
    pub async fn has_next(&self) -> bool {
        self.next.lock().await.is_some()
    }

    pub async fn get_next(&self) -> Option<PreparedFrame> {
        (*self.next.lock().await).clone()
    }

    pub async fn set_next(&self, next: PreparedFrame) {
        *self.next.lock().await = Some(next);
    }
}

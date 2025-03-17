use std::sync::Arc;

use bytes::Bytes;
use log::debug;
use tokio::sync::{Mutex, mpsc::Receiver};
use tokio_stream::StreamExt;

use crate::{CONTEXT, shoutcast::ListenerID};

use super::{FrameWithMeta, PlaylistChild, listener_frame_data::ListenerFrameData};

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
    finished: Mutex<bool>,
    child_recv: Mutex<Receiver<anyhow::Result<FrameWithMeta>>>,
    newest_prepared_frames: Mutex<PreparedFrame>,
    listener_frame_data_db: ListenerFrameData,
    content_type: Mutex<Arc<String>>,
}

impl Playlist {
    pub async fn new(name: String, child: Box<dyn PlaylistChild>) -> Self {
        let frame = PreparedFrame {
            frame_with_meta: FrameWithMeta {
                frame: Bytes::new(),
                duration: 0.0,
                title: Arc::new("".to_string()),
                artist: Arc::new("".to_string()),
                content_type: Arc::new("".to_string()),
            },
            id: CONTEXT.get_id().await,
            next: Arc::new(Mutex::new(None)),
        };

        let (sender, child_recv) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            let mut child = child;
            let stream = child.stream_frame_with_meta().await;
            // if err, send the err to the receiver
            if let Err(e) = stream {
                let e = sender.send(Err(e)).await;
                if let Err(e) = e {
                    log::error!("failed to send error to child_recv: {}", e);
                }
                return;
            }
            let mut stream = stream.unwrap();

            while let Some(frame_with_meta) = stream.next().await {
                match sender.send(frame_with_meta).await {
                    Ok(_) => {}
                    Err(e) => {
                        // the receiver is dropped, we should stop the stream
                        log::error!("failed to send frame_with_meta: {}", e);
                        return;
                    }
                }
            }

            // the stream is finished, the sender is dropped
        });

        Self {
            name: name.into(),
            child_recv: (Mutex::new(child_recv)),
            finished: Mutex::new(false),
            newest_prepared_frames: Mutex::new(frame.clone()),
            listener_frame_data_db: ListenerFrameData::new(Arc::new(Mutex::new(frame.clone()))),
            content_type: Mutex::new(frame.frame_with_meta.content_type),
        }
    }

    /// get the content type of the playlist
    pub async fn get_content_type(&self) -> Arc<String> {
        self.content_type.lock().await.clone()
    }

    /// log the listener current frame
    pub async fn log_current_frame(&self, listener_id: &ListenerID, frame: PreparedFrame) {
        self.listener_frame_data_db
            .log_current_frame(listener_id, frame)
            .await;
    }

    /// check if the Playlist is finished
    pub async fn is_finished(&self) -> anyhow::Result<bool> {
        Ok(*self.finished.lock().await)
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

        let frame_with_meta = match self.child_recv.lock().await.recv().await {
            Some(frame) => frame?,
            None => {
                // the child is finished
                *self.finished.lock().await = true;
                return Ok(());
            }
        };

        // set the content type of the playlist
        *self.content_type.lock().await = frame_with_meta.content_type.clone();

        let prepared_frame = PreparedFrame {
            id: CONTEXT.get_id().await,
            frame_with_meta,
            next: Arc::new(Mutex::new(None)),
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
    pub frame_with_meta: FrameWithMeta,
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
        (*self.next.lock().await).clone()
    }

    pub async fn set_next(&self, next: PreparedFrame) {
        *self.next.lock().await = Some(next);
    }
}

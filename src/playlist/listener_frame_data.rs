use std::{sync::Arc, time::Duration};

use moka::notification::ListenerFuture;
use tokio::sync::Mutex;

use crate::{playlist::MAX_WRITE_AHEAD_DURATION, shoutcast::ListenerID};

use super::PreparedFrame;

pub struct ListenerFrameData {
    session_id_2_listener_id: moka::future::Cache<String, usize>,
    listener_id_2_frame: moka::future::Cache<usize, PreparedFrame>,
    listener_id_frame_group: moka::future::Cache<(usize, usize), ()>,
    current_frame: Arc<Mutex<PreparedFrame>>,
}

impl ListenerFrameData {
    /// new
    pub fn new(current_frame: Arc<Mutex<PreparedFrame>>) -> Self {
        let session_id_2_listener_id: moka::future::Cache<String, usize> =
            moka::future::Cache::builder()
                .time_to_idle(Duration::from_millis(MAX_WRITE_AHEAD_DURATION as u64))
                .build();
        let listener_id_2_frame: moka::future::Cache<usize, PreparedFrame> =
            moka::future::Cache::builder()
                .time_to_idle(Duration::from_millis(MAX_WRITE_AHEAD_DURATION as u64))
                .build();
        let current_frame1 = current_frame.clone();
        let eviction_listener = move |k: Arc<(usize, usize)>, _, _| -> ListenerFuture {
            let (_, frame_id) = k.as_ref();
            let frame_id = *frame_id;
            let current_frame = current_frame1.clone();
            Box::pin(async move {
                let current_frame_lock = current_frame.lock().await;
                let mut current_frame_id = current_frame_lock.id;
                drop(current_frame_lock);

                while current_frame_id < frame_id {
                    let mut current_frame_lock = current_frame.lock().await;
                    if let Some(next) = current_frame_lock.get_next().await {
                        *current_frame_lock = next;
                    } else {
                        break;
                    }
                    current_frame_id = current_frame_lock.id;
                }
            })
        };
        let listener_id_frame_group: moka::future::Cache<(usize, usize), ()> =
            moka::future::Cache::builder()
                .time_to_live(Duration::from_millis(MAX_WRITE_AHEAD_DURATION as u64))
                .async_eviction_listener(eviction_listener)
                .build();

        Self {
            session_id_2_listener_id,
            listener_id_2_frame,
            listener_id_frame_group,
            current_frame,
        }
    }

    pub async fn get_current_frame(&self) -> PreparedFrame {
        let current_frame = self.current_frame.lock().await;
        current_frame.clone()
    }

    /// log the listener current frame
    pub async fn log_current_frame(&self, listener_id: &ListenerID, frame: PreparedFrame) {
        // refresh session_id_2_listener_id
        if let Some(session_id) = &listener_id.session_id {
            // TODO Optimize this
            self.session_id_2_listener_id
                .insert(session_id.clone(), listener_id.listener_id)
                .await;
        }
        self.listener_id_frame_group
            .insert((listener_id.listener_id, frame.id), ())
            .await;
        self.listener_id_2_frame
            .insert(listener_id.listener_id, frame)
            .await;
    }

    pub async fn get_frame_with_id(&self, id: &ListenerID) -> Option<PreparedFrame> {
        if let Some(session_id) = &id.session_id {
            let l_id = self.session_id_2_listener_id.get(session_id).await;
            if let Some(l_id) = l_id {
                return self.listener_id_2_frame.get(&l_id).await;
            }
        }

        self.listener_id_2_frame.get(&id.listener_id).await
    }
}

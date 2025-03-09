use std::{pin::Pin, sync::Arc, task::Poll};

use futures::Stream;

use crate::{
    playlist::{MAX_WRITE_AHEAD_DURATION, PreparedFrame},
    shoutcast::ListenerID,
};

use super::Playlist;

type PlaylistFrameStreamPendingFuture =
    Pin<Box<dyn futures::Future<Output = anyhow::Result<Option<PreparedFrame>>> + Send>>;

pub struct PlaylistFrameStream {
    playlist: Arc<Playlist>,
    current_stream_frame: PreparedFrame,
    /// duration that has been written to the client in milliseconds
    write_ahead_duration: f64,
    /// created time of the stream in milliseconds since epoch
    created_time: u128,
    pending_future: Option<PlaylistFrameStreamPendingFuture>,
    waiting_pending_future: Option<Pin<Box<dyn futures::Future<Output = ()> + Send>>>,
}

impl PlaylistFrameStream {
    pub async fn new(playlist: Arc<Playlist>, listener_id: &ListenerID) -> Self {
        let current_stream_frame = match playlist.get_frame_with_id(listener_id).await {
            Some(frame) => frame,
            None => playlist.get_oldest_prepared_frames().await,
        };
        Self {
            playlist,
            current_stream_frame,
            write_ahead_duration: 0.0,
            created_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            pending_future: None,
            waiting_pending_future: None,
        }
    }
}

impl Stream for PlaylistFrameStream {
    type Item = anyhow::Result<PreparedFrame>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        async fn next_frame(
            playlist: Arc<Playlist>,
            current_stream_frame: PreparedFrame,
        ) -> anyhow::Result<Option<PreparedFrame>> {
            let next_stream = current_stream_frame.get_next().await;
            if let Some(next_stream) = next_stream {
                return Ok(Some(next_stream));
            }
            drop(next_stream);

            // prepare frame should change the current prepared frame if the playlist is not finished
            // if the next current_stream_frame is still none after prepare_frame, then the playlist is finished
            playlist.prepare_frame().await?;
            Ok(current_stream_frame.get_next().await)
        }

        if let Some(ref mut future) = self.waiting_pending_future {
            // Poll the future and check if it's ready
            match future.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    self.waiting_pending_future = None; // Reset future
                }
                Poll::Pending => return Poll::Pending, // Still waiting
            }
        }

        if let Some(ref mut future) = self.pending_future {
            // Poll the future and check if it's ready
            match future.as_mut().poll(cx) {
                Poll::Ready(data) => {
                    self.pending_future = None; // Reset future
                    match data {
                        Ok(Some(frame)) => {
                            self.current_stream_frame = frame.clone();
                            self.write_ahead_duration += frame.duration;
                            self.waiting_pending_future = Some(Box::pin(tokio::time::sleep(
                                std::time::Duration::from_millis(frame.duration.floor() as u64),
                            )));
                            return Poll::Ready(Some(Ok(frame)));
                        }
                        Ok(None) => {
                            // End of stream
                            return Poll::Ready(None);
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(e)));
                        }
                    }
                }
                Poll::Pending => {
                    return Poll::Pending; // Still waiting
                }
            }
        }

        // If write_ahead_duration + created_time - current_time > MAX_WRITE_AHEAD_DURATION,
        // wait for 5 seconds before checking again
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if self.write_ahead_duration.ceil() as u128 + self.created_time
            > MAX_WRITE_AHEAD_DURATION + current_time
        {
            // pin the wait future
            self.waiting_pending_future = Some(Box::pin(tokio::time::sleep(
                std::time::Duration::from_secs(5),
            )));
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        // If no future is running and there are items left, start one
        self.pending_future = Some(Box::pin(next_frame(
            self.playlist.clone(),
            self.current_stream_frame.clone(),
        ))); // Store new future
        cx.waker().wake_by_ref(); // Wake up poller to retry
        Poll::Pending
    }
}

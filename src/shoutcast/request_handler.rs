use async_trait::async_trait;
use bytes::Bytes;
use futures::SinkExt;
use http::Request;
use log::debug;
use std::{sync::Arc, vec};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::playlist::{Playlist, PlaylistFrameStream};

/// MetaDataInterval is the data interval in which meta data is send
const META_DATA_INTERVAL: usize = 65536;
const META_DATA_INTERVAL_STR: &str = "65536";

/// MaxMetaDataSize is the maximum size for meta data (everything over is truncated)
///
/// Must be a multiple of 16 which fits into one byte. Maximum: 16 * 255 = 4080
const MAX_META_DATA_SIZE: usize = 4080;

/// Get whether the request support meta data
fn meta_data_support(request: &Request<()>) -> bool {
    let header_map = request.headers();
    let meta_data_support = header_map.get("Icy-MetaData");
    if let Some(v) = meta_data_support {
        return v == "1";
    }

    let meta_data_support = header_map.get("icy-metadata");
    if let Some(v) = meta_data_support {
        return v == "1";
    }

    false
}

struct MySink(Arc<Mutex<tokio_util::codec::Framed<tokio::net::TcpStream, super::Http>>>);

#[async_trait]
trait Send2Sink<D> {
    async fn send(&self, data: D) -> anyhow::Result<()>;
}

macro_rules! impl_send2_sink {
    ( $($t:ty),* ) => {
        $(
            /// Writing a String to the TcpStream, in the ShoutCast protocol,
            /// we do not need to write a whole http response, just the body.
            impl tokio_util::codec::Encoder<$t> for super::Http {
                type Error = std::io::Error;

                fn encode(&mut self, i: $t, dst: &mut bytes::BytesMut) -> std::io::Result<()> {
                    dst.extend_from_slice(i.as_bytes());
                    Ok(())
                }
            }

            #[async_trait]
            impl Send2Sink<$t> for MySink
                {
                    async fn send(&self, data: $t) -> anyhow::Result<()> {
                        self.0.lock().await.send(data).await?;
                        Ok(())
                    }
                }
        ) *
    }
}
impl_send2_sink! { String, Arc<String>, &'static str }

/// Writing a String to the TcpStream, in the ShoutCast protocol,
/// we do not need to write a whole http response, just the body.
impl tokio_util::codec::Encoder<Bytes> for super::Http {
    type Error = std::io::Error;

    fn encode(&mut self, i: Bytes, dst: &mut bytes::BytesMut) -> std::io::Result<()> {
        dst.extend_from_slice(&i);
        Ok(())
    }
}
#[async_trait]
impl Send2Sink<Bytes> for MySink {
    async fn send(&self, data: Bytes) -> anyhow::Result<()> {
        self.0.lock().await.send(data).await?;
        Ok(())
    }
}

pub struct RequestHandler {
    sink: MySink,
    playlist: Arc<Playlist>,
    meta_data_support: bool,
}
impl RequestHandler {
    // new creates a new RequestHandler
    pub fn new(
        sink: Arc<Mutex<tokio_util::codec::Framed<tokio::net::TcpStream, super::Http>>>,
        playlist: Arc<Playlist>,
        request: Request<()>,
    ) -> Self {
        let meta_data_support = meta_data_support(&request);
        Self {
            sink: MySink(sink),
            playlist,
            meta_data_support,
        }
    }

    /// HandleRequest handles requests from streaming clients. It tries to extract
    /// the path and if meta data is supported. Once a request has been successfully
    /// decoded ServeRequest is called. The connection is closed once HandleRequest
    /// finishes.
    pub async fn handle_request(&mut self) -> anyhow::Result<()> {
        debug!("handle request for playlist: {:?}", self.playlist.name);

        self.write_stream_start_response().await?;

        let mut frame_stream = PlaylistFrameStream::new(self.playlist.clone()).await;
        let mut bytes_before_next_meta_data = META_DATA_INTERVAL;

        async fn write_next_frame(
            handler: &mut RequestHandler,
            frame_stream: &mut PlaylistFrameStream,
            bytes_before_next_meta_data: &mut usize,
        ) -> anyhow::Result<bool> {
            let frame = match frame_stream.next().await {
                Some(frame) => frame,
                None => return Ok(true),
            };
            let frame = frame?;
            handler
                .playlist
                .log_current_frame(frame_stream.listener_id, frame.id)
                .await?;

            *bytes_before_next_meta_data = handler
                .write_frame(frame.frame, *bytes_before_next_meta_data)
                .await?;

            Ok(false)
        }

        loop {
            if let Err(e) =
                write_next_frame(self, &mut frame_stream, &mut bytes_before_next_meta_data).await
            {
                self.playlist
                    .delete_listener_data(frame_stream.listener_id)
                    .await?;
                return Err(e);
            }
            if let Ok(true) =
                write_next_frame(self, &mut frame_stream, &mut bytes_before_next_meta_data).await
            {
                break;
            }
        }

        self.playlist
            .delete_listener_data(frame_stream.listener_id)
            .await?;

        Ok(())
    }
    /// writeStreamStartResponse writes the start response to the client.
    async fn write_stream_start_response(&mut self) -> anyhow::Result<()> {
        debug!("write stream start response");
        self.sink.send("ICY 200 OK\r\n").await?;

        self.sink.send("Content-Type: ").await?;
        self.sink.send(self.playlist.content_type().await).await?;
        self.sink.send("\r\n").await?;

        self.sink.send("icy-name: ").await?;
        self.sink.send(self.playlist.name.clone()).await?;
        self.sink.send("\r\n").await?;

        if self.meta_data_support {
            debug!("meta data support enabled");
            self.sink.send("icy-metadata: 1\r\n").await?;

            self.sink.send("icy-metaint: ").await?;
            self.sink.send(META_DATA_INTERVAL_STR).await?;
            self.sink.send("\r\n").await?;
        }

        self.sink.send("\r\n").await?;
        Ok(())
    }

    /// writeFrame writes a frame to a client.
    async fn write_frame(
        &mut self,
        frame: Bytes,
        bytes_before_next_meta_data: usize,
    ) -> anyhow::Result<usize> {
        let mut frame = frame;
        let mut bytes_before_next_meta_data = bytes_before_next_meta_data;
        while bytes_before_next_meta_data < frame.len() {
            let first = frame.split_to(bytes_before_next_meta_data);
            self.sink.send(first).await?;
            self.write_stream_meta_data().await?;
            bytes_before_next_meta_data = META_DATA_INTERVAL;
        }

        let len = frame.len();
        if len > 0 {
            self.sink.send(frame).await?;
            bytes_before_next_meta_data -= len;
        }

        Ok(bytes_before_next_meta_data)
    }

    /// writeStreamMetaData writes meta data information into the stream.
    async fn write_stream_meta_data(&mut self) -> anyhow::Result<()> {
        // todo optimize this
        let stream_title = format!(
            "{} - {}",
            self.playlist.current_title().await,
            self.playlist.current_artist().await
        );
        let stream_title = if stream_title.len() > MAX_META_DATA_SIZE - 15 {
            // Truncate stream title if necessary
            format!(
                "StreamTitle='{}';",
                stream_title.split_at(MAX_META_DATA_SIZE - 15).0
            )
        } else {
            format!("StreamTitle='{}';", stream_title)
        };

        // padding with 0 to make the length a multiple of 16
        let padding = 16 - (stream_title.len() % 16);
        let padding = if padding == 16 { 0 } else { padding };

        self.sink
            .send(Bytes::from(vec![
                ((stream_title.len() + padding) / 16) as u8,
            ]))
            .await?;
        self.sink.send(stream_title).await?;
        // TODO optimize this
        self.sink.send(Bytes::from(vec![0; padding])).await?;

        Ok(())
    }
}

/*

/// writeStreamNotFoundResponse writes the not found response to the client.
func (drh *DefaultRequestHandler) writeStreamNotFoundResponse(c net.Conn) error {
    _, err := c.Write([]byte("HTTP/1.1 404 Not found\r\n\r\n"))

    return err
}

/// writeUnauthorized writes the Unauthorized response to the client.
func (drh *DefaultRequestHandler) writeUnauthorized(c net.Conn) error {
    _, err := c.Write([]byte("HTTP/1.1 401 Authorization Required\r\nWWW-Authenticate: Basic realm=\"DudelDu Streaming Server\"\r\n\r\n"))

    return err
}
*/

use crate::playlist::Playlist;
use bytes::BytesMut;
use http::{Request, header::HeaderValue};
use log::{debug, error, info};
use std::{collections::HashMap, io, sync::Arc, vec};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

mod request_handler;

use request_handler::RequestHandler;

pub async fn listen(
    host: &str,
    port: u16,
    playlists: Arc<HashMap<String, Arc<Playlist>>>,
) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    let server = TcpListener::bind(&addr).await?;
    info!("Listening on: {addr}");

    loop {
        let (stream, _) = server.accept().await?;
        let playlists = playlists.clone();
        tokio::spawn(async move {
            if let Err(e) = process(stream, playlists).await {
                error!("failed to process connection; error = {e}");
            }
        });
    }
}

async fn process(
    stream: TcpStream,
    playlists: Arc<HashMap<String, Arc<Playlist>>>,
) -> anyhow::Result<()> {
    let transport = Arc::new(Mutex::new(Framed::new(stream, Http)));

    let mut transport_lock = transport.lock().await;
    let request = match transport_lock.next().await {
        Some(request) => request?,
        None => return Ok(()),
    };
    drop(transport_lock);

    debug!("handle request: {:?}", request);
    let path = request.uri().path();
    let path = path.trim_matches('/');
    let playlist = playlists.get(path);
    if let Some(playlist) = playlist {
        let mut handler = RequestHandler::new(transport.clone(), playlist.clone(), request);
        handler.handle_request().await?;
    } else {
        debug!("playlist not found for path: {path}");
    }

    debug!("connection closed");

    Ok(())
}

struct Http;

/// Implementation of decoding an HTTP request from the bytes we've read so far.
/// This leverages the `httparse` crate to do the actual parsing and then we use
/// that information to construct an instance of a `http::Request` object,
/// trying to avoid allocations where possible.
impl Decoder for Http {
    type Item = Request<()>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Request<()>>> {
        let (method, path, amt, headers) = {
            let mut parsed_headers = vec![httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut parsed_headers);
            let amt = loop {
                let e = r.parse(src);
                match e {
                    Ok(httparse::Status::Complete(amt)) => break amt,
                    Ok(httparse::Status::Partial) => return Ok(None),
                    Err(httparse::Error::TooManyHeaders) => {
                        parsed_headers = vec![httparse::EMPTY_HEADER; parsed_headers.len() * 2];
                        r = httparse::Request::new(&mut parsed_headers);
                        continue;
                    }
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to parse http request: {e:?}"),
                        ));
                    }
                }
            };

            let mut headers = vec![None; r.headers.len()];

            let to_slice_fn = |a: &[u8]| {
                let start = a.as_ptr() as usize - src.as_ptr() as usize;
                assert!(start < src.len());
                (start, start + a.len())
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = to_slice_fn(header.name.as_bytes());
                let v = to_slice_fn(header.value);
                headers[i] = Some((k, v));
            }

            let method = http::Method::try_from(r.method.map_or(
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No HTTP Method specified",
                )),
                |m| Ok(m),
            )?)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let path = match r.path {
                Some(path) => to_slice_fn(path.as_bytes()),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No HTTP Path specified",
                    ));
                }
            };

            // check version
            match r.version {
                Some(1) => {}
                Some(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "only HTTP/1.1 accepted",
                    ));
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No HTTP version specified",
                    ));
                }
            };

            (method, path, amt, headers)
        };

        let data = src.split_to(amt).freeze();
        let mut ret = Request::builder();
        ret = ret.method(method);
        let s = data.slice(path.0..path.1);
        let s = String::from_utf8(Vec::from(s.as_ref()))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "path decode error"))?;
        ret = ret.uri(s);
        ret = ret.version(http::Version::HTTP_11);
        for header in headers.iter() {
            let (k, v) = match *header {
                Some((ref k, ref v)) => (k, v),
                None => break,
            };
            let value = HeaderValue::from_bytes(data.slice(v.0..v.1).as_ref())
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "header decode error"))?;
            ret = ret.header(&data[k.0..k.1], value);
        }

        let req = ret
            .body(())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Some(req))
    }
}

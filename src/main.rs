// TODO cache x-playback-session-id

use object_store::aws::AmazonS3Builder;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

mod context;

mod file_provider;
mod playlist;
mod shoutcast;

pub use context::CONTEXT;
pub use file_provider::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // Apply globally
        .apply()
        .unwrap();

    let s3_builder = AmazonS3Builder::new()
        .with_endpoint("https://4e2f1d3d2666e9a944e48be955fce140.r2.cloudflarestorage.com")
        .with_secret_access_key("287d17df6262bde9b8eb9c7c3f930b546bbc8d2d1ede9db62188cc622f65180e")
        .with_access_key_id("f8f461c5d4a9f9cf3bb488732c50b72a")
        .with_bucket_name("personal-radio-station-music");

    /*
        let local_track = playlist::LocalFolder::new(
            "/Users/zifanhua/Documents/Music/ist1/".to_string(),
            Some(true),
            Some(true),
            Arc::new(LocalFileProvider::new()),
        )
        .await?;
    */
    let local_track = playlist::LocalFolder::new(
        "/".to_string(),
        Some(true),
        Some(true),
        Arc::new(AwsS3FileProvider::new(s3_builder).await?),
    )
    .await?;

    let playlist =
        playlist::Playlist::new("playlist".to_string(), Arc::new(Mutex::new(local_track))).await;

    let mut playlists = HashMap::new();
    playlists.insert("".to_string(), Arc::new(playlist));

    shoutcast::listen("127.0.0.1", 8080, Arc::new(playlists)).await
}

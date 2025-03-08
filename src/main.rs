// # [global_allocator]
// static A: std::alloc::System = std::alloc::System;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

mod context;
#[cfg(feature = "db")]
mod db;

mod file_provider;
mod playlist;
mod shoutcast;

pub use context::CONTEXT;
pub use file_provider::*;

#[cfg(feature = "db")]
pub use db::DB;

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

    #[cfg(feature = "db")]
    let db = db::DB::new().await?;

    let local_track = playlist::LocalFolder::new(
        "/Users/zifanhua/Documents/Music/ist1/".to_string(),
        Some(true),
        Some(true),
        Arc::new(LocalFileProvider::new()),
    )
    .await?;

    let playlist =
        playlist::Playlist::new("playlist".to_string(), Arc::new(Mutex::new(local_track))).await;

    let mut playlists = HashMap::new();
    playlists.insert("".to_string(), Arc::new(playlist));

    shoutcast::listen("127.0.0.1", 8080, Arc::new(playlists)).await
}

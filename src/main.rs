// TODO cache x-playback-session-id

use std::{collections::HashMap, sync::Arc};

use clap::Parser;

mod context;

pub mod config;
mod file_provider;
mod playlist;
mod shoutcast;

use config::ShoutCastOutput;
pub use context::CONTEXT;
pub use file_provider::*;
use playlist::{Playlist, build_playlist_from_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    {
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
    }

    let config = config::ClapArgs::parse();
    let config::GlobalConfig {
        file_provider,
        playlists,
        outputs,
        ..
    } = config::GlobalConfig::from_clap_args(config).await?;

    let file_provider = Arc::new(file_provider::build_file_provider(file_provider).await?);
    let playlists = build_playlist_from_config(playlists, file_provider).await?;

    let mut outputs_map = HashMap::new();
    for shoutcast_config in outputs {
        let ShoutCastOutput {
            host,
            port,
            path,
            playlist,
        } = shoutcast_config;
        let path = path.trim_matches('/').to_string();

        let playlist = match playlists.get(&playlist) {
            Some(playlist) => playlist.clone(),
            None => {
                log::error!("playlist not found: {}", playlist);
                return Err(anyhow::anyhow!("playlist not found: {}", playlist));
            }
        };
        let fut: Option<&mut HashMap<String, Arc<Playlist>>> =
            outputs_map.get_mut(&(host.clone(), port));
        if let Some(fut) = fut {
            let res = fut.insert(path.clone(), playlist);
            if res.is_some() {
                let msg = format!(
                    "You have configured the same URL path({}) twice on the same server({}:{}). This creates a conflict because the system doesn't know which configuration to use when a request comes in for that path.",
                    path, host, port
                );
                log::error!("{}", msg);
                return Err(anyhow::anyhow!("{}", msg));
            }
        } else {
            let mut fut = HashMap::new();
            fut.insert(path, playlist);
            outputs_map.insert((host, port), fut);
        }
    }

    let mut output_fut = Vec::with_capacity(outputs_map.len());

    for ((host, port), mut playlists) in outputs_map {
        playlists.shrink_to_fit();
        output_fut.push(shoutcast::listen(host, port, Arc::new(playlists)));
    }

    futures::future::join_all(output_fut).await;

    Ok(())
}

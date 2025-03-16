use std::{collections::HashMap, pin::Pin, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    FileProvider, LocalFileProvider,
    config::{PlaylistChildConfig, PlaylistConfig},
};

use super::{Playlist, PlaylistChild};

pub async fn build_playlist_from_config(
    playlist: HashMap<String, PlaylistConfig>,
    file_provider: Arc<HashMap<String, Arc<dyn FileProvider>>>,
) -> anyhow::Result<HashMap<String, Arc<Playlist>>> {
    let mut res = HashMap::new();
    for (key, playlist) in playlist {
        let PlaylistConfig { child, name } = playlist;
        let child = build_playlist_child_from_config(child, file_provider.clone()).await?;
        let playlist = Playlist::new(name, Arc::new(Mutex::new(child))).await;
        res.insert(key, Arc::new(playlist));
    }

    Ok(res)
}

async fn build_playlist_child_from_config(
    playlist: PlaylistChildConfig,
    file_provider: Arc<HashMap<String, Arc<dyn FileProvider>>>,
) -> anyhow::Result<Box<dyn PlaylistChild>> {
    let child: Box<dyn PlaylistChild> = match playlist {
         PlaylistChildConfig::LocalFolder {
             folder,
             repeat,
             shuffle,
             // TODO add fail_over functionality
             // TODO add recursive find file functionality
             ..
         } => {
             let file_provider = Arc::new(LocalFileProvider::new());
             Box::new(
                 crate::playlist::LocalFolder::new(folder, repeat, shuffle, file_provider).await?,
             )
         }
         PlaylistChildConfig::Silent => todo!("Implement this match arm"),
         PlaylistChildConfig::LocalFiles { files, repeat, shuffle, ..
             // TODO add fail_over functionality
         } => {
             let file_provider: Arc<dyn FileProvider> = Arc::new(LocalFileProvider::new());
             Box::new(
                 crate::playlist::LocalFileTrackList::new(files, repeat, shuffle, file_provider).await?,
             )
         },
         PlaylistChildConfig::RemoteFolder { folder, remote_client, repeat, shuffle, ..
             // TODO add fail_over functionality
         } => {
             let file_provider = match file_provider.get(&remote_client){
                 Some(provider) => provider.clone(),
                 None => return Err(anyhow::anyhow!("No file provider found for {}", remote_client)),
             };
             Box::new(
                 crate::playlist::LocalFolder::new(folder, repeat, shuffle, file_provider).await?,
             )
         },
         PlaylistChildConfig::RemoteFiles { files, remote_client, repeat, shuffle, ..
        // TODO add fail_over functionality
         } => {
             let file_provider = match file_provider.get(&remote_client){
                 Some(provider) => provider.clone(),
                 None => return Err(anyhow::anyhow!("No file provider found for {}", remote_client)),
             };
             Box::new(
                 crate::playlist::LocalFileTrackList::new(files, repeat, shuffle, file_provider).await?,
             )
         },

         PlaylistChildConfig::Playlists { children, repeat, shuffle, ..
        // TODO add fail_over functionality
         } => {
             let children = *children;
             type PlaylistChildOutPin = Pin<
                 Box<
                     dyn futures::Future<Output = anyhow::Result<Box<dyn PlaylistChild>>>
                         + std::marker::Send,
                 >,
             >;

             fn init_fn(c: PlaylistChildConfig, f: Arc<HashMap<String, Arc<dyn FileProvider >>>) -> PlaylistChildOutPin {
                 Box::pin(async move {
                     build_playlist_child_from_config(c, f).await
                 })
             }

             // async fn init(c: PlaylistChildConfig) -> anyhow::Result<Box<dyn PlaylistChild>> {}
             Box::new(
                 crate::playlist::PlaylistChildList::new(children, repeat, shuffle, Some(file_provider),Some(init_fn)).await?,
             )
         },

    };

    Ok(child)
}

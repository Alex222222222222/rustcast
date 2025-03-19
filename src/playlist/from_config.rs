use std::{collections::HashMap, pin::Pin, sync::Arc};

use async_stream::stream;
use futures::Stream;

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
        let playlist = Playlist::new(name, child).await;
        res.insert(key, Arc::new(playlist));
    }

    Ok(res)
}

async fn build_playlist_child_from_config(
    playlist: PlaylistChildConfig,
    file_provider: Arc<HashMap<String, Arc<dyn FileProvider>>>,
) -> anyhow::Result<Box<dyn PlaylistChild>> {
    let child: Box<dyn PlaylistChild> = match playlist {
        PlaylistChildConfig::Silent => Box::new(crate::playlist::Silent::new()?),
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
                 crate::playlist::LocalFolder::new(folder, repeat, shuffle, file_provider)?,
             )
         }

         PlaylistChildConfig::LocalFiles { files, repeat, shuffle, ..
             // TODO add fail_over functionality
         } => {
             let file_provider: Arc<dyn FileProvider> = Arc::new(LocalFileProvider::new());
             Box::new(
                 crate::playlist::LocalFileTrackList::new(files, repeat, shuffle, file_provider)?,
             )
         },
         PlaylistChildConfig::RemoteFolder { folder, remote_client, repeat, shuffle, ..
             // TODO add fail_over functionality
         } => {
             let file_provider = match file_provider.get(remote_client.as_str()){
                 Some(provider) => provider.clone(),
                 None => return Err(anyhow::anyhow!("No file provider found for {}", remote_client)),
             };
             Box::new(
                 crate::playlist::LocalFolder::new(folder, repeat, shuffle, file_provider)?,
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
                 crate::playlist::LocalFileTrackList::new(files, repeat, shuffle, file_provider)?,
             )
         },

         PlaylistChildConfig::Playlists { children, repeat, shuffle, ..
        // TODO add fail_over functionality
         } => {
             type ReturnStream = Pin<Box<dyn Stream<Item = anyhow::Result<Box<dyn PlaylistChild>>> + Send>>;

             fn init_fn(
                 p: Arc<Vec<Arc<PlaylistChildConfig>>>,
                 fp: Arc<HashMap<String, Arc<dyn FileProvider >>>,
             ) -> Pin<
                 Box<
                     dyn Future<Output = anyhow::Result<ReturnStream>>
                         + Send,
                 >,
             > {
                 let s = stream! {
                        for i in p.iter() {
                            let c = build_playlist_child_from_config((**i).clone(), fp.clone()).await;
                            yield c;
                        }
                 };

                     let s: ReturnStream = Box::pin(s);

                         Box::pin(async { Ok(s) })
             }

             Box::new(
                 crate::playlist::PlaylistChildList::<Vec<Arc<PlaylistChildConfig>>, Arc<HashMap<String, Arc<dyn FileProvider >>>>::
                     new(children,
                     repeat,
                     shuffle, init_fn, file_provider,)?,
             )
         },

    };

    Ok(child)
}

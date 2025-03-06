use proc_macro2::Ident;
use quote::quote;
use syn::Generics;

pub fn impl_playlist_child(name: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics PlaylistChild for #name #ty_generics #where_clause{
            /// current_title returns the title of current playing song
            async fn current_title(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.current_title().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }

            /// Artist returns the artist which is currently playing.
            async fn current_artist(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.current_artist().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }

            /// return the current content type of the playlist
            async fn content_type(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.content_type().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }

            /// return a stream representing the current track, and the byte_per_millisecond
            /// the stream should be closed when the track is finished
            /// return none if the playlist is finished
            async fn next_stream(
                &mut self,
            ) -> anyhow::Result<Option<(Box<dyn tokio::io::AsyncRead + Unpin + Sync + std::marker::Send>, u128)>> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.next_stream().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }

            /// check if the Playlist is finished
            async fn is_finished(&mut self) -> anyhow::Result<bool> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.is_finished().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }

            /// reset the played status of the child
            async fn reset(&mut self) -> anyhow::Result<()> {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.reset().await
                } else {
                    anyhow::bail!("inner is none")
                }
            }
        }
    }
}

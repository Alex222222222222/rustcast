use proc_macro2::Ident;
use quote::quote;
use syn::Generics;

pub fn impl_playlist_child(name: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics PlaylistChild for #name #ty_generics #where_clause{
            async fn stream_frame_with_meta(
                &'_ mut self,
            ) -> anyhow::Result<
                std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>>,
            > {
                self.init().await?;
                if let Some(inner) = &mut self.inner {
                    inner.stream_frame_with_meta().await
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
        }
    }
}

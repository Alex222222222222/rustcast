macro_rules! impl_playlist_child_by_redirect_to_self_variable {
    ($t:ty, $e:ident) => {
        #[async_trait]
        impl PlaylistChild for $t {
            async fn stream_frame_with_meta(
                &'_ mut self,
            ) -> anyhow::Result<
                std::pin::Pin<
                    Box<dyn futures::Stream<Item = anyhow::Result<FrameWithMeta>> + Send + '_>,
                >,
            > {
                self.$e.stream_frame_with_meta().await
            }

            async fn is_finished(&mut self) -> anyhow::Result<bool> {
                self.$e.is_finished().await
            }
        }
    };
}

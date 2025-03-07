macro_rules! impl_playlist_child_by_redirect_to_self_variable {
    ($t:ty, $e:ident) => {
        #[async_trait]
        impl PlaylistChild for $t {
            /// current_title returns the title of current playing song
            async fn current_title(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.$e.current_title().await
            }

            /// Artist returns the artist which is currently playing.
            async fn current_artist(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.$e.current_artist().await
            }

            /// return the current content type of the playlist
            async fn content_type(&mut self) -> anyhow::Result<std::sync::Arc<String>> {
                self.$e.content_type().await
            }

            /// return the current byte_per_millisecond
            async fn byte_per_millisecond(&mut self) -> anyhow::Result<u128> {
                self.$e.byte_per_millisecond().await
            }

            /// return a stream representing the current track, and the byte_per_millisecond
            /// the stream should be closed when the track is finished
            /// return none if the playlist is finished
            async fn next_frame(&mut self) -> anyhow::Result<Option<Bytes>> {
                self.$e.next_frame().await
            }

            /// check if the Playlist is finished
            async fn is_finished(&mut self) -> anyhow::Result<bool> {
                self.$e.is_finished().await
            }

            /// reset the played status of the child
            async fn reset(&mut self) -> anyhow::Result<()> {
                self.$e.reset().await
            }
        }
    };
}

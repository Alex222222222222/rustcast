mod playlist;
mod playlist_child;
mod playlist_frame_stream;

pub use playlist::Playlist;
pub use playlist::PreparedFrame;
pub use playlist_child::{LocalFileTrack, PlaylistChild};
pub use playlist_frame_stream::PlaylistFrameStream;


/// default_frame_size: 32768 bytes
const DEFAULT_FRAME_SIZE: usize = 32768;

/// maximum write ahead duration in milliseconds for the playlist
const MAX_WRITE_AHEAD_DURATION: u128 = 300000;
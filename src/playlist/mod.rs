mod listener_frame_data;
mod playlist_child;
mod playlist_frame_stream;
mod playlist_struct;

// re-export the modules
pub use playlist_child::*;
pub use playlist_frame_stream::PlaylistFrameStream;
pub use playlist_struct::{Playlist, PreparedFrame};

/// default_frame_size: 32768 bytes
const DEFAULT_FRAME_SIZE: usize = 2097152;

/// maximum write ahead duration in milliseconds for the playlist
const MAX_WRITE_AHEAD_DURATION: u128 = 60000;

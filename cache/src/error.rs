use thiserror::Error;

/// Errors that can occur during caching.
#[derive(Error, Debug)]
pub enum Error {
    /// Arises when the resource looks like a local file but it doesn't exist.
    #[error("Resource not found at the location ({0})")]
    ResourceNotFound(String),

    /// Arises when the resource looks like a URL, but is invalid.
    #[error("Unable to parse resource URL ({0})")]
    InvalidUrl(String),

    /// Arises when the cache is being used in offline mode, but it couldn't locate
    /// any cached versions of a remote resource.
    #[error("Offline mode is enabled but no cached versions of resouce exist ({0})")]
    NoCachedVersions(String),

    /// Arises when the cache is corrupted for some reason.
    ///
    /// If this error occurs, it is almost certainly the result of an external process
    /// "messing" with the cache directory, since `cached-path` takes great care
    /// to avoid accidental corruption on its own.
    #[error("Cache is corrupted ({0})")]
    CacheCorrupted(String),

    /// Arises when a resource is treated as archive, but the extraction process fails.
    #[error("Extracting archive failed ({0})")]
    ExtractionError(String),

    /// Any IO error that could arise while attempting to cache a remote resource.
    #[error("An IO error occurred")]
    IoError(#[from] std::io::Error),

    /// Failed to get the object storage.
    #[error("Failed to get object storage")]
    ObjectStorageError(#[from] object_store::Error),

    /// Failed to parse a object storage path.
    #[error("Failed to parse object storage path")]
    ObjectStoragePathError(#[from] object_store::path::Error),

    /// An HTTP error that could occur while attempting to fetch a remote resource.
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    /// A method that should never be called was called.
    #[error("Method not implemented")]
    NotImplemented,
}

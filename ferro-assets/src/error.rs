//! Error types for the asset pipeline.

use thiserror::Error;

/// Errors that can occur during asset pipeline processing.
#[derive(Debug, Error)]
pub enum Error {
    /// A transform failed on a specific file.
    #[error("transform '{transform}' failed on '{path}': {cause}")]
    Transform {
        /// The transform that failed (e.g. "html_minify", "image_transcode").
        transform: String,
        /// The logical asset path that caused the failure.
        path: String,
        /// The underlying error message.
        cause: String,
    },

    /// Thread pool or setup error.
    #[error("setup error: {0}")]
    Setup(String),
}

impl Error {
    /// Construct a transform error with full per-file and per-transform context.
    pub fn transform(
        transform: impl Into<String>,
        path: impl Into<String>,
        cause: impl Into<String>,
    ) -> Self {
        Self::Transform {
            transform: transform.into(),
            path: path.into(),
            cause: cause.into(),
        }
    }

    /// Construct a setup error (e.g. rayon pool build failure).
    pub fn setup(cause: impl Into<String>) -> Self {
        Self::Setup(cause.into())
    }
}

//! Crate-local error type for OAuth operations.

/// All errors that can occur during OAuth authorization server operations.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    /// The authorization code is missing, expired, or already used.
    #[error("invalid_grant")]
    InvalidGrant,

    /// The client credentials are invalid or unrecognized.
    #[error("invalid_client")]
    InvalidClient,

    /// The client registration metadata is invalid.
    #[error("invalid_client_metadata: {0}")]
    InvalidClientMetadata(String),

    /// An internal server error occurred.
    #[error("server_error: {0}")]
    ServerError(String),

    /// A JWT operation failed.
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

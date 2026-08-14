//! Error types for the broadcast system.

use thiserror::Error;

/// Errors that can occur during broadcasting.
#[derive(Error, Debug)]
pub enum Error {
    /// WebSocket connection error.
    #[error("websocket error: {0}")]
    WebSocket(String),

    /// Channel not found.
    #[error("channel not found: {0}")]
    ChannelNotFound(String),

    /// Authorization failed.
    #[error("authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Client not connected.
    #[error("client not connected: {0}")]
    ClientNotConnected(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Channel is full (too many subscribers).
    #[error("channel is full")]
    ChannelFull,

    /// Shared-transport error (bus publish/subscribe).
    #[error("transport error: {0}")]
    Transport(String),

    /// Redis transport error (requires the `redis-transport` feature).
    #[cfg(feature = "redis-transport")]
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a WebSocket error.
    pub fn websocket(msg: impl Into<String>) -> Self {
        Self::WebSocket(msg.into())
    }

    /// Create an authorization failed error.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::AuthorizationFailed(msg.into())
    }

    /// Construct a transport error from any displayable message.
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }
}

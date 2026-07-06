//! Error types for the event system.

use thiserror::Error;

/// Errors that can occur in the event system.
#[derive(Debug, Error)]
pub enum Error {
    /// A listener failed to handle the event.
    #[error("Listener '{listener}' failed: {message}")]
    ListenerFailed {
        /// The name of the listener that failed.
        listener: String,
        /// The error message.
        message: String,
    },

    /// Failed to dispatch an event.
    #[error("Failed to dispatch event '{event}': {message}")]
    DispatchFailed {
        /// The name of the event.
        event: String,
        /// The error message.
        message: String,
    },

    /// A queued listener failed to serialize.
    #[error("Failed to serialize event for queue: {0}")]
    SerializationFailed(String),

    /// A queued listener failed to deserialize.
    #[error("Failed to deserialize event from queue: {0}")]
    DeserializationFailed(String),

    /// Queue connection failed.
    #[error("Queue connection failed: {0}")]
    QueueConnectionFailed(String),

    /// Custom error with a message.
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a new listener failed error.
    pub fn listener_failed(listener: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ListenerFailed {
            listener: listener.into(),
            message: message.into(),
        }
    }

    /// Create a new dispatch failed error.
    pub fn dispatch_failed(event: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DispatchFailed {
            event: event.into(),
            message: message.into(),
        }
    }

    /// Create a custom error.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

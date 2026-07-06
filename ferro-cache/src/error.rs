//! Error types for cache operations.

use thiserror::Error;

/// Cache error types.
#[derive(Error, Debug)]
pub enum Error {
    /// Key not found.
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Store not configured.
    #[error("Store not configured: {0}")]
    StoreNotConfigured(String),

    /// Redis error.
    #[cfg(feature = "redis-backend")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

impl Error {
    /// Create a not found error.
    pub fn not_found(key: impl Into<String>) -> Self {
        Self::NotFound(key.into())
    }

    /// Create a serialization error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a deserialization error.
    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }

    /// Create a connection error.
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a store not configured error.
    pub fn store_not_configured(store: impl Into<String>) -> Self {
        Self::StoreNotConfigured(store.into())
    }
}

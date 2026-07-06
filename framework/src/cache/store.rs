//! Cache store trait definition
//!
//! Defines the contract for cache implementations (Redis, InMemory, etc.)

use async_trait::async_trait;
use std::time::Duration;

use crate::error::FrameworkError;

/// Cache store trait - all cache backends must implement this
///
/// This trait uses JSON strings for values to enable dynamic typing.
/// The `Cache` facade handles serialization/deserialization.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Retrieve a raw JSON value from the cache by key
    async fn get_raw(&self, key: &str) -> Result<Option<String>, FrameworkError>;

    /// Store a raw JSON value in the cache
    async fn put_raw(
        &self,
        key: &str,
        value: &str,
        ttl: Option<Duration>,
    ) -> Result<(), FrameworkError>;

    /// Check if a key exists in the cache
    async fn has(&self, key: &str) -> Result<bool, FrameworkError>;

    /// Remove an item from the cache
    async fn forget(&self, key: &str) -> Result<bool, FrameworkError>;

    /// Remove all items from the cache
    async fn flush(&self) -> Result<(), FrameworkError>;

    /// Increment a numeric value
    ///
    /// Returns the new value after incrementing.
    async fn increment(&self, key: &str, amount: i64) -> Result<i64, FrameworkError>;

    /// Decrement a numeric value
    ///
    /// Returns the new value after decrementing.
    async fn decrement(&self, key: &str, amount: i64) -> Result<i64, FrameworkError>;

    /// Set a TTL on an existing key
    ///
    /// Returns `true` if the key existed and TTL was set, `false` if key not found.
    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, FrameworkError>;
}

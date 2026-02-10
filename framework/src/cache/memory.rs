//! In-memory cache implementation for testing and fallback
//!
//! Provides a thread-safe in-memory cache that mimics Redis behavior.
//! Supports TTL expiration.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use super::store::CacheStore;
use crate::error::FrameworkError;

/// In-memory cache entry with optional expiration
#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.expires_at.map(|t| Instant::now() > t).unwrap_or(false)
    }
}

/// In-memory cache implementation
///
/// Thread-safe cache that stores values in memory with optional TTL.
/// Use this as a fallback when Redis is unavailable, or in tests.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::cache::InMemoryCache;
///
/// let cache = InMemoryCache::new();
/// ```
pub struct InMemoryCache {
    store: RwLock<HashMap<String, CacheEntry>>,
    prefix: String,
}

impl InMemoryCache {
    /// Create a new empty in-memory cache
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            prefix: "ferro_cache:".to_string(),
        }
    }

    /// Create with a custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            prefix: prefix.into(),
        }
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheStore for InMemoryCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, FrameworkError> {
        let key = self.prefixed_key(key);

        let store = self
            .store
            .read()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        match store.get(&key) {
            Some(entry) if !entry.is_expired() => Ok(Some(entry.value.clone())),
            _ => Ok(None),
        }
    }

    async fn put_raw(
        &self,
        key: &str,
        value: &str,
        ttl: Option<Duration>,
    ) -> Result<(), FrameworkError> {
        let key = self.prefixed_key(key);

        let entry = CacheEntry {
            value: value.to_string(),
            expires_at: ttl.map(|d| Instant::now() + d),
        };

        let mut store = self
            .store
            .write()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        store.insert(key, entry);
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);

        let store = self
            .store
            .read()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        Ok(store.get(&key).map(|e| !e.is_expired()).unwrap_or(false))
    }

    async fn forget(&self, key: &str) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);

        let mut store = self
            .store
            .write()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        Ok(store.remove(&key).is_some())
    }

    async fn flush(&self) -> Result<(), FrameworkError> {
        let mut store = self
            .store
            .write()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        store.clear();
        Ok(())
    }

    async fn increment(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        let key = self.prefixed_key(key);

        let mut store = self
            .store
            .write()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        let current: i64 = store
            .get(&key)
            .filter(|e| !e.is_expired())
            .and_then(|e| e.value.parse().ok())
            .unwrap_or(0);

        let new_value = current + amount;

        store.insert(
            key,
            CacheEntry {
                value: new_value.to_string(),
                expires_at: None,
            },
        );

        Ok(new_value)
    }

    async fn decrement(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        self.increment(key, -amount).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);

        let mut store = self
            .store
            .write()
            .map_err(|_| FrameworkError::internal("Cache lock poisoned"))?;

        if let Some(entry) = store.get_mut(&key) {
            entry.expires_at = Some(Instant::now() + ttl);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_expire_sets_ttl() {
        let cache = InMemoryCache::new();

        // Increment a key so it exists
        cache.increment("counter", 1).await.unwrap();

        // Set a 1-second TTL
        let result = cache.expire("counter", Duration::from_secs(1)).await.unwrap();
        assert!(result, "expire should return true for existing key");

        // Key should still be accessible immediately
        let val = cache.get_raw("counter").await.unwrap();
        assert_eq!(val, Some("1".to_string()));

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Key should now be expired
        let val = cache.get_raw("counter").await.unwrap();
        assert!(val.is_none(), "key should be expired after TTL");

        // Increment should treat it as new (returns 1)
        let new_val = cache.increment("counter", 1).await.unwrap();
        assert_eq!(new_val, 1, "increment on expired key should return 1");
    }

    #[tokio::test]
    async fn test_expire_missing_key() {
        let cache = InMemoryCache::new();

        let result = cache.expire("nonexistent", Duration::from_secs(10)).await.unwrap();
        assert!(!result, "expire on missing key should return false");
    }

    #[tokio::test]
    async fn test_increment_then_expire_preserves_value() {
        let cache = InMemoryCache::new();

        // Increment to 5
        for _ in 0..5 {
            cache.increment("counter", 1).await.unwrap();
        }

        // Set TTL (long enough not to expire during test)
        let result = cache.expire("counter", Duration::from_secs(10)).await.unwrap();
        assert!(result);

        // Increment again should return 6 (not 1)
        let val = cache.increment("counter", 1).await.unwrap();
        assert_eq!(val, 6, "expire should not reset the value");
    }
}

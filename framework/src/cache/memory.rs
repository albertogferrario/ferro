//! In-memory cache implementation backed by moka
//!
//! Provides a thread-safe in-memory cache with bounded capacity,
//! per-entry TTL via the Expiry trait, and proactive eviction.

use async_trait::async_trait;
use moka::sync::Cache;
use moka::Expiry;
use std::time::{Duration, Instant};

use super::store::CacheStore;
use crate::error::FrameworkError;

/// Cache value with optional per-entry TTL
#[derive(Clone)]
struct CacheValue {
    value: String,
    ttl: Option<Duration>,
}

/// Expiry policy that reads TTL from each CacheValue
struct CacheTtlExpiry;

impl Expiry<String, CacheValue> for CacheTtlExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheValue,
        _created_at: Instant,
    ) -> Option<Duration> {
        value.ttl
    }

    fn expire_after_read(
        &self,
        _key: &String,
        _value: &CacheValue,
        _read_at: Instant,
        duration_until_expiry: Option<Duration>,
        _last_modified_at: Instant,
    ) -> Option<Duration> {
        duration_until_expiry
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &CacheValue,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        value.ttl
    }
}

/// In-memory cache implementation
///
/// Bounded cache backed by moka with per-entry TTL and LRU eviction.
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
    cache: Cache<String, CacheValue>,
    prefix: String,
}

impl InMemoryCache {
    /// Create a new in-memory cache with default capacity (10,000 entries)
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    /// Create with a custom maximum capacity
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(capacity)
                .expire_after(CacheTtlExpiry)
                .build(),
            prefix: "ferro_cache:".to_string(),
        }
    }

    /// Create with a custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(10_000)
                .expire_after(CacheTtlExpiry)
                .build(),
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
        Ok(self.cache.get(&key).map(|cv| cv.value))
    }

    async fn put_raw(
        &self,
        key: &str,
        value: &str,
        ttl: Option<Duration>,
    ) -> Result<(), FrameworkError> {
        let key = self.prefixed_key(key);
        self.cache.insert(
            key,
            CacheValue {
                value: value.to_string(),
                ttl,
            },
        );
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);
        Ok(self.cache.contains_key(&key))
    }

    async fn forget(&self, key: &str) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);
        let existed = self.cache.contains_key(&key);
        self.cache.remove(&key);
        Ok(existed)
    }

    async fn flush(&self) -> Result<(), FrameworkError> {
        self.cache.invalidate_all();
        Ok(())
    }

    async fn increment(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        let key = self.prefixed_key(key);

        let current: i64 = self
            .cache
            .get(&key)
            .and_then(|cv| cv.value.parse().ok())
            .unwrap_or(0);

        let new_value = current + amount;

        self.cache.insert(
            key,
            CacheValue {
                value: new_value.to_string(),
                ttl: None,
            },
        );

        Ok(new_value)
    }

    async fn decrement(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        self.increment(key, -amount).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, FrameworkError> {
        let key = self.prefixed_key(key);

        match self.cache.get(&key) {
            Some(cv) => {
                self.cache.insert(
                    key,
                    CacheValue {
                        value: cv.value,
                        ttl: Some(ttl),
                    },
                );
                Ok(true)
            }
            None => Ok(false),
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
        let result = cache
            .expire("counter", Duration::from_secs(1))
            .await
            .unwrap();
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

        let result = cache
            .expire("nonexistent", Duration::from_secs(10))
            .await
            .unwrap();
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
        let result = cache
            .expire("counter", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(result);

        // Increment again should return 6 (not 1)
        let val = cache.increment("counter", 1).await.unwrap();
        assert_eq!(val, 6, "expire should not reset the value");
    }

    #[tokio::test]
    async fn test_put_get_forget_flush() {
        let cache = InMemoryCache::new();

        // put + get
        cache.put_raw("key1", "value1", None).await.unwrap();
        assert_eq!(
            cache.get_raw("key1").await.unwrap(),
            Some("value1".to_string())
        );
        assert!(cache.has("key1").await.unwrap());

        // get missing key
        assert!(cache.get_raw("missing").await.unwrap().is_none());
        assert!(!cache.has("missing").await.unwrap());

        // forget existing
        assert!(cache.forget("key1").await.unwrap());
        assert!(cache.get_raw("key1").await.unwrap().is_none());

        // forget missing
        assert!(!cache.forget("key1").await.unwrap());

        // flush
        cache.put_raw("a", "1", None).await.unwrap();
        cache.put_raw("b", "2", None).await.unwrap();
        cache.flush().await.unwrap();
        assert!(!cache.has("a").await.unwrap());
        assert!(!cache.has("b").await.unwrap());
    }

    #[tokio::test]
    async fn test_capacity_eviction() {
        let cache = InMemoryCache::with_capacity(100);

        for i in 0..200u64 {
            cache
                .put_raw(&format!("key{i}"), &format!("val{i}"), None)
                .await
                .unwrap();
        }

        // Trigger pending evictions
        cache.cache.run_pending_tasks();

        let count = cache.cache.entry_count();
        assert!(
            count <= 110,
            "cache should be bounded near capacity, got {count}"
        );
    }

    #[tokio::test]
    async fn test_expired_entries_not_returned() {
        let cache = InMemoryCache::new();

        cache
            .put_raw("short-lived", "data", Some(Duration::from_millis(100)))
            .await
            .unwrap();

        // Exists immediately
        assert!(cache.has("short-lived").await.unwrap());
        assert_eq!(
            cache.get_raw("short-lived").await.unwrap(),
            Some("data".to_string())
        );

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Moka proactively filters expired entries
        assert!(
            cache.get_raw("short-lived").await.unwrap().is_none(),
            "expired entry should not be returned"
        );
        assert!(
            !cache.has("short-lived").await.unwrap(),
            "has() should return false for expired entry"
        );
    }
}

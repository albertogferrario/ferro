//! In-memory cache store using moka.

use crate::cache::CacheStore;
use crate::error::Error;
use async_trait::async_trait;
use dashmap::DashMap;
use moka::future::Cache as MokaCache;
use moka::policy::Expiry;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Wrapper that stores data alongside its per-entry TTL.
#[derive(Clone)]
struct CacheValue {
    data: Vec<u8>,
    ttl: Duration,
}

/// Per-entry expiry policy: each entry expires after its own TTL.
struct PerEntryExpiry;

impl Expiry<String, CacheValue> for PerEntryExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheValue,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(value.ttl)
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &CacheValue,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        Some(value.ttl)
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
}

/// In-memory cache store.
pub struct MemoryStore {
    cache: MokaCache<String, CacheValue>,
    tags: Arc<DashMap<String, HashSet<String>>>,
    counters: Arc<DashMap<String, i64>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    /// Create a new memory store.
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    /// Create with custom capacity.
    pub fn with_capacity(capacity: u64) -> Self {
        let tags: Arc<DashMap<String, HashSet<String>>> = Arc::new(DashMap::new());
        let tags_clone = tags.clone();

        let cache = MokaCache::builder()
            .max_capacity(capacity)
            .expire_after(PerEntryExpiry)
            .eviction_listener(move |key: Arc<String>, _value, _cause| {
                tags_clone.retain(|_tag, members| {
                    members.remove(key.as_str());
                    !members.is_empty()
                });
            })
            .build();

        Self {
            cache,
            tags,
            counters: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl CacheStore for MemoryStore {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        Ok(self.cache.get(key).await.map(|cv| cv.data))
    }

    async fn put_raw(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<(), Error> {
        let cv = CacheValue { data: value, ttl };
        self.cache.insert(key.to_string(), cv).await;
        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool, Error> {
        Ok(self.cache.contains_key(key))
    }

    async fn forget(&self, key: &str) -> Result<bool, Error> {
        let existed = self.cache.contains_key(key);
        self.cache.remove(key).await;
        self.counters.remove(key);
        Ok(existed)
    }

    async fn flush(&self) -> Result<(), Error> {
        self.cache.invalidate_all();
        self.tags.clear();
        self.counters.clear();
        Ok(())
    }

    async fn increment(&self, key: &str, value: i64) -> Result<i64, Error> {
        let mut entry = self.counters.entry(key.to_string()).or_insert(0);
        *entry += value;
        Ok(*entry)
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64, Error> {
        let mut entry = self.counters.entry(key.to_string()).or_insert(0);
        *entry -= value;
        Ok(*entry)
    }

    async fn tag_add(&self, tag: &str, key: &str) -> Result<(), Error> {
        self.tags
            .entry(tag.to_string())
            .or_default()
            .insert(key.to_string());
        Ok(())
    }

    async fn tag_members(&self, tag: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .tags
            .get(tag)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default())
    }

    async fn tag_flush(&self, tag: &str) -> Result<(), Error> {
        if let Some((_, keys)) = self.tags.remove(tag) {
            for key in keys {
                self.cache.remove(&key).await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store_put_get() {
        let store = MemoryStore::new();

        store
            .put_raw("key", b"value".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        let value = store.get_raw("key").await.unwrap();
        assert_eq!(value, Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_memory_store_has() {
        let store = MemoryStore::new();

        assert!(!store.has("missing").await.unwrap());

        store
            .put_raw("exists", b"value".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        assert!(store.has("exists").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_store_forget() {
        let store = MemoryStore::new();

        store
            .put_raw("key", b"value".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        let removed = store.forget("key").await.unwrap();
        assert!(removed);
        assert!(!store.has("key").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_store_increment_decrement() {
        let store = MemoryStore::new();

        let val = store.increment("counter", 5).await.unwrap();
        assert_eq!(val, 5);

        let val = store.increment("counter", 3).await.unwrap();
        assert_eq!(val, 8);

        let val = store.decrement("counter", 2).await.unwrap();
        assert_eq!(val, 6);
    }

    #[tokio::test]
    async fn test_memory_store_tags() {
        let store = MemoryStore::new();

        store
            .put_raw("user:1", b"alice".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();
        store
            .put_raw("user:2", b"bob".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        store.tag_add("users", "user:1").await.unwrap();
        store.tag_add("users", "user:2").await.unwrap();

        let members = store.tag_members("users").await.unwrap();
        assert_eq!(members.len(), 2);

        store.tag_flush("users").await.unwrap();

        assert!(!store.has("user:1").await.unwrap());
        assert!(!store.has("user:2").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_store_flush() {
        let store = MemoryStore::new();

        store
            .put_raw("key1", b"value1".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();
        store
            .put_raw("key2", b"value2".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        store.flush().await.unwrap();

        assert!(!store.has("key1").await.unwrap());
        assert!(!store.has("key2").await.unwrap());
    }
}

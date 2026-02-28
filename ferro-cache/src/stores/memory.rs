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
    counters: MokaCache<String, i64>,
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

        let counters = MokaCache::builder().max_capacity(capacity).build();

        Self {
            cache,
            tags,
            counters,
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
        self.counters.remove(key).await;
        Ok(existed)
    }

    async fn flush(&self) -> Result<(), Error> {
        self.cache.invalidate_all();
        self.tags.clear();
        self.counters.invalidate_all();
        Ok(())
    }

    async fn increment(&self, key: &str, value: i64) -> Result<i64, Error> {
        let current = self.counters.get(key).await.unwrap_or(0);
        let new_val = current + value;
        self.counters.insert(key.to_string(), new_val).await;
        Ok(new_val)
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64, Error> {
        let current = self.counters.get(key).await.unwrap_or(0);
        let new_val = current - value;
        self.counters.insert(key.to_string(), new_val).await;
        Ok(new_val)
    }

    async fn tag_add(&self, tag: &str, key: &str) -> Result<(), Error> {
        self.tags
            .entry(tag.to_string())
            .or_default()
            .insert(key.to_string());
        Ok(())
    }

    async fn tag_members(&self, tag: &str) -> Result<Vec<String>, Error> {
        let Some(mut entry) = self.tags.get_mut(tag) else {
            return Ok(Vec::new());
        };
        // Lazy cleanup: remove keys no longer present in the cache.
        entry.retain(|k| self.cache.contains_key(k));
        let members: Vec<String> = entry.iter().cloned().collect();
        let is_empty = entry.is_empty();
        drop(entry);
        if is_empty {
            self.tags.remove(tag);
        }
        Ok(members)
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

    #[tokio::test]
    async fn test_per_entry_ttl_respected() {
        let store = MemoryStore::new();

        store
            .put_raw("short", b"data".to_vec(), Duration::from_millis(100))
            .await
            .unwrap();

        // Entry exists immediately.
        assert!(store.has("short").await.unwrap());

        tokio::time::sleep(Duration::from_millis(200)).await;
        store.cache.run_pending_tasks().await;

        // Entry expired after its TTL.
        assert!(store.get_raw("short").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_tag_deduplication() {
        let store = MemoryStore::new();

        store
            .put_raw("item", b"val".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();

        // Add same key to same tag twice.
        store.tag_add("dup-tag", "item").await.unwrap();
        store.tag_add("dup-tag", "item").await.unwrap();

        let members = store.tag_members("dup-tag").await.unwrap();
        assert_eq!(members.len(), 1, "duplicate tag entries must be prevented");
    }

    #[tokio::test]
    async fn test_eviction_cleans_tags() {
        let store = MemoryStore::new();

        // Insert entries with short TTL and tag them.
        for i in 0..5u64 {
            let key = format!("ephemeral{i}");
            store
                .put_raw(&key, b"v".to_vec(), Duration::from_millis(100))
                .await
                .unwrap();
            store.tag_add("temp", &key).await.unwrap();
        }

        assert_eq!(store.tag_members("temp").await.unwrap().len(), 5);

        // Wait for TTL expiry.
        tokio::time::sleep(Duration::from_millis(200)).await;
        store.cache.run_pending_tasks().await;

        // tag_members performs lazy cleanup of stale references.
        let members = store.tag_members("temp").await.unwrap();
        assert!(
            members.is_empty(),
            "stale tag references should be cleaned on read, got {} members",
            members.len()
        );

        // Empty tag set should be pruned.
        assert_eq!(store.tags.len(), 0, "empty tag sets should be removed");
    }

    #[tokio::test]
    async fn test_eviction_listener_on_explicit_remove() {
        let store = MemoryStore::new();

        store
            .put_raw("tagged-key", b"data".to_vec(), Duration::from_secs(60))
            .await
            .unwrap();
        store.tag_add("group", "tagged-key").await.unwrap();

        // Explicit removal triggers the eviction listener.
        store.cache.remove("tagged-key").await;
        store.cache.run_pending_tasks().await;

        // Listener should have cleaned the key from tag sets.
        let raw_members: Vec<String> = store
            .tags
            .get("group")
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        assert!(
            raw_members.is_empty(),
            "eviction listener should remove key from tags on explicit removal"
        );
    }

    #[tokio::test]
    async fn test_counters_bounded() {
        let store = MemoryStore::with_capacity(50);

        // Insert many unique counter keys.
        for i in 0..200u64 {
            store.increment(&format!("c{i}"), 1).await.unwrap();
        }
        store.counters.run_pending_tasks().await;

        let count = store.counters.entry_count();
        assert!(
            count <= 60,
            "counter count should be bounded near capacity, got {count}"
        );
    }
}

//! Tagged cache for bulk invalidation.

use crate::cache::{CacheConfig, CacheStore};
use crate::error::Error;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// A cache instance with tags for grouped operations.
pub struct TaggedCache {
    store: Arc<dyn CacheStore>,
    tags: Vec<String>,
    config: CacheConfig,
}

impl TaggedCache {
    /// Create a new tagged cache.
    pub(crate) fn new(store: Arc<dyn CacheStore>, tags: Vec<String>, config: CacheConfig) -> Self {
        Self {
            store,
            tags,
            config,
        }
    }

    /// Generate a tag namespace prefix for keys.
    fn tagged_key(&self, key: &str) -> String {
        let tag_prefix: String = self.tags.join(":");
        if self.config.prefix.is_empty() {
            format!("tag:{tag_prefix}:{key}")
        } else {
            format!("{}:tag:{}:{}", self.config.prefix, tag_prefix, key)
        }
    }

    /// Get a value from the cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        let key = self.tagged_key(key);
        match self.store.get_raw(&key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| Error::deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Put a value in the cache with tags.
    pub async fn put<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), Error> {
        let tagged_key = self.tagged_key(key);
        let bytes = serde_json::to_vec(value).map_err(|e| Error::serialization(e.to_string()))?;

        // Store the value
        self.store.put_raw(&tagged_key, bytes, ttl).await?;

        // Associate with all tags
        for tag in &self.tags {
            let tag_set_key = format!("tag_set:{tag}");
            self.store.tag_add(&tag_set_key, &tagged_key).await?;
        }

        Ok(())
    }

    /// Put a value using the default TTL.
    pub async fn put_default<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
        self.put(key, value, self.config.default_ttl).await
    }

    /// Store a value forever.
    pub async fn forever<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
        self.put(key, value, Duration::from_secs(315_360_000)).await
    }

    /// Check if a key exists.
    pub async fn has(&self, key: &str) -> Result<bool, Error> {
        let key = self.tagged_key(key);
        self.store.has(&key).await
    }

    /// Remove a key from the cache.
    pub async fn forget(&self, key: &str) -> Result<bool, Error> {
        let key = self.tagged_key(key);
        self.store.forget(&key).await
    }

    /// Flush all cache entries with any of the configured tags.
    pub async fn flush(&self) -> Result<(), Error> {
        for tag in &self.tags {
            let tag_set_key = format!("tag_set:{tag}");
            self.store.tag_flush(&tag_set_key).await?;
        }
        Ok(())
    }

    /// Get a value or compute it if not cached.
    pub async fn remember<T, F, Fut>(&self, key: &str, ttl: Duration, f: F) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        if let Some(value) = self.get(key).await? {
            return Ok(value);
        }

        let value = f().await;
        self.put(key, &value, ttl).await?;
        Ok(value)
    }

    /// Get a value or compute it, caching forever.
    pub async fn remember_forever<T, F, Fut>(&self, key: &str, f: F) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.remember(key, Duration::from_secs(315_360_000), f)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::MemoryStore;

    #[tokio::test]
    async fn test_tagged_cache_put_get() {
        let store = Arc::new(MemoryStore::new());
        let cache = TaggedCache::new(store, vec!["users".to_string()], CacheConfig::default());

        cache
            .put("user:1", &"Alice", Duration::from_secs(60))
            .await
            .unwrap();

        let value: Option<String> = cache.get("user:1").await.unwrap();
        assert_eq!(value, Some("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_tagged_cache_flush() {
        let store = Arc::new(MemoryStore::new());
        let cache = TaggedCache::new(
            store.clone(),
            vec!["users".to_string()],
            CacheConfig::default(),
        );

        cache
            .put("user:1", &"Alice", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .put("user:2", &"Bob", Duration::from_secs(60))
            .await
            .unwrap();

        assert!(cache.has("user:1").await.unwrap());
        assert!(cache.has("user:2").await.unwrap());

        cache.flush().await.unwrap();

        assert!(!cache.has("user:1").await.unwrap());
        assert!(!cache.has("user:2").await.unwrap());
    }

    #[tokio::test]
    async fn test_tagged_cache_remember() {
        let store = Arc::new(MemoryStore::new());
        let cache = TaggedCache::new(store, vec!["data".to_string()], CacheConfig::default());

        let mut call_count = 0;

        let value: i32 = cache
            .remember("computed", Duration::from_secs(60), || async {
                call_count += 1;
                42
            })
            .await
            .unwrap();

        assert_eq!(value, 42);

        // Second call should use cache
        let value2: i32 = cache
            .remember("computed", Duration::from_secs(60), || async {
                call_count += 1;
                99
            })
            .await
            .unwrap();

        assert_eq!(value2, 42); // Should still be 42, not 99
    }

    #[tokio::test]
    async fn test_tagged_cache_multiple_tags() {
        let store = Arc::new(MemoryStore::new());

        // Cache with two tags
        let cache = TaggedCache::new(
            store.clone(),
            vec!["users".to_string(), "admins".to_string()],
            CacheConfig::default(),
        );

        cache
            .put("admin:1", &"Super Admin", Duration::from_secs(60))
            .await
            .unwrap();

        // Create a cache with just "users" tag
        let users_cache = TaggedCache::new(
            store.clone(),
            vec!["users".to_string()],
            CacheConfig::default(),
        );

        // Flushing "users" should also flush our entry (since it has both tags)
        users_cache.flush().await.unwrap();

        assert!(!cache.has("admin:1").await.unwrap());
    }
}

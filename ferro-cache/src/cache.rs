//! Core cache trait and facade.

use crate::error::Error;
use crate::tagged::TaggedCache;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::env;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Cache store configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for cache entries.
    pub default_ttl: Duration,
    /// Cache key prefix.
    pub prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),
            prefix: String::new(),
        }
    }
}

impl CacheConfig {
    /// Create a new cache config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `CACHE_PREFIX`: Key prefix for all cache entries (default: "")
    /// - `CACHE_TTL`: Default TTL in seconds (default: 3600)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_cache::CacheConfig;
    ///
    /// let config = CacheConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        let prefix = env::var("CACHE_PREFIX").unwrap_or_default();
        let default_ttl = env::var("CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(3600));

        Self {
            default_ttl,
            prefix,
        }
    }

    /// Set default TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Set key prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

/// Cache store trait.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Get a value from the cache.
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, Error>;

    /// Put a value in the cache.
    async fn put_raw(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<(), Error>;

    /// Check if a key exists.
    async fn has(&self, key: &str) -> Result<bool, Error>;

    /// Remove a key from the cache.
    async fn forget(&self, key: &str) -> Result<bool, Error>;

    /// Remove all items from the cache.
    async fn flush(&self) -> Result<(), Error>;

    /// Increment a numeric value.
    async fn increment(&self, key: &str, value: i64) -> Result<i64, Error>;

    /// Decrement a numeric value.
    async fn decrement(&self, key: &str, value: i64) -> Result<i64, Error>;

    /// Add a key to a tag set.
    async fn tag_add(&self, tag: &str, key: &str) -> Result<(), Error>;

    /// Get all keys in a tag set.
    async fn tag_members(&self, tag: &str) -> Result<Vec<String>, Error>;

    /// Remove a tag set.
    async fn tag_flush(&self, tag: &str) -> Result<(), Error>;
}

/// Main cache facade.
#[derive(Clone)]
pub struct Cache {
    store: Arc<dyn CacheStore>,
    config: CacheConfig,
}

impl Cache {
    /// Create a new cache with the given store.
    pub fn new(store: Arc<dyn CacheStore>) -> Self {
        Self {
            store,
            config: CacheConfig::default(),
        }
    }

    /// Create a cache with configuration.
    pub fn with_config(store: Arc<dyn CacheStore>, config: CacheConfig) -> Self {
        Self { store, config }
    }

    /// Create an in-memory cache.
    #[cfg(feature = "memory")]
    pub fn memory() -> Self {
        Self::new(Arc::new(crate::stores::MemoryStore::new()))
    }

    /// Create a Redis-backed cache.
    #[cfg(feature = "redis-backend")]
    pub async fn redis(url: &str) -> Result<Self, Error> {
        let store = crate::stores::RedisStore::new(url).await?;
        Ok(Self::new(Arc::new(store)))
    }

    /// Create a cache from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `CACHE_DRIVER`: Cache driver ("memory" or "redis", default: "memory")
    /// - `CACHE_PREFIX`: Key prefix for all cache entries (default: "")
    /// - `CACHE_TTL`: Default TTL in seconds (default: 3600)
    /// - `CACHE_MEMORY_CAPACITY`: Max entries for memory store (default: 10000)
    /// - `REDIS_URL`: Redis connection URL (required if CACHE_DRIVER=redis)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_cache::Cache;
    ///
    /// let cache = Cache::from_env().await?;
    /// ```
    #[cfg(feature = "memory")]
    pub async fn from_env() -> Result<Self, Error> {
        let driver = env::var("CACHE_DRIVER").unwrap_or_else(|_| "memory".to_string());
        let config = CacheConfig::from_env();

        let store: Arc<dyn CacheStore> = match driver.as_str() {
            #[cfg(feature = "redis-backend")]
            "redis" => {
                let url =
                    env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
                Arc::new(crate::stores::RedisStore::new(&url).await?)
            }
            _ => {
                // Default to memory
                let capacity = env::var("CACHE_MEMORY_CAPACITY")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10_000);
                Arc::new(crate::stores::MemoryStore::with_capacity(capacity))
            }
        };

        Ok(Self::with_config(store, config))
    }

    /// Get a prefixed key.
    fn prefixed_key(&self, key: &str) -> String {
        if self.config.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.config.prefix, key)
        }
    }

    /// Get a value from the cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        let key = self.prefixed_key(key);
        match self.store.get_raw(&key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| Error::deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Put a value in the cache.
    pub async fn put<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), Error> {
        let key = self.prefixed_key(key);
        let bytes = serde_json::to_vec(value).map_err(|e| Error::serialization(e.to_string()))?;
        self.store.put_raw(&key, bytes, ttl).await
    }

    /// Put a value using the default TTL.
    pub async fn put_default<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
        self.put(key, value, self.config.default_ttl).await
    }

    /// Store a value forever (very long TTL).
    pub async fn forever<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
        self.put(key, value, Duration::from_secs(315_360_000)).await // 10 years
    }

    /// Check if a key exists.
    pub async fn has(&self, key: &str) -> Result<bool, Error> {
        let key = self.prefixed_key(key);
        self.store.has(&key).await
    }

    /// Remove a key from the cache.
    pub async fn forget(&self, key: &str) -> Result<bool, Error> {
        let key = self.prefixed_key(key);
        self.store.forget(&key).await
    }

    /// Remove all items from the cache.
    pub async fn flush(&self) -> Result<(), Error> {
        self.store.flush().await
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

    /// Pull a value from cache and remove it.
    pub async fn pull<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        let value = self.get(key).await?;
        if value.is_some() {
            self.forget(key).await?;
        }
        Ok(value)
    }

    /// Increment a numeric value.
    pub async fn increment(&self, key: &str, value: i64) -> Result<i64, Error> {
        let key = self.prefixed_key(key);
        self.store.increment(&key, value).await
    }

    /// Decrement a numeric value.
    pub async fn decrement(&self, key: &str, value: i64) -> Result<i64, Error> {
        let key = self.prefixed_key(key);
        self.store.decrement(&key, value).await
    }

    /// Create a tagged cache instance.
    pub fn tags(&self, tags: &[&str]) -> TaggedCache {
        TaggedCache::new(
            self.store.clone(),
            tags.iter().map(|s| s.to_string()).collect(),
            self.config.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::new()
            .with_ttl(Duration::from_secs(1800))
            .with_prefix("myapp");

        assert_eq!(config.default_ttl, Duration::from_secs(1800));
        assert_eq!(config.prefix, "myapp");
    }

    #[test]
    fn test_prefixed_key() {
        let config = CacheConfig::new().with_prefix("test");
        let cache = Cache::with_config(Arc::new(crate::stores::MemoryStore::new()), config);

        assert_eq!(cache.prefixed_key("key"), "test:key");
    }

    #[test]
    fn test_prefixed_key_no_prefix() {
        let cache = Cache::new(Arc::new(crate::stores::MemoryStore::new()));
        assert_eq!(cache.prefixed_key("key"), "key");
    }
}

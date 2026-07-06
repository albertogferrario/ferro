//! Redis-backed cache implementation

use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use std::time::Duration;

use super::config::CacheConfig;
use super::store::CacheStore;
use crate::error::FrameworkError;

/// Redis cache implementation
///
/// Uses redis-rs with async/tokio runtime for high-performance caching.
pub struct RedisCache {
    conn: ConnectionManager,
    prefix: String,
    default_ttl: Option<Duration>,
}

impl RedisCache {
    /// Create a new Redis cache connection with a 2-second timeout
    pub async fn connect(config: &CacheConfig) -> Result<Self, FrameworkError> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| FrameworkError::internal(format!("Redis connection error: {e}")))?;

        // Timeout after 2 seconds to avoid hanging when Redis is unavailable
        let conn = tokio::time::timeout(Duration::from_secs(2), ConnectionManager::new(client))
            .await
            .map_err(|_| FrameworkError::internal("Redis connection timeout".to_string()))?
            .map_err(|e| {
                FrameworkError::internal(format!("Redis connection manager error: {e}"))
            })?;

        let default_ttl = if config.default_ttl > 0 {
            Some(Duration::from_secs(config.default_ttl))
        } else {
            None
        };

        Ok(Self {
            conn,
            prefix: config.prefix.clone(),
            default_ttl,
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait]
impl CacheStore for RedisCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache get error: {e}")))?;

        Ok(value)
    }

    async fn put_raw(
        &self,
        key: &str,
        value: &str,
        ttl: Option<Duration>,
    ) -> Result<(), FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let effective_ttl = ttl.or(self.default_ttl);

        if let Some(duration) = effective_ttl {
            conn.set_ex::<_, _, ()>(&key, value, duration.as_secs())
                .await
                .map_err(|e| FrameworkError::internal(format!("Cache set error: {e}")))?;
        } else {
            conn.set::<_, _, ()>(&key, value)
                .await
                .map_err(|e| FrameworkError::internal(format!("Cache set error: {e}")))?;
        }

        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache exists error: {e}")))?;

        Ok(exists)
    }

    async fn forget(&self, key: &str) -> Result<bool, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let deleted: i64 = conn
            .del(&key)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache delete error: {e}")))?;

        Ok(deleted > 0)
    }

    async fn flush(&self) -> Result<(), FrameworkError> {
        let mut conn = self.conn.clone();

        // Use KEYS to find and delete all keys with our prefix
        // Note: KEYS is O(N) and should be used carefully in production
        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache flush scan error: {e}")))?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys)
                .await
                .map_err(|e| FrameworkError::internal(format!("Cache flush delete error: {e}")))?;
        }

        Ok(())
    }

    async fn increment(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let value: i64 = conn
            .incr(&key, amount)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache increment error: {e}")))?;

        Ok(value)
    }

    async fn decrement(&self, key: &str, amount: i64) -> Result<i64, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let value: i64 = conn
            .decr(&key, amount)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache decrement error: {e}")))?;

        Ok(value)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, FrameworkError> {
        let mut conn = self.conn.clone();
        let key = self.prefixed_key(key);

        let result: bool = conn
            .expire(&key, ttl.as_secs() as i64)
            .await
            .map_err(|e| FrameworkError::internal(format!("Cache expire error: {e}")))?;

        Ok(result)
    }
}

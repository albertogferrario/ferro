//! Redis cache store.

use crate::cache::CacheStore;
use crate::error::Error;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::time::Duration;
use tracing::debug;

/// Redis-backed cache store.
pub struct RedisStore {
    client: ConnectionManager,
}

impl RedisStore {
    /// Create a new Redis store.
    pub async fn new(url: &str) -> Result<Self, Error> {
        let client = redis::Client::open(url).map_err(|e| Error::connection(e.to_string()))?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| Error::connection(e.to_string()))?;

        debug!("Connected to Redis at {}", url);

        Ok(Self { client: manager })
    }
}

#[async_trait]
impl CacheStore for RedisStore {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let mut conn = self.client.clone();
        let value: Option<Vec<u8>> = conn.get(key).await?;
        Ok(value)
    }

    async fn put_raw(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<(), Error> {
        let mut conn = self.client.clone();
        let seconds = ttl.as_secs() as i64;

        if seconds > 0 {
            conn.set_ex::<_, _, ()>(key, value, seconds as u64).await?;
        } else {
            conn.set::<_, _, ()>(key, value).await?;
        }

        Ok(())
    }

    async fn has(&self, key: &str) -> Result<bool, Error> {
        let mut conn = self.client.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    async fn forget(&self, key: &str) -> Result<bool, Error> {
        let mut conn = self.client.clone();
        let deleted: i64 = conn.del(key).await?;
        Ok(deleted > 0)
    }

    async fn flush(&self) -> Result<(), Error> {
        let mut conn = self.client.clone();
        redis::cmd("FLUSHDB").query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn increment(&self, key: &str, value: i64) -> Result<i64, Error> {
        let mut conn = self.client.clone();
        let result: i64 = conn.incr(key, value).await?;
        Ok(result)
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64, Error> {
        let mut conn = self.client.clone();
        let result: i64 = conn.decr(key, value).await?;
        Ok(result)
    }

    async fn tag_add(&self, tag: &str, key: &str) -> Result<(), Error> {
        let mut conn = self.client.clone();
        conn.sadd::<_, _, ()>(tag, key).await?;
        Ok(())
    }

    async fn tag_members(&self, tag: &str) -> Result<Vec<String>, Error> {
        let mut conn = self.client.clone();
        let members: Vec<String> = conn.smembers(tag).await?;
        Ok(members)
    }

    async fn tag_flush(&self, tag: &str) -> Result<(), Error> {
        let mut conn = self.client.clone();

        // Get all keys in the tag set
        let keys: Vec<String> = conn.smembers(tag).await?;

        // Delete all keys
        if !keys.is_empty() {
            for key in &keys {
                conn.del::<_, ()>(key).await?;
            }
        }

        // Delete the tag set itself
        conn.del::<_, ()>(tag).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Redis tests require a running Redis instance
    // These are integration tests that should be run separately
}

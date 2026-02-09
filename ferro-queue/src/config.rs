//! Queue configuration.

use std::env;
use std::time::Duration;

/// Queue system configuration.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Redis connection URL.
    pub redis_url: String,
    /// Default queue name.
    pub default_queue: String,
    /// Prefix for queue keys in Redis.
    pub prefix: String,
    /// How long to block waiting for jobs (in seconds).
    pub block_timeout: Duration,
    /// Maximum number of concurrent jobs per worker.
    pub max_concurrent_jobs: usize,
    /// How often to check for delayed jobs.
    pub delayed_job_poll_interval: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_queue: "default".to_string(),
            prefix: "ferro_queue".to_string(),
            block_timeout: Duration::from_secs(5),
            max_concurrent_jobs: 10,
            delayed_job_poll_interval: Duration::from_secs(1),
        }
    }
}

impl QueueConfig {
    /// Create a new configuration with a Redis URL.
    pub fn new(redis_url: impl Into<String>) -> Self {
        Self {
            redis_url: redis_url.into(),
            ..Default::default()
        }
    }

    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `QUEUE_CONNECTION`: "sync" or "redis" (defaults to "sync")
    /// - `QUEUE_DEFAULT`: Default queue name (defaults to "default")
    /// - `QUEUE_PREFIX`: Key prefix in Redis (defaults to "ferro_queue")
    /// - `QUEUE_BLOCK_TIMEOUT`: Seconds to block waiting for jobs (defaults to 5)
    /// - `QUEUE_MAX_CONCURRENT`: Max concurrent jobs per worker (defaults to 10)
    /// - `REDIS_URL`: Full Redis URL (takes precedence if set)
    /// - `REDIS_HOST`: Redis host (defaults to "127.0.0.1")
    /// - `REDIS_PORT`: Redis port (defaults to 6379)
    /// - `REDIS_PASSWORD`: Redis password (optional)
    /// - `REDIS_DATABASE`: Redis database number (defaults to 0)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_queue::QueueConfig;
    ///
    /// // In bootstrap.rs
    /// let config = QueueConfig::from_env();
    /// Queue::init(config).await?;
    /// ```
    pub fn from_env() -> Self {
        let redis_url = Self::build_redis_url();

        Self {
            redis_url,
            default_queue: env::var("QUEUE_DEFAULT").unwrap_or_else(|_| "default".to_string()),
            prefix: env::var("QUEUE_PREFIX").unwrap_or_else(|_| "ferro_queue".to_string()),
            block_timeout: Duration::from_secs(
                env::var("QUEUE_BLOCK_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
            ),
            max_concurrent_jobs: env::var("QUEUE_MAX_CONCURRENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            delayed_job_poll_interval: Duration::from_secs(1),
        }
    }

    /// Build Redis URL from environment variables.
    fn build_redis_url() -> String {
        // Check for explicit REDIS_URL first
        if let Ok(url) = env::var("REDIS_URL") {
            return url;
        }

        let host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let password = env::var("REDIS_PASSWORD").ok().filter(|p| !p.is_empty());
        let database = env::var("REDIS_DATABASE").unwrap_or_else(|_| "0".to_string());

        match password {
            Some(pass) => format!("redis://:{}@{}:{}/{}", pass, host, port, database),
            None => format!("redis://{}:{}/{}", host, port, database),
        }
    }

    /// Check if sync queue mode is configured.
    ///
    /// When QUEUE_CONNECTION=sync, jobs are processed immediately instead
    /// of being pushed to Redis.
    pub fn is_sync_mode() -> bool {
        env::var("QUEUE_CONNECTION")
            .map(|v| v.to_lowercase() == "sync")
            .unwrap_or(true) // Default to sync for development
    }

    /// Set the default queue name.
    pub fn default_queue(mut self, queue: impl Into<String>) -> Self {
        self.default_queue = queue.into();
        self
    }

    /// Set the key prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set the block timeout.
    pub fn block_timeout(mut self, timeout: Duration) -> Self {
        self.block_timeout = timeout;
        self
    }

    /// Set max concurrent jobs.
    pub fn max_concurrent_jobs(mut self, count: usize) -> Self {
        self.max_concurrent_jobs = count;
        self
    }

    /// Get the Redis key for a queue.
    pub fn queue_key(&self, queue: &str) -> String {
        format!("{}:{}", self.prefix, queue)
    }

    /// Get the Redis key for delayed jobs.
    pub fn delayed_key(&self, queue: &str) -> String {
        format!("{}:{}:delayed", self.prefix, queue)
    }

    /// Get the Redis key for reserved jobs.
    pub fn reserved_key(&self, queue: &str) -> String {
        format!("{}:{}:reserved", self.prefix, queue)
    }

    /// Get the Redis key for failed jobs.
    pub fn failed_key(&self) -> String {
        format!("{}:failed", self.prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Guard that removes environment variables on drop, ensuring cleanup even on panic.
    struct EnvGuard {
        vars: Vec<String>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            env::set_var(key, value);
            Self {
                vars: vec![key.to_string()],
            }
        }

        fn also_set(&mut self, key: &str, value: &str) {
            env::set_var(key, value);
            self.vars.push(key.to_string());
        }

        fn also_remove(&mut self, key: &str) {
            env::remove_var(key);
            self.vars.push(key.to_string());
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                env::remove_var(var);
            }
        }
    }

    #[test]
    fn test_default_config() {
        let config = QueueConfig::default();
        assert_eq!(config.default_queue, "default");
        assert_eq!(config.prefix, "ferro_queue");
    }

    #[test]
    fn test_queue_key() {
        let config = QueueConfig::default();
        assert_eq!(config.queue_key("emails"), "ferro_queue:emails");
        assert_eq!(config.delayed_key("emails"), "ferro_queue:emails:delayed");
    }

    #[test]
    fn test_builder_pattern() {
        let config = QueueConfig::new("redis://localhost:6380")
            .default_queue("high-priority")
            .prefix("myapp")
            .max_concurrent_jobs(5);

        assert_eq!(config.redis_url, "redis://localhost:6380");
        assert_eq!(config.default_queue, "high-priority");
        assert_eq!(config.prefix, "myapp");
        assert_eq!(config.max_concurrent_jobs, 5);
    }

    #[test]
    #[serial]
    fn test_from_env_defaults() {
        let mut guard = EnvGuard { vars: Vec::new() };
        // Clear any existing env vars (guard tracks them for cleanup)
        guard.also_remove("QUEUE_DEFAULT");
        guard.also_remove("QUEUE_PREFIX");
        guard.also_remove("QUEUE_BLOCK_TIMEOUT");
        guard.also_remove("QUEUE_MAX_CONCURRENT");
        guard.also_remove("REDIS_URL");
        guard.also_remove("REDIS_HOST");
        guard.also_remove("REDIS_PORT");
        guard.also_remove("REDIS_PASSWORD");
        guard.also_remove("REDIS_DATABASE");

        let config = QueueConfig::from_env();
        assert_eq!(config.default_queue, "default");
        assert_eq!(config.prefix, "ferro_queue");
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379/0");
        assert_eq!(config.max_concurrent_jobs, 10);
    }

    #[test]
    #[serial]
    fn test_from_env_with_redis_url() {
        let _guard = EnvGuard::set("REDIS_URL", "redis://custom:6380/5");
        let config = QueueConfig::from_env();
        assert_eq!(config.redis_url, "redis://custom:6380/5");
    }

    #[test]
    #[serial]
    fn test_build_redis_url_with_password() {
        let mut guard = EnvGuard { vars: Vec::new() };
        guard.also_remove("REDIS_URL");
        guard.also_set("REDIS_HOST", "redis.example.com");
        guard.also_set("REDIS_PORT", "6380");
        guard.also_set("REDIS_PASSWORD", "secret123");
        guard.also_set("REDIS_DATABASE", "3");

        let url = QueueConfig::build_redis_url();
        assert_eq!(url, "redis://:secret123@redis.example.com:6380/3");
    }

    #[test]
    #[serial]
    fn test_is_sync_mode() {
        let mut guard = EnvGuard { vars: Vec::new() };
        guard.also_remove("QUEUE_CONNECTION");
        assert!(QueueConfig::is_sync_mode()); // default is sync

        guard.also_set("QUEUE_CONNECTION", "sync");
        assert!(QueueConfig::is_sync_mode());

        env::set_var("QUEUE_CONNECTION", "redis");
        assert!(!QueueConfig::is_sync_mode());

        env::set_var("QUEUE_CONNECTION", "SYNC");
        assert!(QueueConfig::is_sync_mode()); // case insensitive
    }
}

//! Queue configuration.

use std::env;
use std::time::Duration;

/// Queue system configuration.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Default queue name.
    pub default_queue: String,
    /// Maximum number of concurrent jobs per worker.
    pub max_concurrent_jobs: usize,
    /// How long to sleep between poll cycles when no jobs are available.
    pub sleep_duration: Duration,
    /// How long a claimed job stays invisible before being reclaimed by the reaper.
    pub visibility_timeout: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            default_queue: "default".to_string(),
            max_concurrent_jobs: 10,
            sleep_duration: Duration::from_millis(500),
            visibility_timeout: Duration::from_secs(300),
        }
    }
}

impl QueueConfig {
    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `QUEUE_CONNECTION`: "sync" disables the DB worker loop (defaults to "sync")
    /// - `QUEUE_DEFAULT`: Default queue name (defaults to "default")
    /// - `QUEUE_MAX_CONCURRENT`: Max concurrent jobs per worker (defaults to 10)
    /// - `QUEUE_VISIBILITY_TIMEOUT_SECS`: Seconds before claimed jobs are reclaimed (defaults to 300)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_queue::QueueConfig;
    ///
    /// // In bootstrap.rs
    /// let config = QueueConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        Self {
            default_queue: env::var("QUEUE_DEFAULT").unwrap_or_else(|_| "default".to_string()),
            max_concurrent_jobs: env::var("QUEUE_MAX_CONCURRENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            sleep_duration: Duration::from_millis(500),
            visibility_timeout: Duration::from_secs(
                env::var("QUEUE_VISIBILITY_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
            ),
        }
    }

    /// Check if sync queue mode is configured.
    ///
    /// When `QUEUE_CONNECTION=sync`, jobs are processed immediately in the
    /// calling process instead of being written to the database.
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

    /// Set max concurrent jobs.
    pub fn max_concurrent_jobs(mut self, count: usize) -> Self {
        self.max_concurrent_jobs = count;
        self
    }

    /// Set the sleep duration between poll cycles.
    pub fn with_sleep_duration(mut self, d: Duration) -> Self {
        self.sleep_duration = d;
        self
    }

    /// Set the visibility timeout for claimed jobs.
    pub fn with_visibility_timeout(mut self, d: Duration) -> Self {
        self.visibility_timeout = d;
        self
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
        fn new() -> Self {
            Self { vars: Vec::new() }
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
        assert_eq!(config.max_concurrent_jobs, 10);
        assert_eq!(config.sleep_duration, Duration::from_millis(500));
        assert_eq!(config.visibility_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_builder_pattern() {
        let config = QueueConfig::default()
            .default_queue("high-priority")
            .max_concurrent_jobs(5)
            .with_visibility_timeout(Duration::from_secs(60));

        assert_eq!(config.default_queue, "high-priority");
        assert_eq!(config.max_concurrent_jobs, 5);
        assert_eq!(config.visibility_timeout, Duration::from_secs(60));
    }

    #[test]
    fn visibility_timeout_default_is_five_minutes() {
        let config = QueueConfig::default();
        assert_eq!(config.visibility_timeout, Duration::from_secs(300));
    }

    #[test]
    #[serial]
    fn test_from_env_defaults() {
        let mut guard = EnvGuard::new();
        guard.also_remove("QUEUE_DEFAULT");
        guard.also_remove("QUEUE_MAX_CONCURRENT");
        guard.also_remove("QUEUE_VISIBILITY_TIMEOUT_SECS");

        let config = QueueConfig::from_env();
        assert_eq!(config.default_queue, "default");
        assert_eq!(config.max_concurrent_jobs, 10);
        assert_eq!(config.visibility_timeout, Duration::from_secs(300));
    }

    #[test]
    #[serial]
    fn test_from_env_visibility_timeout() {
        let mut guard = EnvGuard::new();
        guard.also_set("QUEUE_VISIBILITY_TIMEOUT_SECS", "120");

        let config = QueueConfig::from_env();
        assert_eq!(config.visibility_timeout, Duration::from_secs(120));
    }

    #[test]
    #[serial]
    fn test_is_sync_mode() {
        let mut guard = EnvGuard::new();
        guard.also_remove("QUEUE_CONNECTION");
        assert!(QueueConfig::is_sync_mode()); // default is sync

        env::set_var("QUEUE_CONNECTION", "sync");
        assert!(QueueConfig::is_sync_mode());

        env::set_var("QUEUE_CONNECTION", "db");
        assert!(!QueueConfig::is_sync_mode());

        env::set_var("QUEUE_CONNECTION", "SYNC");
        assert!(QueueConfig::is_sync_mode()); // case insensitive
    }
}

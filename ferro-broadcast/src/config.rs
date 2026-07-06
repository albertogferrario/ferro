//! Configuration for the broadcast system.

use std::env;
use std::time::Duration;

/// Configuration for the broadcaster.
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Maximum subscribers per channel (0 = unlimited).
    pub max_subscribers_per_channel: usize,
    /// Maximum channels (0 = unlimited).
    pub max_channels: usize,
    /// Heartbeat interval for WebSocket connections.
    pub heartbeat_interval: Duration,
    /// Client timeout (disconnect if no activity).
    pub client_timeout: Duration,
    /// Whether to allow client-to-client messages (whisper).
    pub allow_client_events: bool,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            max_subscribers_per_channel: 0,
            max_channels: 0,
            heartbeat_interval: Duration::from_secs(30),
            client_timeout: Duration::from_secs(60),
            allow_client_events: true,
        }
    }
}

impl BroadcastConfig {
    /// Create a new broadcast config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `BROADCAST_MAX_SUBSCRIBERS`: Max subscribers per channel (default: unlimited)
    /// - `BROADCAST_MAX_CHANNELS`: Max total channels (default: unlimited)
    /// - `BROADCAST_HEARTBEAT_INTERVAL`: Heartbeat interval in seconds (default: 30)
    /// - `BROADCAST_CLIENT_TIMEOUT`: Client timeout in seconds (default: 60)
    /// - `BROADCAST_ALLOW_CLIENT_EVENTS`: Allow whisper messages (default: true)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_broadcast::BroadcastConfig;
    ///
    /// let config = BroadcastConfig::from_env();
    /// let broadcaster = Broadcaster::with_config(config);
    /// ```
    pub fn from_env() -> Self {
        Self {
            max_subscribers_per_channel: env::var("BROADCAST_MAX_SUBSCRIBERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            max_channels: env::var("BROADCAST_MAX_CHANNELS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            heartbeat_interval: Duration::from_secs(
                env::var("BROADCAST_HEARTBEAT_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
            ),
            client_timeout: Duration::from_secs(
                env::var("BROADCAST_CLIENT_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
            ),
            allow_client_events: env::var("BROADCAST_ALLOW_CLIENT_EVENTS")
                .map(|v| v.to_lowercase() != "false" && v != "0")
                .unwrap_or(true),
        }
    }

    /// Set maximum subscribers per channel.
    pub fn max_subscribers_per_channel(mut self, max: usize) -> Self {
        self.max_subscribers_per_channel = max;
        self
    }

    /// Set maximum channels.
    pub fn max_channels(mut self, max: usize) -> Self {
        self.max_channels = max;
        self
    }

    /// Set heartbeat interval.
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set client timeout.
    pub fn client_timeout(mut self, timeout: Duration) -> Self {
        self.client_timeout = timeout;
        self
    }

    /// Set whether client events (whisper) are allowed.
    pub fn allow_client_events(mut self, allow: bool) -> Self {
        self.allow_client_events = allow;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_config_defaults() {
        let config = BroadcastConfig::default();
        assert_eq!(config.max_subscribers_per_channel, 0);
        assert_eq!(config.max_channels, 0);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.client_timeout, Duration::from_secs(60));
        assert!(config.allow_client_events);
    }

    #[test]
    fn test_broadcast_config_builder() {
        let config = BroadcastConfig::new()
            .max_subscribers_per_channel(100)
            .max_channels(50)
            .heartbeat_interval(Duration::from_secs(15))
            .allow_client_events(false);

        assert_eq!(config.max_subscribers_per_channel, 100);
        assert_eq!(config.max_channels, 50);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(15));
        assert!(!config.allow_client_events);
    }
}

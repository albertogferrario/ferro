//! CDN cache-invalidation primitives for ferro-storage.
//!
//! This module provides the [`PurgeApi`] trait abstracting CDN cache invalidation, and the
//! batteries-included [`DoSpacesCdn`] adapter for DigitalOcean Spaces CDN.
//!
//! # Relative paths
//!
//! The [`PurgeApi`] trait uses relative paths (e.g. `"index.html"`, `"assets/*"`).
//! Implementations handle batching and rate limiting internally. Do NOT pass full CDN
//! URLs — paths are relative to the CDN endpoint's origin.
//!
//! # DigitalOcean Spaces CDN adapter
//!
//! [`DoSpacesCdn`] encapsulates the operationally fiddly parts of the DO CDN purge API:
//! - Batching: ≤50 files per `DELETE /v2/cdn/endpoints/{id}/cache` request.
//! - Rate limiting: an internal sliding-window throttle enforces ≤5 requests per 10 s.
//! - Wildcard slot accounting: a wildcard path (e.g. `"dir/*"`) counts as 1 file slot.
//! - Missing endpoint id: `purge()` is a logged no-op returning `Ok(())` (no HTTP request).
//!
//! # Security
//!
//! The `DIGITALOCEAN_ACCESS_TOKEN` is never logged or printed. [`DoSpacesCdnConfig`]
//! implements a hand-written `Debug` that redacts the token field.

use crate::Error;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

const DO_CDN_API_BASE: &str = "https://api.digitalocean.com";
const BATCH_SIZE: usize = 50;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);
const RATE_LIMIT_MAX: usize = 5;

/// Cache invalidation abstraction for CDN backends.
///
/// Paths are relative (e.g. `"index.html"`, `"assets/*"`); implementations handle
/// batching and rate limiting internally.
#[async_trait]
pub trait PurgeApi: Send + Sync {
    /// Purge cached content at the given relative paths.
    ///
    /// - An empty slice returns `Ok(())` with zero HTTP requests.
    /// - Implementations handle batching, rate limiting, and wildcard slots internally.
    async fn purge(&self, paths: &[String]) -> Result<(), Error>;
}

/// Configuration for the DigitalOcean Spaces CDN adapter.
///
/// Read from environment via [`DoSpacesCdnConfig::from_env()`].
///
/// # Token security
///
/// `api_token` is never logged. The `Debug` implementation prints `<redacted>` for this field.
#[derive(Clone)]
pub struct DoSpacesCdnConfig {
    /// DO CDN endpoint id (`DO_SPACES_CDN_ID`). `None` → `purge()` is a logged no-op.
    pub endpoint_id: Option<String>,
    /// DO API token (`DIGITALOCEAN_ACCESS_TOKEN`). Never logged.
    pub api_token: String,
    /// API base URL override for tests only. Production uses the DO API base constant.
    pub(crate) api_base: Option<String>,
}

impl std::fmt::Debug for DoSpacesCdnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoSpacesCdnConfig")
            .field("endpoint_id", &self.endpoint_id)
            .field("api_token", &"<redacted>")
            .field("api_base", &self.api_base)
            .finish()
    }
}

impl DoSpacesCdnConfig {
    /// Read config from environment.
    ///
    /// - `DO_SPACES_CDN_ID` — optional. When absent, `purge()` is a logged no-op.
    /// - `DIGITALOCEAN_ACCESS_TOKEN` — required when endpoint id is set; otherwise unused.
    pub fn from_env() -> Self {
        Self {
            endpoint_id: std::env::var("DO_SPACES_CDN_ID").ok(),
            api_token: std::env::var("DIGITALOCEAN_ACCESS_TOKEN").unwrap_or_default(),
            api_base: None,
        }
    }
}

/// DigitalOcean Spaces CDN adapter implementing [`PurgeApi`].
///
/// Encapsulates the DO CDN purge API: `DELETE /v2/cdn/endpoints/{id}/cache` with
/// `{"files": [...]}` body, ≤50-file batching, and a 5-req/10s internal throttle.
pub struct DoSpacesCdn {
    config: DoSpacesCdnConfig,
    client: reqwest::Client,
    request_times: Mutex<VecDeque<Instant>>,
}

impl DoSpacesCdn {
    /// Construct from config. Builds a single `reqwest::Client` shared across all requests.
    pub fn new(config: DoSpacesCdnConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            request_times: Mutex::new(VecDeque::new()),
        }
    }

    fn api_base(&self) -> &str {
        self.config.api_base.as_deref().unwrap_or(DO_CDN_API_BASE)
    }

    /// Sliding-window rate limiter: ensures ≤ RATE_LIMIT_MAX requests per RATE_LIMIT_WINDOW.
    ///
    /// Uses `tokio::sync::Mutex` (not `std::sync::Mutex`) so the lock can be held across
    /// the `.await` in the sleep path without deadlocking.
    async fn throttle(&self) {
        let mut times = self.request_times.lock().await;
        let now = Instant::now();
        // Evict entries older than the window
        while times
            .front()
            .map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW)
            .unwrap_or(false)
        {
            times.pop_front();
        }
        if times.len() >= RATE_LIMIT_MAX {
            // Sleep until the oldest entry falls out of the window
            let oldest = *times.front().unwrap();
            let sleep_for = RATE_LIMIT_WINDOW - now.duration_since(oldest);
            drop(times);
            tokio::time::sleep(sleep_for).await;
            // Re-acquire and re-evict after sleeping
            let mut times = self.request_times.lock().await;
            let now = Instant::now();
            while times
                .front()
                .map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW)
                .unwrap_or(false)
            {
                times.pop_front();
            }
            times.push_back(Instant::now());
        } else {
            times.push_back(now);
        }
    }
}

#[async_trait]
impl PurgeApi for DoSpacesCdn {
    async fn purge(&self, paths: &[String]) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }
        let Some(id) = &self.config.endpoint_id else {
            tracing::info!("DO_SPACES_CDN_ID not set — CDN purge is a no-op");
            return Ok(());
        };
        if self.config.api_token.is_empty() {
            return Err(Error::cdn(
                "DIGITALOCEAN_ACCESS_TOKEN not set — cannot purge CDN cache",
            ));
        }
        let url = format!("{}/v2/cdn/endpoints/{}/cache", self.api_base(), id);
        let mut batches = 0usize;
        for chunk in paths.chunks(BATCH_SIZE) {
            self.throttle().await;
            let resp = self
                .client
                .delete(&url)
                .bearer_auth(&self.config.api_token)
                .json(&serde_json::json!({ "files": chunk }))
                .send()
                .await
                .map_err(|e| Error::cdn(e.to_string()))?;
            if resp.status().as_u16() != 204 {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                return Err(Error::cdn(format!("DO CDN purge status {status}: {body}")));
            }
            batches += 1;
        }
        tracing::info!("purged {} paths in {} request(s)", paths.len(), batches);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for Task 1 acceptance criteria (structural/behavioral verification without HTTP).
    // HTTP-backed tests are in Task 2 (wiremock suite).

    #[test]
    fn debug_does_not_contain_token() {
        let cfg = DoSpacesCdnConfig {
            endpoint_id: Some("ep-123".into()),
            api_token: "secret-token-abc".into(),
            api_base: None,
        };
        let dbg = format!("{cfg:?}");
        assert!(
            !dbg.contains("secret-token-abc"),
            "Debug output must not contain the token: {dbg}"
        );
        assert!(
            dbg.contains("<redacted>"),
            "Debug output must show <redacted>: {dbg}"
        );
    }

    #[tokio::test]
    async fn purge_empty_slice_returns_ok_no_panic() {
        let cfg = DoSpacesCdnConfig {
            endpoint_id: Some("ep-123".into()),
            api_token: "tok".into(),
            api_base: None,
        };
        let purger = DoSpacesCdn::new(cfg);
        // Should return Ok(()) immediately, no network calls
        purger.purge(&[]).await.unwrap();
    }

    #[tokio::test]
    async fn missing_token_with_id_set_returns_error() {
        let cfg = DoSpacesCdnConfig {
            endpoint_id: Some("ep-123".into()),
            api_token: "".into(),
            api_base: Some("http://127.0.0.1:9".into()), // unreachable — no request should be made
        };
        let purger = DoSpacesCdn::new(cfg);
        let result = purger.purge(&["index.html".into()]).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("DIGITALOCEAN_ACCESS_TOKEN"),
            "Error must mention the token env var: {msg}"
        );
    }
}

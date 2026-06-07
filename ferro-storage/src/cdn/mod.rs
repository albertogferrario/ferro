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

#[cfg(feature = "cdn-bunny")]
pub mod bunny;
#[cfg(feature = "cdn-bunny")]
pub use bunny::{BunnyCdn, BunnyCdnConfig};

#[cfg(feature = "cdn-cloudflare")]
pub mod cloudflare;
#[cfg(feature = "cdn-cloudflare")]
pub use cloudflare::{CloudflareCdn, CloudflareCdnConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Build a DoSpacesCdnConfig pointing at the wiremock server URI.
    fn cfg(server_uri: &str, id: Option<&str>, token: &str) -> DoSpacesCdnConfig {
        DoSpacesCdnConfig {
            endpoint_id: id.map(String::from),
            api_token: token.to_string(),
            api_base: Some(server_uri.to_string()),
        }
    }

    // --- Structural / no-HTTP tests ---

    #[test]
    fn debug_does_not_contain_token() {
        let config = DoSpacesCdnConfig {
            endpoint_id: Some("ep-123".into()),
            api_token: "secret-token-abc".into(),
            api_base: None,
        };
        let dbg = format!("{config:?}");
        assert!(
            !dbg.contains("secret-token-abc"),
            "Debug output must not contain the token: {dbg}"
        );
        assert!(
            dbg.contains("<redacted>"),
            "Debug output must show <redacted>: {dbg}"
        );
    }

    // --- wiremock-backed tests ---

    /// 1 path → exactly 1 DELETE to /v2/cdn/endpoints/test-id/cache, correct auth, correct body.
    #[tokio::test]
    async fn do_adapter_request_shape() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
            .and(header("Authorization", "Bearer test-token"))
            .and(body_json(serde_json::json!({ "files": ["index.html"] })))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        purger.purge(&["index.html".to_string()]).await.unwrap();
        // wiremock verifies .expect(1) on drop
    }

    /// 55 paths → exactly 2 DELETE requests (chunks of 50 + 5).
    #[tokio::test]
    async fn do_adapter_batches_over_50() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(2)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        let paths: Vec<String> = (0..55).map(|i| format!("file{i}.html")).collect();
        purger.purge(&paths).await.unwrap();
    }

    /// 50 plain paths + 1 wildcard "dir/*" = 51 elements → 2 requests.
    /// Wildcard counts as 1 slot, not expanded.
    #[tokio::test]
    async fn do_adapter_wildcard_slot() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(2)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        let mut paths: Vec<String> = (0..50).map(|i| format!("file{i}.html")).collect();
        paths.push("dir/*".to_string()); // 51 total → 2 requests
        purger.purge(&paths).await.unwrap();
    }

    /// Missing endpoint id → purge() returns Ok(()), zero HTTP requests.
    #[tokio::test]
    async fn do_adapter_noop_missing_id() {
        let server = MockServer::start().await;
        // Expect zero requests — wiremock will fail the test if any arrive
        Mock::given(method("DELETE"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), None, "test-token"));
        purger.purge(&["a".to_string()]).await.unwrap();
    }

    /// purge(&[]) → Ok(()), zero HTTP requests.
    #[tokio::test]
    async fn purge_empty_noop() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        purger.purge(&[]).await.unwrap();
    }

    /// Non-204 response → Err(Error::Cdn(...)) whose message contains the status code.
    #[tokio::test]
    async fn do_adapter_error_on_non_204() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        let err = purger.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(
            err.to_string().contains("403"),
            "Error must contain status 403: {err}"
        );
    }

    /// Empty token with id set → Err(Error::Cdn) mentioning DIGITALOCEAN_ACCESS_TOKEN.
    /// Zero HTTP requests are made.
    #[tokio::test]
    async fn do_adapter_missing_token_errors() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), ""));
        let err = purger.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(
            err.to_string().contains("DIGITALOCEAN_ACCESS_TOKEN"),
            "Error must mention the token env var: {err}"
        );
    }

    /// 300 paths → 6 chunks. The 6th request must wait for the rate-limit window.
    /// Total elapsed must be >= ~9 s (RATE_LIMIT_WINDOW - 1s tolerance).
    ///
    /// NOTE: This test takes ~10 s due to the real rate-limit sleep. This is intentional —
    /// it asserts that the throttle actually serializes under the 5-req/10s window without
    /// relying on tokio::time::pause (which could give a false pass).
    #[tokio::test]
    async fn do_adapter_throttle_serializes() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/test-id/cache"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(6)
            .mount(&server)
            .await;

        let purger = DoSpacesCdn::new(cfg(&server.uri(), Some("test-id"), "test-token"));
        // 300 paths → 6 chunks of 50 — the 6th must wait for the window
        let paths: Vec<String> = (0..300).map(|i| format!("file{i}.html")).collect();

        let start = std::time::Instant::now();
        purger.purge(&paths).await.unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_secs(9),
            "Throttle should have held the 6th request for ~10s; elapsed: {elapsed:?}"
        );
    }
}

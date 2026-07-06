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

use crate::env_helpers::env_with_fallback;
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
    /// Reads the provider-agnostic Phase 205 names first, with fallback to the
    /// legacy DO-specific names so existing deployments keep working without
    /// re-keying their secrets:
    /// - `CDN_PURGE_ZONE` (modern) → falls back to `DO_SPACES_CDN_ID` (legacy).
    ///   Optional. When neither is set, `purge()` is a logged no-op.
    /// - `CDN_PURGE_TOKEN` (modern) → falls back to `DIGITALOCEAN_ACCESS_TOKEN`
    ///   (legacy). Required when endpoint id is set; otherwise unused.
    pub fn from_env() -> Self {
        Self {
            endpoint_id: crate::env_helpers::env_with_fallback(
                "CDN_PURGE_ZONE",
                &["DO_SPACES_CDN_ID"],
            ),
            api_token: crate::env_helpers::env_with_fallback(
                "CDN_PURGE_TOKEN",
                &["DIGITALOCEAN_ACCESS_TOKEN"],
            )
            .unwrap_or_default(),
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
    /// Loops until a free slot is confirmed while holding the lock, so concurrent callers
    /// cannot both observe `len < RATE_LIMIT_MAX` and both proceed. The lock is never held
    /// across the `.await` sleep to avoid deadlocking other waiters.
    async fn throttle(&self) {
        loop {
            let mut times = self.request_times.lock().await;
            let now = Instant::now();
            // Evict entries older than the window.
            while times
                .front()
                .map(|t| now.duration_since(*t) >= RATE_LIMIT_WINDOW)
                .unwrap_or(false)
            {
                times.pop_front();
            }
            if times.len() < RATE_LIMIT_MAX {
                // Slot available — record this request and proceed.
                times.push_back(Instant::now());
                return;
            }
            // Window full: compute sleep duration, drop the lock, then re-check.
            let oldest = *times.front().unwrap();
            let sleep_for = RATE_LIMIT_WINDOW - now.duration_since(oldest);
            drop(times);
            tokio::time::sleep(sleep_for).await;
            // Loop back to re-acquire and re-check under the lock.
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

/// CDN provider selected for cache invalidation (`CDN_PROVIDER`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CdnProvider {
    /// No purge provider — `purge()` is an explicit logged no-op.
    #[default]
    None,
    /// DigitalOcean Spaces CDN (always available — no cargo feature).
    DigitalOcean,
    /// Bunny CDN (requires the `cdn-bunny` feature).
    Bunny,
    /// Cloudflare (requires the `cdn-cloudflare` feature).
    Cloudflare,
}

impl CdnProvider {
    /// Parse case-insensitively; unknown value → `Error::CdnInvalidProvider`.
    pub fn from_str_ci(s: &str) -> Result<Self, Error> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "digitalocean" => Ok(Self::DigitalOcean),
            "bunny" => Ok(Self::Bunny),
            "cloudflare" => Ok(Self::Cloudflare),
            other => Err(Error::cdn_invalid_provider(other)),
        }
    }
}

/// Provider-agnostic CDN configuration. Read via [`Config::from_env`];
/// build the active purge adapter with [`Config::build_purge_api`].
///
/// # Token security
/// `purge_token` is never logged. `Debug` prints `<redacted>` for it.
#[derive(Clone, Default)]
pub struct Config {
    /// CDN base URL fronting the bucket (`CDN_URL`). Drives `Disk::cdn_url()`.
    pub url: Option<String>,
    /// Selected provider for cache invalidation (`CDN_PROVIDER`).
    pub provider: CdnProvider,
    /// Provider API credential (`CDN_PURGE_TOKEN`). Never logged.
    pub purge_token: Option<String>,
    /// Provider-specific zone or endpoint id (`CDN_PURGE_ZONE`).
    pub purge_zone: Option<String>,
    /// Set when `CDN_PROVIDER` is explicitly provided but fails to parse.
    /// `build_purge_api` converts this into a boot `Error` (D-03 / SC-5b).
    pub provider_error: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("cdn::Config")
            .field("url", &self.url)
            .field("provider", &self.provider)
            .field("purge_token", &"<redacted>")
            .field("purge_zone", &self.purge_zone)
            .field("provider_error", &self.provider_error)
            .finish()
    }
}

impl Config {
    /// Read CDN config from environment: the quartet primary + per-var legacy fallbacks.
    ///
    /// Provider resolution runs first; token/zone aliases are then scoped to the resolved
    /// provider so that a stray credential from a different provider's legacy cluster cannot
    /// win the fallback race.
    ///
    /// If `CDN_PROVIDER` is set to an unrecognized value the struct is returned with
    /// `provider_error` set; the parse failure is surfaced as a boot `Error` by
    /// [`Config::build_purge_api`].
    pub fn from_env() -> Self {
        // 1. Resolve the CDN display URL (provider-independent).
        let url = env_with_fallback("CDN_URL", &["AWS_CDN_URL", "CF_CDN_URL", "BUNNY_CDN_URL"]);

        // 2. Resolve provider first so token/zone aliases can be scoped to it.
        let mut provider_error: Option<String> = None;
        let provider = if let Ok(val) = std::env::var("CDN_PROVIDER") {
            match CdnProvider::from_str_ci(&val) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("{e}; set CDN_PROVIDER to a valid value to enable purging");
                    provider_error = Some(val);
                    CdnProvider::None
                }
            }
        } else if std::env::var("DO_SPACES_CDN_ID").is_ok() {
            tracing::warn!("CDN_PROVIDER unset; inferred digitalocean from DO_SPACES_CDN_ID. Set CDN_PROVIDER=digitalocean to silence.");
            CdnProvider::DigitalOcean
        } else if std::env::var("CF_ZONE_ID").is_ok() {
            tracing::warn!("CDN_PROVIDER unset; inferred cloudflare from CF_ZONE_ID. Set CDN_PROVIDER=cloudflare to silence.");
            CdnProvider::Cloudflare
        } else if std::env::var("BUNNY_CDN_URL").is_ok()
            || std::env::var("BUNNY_ACCESS_KEY").is_ok()
        {
            tracing::warn!("CDN_PROVIDER unset; inferred bunny from BUNNY_CDN_URL/BUNNY_ACCESS_KEY. Set CDN_PROVIDER=bunny to silence.");
            CdnProvider::Bunny
        } else {
            CdnProvider::None
        };

        // 3. Resolve token/zone using only the aliases that belong to the resolved provider,
        //    preventing cross-provider credential contamination in mixed legacy deployments.
        let (purge_token, purge_zone) = match &provider {
            CdnProvider::DigitalOcean => (
                env_with_fallback("CDN_PURGE_TOKEN", &["DIGITALOCEAN_ACCESS_TOKEN"]),
                env_with_fallback("CDN_PURGE_ZONE", &["DO_SPACES_CDN_ID"]),
            ),
            CdnProvider::Cloudflare => (
                env_with_fallback("CDN_PURGE_TOKEN", &["CF_API_TOKEN"]),
                env_with_fallback("CDN_PURGE_ZONE", &["CF_ZONE_ID"]),
            ),
            CdnProvider::Bunny => (
                env_with_fallback("CDN_PURGE_TOKEN", &["BUNNY_ACCESS_KEY"]),
                None,
            ),
            CdnProvider::None => (
                env_with_fallback("CDN_PURGE_TOKEN", &[]),
                env_with_fallback("CDN_PURGE_ZONE", &[]),
            ),
        };

        Self {
            url,
            provider,
            purge_token,
            purge_zone,
            provider_error,
        }
    }

    /// Build the active purge adapter, or `Ok(None)` for provider `None`.
    ///
    /// Returns `Err(CdnInvalidProvider)` when `CDN_PROVIDER` was set to an unrecognized
    /// value during `from_env` (D-03 boot error, SC-5b).
    /// Returns `Err(CdnFeatureRequired)` if the selected provider's cargo feature is off.
    pub fn build_purge_api(&self) -> Result<Option<Box<dyn PurgeApi>>, Error> {
        // Surface an invalid CDN_PROVIDER value as a boot error (D-03 / SC-5b).
        if let Some(bad) = &self.provider_error {
            return Err(Error::cdn_invalid_provider(bad));
        }

        match &self.provider {
            CdnProvider::None => {
                tracing::info!("CDN_PROVIDER=none — purge is a no-op");
                Ok(None)
            }
            CdnProvider::DigitalOcean => {
                let cfg = DoSpacesCdnConfig {
                    endpoint_id: self.purge_zone.clone(),
                    api_token: self.purge_token.clone().unwrap_or_default(),
                    api_base: None,
                };
                Ok(Some(Box::new(DoSpacesCdn::new(cfg))))
            }
            CdnProvider::Bunny => {
                #[cfg(feature = "cdn-bunny")]
                {
                    let cfg = BunnyCdnConfig {
                        cdn_base_url: self.url.clone().unwrap_or_default(),
                        access_key: self.purge_token.clone().unwrap_or_default(),
                    };
                    Ok(Some(Box::new(BunnyCdn::new(cfg))))
                }
                #[cfg(not(feature = "cdn-bunny"))]
                {
                    Err(Error::cdn_feature_required("bunny", "cdn-bunny"))
                }
            }
            CdnProvider::Cloudflare => {
                #[cfg(feature = "cdn-cloudflare")]
                {
                    let cfg = CloudflareCdnConfig {
                        zone_id: self.purge_zone.clone().unwrap_or_default(),
                        api_token: self.purge_token.clone().unwrap_or_default(),
                        cdn_base_url: self.url.clone().unwrap_or_default(),
                    };
                    Ok(Some(Box::new(CloudflareCdn::new(cfg))))
                }
                #[cfg(not(feature = "cdn-cloudflare"))]
                {
                    Err(Error::cdn_feature_required("cloudflare", "cdn-cloudflare"))
                }
            }
        }
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

    // --- Config / CdnProvider unit tests ---

    #[test]
    fn cdn_config_debug_redacts_token() {
        let config = Config {
            purge_token: Some("secret-xyz".into()),
            ..Default::default()
        };
        let dbg = format!("{config:?}");
        assert!(
            !dbg.contains("secret-xyz"),
            "Debug output must not contain the token: {dbg}"
        );
        assert!(
            dbg.contains("<redacted>"),
            "Debug output must show <redacted>: {dbg}"
        );
    }

    #[test]
    fn cdn_invalid_provider() {
        let result = CdnProvider::from_str_ci("Fastly");
        assert!(result.is_err(), "Expected Err for unknown provider");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("none, digitalocean, bunny, cloudflare"),
            "Error must list valid values: {msg}"
        );
    }

    /// When CDN_PROVIDER is explicitly set to an invalid value, build_purge_api must return
    /// Err listing valid values (D-03 / SC-5b boot error via the env path).
    #[test]
    fn cdn_invalid_provider_from_env_errors() {
        let config = Config {
            provider_error: Some("Fastly".into()),
            ..Default::default()
        };
        let result = config.build_purge_api();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("build_purge_api must return Err when provider_error is set"),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("none, digitalocean, bunny, cloudflare"),
            "Error must list valid values: {msg}"
        );
    }

    #[test]
    fn cdn_provider_none_no_op() {
        let config = Config::default();
        assert!(
            matches!(config.build_purge_api(), Ok(None)),
            "provider=None must return Ok(None)"
        );
    }

    #[test]
    #[serial_test::serial]
    fn cdn_config_from_env_quartet() {
        std::env::set_var("CDN_URL", "https://q.example.com");
        std::env::set_var("CDN_PROVIDER", "digitalocean");
        std::env::set_var("CDN_PURGE_TOKEN", "tok");
        std::env::set_var("CDN_PURGE_ZONE", "ep-9");

        let config = Config::from_env();

        std::env::remove_var("CDN_URL");
        std::env::remove_var("CDN_PROVIDER");
        std::env::remove_var("CDN_PURGE_TOKEN");
        std::env::remove_var("CDN_PURGE_ZONE");

        assert_eq!(config.url, Some("https://q.example.com".into()));
        assert_eq!(config.provider, CdnProvider::DigitalOcean);
        assert_eq!(config.purge_token, Some("tok".into()));
        assert_eq!(config.purge_zone, Some("ep-9".into()));
    }

    #[test]
    #[serial_test::serial]
    fn cdn_fallback_aws_cdn_url() {
        std::env::remove_var("CDN_URL");
        std::env::set_var("AWS_CDN_URL", "https://legacy");

        let config = Config::from_env();

        std::env::remove_var("AWS_CDN_URL");

        assert_eq!(config.url, Some("https://legacy".into()));
    }

    #[test]
    #[serial_test::serial]
    fn cdn_fallback_do_zone_and_token() {
        std::env::remove_var("CDN_PURGE_ZONE");
        std::env::remove_var("CDN_PURGE_TOKEN");
        std::env::remove_var("CDN_PROVIDER");
        std::env::set_var("DO_SPACES_CDN_ID", "ep-1");
        std::env::set_var("DIGITALOCEAN_ACCESS_TOKEN", "t");

        let config = Config::from_env();

        std::env::remove_var("DO_SPACES_CDN_ID");
        std::env::remove_var("DIGITALOCEAN_ACCESS_TOKEN");

        assert_eq!(config.purge_zone, Some("ep-1".into()));
        assert_eq!(config.purge_token, Some("t".into()));
        assert_eq!(config.provider, CdnProvider::DigitalOcean);
    }

    #[cfg(not(feature = "cdn-bunny"))]
    #[test]
    fn cdn_feature_required_bunny() {
        let config = Config {
            provider: CdnProvider::Bunny,
            ..Default::default()
        };
        let result = config.build_purge_api();
        assert!(
            result.is_err(),
            "Expected Err when cdn-bunny feature is off"
        );
        let msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => unreachable!(),
        };
        assert!(
            msg.contains("cdn-bunny"),
            "Error must mention the cdn-bunny feature: {msg}"
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

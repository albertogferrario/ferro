//! Bunny CDN purge adapter (feature `cdn-bunny`).
use crate::cdn::PurgeApi;
use crate::Error;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// Bunny's documented purge rate limit is 1000 req/min per zone.
/// A conservative internal bound of 100 requests per 10-second window (~600/min) stays well
/// below the documented limit while still allowing high throughput for typical batch sizes.
const BUNNY_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);
const BUNNY_RATE_LIMIT_MAX: usize = 100;

/// Configuration for the Bunny CDN URL-purge adapter.
#[derive(Clone)]
pub struct BunnyCdnConfig {
    /// CDN zone base URL, e.g. `https://myzone.b-cdn.net` (Bunny requires full URLs).
    pub cdn_base_url: String,
    /// Bunny API access key (`BUNNY_ACCESS_KEY`). Never logged.
    pub access_key: String,
}

impl std::fmt::Debug for BunnyCdnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BunnyCdnConfig")
            .field("cdn_base_url", &self.cdn_base_url)
            .field("access_key", &"<redacted>")
            .finish()
    }
}

impl BunnyCdnConfig {
    /// Read config from environment.
    ///
    /// - `BUNNY_CDN_URL` — CDN zone base URL.
    /// - `BUNNY_ACCESS_KEY` — Bunny API access key.
    pub fn from_env() -> Self {
        Self {
            cdn_base_url: std::env::var("BUNNY_CDN_URL").unwrap_or_default(),
            access_key: std::env::var("BUNNY_ACCESS_KEY").unwrap_or_default(),
        }
    }
}

/// Bunny CDN adapter implementing [`PurgeApi`].
///
/// Uses per-URL `POST https://api.bunny.net/purge?url=...&async=false` calls.
/// Bunny requires full URLs; the adapter prepends `cdn_base_url` to each relative path.
///
/// An internal sliding-window throttle enforces ≤100 requests per 10 s (conservative bound
/// below Bunny's documented 1000 req/min limit). Rate limiting is handled internally;
/// callers do not need to manage it.
pub struct BunnyCdn {
    config: BunnyCdnConfig,
    client: reqwest::Client,
    request_times: Mutex<VecDeque<Instant>>,
}

impl BunnyCdn {
    /// Construct from config. Builds a single `reqwest::Client` shared across all requests.
    pub fn new(config: BunnyCdnConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            request_times: Mutex::new(VecDeque::new()),
        }
    }

    /// Sliding-window rate limiter: ensures ≤ BUNNY_RATE_LIMIT_MAX requests per
    /// BUNNY_RATE_LIMIT_WINDOW. Loops until a free slot is confirmed while holding the lock;
    /// the lock is never held across the sleep to avoid blocking other waiters.
    async fn throttle(&self) {
        loop {
            let mut times = self.request_times.lock().await;
            let now = Instant::now();
            while times
                .front()
                .map(|t| now.duration_since(*t) >= BUNNY_RATE_LIMIT_WINDOW)
                .unwrap_or(false)
            {
                times.pop_front();
            }
            if times.len() < BUNNY_RATE_LIMIT_MAX {
                times.push_back(Instant::now());
                return;
            }
            let oldest = *times.front().unwrap();
            let sleep_for = BUNNY_RATE_LIMIT_WINDOW - now.duration_since(oldest);
            drop(times);
            tokio::time::sleep(sleep_for).await;
        }
    }
}

#[async_trait]
impl PurgeApi for BunnyCdn {
    async fn purge(&self, paths: &[String]) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }
        if self.config.access_key.is_empty() {
            return Err(Error::cdn("BUNNY_ACCESS_KEY not set"));
        }
        if self.config.cdn_base_url.is_empty() {
            return Err(Error::cdn("BUNNY_CDN_URL not set"));
        }
        for path in paths {
            self.throttle().await;
            let full_url = format!(
                "{}/{}",
                self.config.cdn_base_url.trim_end_matches('/'),
                path.trim_start_matches('/')
            );
            let resp = self
                .client
                .post("https://api.bunny.net/purge")
                .query(&[("url", full_url.as_str()), ("async", "false")])
                .header("AccessKey", &self.config.access_key)
                .send()
                .await
                .map_err(|e| Error::cdn(e.to_string()))?;
            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                return Err(Error::cdn(format!("Bunny purge status {status}: {body}")));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_cdn() -> BunnyCdn {
        BunnyCdn::new(BunnyCdnConfig {
            cdn_base_url: "https://myzone.b-cdn.net".to_string(),
            access_key: "test-key".to_string(),
        })
    }

    /// purge(&[]) → Ok(()), no HTTP.
    #[tokio::test]
    async fn bunny_adapter_empty_noop() {
        valid_cdn().purge(&[]).await.unwrap();
    }

    /// Empty access_key → Err before any HTTP call.
    #[tokio::test]
    async fn bunny_adapter_missing_key_errors() {
        let cdn = BunnyCdn::new(BunnyCdnConfig {
            cdn_base_url: "https://myzone.b-cdn.net".to_string(),
            access_key: "".to_string(),
        });
        let err = cdn.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(err.to_string().contains("BUNNY_ACCESS_KEY"), "Error: {err}");
    }

    /// Empty cdn_base_url → Err before any HTTP call.
    #[tokio::test]
    async fn bunny_adapter_missing_cdn_url_errors() {
        let cdn = BunnyCdn::new(BunnyCdnConfig {
            cdn_base_url: "".to_string(),
            access_key: "test-key".to_string(),
        });
        let err = cdn.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(err.to_string().contains("BUNNY_CDN_URL"), "Error: {err}");
    }
}

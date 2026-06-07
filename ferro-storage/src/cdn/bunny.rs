//! Bunny CDN purge adapter (feature `cdn-bunny`).
use crate::cdn::PurgeApi;
use crate::Error;
use async_trait::async_trait;

/// Configuration for the Bunny CDN URL-purge adapter.
#[derive(Clone)]
pub struct BunnyCdnConfig {
    /// CDN zone base URL, e.g. "https://myzone.b-cdn.net" (Bunny requires full URLs).
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
pub struct BunnyCdn {
    config: BunnyCdnConfig,
    client: reqwest::Client,
}

impl BunnyCdn {
    /// Construct from config. Builds a single `reqwest::Client` shared across all requests.
    pub fn new(config: BunnyCdnConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
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
        for path in paths {
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

//! Cloudflare CDN purge adapter (feature `cdn-cloudflare`).
use crate::cdn::PurgeApi;
use crate::Error;
use async_trait::async_trait;

/// Cloudflare purge API rejects more than 30 URLs per request.
const CF_BATCH_SIZE: usize = 30;

/// Configuration for the Cloudflare CDN purge adapter.
#[derive(Clone)]
pub struct CloudflareCdnConfig {
    /// Cloudflare zone ID (`CF_ZONE_ID`).
    pub zone_id: String,
    /// Cloudflare API token (`CF_API_TOKEN`). Never logged.
    pub api_token: String,
    /// CDN base URL, e.g. `https://example.com` (Cloudflare requires full URLs).
    pub cdn_base_url: String,
}

impl std::fmt::Debug for CloudflareCdnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudflareCdnConfig")
            .field("zone_id", &self.zone_id)
            .field("api_token", &"<redacted>")
            .field("cdn_base_url", &self.cdn_base_url)
            .finish()
    }
}

impl CloudflareCdnConfig {
    /// Read config from environment.
    ///
    /// - `CF_ZONE_ID` — Cloudflare zone ID.
    /// - `CF_API_TOKEN` — Cloudflare API token.
    /// - `CF_CDN_URL` — CDN base URL (prepended to relative paths for full-URL purge).
    pub fn from_env() -> Self {
        Self {
            zone_id: std::env::var("CF_ZONE_ID").unwrap_or_default(),
            api_token: std::env::var("CF_API_TOKEN").unwrap_or_default(),
            cdn_base_url: std::env::var("CF_CDN_URL").unwrap_or_default(),
        }
    }
}

/// Cloudflare CDN adapter implementing [`PurgeApi`].
///
/// Uses `POST https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache`
/// with `{"files": [...full_urls...]}` body and `Authorization: Bearer {api_token}`.
/// Cloudflare requires full URLs; the adapter prepends `cdn_base_url` to each relative path.
pub struct CloudflareCdn {
    config: CloudflareCdnConfig,
    client: reqwest::Client,
}

impl CloudflareCdn {
    /// Construct from config. Builds a single `reqwest::Client` shared across all requests.
    pub fn new(config: CloudflareCdnConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PurgeApi for CloudflareCdn {
    async fn purge(&self, paths: &[String]) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }
        if self.config.api_token.is_empty() {
            return Err(Error::cdn("CF_API_TOKEN not set"));
        }
        if self.config.zone_id.is_empty() {
            return Err(Error::cdn("CF_ZONE_ID not set"));
        }
        if self.config.cdn_base_url.is_empty() {
            return Err(Error::cdn("CF_CDN_URL not set"));
        }
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            self.config.zone_id
        );
        let full_urls: Vec<String> = paths
            .iter()
            .map(|p| {
                format!(
                    "{}/{}",
                    self.config.cdn_base_url.trim_end_matches('/'),
                    p.trim_start_matches('/')
                )
            })
            .collect();
        for chunk in full_urls.chunks(CF_BATCH_SIZE) {
            let resp = self
                .client
                .post(&url)
                .bearer_auth(&self.config.api_token)
                .json(&serde_json::json!({ "files": chunk }))
                .send()
                .await
                .map_err(|e| Error::cdn(e.to_string()))?;
            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                return Err(Error::cdn(format!(
                    "Cloudflare purge status {status}: {body}"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_cdn() -> CloudflareCdn {
        CloudflareCdn::new(CloudflareCdnConfig {
            zone_id: "zone-id".to_string(),
            api_token: "test-token".to_string(),
            cdn_base_url: "https://cdn.example.com".to_string(),
        })
    }

    /// CF_BATCH_SIZE is 30; 35 paths must produce 2 chunks.
    #[test]
    fn cf_batch_size_chunks_correctly() {
        let paths: Vec<String> = (0..35).map(|i| format!("file{i}.html")).collect();
        let cdn_base = "https://cdn.example.com";
        let full_urls: Vec<String> = paths.iter().map(|p| format!("{cdn_base}/{p}")).collect();
        let chunks: Vec<&[String]> = full_urls.chunks(CF_BATCH_SIZE).collect();
        assert_eq!(chunks.len(), 2, "35 paths → 2 chunks of ≤30");
        assert_eq!(chunks[0].len(), 30);
        assert_eq!(chunks[1].len(), 5);
    }

    /// purge(&[]) → Ok(()), no HTTP.
    #[tokio::test]
    async fn cf_adapter_empty_noop() {
        valid_cdn().purge(&[]).await.unwrap();
    }

    /// Empty api_token → Err before any HTTP call.
    #[tokio::test]
    async fn cf_adapter_missing_token_errors() {
        let cdn = CloudflareCdn::new(CloudflareCdnConfig {
            zone_id: "zone-id".to_string(),
            api_token: "".to_string(),
            cdn_base_url: "https://cdn.example.com".to_string(),
        });
        let err = cdn.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(err.to_string().contains("CF_API_TOKEN"), "Error: {err}");
    }

    /// Empty zone_id → Err before any HTTP call.
    #[tokio::test]
    async fn cf_adapter_missing_zone_id_errors() {
        let cdn = CloudflareCdn::new(CloudflareCdnConfig {
            zone_id: "".to_string(),
            api_token: "test-token".to_string(),
            cdn_base_url: "https://cdn.example.com".to_string(),
        });
        let err = cdn.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(err.to_string().contains("CF_ZONE_ID"), "Error: {err}");
    }

    /// Empty cdn_base_url → Err before any HTTP call.
    #[tokio::test]
    async fn cf_adapter_missing_cdn_url_errors() {
        let cdn = CloudflareCdn::new(CloudflareCdnConfig {
            zone_id: "zone-id".to_string(),
            api_token: "test-token".to_string(),
            cdn_base_url: "".to_string(),
        });
        let err = cdn.purge(&["index.html".to_string()]).await.unwrap_err();
        assert!(err.to_string().contains("CF_CDN_URL"), "Error: {err}");
    }
}

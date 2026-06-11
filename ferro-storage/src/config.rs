//! Configuration for the storage system.

use crate::facade::DiskConfig;
#[cfg(feature = "s3")]
use crate::facade::DiskDriver;
use std::collections::HashMap;
use std::env;

/// Configuration for the storage system.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Default disk name.
    pub default: String,
    /// Disk configurations.
    pub disks: HashMap<String, DiskConfig>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let mut disks = HashMap::new();
        disks.insert("local".to_string(), DiskConfig::local("./storage"));

        Self {
            default: "local".to_string(),
            disks,
        }
    }
}

impl StorageConfig {
    /// Create a new storage config with a default disk.
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            default: default.into(),
            disks: HashMap::new(),
        }
    }

    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `FILESYSTEM_DISK`: Default disk name (default: "local")
    /// - `FILESYSTEM_LOCAL_ROOT`: Root path for local disk (default: "./storage")
    /// - `FILESYSTEM_LOCAL_URL`: Public URL for local files
    /// - `FILESYSTEM_PUBLIC_ROOT`: Root path for public disk (default: "./storage/public")
    /// - `FILESYSTEM_PUBLIC_URL`: Public URL for public files (default: "/storage")
    ///
    /// With `s3` feature:
    /// - `AWS_ACCESS_KEY_ID`: S3 access key
    /// - `AWS_SECRET_ACCESS_KEY`: S3 secret key
    /// - `AWS_DEFAULT_REGION`: S3 region (default: "us-east-1")
    /// - `AWS_BUCKET`: S3 bucket name
    /// - `AWS_PUBLIC_URL`: Public base URL for generated file URLs (overrides `AWS_URL` for this purpose)
    /// - `AWS_URL`: S3 API endpoint; also used as public URL base if `AWS_PUBLIC_URL` is not set
    /// - `AWS_CDN_URL`: CDN base URL fronting the Spaces bucket (optional; used by `cdn_url()`)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_storage::{StorageConfig, Storage};
    ///
    /// let config = StorageConfig::from_env();
    /// let storage = Storage::with_storage_config(config);
    /// ```
    pub fn from_env() -> Self {
        let default = env::var("FILESYSTEM_DISK").unwrap_or_else(|_| "local".to_string());
        let mut disks = HashMap::new();

        // Local disk
        let local_root =
            env::var("FILESYSTEM_LOCAL_ROOT").unwrap_or_else(|_| "./storage".to_string());
        let mut local_config = DiskConfig::local(&local_root);
        if let Ok(url) = env::var("FILESYSTEM_LOCAL_URL") {
            local_config = local_config.with_url(url);
        }
        disks.insert("local".to_string(), local_config);

        // Public disk (for publicly accessible files)
        let public_root =
            env::var("FILESYSTEM_PUBLIC_ROOT").unwrap_or_else(|_| "./storage/public".to_string());
        let public_url =
            env::var("FILESYSTEM_PUBLIC_URL").unwrap_or_else(|_| "/storage".to_string());
        let public_config = DiskConfig::local(&public_root).with_url(public_url);
        disks.insert("public".to_string(), public_config);

        // S3 disk (if configured)
        #[cfg(feature = "s3")]
        if let Ok(bucket) = env::var("AWS_BUCKET") {
            let region = env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "us-east-1".to_string());
            let mut s3_config = DiskConfig {
                driver: DiskDriver::S3,
                root: None,
                url: None,
                cdn_url: None,
                bucket: Some(bucket.clone()),
                region: Some(region),
            };
            // Resolve public file URL base (used by Storage::url() to build asset URLs).
            // Priority: AWS_PUBLIC_URL → computed from AWS_URL+bucket → AWS_URL bare.
            // Auto-compute handles providers like DigitalOcean Spaces and Cloudflare R2
            // where the public URL is {bucket}.{endpoint_host} but the API endpoint is
            // just {endpoint_host}.
            let public_url = if let Ok(explicit) = env::var("AWS_PUBLIC_URL") {
                Some(explicit)
            } else if let Ok(api_url) = env::var("AWS_URL") {
                let host = api_url
                    .trim_start_matches("https://")
                    .trim_start_matches("http://");
                let scheme = if api_url.starts_with("https://") {
                    "https"
                } else {
                    "http"
                };
                Some(format!("{scheme}://{bucket}.{host}"))
            } else {
                None
            };
            s3_config.url = public_url;
            // Read the unified CDN config (handles CDN_URL quartet + AWS_CDN_URL fallback + deprecation warn).
            let cdn_config = crate::cdn::Config::from_env();
            if let Some(cdn_url) = cdn_config.url {
                s3_config = s3_config.with_cdn_url(cdn_url);
            }
            disks.insert("s3".to_string(), s3_config);
        }

        Self { default, disks }
    }

    /// Add a disk configuration.
    pub fn disk(mut self, name: impl Into<String>, config: DiskConfig) -> Self {
        self.disks.insert(name.into(), config);
        self
    }

    /// Set the default disk.
    pub fn default_disk(mut self, name: impl Into<String>) -> Self {
        self.default = name.into();
        self
    }

    /// Get the default disk name.
    pub fn get_default(&self) -> &str {
        &self.default
    }

    /// Get a disk configuration by name.
    pub fn get_disk(&self, name: &str) -> Option<&DiskConfig> {
        self.disks.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.default, "local");
        assert!(config.disks.contains_key("local"));
    }

    #[test]
    fn test_storage_config_builder() {
        let config = StorageConfig::new("s3")
            .disk("local", DiskConfig::local("./storage"))
            .disk("public", DiskConfig::local("./public").with_url("/files"));

        assert_eq!(config.default, "s3");
        assert!(config.disks.contains_key("local"));
        assert!(config.disks.contains_key("public"));
    }

    #[test]
    #[serial]
    fn test_storage_config_from_env() {
        // Test with default env (no env vars set)
        let config = StorageConfig::from_env();
        assert_eq!(config.default, "local");
        assert!(config.disks.contains_key("local"));
        assert!(config.disks.contains_key("public"));
    }

    #[cfg(feature = "s3")]
    #[test]
    #[serial]
    fn from_env_cdn_url() {
        std::env::set_var("AWS_BUCKET", "test-bucket");
        std::env::set_var("AWS_CDN_URL", "https://cdn.test.example.com");
        let config = StorageConfig::from_env();
        let s3_disk = config.get_disk("s3").expect("s3 disk should be configured");
        assert_eq!(
            s3_disk.cdn_url,
            Some("https://cdn.test.example.com".to_string())
        );
        std::env::remove_var("AWS_BUCKET");
        std::env::remove_var("AWS_CDN_URL");
    }

    /// SC-3: AWS_CDN_URL-only env → Disk::cdn_url bytes identical to the pre-phase direct read.
    /// The fallback path in cdn::Config::from_env() must yield the same URL.
    #[cfg(feature = "s3")]
    #[test]
    #[serial]
    fn cdn_url_parity_aws_fallback() {
        std::env::remove_var("CDN_URL");
        std::env::set_var("AWS_BUCKET", "test-bucket");
        std::env::set_var("AWS_CDN_URL", "https://cdn.parity.example.com");
        let config = StorageConfig::from_env();
        let s3_disk = config.get_disk("s3").expect("s3 disk should be configured");
        assert_eq!(
            s3_disk.cdn_url,
            Some("https://cdn.parity.example.com".to_string()),
            "AWS_CDN_URL fallback must yield byte-identical URL (SC-3 parity)"
        );
        std::env::remove_var("AWS_BUCKET");
        std::env::remove_var("AWS_CDN_URL");
    }

    /// SC-4: legacy DO vars (DO_SPACES_CDN_ID + DIGITALOCEAN_ACCESS_TOKEN) → DoSpacesCdn purge
    /// hits the same DO Spaces CDN endpoint and auth as the pre-phase code (T-204-PURGE-PARITY).
    #[tokio::test]
    #[serial]
    async fn purge_parity_legacy_do() {
        use crate::cdn::PurgeApi;
        use wiremock::matchers::{header, method, path_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        // Simulate a legacy-DO-only deployment — clear the quartet so inference kicks in.
        std::env::remove_var("CDN_PROVIDER");
        std::env::remove_var("CDN_PURGE_ZONE");
        std::env::remove_var("CDN_PURGE_TOKEN");
        std::env::set_var("DO_SPACES_CDN_ID", "legacy-id");
        std::env::set_var("DIGITALOCEAN_ACCESS_TOKEN", "legacy-token");

        let cfg = crate::cdn::Config::from_env();
        // Provider inferred from DO_SPACES_CDN_ID; zone/token read from legacy fallbacks.
        assert_eq!(cfg.provider, crate::cdn::CdnProvider::DigitalOcean);
        assert_eq!(cfg.purge_zone.as_deref(), Some("legacy-id"));
        assert_eq!(cfg.purge_token.as_deref(), Some("legacy-token"));

        // Build the adapter config the unified path yields, point it at the mock server,
        // and assert identical endpoint + auth to the pre-phase DO purge path.
        let server = MockServer::start().await;
        let do_cfg = crate::DoSpacesCdnConfig {
            endpoint_id: cfg.purge_zone.clone(),
            api_token: cfg.purge_token.clone().unwrap_or_default(),
            api_base: Some(server.uri()),
        };
        Mock::given(method("DELETE"))
            .and(path_regex(r"/v2/cdn/endpoints/legacy-id/cache"))
            .and(header("Authorization", "Bearer legacy-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let purger = crate::DoSpacesCdn::new(do_cfg);
        purger.purge(&["index.html".to_string()]).await.unwrap();

        std::env::remove_var("DO_SPACES_CDN_ID");
        std::env::remove_var("DIGITALOCEAN_ACCESS_TOKEN");
    }
}

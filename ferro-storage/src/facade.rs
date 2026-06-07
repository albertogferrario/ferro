//! Storage facade for managing multiple disks.

use crate::config::StorageConfig;
use crate::drivers::{LocalDriver, MemoryDriver};
use crate::storage::{FileMetadata, PutOptions, StorageDriver};
use crate::Error;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;

/// Storage disk configuration.
#[derive(Debug, Clone)]
pub struct DiskConfig {
    /// Disk driver type.
    pub driver: DiskDriver,
    /// Root path for local driver.
    pub root: Option<String>,
    /// URL base for generating URLs.
    pub url: Option<String>,
    /// CDN base URL for generating edge URLs (falls back to url when None).
    pub cdn_url: Option<String>,
    /// S3 bucket for S3 driver.
    #[cfg(feature = "s3")]
    pub bucket: Option<String>,
    /// S3 region.
    #[cfg(feature = "s3")]
    pub region: Option<String>,
}

/// Available disk drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskDriver {
    /// Local filesystem.
    Local,
    /// In-memory (for testing).
    Memory,
    /// Amazon S3.
    #[cfg(feature = "s3")]
    S3,
}

impl Default for DiskConfig {
    fn default() -> Self {
        Self {
            driver: DiskDriver::Local,
            root: Some("storage".to_string()),
            url: None,
            cdn_url: None,
            #[cfg(feature = "s3")]
            bucket: None,
            #[cfg(feature = "s3")]
            region: None,
        }
    }
}

impl DiskConfig {
    /// Create a local disk config.
    pub fn local(root: impl Into<String>) -> Self {
        Self {
            driver: DiskDriver::Local,
            root: Some(root.into()),
            url: None,
            cdn_url: None,
            #[cfg(feature = "s3")]
            bucket: None,
            #[cfg(feature = "s3")]
            region: None,
        }
    }

    /// Create a memory disk config.
    pub fn memory() -> Self {
        Self {
            driver: DiskDriver::Memory,
            root: None,
            url: None,
            cdn_url: None,
            #[cfg(feature = "s3")]
            bucket: None,
            #[cfg(feature = "s3")]
            region: None,
        }
    }

    /// Set URL base.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the CDN base URL.
    pub fn with_cdn_url(mut self, cdn_url: impl Into<String>) -> Self {
        self.cdn_url = Some(cdn_url.into());
        self
    }
}

/// Storage facade for file operations.
#[derive(Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

struct StorageInner {
    disks: DashMap<String, (Arc<dyn StorageDriver>, Option<String>)>,
    default_disk: String,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    /// Create a new storage instance with a default local disk.
    pub fn new() -> Self {
        let inner = StorageInner {
            disks: DashMap::new(),
            default_disk: "local".to_string(),
        };

        let storage = Self {
            inner: Arc::new(inner),
        };

        // Add default local disk
        let local = LocalDriver::new("storage");
        storage
            .inner
            .disks
            .insert("local".to_string(), (Arc::new(local), None));

        storage
    }

    /// Create storage with custom configuration.
    pub fn with_config(default_disk: &str, configs: Vec<(&str, DiskConfig)>) -> Self {
        let inner = StorageInner {
            disks: DashMap::new(),
            default_disk: default_disk.to_string(),
        };

        let storage = Self {
            inner: Arc::new(inner),
        };

        for (name, config) in configs {
            let driver = Self::create_driver(&config);
            storage
                .inner
                .disks
                .insert(name.to_string(), (driver, config.cdn_url.clone()));
        }

        storage
    }

    /// Create storage from a StorageConfig (typically loaded from environment).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_storage::{Storage, StorageConfig};
    ///
    /// // Load configuration from environment variables
    /// let config = StorageConfig::from_env();
    /// let storage = Storage::with_storage_config(config);
    ///
    /// // Or use the builder pattern
    /// let config = StorageConfig::new("local")
    ///     .disk("local", DiskConfig::local("./storage"))
    ///     .disk("public", DiskConfig::local("./public").with_url("/storage"));
    /// let storage = Storage::with_storage_config(config);
    /// ```
    pub fn with_storage_config(config: StorageConfig) -> Self {
        let inner = StorageInner {
            disks: DashMap::new(),
            default_disk: config.default,
        };

        let storage = Self {
            inner: Arc::new(inner),
        };

        for (name, disk_config) in config.disks {
            let driver = Self::create_driver(&disk_config);
            storage
                .inner
                .disks
                .insert(name, (driver, disk_config.cdn_url.clone()));
        }

        storage
    }

    /// Create a driver from disk configuration.
    fn create_driver(config: &DiskConfig) -> Arc<dyn StorageDriver> {
        match config.driver {
            DiskDriver::Local => {
                let root = config.root.clone().unwrap_or_else(|| "storage".to_string());
                let mut driver = LocalDriver::new(root);
                if let Some(url) = &config.url {
                    driver = driver.with_url_base(url);
                }
                Arc::new(driver)
            }
            DiskDriver::Memory => {
                let mut driver = MemoryDriver::new();
                if let Some(url) = &config.url {
                    driver = driver.with_url_base(url);
                }
                Arc::new(driver)
            }
            #[cfg(feature = "s3")]
            DiskDriver::S3 => {
                let bucket = config.bucket.clone().unwrap_or_default();
                let region = config
                    .region
                    .clone()
                    .unwrap_or_else(|| "us-east-1".to_string());
                let url_base = config.url.clone();
                let endpoint_url = std::env::var("AWS_URL").ok();
                Arc::new(crate::drivers::S3Driver::new(
                    bucket,
                    region,
                    url_base,
                    endpoint_url,
                ))
            }
        }
    }

    /// Get a specific disk.
    pub fn disk(&self, name: &str) -> Result<Disk, Error> {
        let entry = self
            .inner
            .disks
            .get(name)
            .ok_or_else(|| Error::disk_not_configured(name))?;
        let (driver, cdn_url) = entry.clone();

        Ok(Disk::new(driver, cdn_url))
    }

    /// Get the default disk.
    pub fn default_disk(&self) -> Result<Disk, Error> {
        self.disk(&self.inner.default_disk)
    }

    /// Register a disk without a CDN URL. `cdn_url()` on this disk falls back to the origin URL.
    pub fn register_disk(&self, name: impl Into<String>, driver: Arc<dyn StorageDriver>) {
        self.inner.disks.insert(name.into(), (driver, None));
    }

    /// Register a disk with an optional CDN base URL.
    ///
    /// When `cdn_url` is `Some`, `Disk::cdn_url()` returns `{cdn_url}/{path}` instead of
    /// falling back to the origin URL.
    pub fn register_disk_with_cdn(
        &self,
        name: impl Into<String>,
        driver: Arc<dyn StorageDriver>,
        cdn_url: Option<String>,
    ) {
        self.inner.disks.insert(name.into(), (driver, cdn_url));
    }

    // Convenience methods that operate on the default disk

    /// Check if a file exists.
    pub async fn exists(&self, path: &str) -> Result<bool, Error> {
        self.default_disk()?.exists(path).await
    }

    /// Get file contents.
    pub async fn get(&self, path: &str) -> Result<Bytes, Error> {
        self.default_disk()?.get(path).await
    }

    /// Get file as string.
    pub async fn get_string(&self, path: &str) -> Result<String, Error> {
        self.default_disk()?.get_string(path).await
    }

    /// Put file contents.
    pub async fn put(&self, path: &str, contents: impl Into<Bytes>) -> Result<(), Error> {
        self.default_disk()?.put(path, contents).await
    }

    /// Put with options.
    pub async fn put_with_options(
        &self,
        path: &str,
        contents: impl Into<Bytes>,
        options: PutOptions,
    ) -> Result<(), Error> {
        self.default_disk()?
            .put_with_options(path, contents, options)
            .await
    }

    /// Delete a file.
    pub async fn delete(&self, path: &str) -> Result<(), Error> {
        self.default_disk()?.delete(path).await
    }

    /// Copy a file.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        self.default_disk()?.copy(from, to).await
    }

    /// Move a file.
    pub async fn rename(&self, from: &str, to: &str) -> Result<(), Error> {
        self.default_disk()?.rename(from, to).await
    }

    /// Get file URL.
    pub async fn url(&self, path: &str) -> Result<String, Error> {
        self.default_disk()?.url(path).await
    }

    /// Get the CDN edge URL for a path (origin fallback when no CDN configured).
    pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
        self.default_disk()?.cdn_url(path).await
    }
}

/// A handle to a specific disk.
#[derive(Clone)]
pub struct Disk {
    driver: Arc<dyn StorageDriver>,
    cdn_url: Option<String>,
}

impl Disk {
    /// Create a disk from a driver.
    pub fn new(driver: Arc<dyn StorageDriver>, cdn_url: Option<String>) -> Self {
        Self { driver, cdn_url }
    }

    /// Check if a file exists.
    pub async fn exists(&self, path: &str) -> Result<bool, Error> {
        self.driver.exists(path).await
    }

    /// Get file contents.
    pub async fn get(&self, path: &str) -> Result<Bytes, Error> {
        self.driver.get(path).await
    }

    /// Get file as string.
    pub async fn get_string(&self, path: &str) -> Result<String, Error> {
        self.driver.get_string(path).await
    }

    /// Put file contents.
    pub async fn put(&self, path: &str, contents: impl Into<Bytes>) -> Result<(), Error> {
        self.driver
            .put(path, contents.into(), PutOptions::new())
            .await
    }

    /// Put with options.
    pub async fn put_with_options(
        &self,
        path: &str,
        contents: impl Into<Bytes>,
        options: PutOptions,
    ) -> Result<(), Error> {
        self.driver.put(path, contents.into(), options).await
    }

    /// Delete a file.
    pub async fn delete(&self, path: &str) -> Result<(), Error> {
        self.driver.delete(path).await
    }

    /// Copy a file.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        self.driver.copy(from, to).await
    }

    /// Move a file.
    pub async fn rename(&self, from: &str, to: &str) -> Result<(), Error> {
        self.driver.rename(from, to).await
    }

    /// Get file size.
    pub async fn size(&self, path: &str) -> Result<u64, Error> {
        self.driver.size(path).await
    }

    /// Get file metadata.
    pub async fn metadata(&self, path: &str) -> Result<FileMetadata, Error> {
        self.driver.metadata(path).await
    }

    /// Get file URL.
    pub async fn url(&self, path: &str) -> Result<String, Error> {
        self.driver.url(path).await
    }

    /// Returns the CDN edge URL for a stored path, falling back to the origin URL when no CDN is configured.
    pub async fn cdn_url(&self, path: &str) -> Result<String, Error> {
        match &self.cdn_url {
            Some(base) => {
                let path = path.trim_start_matches('/');
                Ok(format!("{}/{}", base.trim_end_matches('/'), path))
            }
            None => self.url(path).await,
        }
    }

    /// Get temporary URL.
    pub async fn temporary_url(&self, path: &str, expiration: Duration) -> Result<String, Error> {
        self.driver.temporary_url(path, expiration).await
    }

    /// List files in a directory.
    pub async fn files(&self, directory: &str) -> Result<Vec<String>, Error> {
        self.driver.files(directory).await
    }

    /// List all files recursively.
    pub async fn all_files(&self, directory: &str) -> Result<Vec<String>, Error> {
        self.driver.all_files(directory).await
    }

    /// List directories.
    pub async fn directories(&self, directory: &str) -> Result<Vec<String>, Error> {
        self.driver.directories(directory).await
    }

    /// Create a directory.
    pub async fn make_directory(&self, path: &str) -> Result<(), Error> {
        self.driver.make_directory(path).await
    }

    /// Delete a directory.
    pub async fn delete_directory(&self, path: &str) -> Result<(), Error> {
        self.driver.delete_directory(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cdn_url_returns_cdn_when_configured() {
        let storage = Storage::with_config(
            "memory",
            vec![(
                "memory",
                DiskConfig::memory()
                    .with_url("https://origin.example.com")
                    .with_cdn_url("https://cdn.example.com"),
            )],
        );
        let url = storage
            .disk("memory")
            .unwrap()
            .cdn_url("images/photo.jpg")
            .await
            .unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn cdn_url_falls_back_to_origin() {
        let storage = Storage::with_config(
            "memory",
            vec![(
                "memory",
                DiskConfig::memory().with_url("https://origin.example.com"),
            )],
        );
        let url = storage
            .disk("memory")
            .unwrap()
            .cdn_url("images/photo.jpg")
            .await
            .unwrap();
        assert_eq!(url, "https://origin.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn cdn_url_no_double_slash() {
        let storage = Storage::with_config(
            "memory",
            vec![(
                "memory",
                DiskConfig::memory().with_cdn_url("https://cdn.example.com/"),
            )],
        );
        let url = storage
            .disk("memory")
            .unwrap()
            .cdn_url("/index.html")
            .await
            .unwrap();
        assert_eq!(url, "https://cdn.example.com/index.html");
    }

    #[tokio::test]
    async fn cdn_url_via_storage_facade() {
        let storage = Storage::with_config(
            "memory",
            vec![(
                "memory",
                DiskConfig::memory()
                    .with_url("https://origin.example.com")
                    .with_cdn_url("https://cdn.example.com"),
            )],
        );
        let url = storage.cdn_url("images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn test_storage_default_disk() {
        let storage = Storage::with_config("memory", vec![("memory", DiskConfig::memory())]);

        storage.put("test.txt", "hello world").await.unwrap();
        let contents = storage.get_string("test.txt").await.unwrap();
        assert_eq!(contents, "hello world");
    }

    #[tokio::test]
    async fn test_storage_multiple_disks() {
        let storage = Storage::with_config(
            "primary",
            vec![
                ("primary", DiskConfig::memory()),
                ("backup", DiskConfig::memory()),
            ],
        );

        // Write to primary
        storage
            .disk("primary")
            .unwrap()
            .put("data.txt", "primary data")
            .await
            .unwrap();

        // Write to backup
        storage
            .disk("backup")
            .unwrap()
            .put("data.txt", "backup data")
            .await
            .unwrap();

        // Verify they're independent
        let primary = storage
            .disk("primary")
            .unwrap()
            .get_string("data.txt")
            .await
            .unwrap();
        let backup = storage
            .disk("backup")
            .unwrap()
            .get_string("data.txt")
            .await
            .unwrap();

        assert_eq!(primary, "primary data");
        assert_eq!(backup, "backup data");
    }

    #[tokio::test]
    async fn test_disk_not_configured() {
        let storage = Storage::with_config("memory", vec![("memory", DiskConfig::memory())]);
        let result = storage.disk("nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_disk() {
        let storage = Storage::new();
        let memory_driver = Arc::new(MemoryDriver::new());
        storage.register_disk("test", memory_driver);

        storage
            .disk("test")
            .unwrap()
            .put("file.txt", "content")
            .await
            .unwrap();

        assert!(storage
            .disk("test")
            .unwrap()
            .exists("file.txt")
            .await
            .unwrap());
    }

    /// register_disk_with_cdn(Some(url)) makes cdn_url() return the CDN-prefixed URL.
    #[tokio::test]
    async fn test_register_disk_with_cdn_url() {
        let storage = Storage::new();
        let driver = Arc::new(MemoryDriver::new());
        storage.register_disk_with_cdn(
            "cdn-disk",
            driver,
            Some("https://cdn.example.com".to_string()),
        );
        let url = storage
            .disk("cdn-disk")
            .unwrap()
            .cdn_url("assets/app.js")
            .await
            .unwrap();
        assert_eq!(url, "https://cdn.example.com/assets/app.js");
    }

    /// register_disk_with_cdn(None) behaves identically to register_disk.
    #[tokio::test]
    async fn test_register_disk_with_cdn_none_falls_back() {
        let storage = Storage::new();
        storage.register_disk_with_cdn("no-cdn", Arc::new(MemoryDriver::new()), None);
        // Disk is registered and accessible; cdn_url() delegates to driver.url() (origin fallback).
        assert!(storage.disk("no-cdn").is_ok());
    }
}

//! # Ferro Storage
//!
//! File storage abstraction for the Ferro framework.
//!
//! Provides a unified API for working with different storage backends:
//! - Local filesystem
//! - In-memory (for testing)
//! - Amazon S3 (with `s3` feature)
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_storage::{Storage, DiskConfig};
//!
//! // Create storage with configuration
//! let storage = Storage::with_config(
//!     "local",
//!     vec![
//!         ("local", DiskConfig::local("storage/app")),
//!         ("public", DiskConfig::local("storage/public").with_url("/storage")),
//!     ],
//! );
//!
//! // Store a file
//! storage.put("documents/report.pdf", file_contents).await?;
//!
//! // Get a file
//! let contents = storage.get("documents/report.pdf").await?;
//!
//! // Get URL
//! let url = storage.disk("public")?.url("images/logo.png").await?;
//! ```
//!
//! ## Multiple Disks
//!
//! You can configure multiple disks and switch between them:
//!
//! ```rust,ignore
//! use ferro_storage::Storage;
//!
//! // Use specific disk
//! let disk = storage.disk("s3")?;
//! disk.put("backups/data.json", data).await?;
//!
//! // Use default disk
//! storage.put("temp/cache.txt", cache_data).await?;
//! ```

pub mod cdn;
mod config;
mod drivers;
mod env_helpers;
mod error;
mod facade;
mod storage;

#[cfg(feature = "s3")]
pub use drivers::S3Driver;
pub use drivers::{LocalDriver, MemoryDriver};

pub use cdn::{CdnProvider, Config as CdnConfig, DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};

#[cfg(feature = "cdn-bunny")]
pub use cdn::{BunnyCdn, BunnyCdnConfig};
#[cfg(feature = "cdn-cloudflare")]
pub use cdn::{CloudflareCdn, CloudflareCdnConfig};

pub use config::StorageConfig;
pub use error::Error;
pub use facade::{Disk, DiskConfig, DiskDriver, Storage};
pub use storage::{FileMetadata, PutOptions, StorageDriver, Visibility};

/// Re-export bytes for convenience.
pub use bytes::Bytes;

/// Re-export async_trait for implementing StorageDriver.
pub use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let storage = Storage::with_config(
            "memory",
            vec![(
                "memory",
                DiskConfig::memory().with_url("https://cdn.example.com"),
            )],
        );

        // Put file
        storage.put("test/file.txt", "test content").await.unwrap();

        // Verify exists
        assert!(storage.exists("test/file.txt").await.unwrap());

        // Get file
        let contents = storage.get_string("test/file.txt").await.unwrap();
        assert_eq!(contents, "test content");

        // Get URL
        let url = storage.url("test/file.txt").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/test/file.txt");

        // Delete file
        storage.delete("test/file.txt").await.unwrap();
        assert!(!storage.exists("test/file.txt").await.unwrap());
    }
}

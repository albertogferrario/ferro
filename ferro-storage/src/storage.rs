//! Core storage trait and types.

use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// File metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File path.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modified time.
    pub last_modified: Option<SystemTime>,
    /// MIME type.
    pub mime_type: Option<String>,
}

impl FileMetadata {
    /// Create new file metadata.
    pub fn new(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            size,
            last_modified: None,
            mime_type: None,
        }
    }

    /// Set last modified time.
    pub fn with_last_modified(mut self, time: SystemTime) -> Self {
        self.last_modified = Some(time);
        self
    }

    /// Set MIME type.
    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
}

/// Visibility of stored files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// File is publicly accessible.
    Public,
    /// File is private.
    #[default]
    Private,
}

/// Options for storing files.
#[derive(Debug, Clone, Default)]
pub struct PutOptions {
    /// File visibility.
    pub visibility: Visibility,
    /// Content type override.
    pub content_type: Option<String>,
    /// Custom metadata.
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl PutOptions {
    /// Create new put options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set visibility.
    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set content type.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Make file public.
    pub fn public(mut self) -> Self {
        self.visibility = Visibility::Public;
        self
    }

    /// Make file private.
    pub fn private(mut self) -> Self {
        self.visibility = Visibility::Private;
        self
    }
}

/// Storage driver trait.
#[async_trait]
pub trait StorageDriver: Send + Sync {
    /// Check if a file exists.
    async fn exists(&self, path: &str) -> Result<bool, Error>;

    /// Get file contents as bytes.
    async fn get(&self, path: &str) -> Result<Bytes, Error>;

    /// Get file contents as string.
    async fn get_string(&self, path: &str) -> Result<String, Error> {
        let bytes = self.get(path).await?;
        String::from_utf8(bytes.to_vec()).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Put file contents.
    async fn put(&self, path: &str, contents: Bytes, options: PutOptions) -> Result<(), Error>;

    /// Put string contents.
    async fn put_string(
        &self,
        path: &str,
        contents: &str,
        options: PutOptions,
    ) -> Result<(), Error> {
        self.put(path, Bytes::from(contents.to_string()), options)
            .await
    }

    /// Delete a file.
    async fn delete(&self, path: &str) -> Result<(), Error>;

    /// Copy a file.
    async fn copy(&self, from: &str, to: &str) -> Result<(), Error>;

    /// Move a file.
    async fn rename(&self, from: &str, to: &str) -> Result<(), Error> {
        self.copy(from, to).await?;
        self.delete(from).await
    }

    /// Get file size.
    async fn size(&self, path: &str) -> Result<u64, Error>;

    /// Get file metadata.
    async fn metadata(&self, path: &str) -> Result<FileMetadata, Error>;

    /// Get URL to a file.
    async fn url(&self, path: &str) -> Result<String, Error>;

    /// Get a temporary URL (for private files).
    async fn temporary_url(
        &self,
        path: &str,
        expiration: std::time::Duration,
    ) -> Result<String, Error>;

    /// List files in a directory.
    async fn files(&self, directory: &str) -> Result<Vec<String>, Error>;

    /// List all files recursively.
    async fn all_files(&self, directory: &str) -> Result<Vec<String>, Error>;

    /// List directories.
    async fn directories(&self, directory: &str) -> Result<Vec<String>, Error>;

    /// Create a directory.
    async fn make_directory(&self, path: &str) -> Result<(), Error>;

    /// Delete a directory.
    async fn delete_directory(&self, path: &str) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metadata() {
        let meta = FileMetadata::new("test.txt", 100).with_mime_type("text/plain");

        assert_eq!(meta.path, "test.txt");
        assert_eq!(meta.size, 100);
        assert_eq!(meta.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_put_options() {
        let opts = PutOptions::new().public().content_type("image/png");

        assert_eq!(opts.visibility, Visibility::Public);
        assert_eq!(opts.content_type, Some("image/png".to_string()));
    }

    #[test]
    fn test_visibility_default() {
        assert_eq!(Visibility::default(), Visibility::Private);
    }
}

//! In-memory storage driver for testing.

use crate::storage::{FileMetadata, PutOptions, StorageDriver, Visibility};
use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

/// Stored file data.
#[derive(Clone)]
struct StoredFile {
    contents: Bytes,
    #[allow(dead_code)]
    visibility: Visibility,
    content_type: Option<String>,
    created_at: SystemTime,
}

/// In-memory storage driver.
#[derive(Clone)]
pub struct MemoryDriver {
    files: Arc<DashMap<String, StoredFile>>,
    url_base: Option<String>,
}

impl Default for MemoryDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryDriver {
    /// Create a new memory driver.
    pub fn new() -> Self {
        Self {
            files: Arc::new(DashMap::new()),
            url_base: None,
        }
    }

    /// Set the base URL.
    pub fn with_url_base(mut self, url: impl Into<String>) -> Self {
        self.url_base = Some(url.into());
        self
    }

    /// Clear all files.
    pub fn clear(&self) {
        self.files.clear();
    }

    /// Get number of stored files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if storage is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Normalize path (remove leading slash, normalize separators).
    fn normalize_path(path: &str) -> String {
        path.trim_start_matches('/').replace('\\', "/")
    }
}

#[async_trait]
impl StorageDriver for MemoryDriver {
    async fn exists(&self, path: &str) -> Result<bool, Error> {
        let path = Self::normalize_path(path);
        Ok(self.files.contains_key(&path))
    }

    async fn get(&self, path: &str) -> Result<Bytes, Error> {
        let path = Self::normalize_path(path);
        self.files
            .get(&path)
            .map(|f| f.contents.clone())
            .ok_or_else(|| Error::not_found(&path))
    }

    async fn put(&self, path: &str, contents: Bytes, options: PutOptions) -> Result<(), Error> {
        let path = Self::normalize_path(path);
        self.files.insert(
            path,
            StoredFile {
                contents,
                visibility: options.visibility,
                content_type: options.content_type,
                created_at: SystemTime::now(),
            },
        );
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<(), Error> {
        let path = Self::normalize_path(path);
        self.files
            .remove(&path)
            .ok_or_else(|| Error::not_found(&path))?;
        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        let from = Self::normalize_path(from);
        let to = Self::normalize_path(to);

        let file = self
            .files
            .get(&from)
            .ok_or_else(|| Error::not_found(&from))?
            .clone();

        self.files.insert(to, file);
        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64, Error> {
        let path = Self::normalize_path(path);
        self.files
            .get(&path)
            .map(|f| f.contents.len() as u64)
            .ok_or_else(|| Error::not_found(&path))
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata, Error> {
        let normalized = Self::normalize_path(path);
        let file = self
            .files
            .get(&normalized)
            .ok_or_else(|| Error::not_found(&normalized))?;

        let mut meta =
            FileMetadata::new(path, file.contents.len() as u64).with_last_modified(file.created_at);

        if let Some(ref content_type) = file.content_type {
            meta = meta.with_mime_type(content_type);
        }

        Ok(meta)
    }

    async fn url(&self, path: &str) -> Result<String, Error> {
        let path = Self::normalize_path(path);
        match &self.url_base {
            Some(base) => Ok(format!("{}/{}", base.trim_end_matches('/'), path)),
            None => Ok(format!("memory://{path}")),
        }
    }

    async fn temporary_url(
        &self,
        path: &str,
        _expiration: std::time::Duration,
    ) -> Result<String, Error> {
        self.url(path).await
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let dir = Self::normalize_path(directory);
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };

        let mut files = Vec::new();
        for entry in self.files.iter() {
            let path = entry.key();
            if path.starts_with(&prefix) || (prefix.is_empty() && !path.contains('/')) {
                let relative = path.strip_prefix(&prefix).unwrap_or(path);
                // Only include files directly in this directory
                if !relative.contains('/') {
                    files.push(relative.to_string());
                }
            }
        }

        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let dir = Self::normalize_path(directory);
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };

        let mut files = Vec::new();
        for entry in self.files.iter() {
            let path = entry.key();
            if path.starts_with(&prefix) {
                let relative = path.strip_prefix(&prefix).unwrap_or(path);
                files.push(relative.to_string());
            } else if prefix.is_empty() {
                files.push(path.clone());
            }
        }

        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>, Error> {
        let dir = Self::normalize_path(directory);
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };

        let mut dirs: HashSet<String> = HashSet::new();
        for entry in self.files.iter() {
            let path = entry.key();
            if path.starts_with(&prefix) {
                let relative = path.strip_prefix(&prefix).unwrap_or(path);
                if let Some(slash_idx) = relative.find('/') {
                    dirs.insert(relative[..slash_idx].to_string());
                }
            }
        }

        Ok(dirs.into_iter().collect())
    }

    async fn make_directory(&self, _path: &str) -> Result<(), Error> {
        // No-op for memory driver - directories are implicit
        Ok(())
    }

    async fn delete_directory(&self, path: &str) -> Result<(), Error> {
        let dir = Self::normalize_path(path);
        let prefix = format!("{dir}/");

        let keys_to_remove: Vec<String> = self
            .files
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.files.remove(&key);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_driver_put_get() {
        let driver = MemoryDriver::new();

        driver
            .put("test.txt", Bytes::from("hello"), PutOptions::new())
            .await
            .unwrap();

        let contents = driver.get("test.txt").await.unwrap();
        assert_eq!(contents, Bytes::from("hello"));
    }

    #[tokio::test]
    async fn test_memory_driver_exists() {
        let driver = MemoryDriver::new();

        assert!(!driver.exists("missing.txt").await.unwrap());

        driver
            .put("exists.txt", Bytes::from("data"), PutOptions::new())
            .await
            .unwrap();

        assert!(driver.exists("exists.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_driver_delete() {
        let driver = MemoryDriver::new();

        driver
            .put("to_delete.txt", Bytes::from("data"), PutOptions::new())
            .await
            .unwrap();

        driver.delete("to_delete.txt").await.unwrap();
        assert!(!driver.exists("to_delete.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_driver_copy() {
        let driver = MemoryDriver::new();

        driver
            .put("original.txt", Bytes::from("content"), PutOptions::new())
            .await
            .unwrap();

        driver.copy("original.txt", "copy.txt").await.unwrap();

        assert_eq!(
            driver.get("copy.txt").await.unwrap(),
            Bytes::from("content")
        );
    }

    #[tokio::test]
    async fn test_memory_driver_url() {
        let driver = MemoryDriver::new().with_url_base("https://cdn.example.com");

        let url = driver.url("images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn test_memory_driver_directories() {
        let driver = MemoryDriver::new();

        driver
            .put("images/a.jpg", Bytes::from("a"), PutOptions::new())
            .await
            .unwrap();
        driver
            .put("images/b.jpg", Bytes::from("b"), PutOptions::new())
            .await
            .unwrap();
        driver
            .put("docs/readme.md", Bytes::from("readme"), PutOptions::new())
            .await
            .unwrap();

        let dirs = driver.directories("").await.unwrap();
        assert!(dirs.contains(&"images".to_string()));
        assert!(dirs.contains(&"docs".to_string()));
    }

    #[tokio::test]
    async fn test_memory_driver_clear() {
        let driver = MemoryDriver::new();

        driver
            .put("a.txt", Bytes::from("a"), PutOptions::new())
            .await
            .unwrap();
        driver
            .put("b.txt", Bytes::from("b"), PutOptions::new())
            .await
            .unwrap();

        assert_eq!(driver.len(), 2);

        driver.clear();
        assert!(driver.is_empty());
    }
}

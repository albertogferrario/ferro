//! Local filesystem storage driver.

use crate::storage::{FileMetadata, PutOptions, StorageDriver};
use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

/// Local filesystem storage driver.
pub struct LocalDriver {
    /// Base path for storage.
    root: PathBuf,
    /// Base URL for public files.
    url_base: Option<String>,
}

impl LocalDriver {
    /// Create a new local driver.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            url_base: None,
        }
    }

    /// Set the base URL for public files.
    pub fn with_url_base(mut self, url: impl Into<String>) -> Self {
        self.url_base = Some(url.into());
        self
    }

    /// Get the full path for a relative path.
    fn full_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    /// Ensure parent directory exists.
    async fn ensure_directory(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl StorageDriver for LocalDriver {
    async fn exists(&self, path: &str) -> Result<bool, Error> {
        let full_path = self.full_path(path);
        Ok(full_path.exists())
    }

    async fn get(&self, path: &str) -> Result<Bytes, Error> {
        let full_path = self.full_path(path);
        debug!(path = %full_path.display(), "Reading file");

        let contents = fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(path)
            } else {
                Error::from(e)
            }
        })?;

        Ok(Bytes::from(contents))
    }

    async fn put(&self, path: &str, contents: Bytes, _options: PutOptions) -> Result<(), Error> {
        let full_path = self.full_path(path);
        debug!(path = %full_path.display(), "Writing file");

        self.ensure_directory(&full_path).await?;
        fs::write(&full_path, &contents).await?;

        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<(), Error> {
        let full_path = self.full_path(path);
        debug!(path = %full_path.display(), "Deleting file");

        fs::remove_file(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(path)
            } else {
                Error::from(e)
            }
        })
    }

    async fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);

        debug!(from = %from_path.display(), to = %to_path.display(), "Copying file");

        self.ensure_directory(&to_path).await?;
        fs::copy(&from_path, &to_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(from)
            } else {
                Error::from(e)
            }
        })?;

        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64, Error> {
        let full_path = self.full_path(path);
        let metadata = fs::metadata(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(path)
            } else {
                Error::from(e)
            }
        })?;

        Ok(metadata.len())
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata, Error> {
        let full_path = self.full_path(path);
        let fs_meta = fs::metadata(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(path)
            } else {
                Error::from(e)
            }
        })?;

        let mime_type = mime_guess::from_path(&full_path)
            .first()
            .map(|m| m.to_string());

        let mut meta = FileMetadata::new(path, fs_meta.len());

        if let Ok(modified) = fs_meta.modified() {
            meta = meta.with_last_modified(modified);
        }

        if let Some(mime) = mime_type {
            meta = meta.with_mime_type(mime);
        }

        Ok(meta)
    }

    async fn url(&self, path: &str) -> Result<String, Error> {
        match &self.url_base {
            Some(base) => Ok(format!("{}/{}", base.trim_end_matches('/'), path)),
            None => Ok(self.full_path(path).to_string_lossy().to_string()),
        }
    }

    async fn temporary_url(
        &self,
        path: &str,
        _expiration: std::time::Duration,
    ) -> Result<String, Error> {
        // Local storage doesn't support temporary URLs, just return the regular URL
        self.url(path).await
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let full_path = self.full_path(directory);
        let mut files = Vec::new();

        if !full_path.exists() {
            return Ok(files);
        }

        let mut entries = fs::read_dir(&full_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let full_path = self.full_path(directory);
        let mut files = Vec::new();

        if !full_path.exists() {
            return Ok(files);
        }

        self.collect_files_recursive(&full_path, &full_path, &mut files)
            .await?;
        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>, Error> {
        let full_path = self.full_path(directory);
        let mut dirs = Vec::new();

        if !full_path.exists() {
            return Ok(dirs);
        }

        let mut entries = fs::read_dir(&full_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    dirs.push(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(dirs)
    }

    async fn make_directory(&self, path: &str) -> Result<(), Error> {
        let full_path = self.full_path(path);
        fs::create_dir_all(&full_path).await?;
        Ok(())
    }

    async fn delete_directory(&self, path: &str) -> Result<(), Error> {
        let full_path = self.full_path(path);
        fs::remove_dir_all(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(path)
            } else {
                Error::from(e)
            }
        })
    }
}

impl LocalDriver {
    #[allow(clippy::only_used_in_recursion)]
    fn collect_files_recursive<'a>(
        &'a self,
        base: &'a Path,
        current: &'a Path,
        files: &'a mut Vec<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = fs::read_dir(current).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(relative) = path.strip_prefix(base) {
                        files.push(relative.to_string_lossy().to_string());
                    }
                } else if path.is_dir() {
                    self.collect_files_recursive(base, &path, files).await?;
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_driver_put_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path());

        driver
            .put("test.txt", Bytes::from("hello world"), PutOptions::new())
            .await
            .unwrap();

        let contents = driver.get("test.txt").await.unwrap();
        assert_eq!(contents, Bytes::from("hello world"));
    }

    #[tokio::test]
    async fn test_local_driver_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path());

        assert!(!driver.exists("missing.txt").await.unwrap());

        driver
            .put("exists.txt", Bytes::from("data"), PutOptions::new())
            .await
            .unwrap();

        assert!(driver.exists("exists.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_local_driver_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path());

        driver
            .put("to_delete.txt", Bytes::from("data"), PutOptions::new())
            .await
            .unwrap();

        driver.delete("to_delete.txt").await.unwrap();
        assert!(!driver.exists("to_delete.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_local_driver_copy() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path());

        driver
            .put(
                "original.txt",
                Bytes::from("original content"),
                PutOptions::new(),
            )
            .await
            .unwrap();

        driver.copy("original.txt", "copy.txt").await.unwrap();

        let contents = driver.get("copy.txt").await.unwrap();
        assert_eq!(contents, Bytes::from("original content"));
    }

    #[tokio::test]
    async fn test_local_driver_nested_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path());

        driver
            .put(
                "a/b/c/deep.txt",
                Bytes::from("deep content"),
                PutOptions::new(),
            )
            .await
            .unwrap();

        let contents = driver.get("a/b/c/deep.txt").await.unwrap();
        assert_eq!(contents, Bytes::from("deep content"));
    }

    #[tokio::test]
    async fn test_local_driver_url() {
        let temp_dir = tempfile::tempdir().unwrap();
        let driver = LocalDriver::new(temp_dir.path()).with_url_base("https://example.com/storage");

        let url = driver.url("images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://example.com/storage/images/photo.jpg");
    }
}

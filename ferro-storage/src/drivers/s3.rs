//! S3 storage driver using aws-sdk-s3.
//!
//! This module is only compiled when the `s3` feature is enabled.

use crate::storage::{FileMetadata, PutOptions, StorageDriver, Visibility};
use crate::Error;
use async_trait::async_trait;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Region};
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectCannedAcl, ObjectIdentifier};
use bytes::Bytes;
use std::time::SystemTime;

/// S3-compatible storage driver.
pub struct S3Driver {
    client: aws_sdk_s3::Client,
    bucket: String,
    region: String,
    url_base: Option<String>,
}

impl S3Driver {
    /// Create a new S3 driver.
    ///
    /// Reads `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` from environment.
    /// When `endpoint_url` is `Some`, enables path-style addressing (for MinIO / R2).
    pub fn new(
        bucket: String,
        region: String,
        url_base: Option<String>,
        endpoint_url: Option<String>,
    ) -> Self {
        let key_id = std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default();
        let secret = std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default();

        let creds = aws_credential_types::Credentials::from_keys(key_id, secret, None);

        let mut config_builder = S3ConfigBuilder::new()
            .behavior_version_latest()
            .region(Region::new(region.clone()))
            .credentials_provider(creds);

        if let Some(endpoint) = endpoint_url {
            config_builder = config_builder.endpoint_url(endpoint).force_path_style(true);
        }

        let client = aws_sdk_s3::Client::from_conf(config_builder.build());

        Self {
            client,
            bucket,
            region,
            url_base,
        }
    }
}

/// Strip leading slash from a path.
fn normalize_path(path: &str) -> &str {
    path.trim_start_matches('/')
}

/// Normalize a directory prefix: strip leading slash and trailing slash, then append a single trailing slash.
fn normalize_dir_prefix(dir: &str) -> String {
    let d = dir.trim_matches('/');
    if d.is_empty() {
        String::new()
    } else {
        format!("{d}/")
    }
}

/// Check whether an SDK error represents a 404 (object not found).
///
/// HEAD requests have no body, so the SDK may not return a typed `NoSuchKey`
/// variant. Fall back to the raw HTTP 404 status code.
fn is_not_found<E>(e: &aws_sdk_s3::error::SdkError<E>) -> bool
where
    E: std::error::Error + aws_sdk_s3::error::ProvideErrorMetadata + 'static,
{
    let typed_404 = e
        .as_service_error()
        .map(|se| se.code() == Some("NoSuchKey") || se.code() == Some("NotFound"))
        .unwrap_or(false);
    let raw_404 = e
        .raw_response()
        .map(|r| r.status().as_u16() == 404)
        .unwrap_or(false);
    typed_404 || raw_404
}

#[async_trait]
impl StorageDriver for S3Driver {
    async fn exists(&self, path: &str) -> Result<bool, Error> {
        let key = normalize_path(path);
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if is_not_found(&e) {
                    Ok(false)
                } else {
                    Err(Error::S3(e.to_string()))
                }
            }
        }
    }

    async fn get(&self, path: &str) -> Result<Bytes, Error> {
        let key = normalize_path(path);
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                if e.as_service_error()
                    .map(|se| se.code() == Some("NoSuchKey"))
                    .unwrap_or(false)
                {
                    Error::not_found(path)
                } else {
                    Error::S3(e.to_string())
                }
            })?;

        let bytes = output
            .body
            .collect()
            .await
            .map_err(|e| Error::S3(e.to_string()))?
            .into_bytes();

        Ok(bytes)
    }

    async fn put(&self, path: &str, contents: Bytes, options: PutOptions) -> Result<(), Error> {
        let key = normalize_path(path);
        let body = ByteStream::from(contents);

        let content_type = options
            .content_type
            .clone()
            .or_else(|| mime_guess::from_path(path).first().map(|m| m.to_string()));

        let mut req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body);

        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }

        if options.visibility == Visibility::Public {
            req = req.acl(ObjectCannedAcl::PublicRead);
        }

        req.send().await.map_err(|e| Error::S3(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<(), Error> {
        let key = normalize_path(path);
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| Error::S3(e.to_string()))?;
        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        let from_key = normalize_path(from);
        let to_key = normalize_path(to);
        let copy_source = format!("{}/{}", self.bucket, from_key);

        self.client
            .copy_object()
            .copy_source(copy_source)
            .bucket(&self.bucket)
            .key(to_key)
            .send()
            .await
            .map_err(|e| Error::S3(e.to_string()))?;
        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64, Error> {
        let key = normalize_path(path);
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => Ok(output.content_length().unwrap_or(0) as u64),
            Err(e) => {
                if is_not_found(&e) {
                    Err(Error::not_found(path))
                } else {
                    Err(Error::S3(e.to_string()))
                }
            }
        }
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata, Error> {
        let key = normalize_path(path);
        let output = match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                if is_not_found(&e) {
                    return Err(Error::not_found(path));
                }
                return Err(Error::S3(e.to_string()));
            }
        };

        let size = output.content_length().unwrap_or(0) as u64;
        let mut meta = FileMetadata::new(path, size);

        if let Some(lm) = output.last_modified() {
            // Convert aws_sdk_s3::primitives::DateTime to SystemTime
            let secs = lm.secs();
            let system_time = if secs >= 0 {
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64)
            } else {
                SystemTime::UNIX_EPOCH
            };
            meta = meta.with_last_modified(system_time);
        }

        if let Some(ct) = output.content_type() {
            meta = meta.with_mime_type(ct);
        }

        Ok(meta)
    }

    async fn url(&self, path: &str) -> Result<String, Error> {
        let key = normalize_path(path);
        match &self.url_base {
            Some(base) => Ok(format!("{}/{}", base.trim_end_matches('/'), key)),
            None => Ok(format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, key
            )),
        }
    }

    async fn temporary_url(
        &self,
        path: &str,
        expiration: std::time::Duration,
    ) -> Result<String, Error> {
        let key = normalize_path(path);
        let presigning_config = PresigningConfig::builder()
            .expires_in(expiration)
            .build()
            .map_err(|e| Error::S3(e.to_string()))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| Error::S3(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let prefix = normalize_dir_prefix(directory);
        let mut paginator = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .delimiter("/")
            .into_paginator()
            .send();

        let mut keys = Vec::new();
        while let Some(page) = paginator.next().await {
            let page = page.map_err(|e| Error::S3(e.to_string()))?;
            for obj in page.contents() {
                if let Some(key) = obj.key() {
                    keys.push(key.to_string());
                }
            }
        }

        Ok(keys)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>, Error> {
        let prefix = normalize_dir_prefix(directory);
        let mut paginator = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .into_paginator()
            .send();

        let mut keys = Vec::new();
        while let Some(page) = paginator.next().await {
            let page = page.map_err(|e| Error::S3(e.to_string()))?;
            for obj in page.contents() {
                if let Some(key) = obj.key() {
                    keys.push(key.to_string());
                }
            }
        }

        Ok(keys)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>, Error> {
        let prefix = normalize_dir_prefix(directory);
        let mut paginator = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .delimiter("/")
            .into_paginator()
            .send();

        let mut dirs = Vec::new();
        while let Some(page) = paginator.next().await {
            let page = page.map_err(|e| Error::S3(e.to_string()))?;
            for cp in page.common_prefixes() {
                if let Some(p) = cp.prefix() {
                    dirs.push(p.trim_end_matches('/').to_string());
                }
            }
        }

        Ok(dirs)
    }

    async fn make_directory(&self, path: &str) -> Result<(), Error> {
        let prefix = normalize_dir_prefix(path);
        let key = format!("{prefix}.keep");

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(Bytes::new()))
            .send()
            .await
            .map_err(|e| Error::S3(e.to_string()))?;

        Ok(())
    }

    async fn delete_directory(&self, path: &str) -> Result<(), Error> {
        let prefix = normalize_dir_prefix(path);

        // List all objects under the prefix
        let mut paginator = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .into_paginator()
            .send();

        let mut all_keys: Vec<String> = Vec::new();
        while let Some(page) = paginator.next().await {
            let page = page.map_err(|e| Error::S3(e.to_string()))?;
            for obj in page.contents() {
                if let Some(key) = obj.key() {
                    all_keys.push(key.to_string());
                }
            }
        }

        if all_keys.is_empty() {
            return Ok(());
        }

        // Batch delete in chunks of 1000
        for chunk in all_keys.chunks(1000) {
            let identifiers: Result<Vec<_>, _> = chunk
                .iter()
                .map(|key| {
                    ObjectIdentifier::builder()
                        .key(key)
                        .build()
                        .map_err(|e| Error::S3(e.to_string()))
                })
                .collect();

            let delete = Delete::builder()
                .set_objects(Some(identifiers?))
                .build()
                .map_err(|e| Error::S3(e.to_string()))?;

            self.client
                .delete_objects()
                .bucket(&self.bucket)
                .delete(delete)
                .send()
                .await
                .map_err(|e| Error::S3(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo/bar.txt"), "foo/bar.txt");
        assert_eq!(normalize_path("/foo/bar.txt"), "foo/bar.txt");
        assert_eq!(normalize_path("//foo"), "foo");
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn test_normalize_dir_prefix() {
        assert_eq!(normalize_dir_prefix("photos"), "photos/");
        assert_eq!(normalize_dir_prefix("/photos"), "photos/");
        assert_eq!(normalize_dir_prefix("photos/"), "photos/");
        assert_eq!(normalize_dir_prefix(""), "");
        assert_eq!(normalize_dir_prefix("/"), "");
    }

    /// Build an S3Driver with dummy credentials for unit tests that don't make network calls.
    fn dummy_driver(url_base: Option<&str>) -> S3Driver {
        // Use a dummy endpoint so no real connections can be made.
        S3Driver::new(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            url_base.map(|s| s.to_string()),
            Some("http://localhost:19999".to_string()),
        )
    }

    #[tokio::test]
    async fn test_url_with_base() {
        let driver = dummy_driver(Some("https://cdn.example.com"));
        let url = driver.url("images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn test_url_with_base_trailing_slash() {
        let driver = dummy_driver(Some("https://cdn.example.com/"));
        let url = driver.url("images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }

    #[tokio::test]
    async fn test_url_fallback() {
        let driver = dummy_driver(None);
        let url = driver.url("images/photo.jpg").await.unwrap();
        assert_eq!(
            url,
            "https://test-bucket.s3.us-east-1.amazonaws.com/images/photo.jpg"
        );
    }

    #[tokio::test]
    async fn test_url_strips_leading_slash() {
        let driver = dummy_driver(Some("https://cdn.example.com"));
        let url = driver.url("/images/photo.jpg").await.unwrap();
        assert_eq!(url, "https://cdn.example.com/images/photo.jpg");
    }
}

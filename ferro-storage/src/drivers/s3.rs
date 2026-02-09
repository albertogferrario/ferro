//! S3 storage driver (placeholder).
//!
//! This module is only compiled when the `s3` feature is enabled.
//! All methods return `Error::NotImplemented` until actual S3 integration is built.

use crate::storage::{FileMetadata, PutOptions, StorageDriver};
use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;

/// S3-compatible storage driver.
pub struct S3Driver;

#[async_trait]
impl StorageDriver for S3Driver {
    async fn exists(&self, _path: &str) -> Result<bool, Error> {
        Err(Error::not_implemented("S3 driver: exists"))
    }

    async fn get(&self, _path: &str) -> Result<Bytes, Error> {
        Err(Error::not_implemented("S3 driver: get"))
    }

    async fn put(&self, _path: &str, _contents: Bytes, _options: PutOptions) -> Result<(), Error> {
        Err(Error::not_implemented("S3 driver: put"))
    }

    async fn delete(&self, _path: &str) -> Result<(), Error> {
        Err(Error::not_implemented("S3 driver: delete"))
    }

    async fn copy(&self, _from: &str, _to: &str) -> Result<(), Error> {
        Err(Error::not_implemented("S3 driver: copy"))
    }

    async fn size(&self, _path: &str) -> Result<u64, Error> {
        Err(Error::not_implemented("S3 driver: size"))
    }

    async fn metadata(&self, _path: &str) -> Result<FileMetadata, Error> {
        Err(Error::not_implemented("S3 driver: metadata"))
    }

    async fn url(&self, _path: &str) -> Result<String, Error> {
        Err(Error::not_implemented("S3 driver: url"))
    }

    async fn temporary_url(
        &self,
        _path: &str,
        _expiration: std::time::Duration,
    ) -> Result<String, Error> {
        Err(Error::not_implemented("S3 driver: temporary_url"))
    }

    async fn files(&self, _directory: &str) -> Result<Vec<String>, Error> {
        Err(Error::not_implemented("S3 driver: files"))
    }

    async fn all_files(&self, _directory: &str) -> Result<Vec<String>, Error> {
        Err(Error::not_implemented("S3 driver: all_files"))
    }

    async fn directories(&self, _directory: &str) -> Result<Vec<String>, Error> {
        Err(Error::not_implemented("S3 driver: directories"))
    }

    async fn make_directory(&self, _path: &str) -> Result<(), Error> {
        Err(Error::not_implemented("S3 driver: make_directory"))
    }

    async fn delete_directory(&self, _path: &str) -> Result<(), Error> {
        Err(Error::not_implemented("S3 driver: delete_directory"))
    }
}

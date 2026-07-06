//! Error types for storage operations.

use std::io;
use thiserror::Error;

/// Storage error types.
#[derive(Error, Debug)]
pub enum Error {
    /// File not found.
    #[error("File not found: {0}")]
    NotFound(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Invalid path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Disk not configured.
    #[error("Disk not configured: {0}")]
    DiskNotConfigured(String),

    /// S3 error.
    #[cfg(feature = "s3")]
    #[error("S3 error: {0}")]
    S3(String),

    /// Feature not yet implemented.
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// CDN operation error.
    #[error("CDN error: {0}")]
    Cdn(String),

    /// CDN provider name is not recognized.
    #[error("CDN_PROVIDER value '{0}' is not valid; valid values: none, digitalocean, bunny, cloudflare")]
    CdnInvalidProvider(String),

    /// Selected CDN provider requires a cargo feature that is not enabled.
    #[error("CDN_PROVIDER={0} requires the '{1}' cargo feature")]
    CdnFeatureRequired(String, &'static str),
}

impl Error {
    /// Create a not found error.
    pub fn not_found(path: impl Into<String>) -> Self {
        Self::NotFound(path.into())
    }

    /// Create a permission denied error.
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an invalid path error.
    pub fn invalid_path(path: impl Into<String>) -> Self {
        Self::InvalidPath(path.into())
    }

    /// Create a disk not configured error.
    pub fn disk_not_configured(disk: impl Into<String>) -> Self {
        Self::DiskNotConfigured(disk.into())
    }

    /// Create a not implemented error.
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented(feature.into())
    }

    /// Create a CDN error.
    pub fn cdn(msg: impl Into<String>) -> Self {
        Self::Cdn(msg.into())
    }

    /// Create a CDN invalid provider error.
    pub fn cdn_invalid_provider(val: impl Into<String>) -> Self {
        Self::CdnInvalidProvider(val.into())
    }

    /// Create a CDN feature-required error.
    pub fn cdn_feature_required(provider: &str, feature: &'static str) -> Self {
        Self::CdnFeatureRequired(provider.to_string(), feature)
    }
}

//! Error types for the queue system.

use thiserror::Error;

/// Errors that can occur in the queue system.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to connect to the queue backend.
    #[error("Queue connection failed: {0}")]
    ConnectionFailed(String),

    /// Failed to serialize a job.
    #[error("Failed to serialize job: {0}")]
    SerializationFailed(String),

    /// Failed to deserialize a job.
    #[error("Failed to deserialize job: {0}")]
    DeserializationFailed(String),

    /// Failed to push a job to the queue.
    #[error("Failed to push job to queue '{queue}': {message}")]
    PushFailed {
        /// The queue name.
        queue: String,
        /// The error message.
        message: String,
    },

    /// Failed to pop a job from the queue.
    #[error("Failed to pop job from queue '{queue}': {message}")]
    PopFailed {
        /// The queue name.
        queue: String,
        /// The error message.
        message: String,
    },

    /// Job execution failed.
    #[error("Job '{job}' failed: {message}")]
    JobFailed {
        /// The job name.
        job: String,
        /// The error message.
        message: String,
    },

    /// Maximum retry attempts exceeded.
    #[error("Job '{job}' exceeded maximum retries ({max_retries})")]
    MaxRetriesExceeded {
        /// The job name.
        job: String,
        /// Maximum retry count.
        max_retries: u32,
    },

    /// Database error.
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// Unsupported database backend.
    #[error("Unsupported database backend")]
    UnsupportedBackend,

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Tenant not found when trying to establish tenant scope for a job.
    #[error("Tenant not found for job: tenant_id={tenant_id}")]
    TenantNotFound {
        /// The tenant ID that could not be resolved.
        tenant_id: i64,
    },

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a job failed error.
    pub fn job_failed(job: impl Into<String>, message: impl Into<String>) -> Self {
        Self::JobFailed {
            job: job.into(),
            message: message.into(),
        }
    }

    /// Create a push failed error.
    pub fn push_failed(queue: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PushFailed {
            queue: queue.into(),
            message: message.into(),
        }
    }

    /// Create a tenant not found error.
    pub fn tenant_not_found(id: i64) -> Self {
        Self::TenantNotFound { tenant_id: id }
    }

    /// Create a custom error.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_not_found_formats_with_id() {
        let err = Error::tenant_not_found(42);
        assert_eq!(err.to_string(), "Tenant not found for job: tenant_id=42");
    }

    #[test]
    fn test_tenant_not_found_constructor() {
        let err = Error::tenant_not_found(99);
        assert!(matches!(err, Error::TenantNotFound { tenant_id: 99 }));
    }
}

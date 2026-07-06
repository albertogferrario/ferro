//! Error types for the deployments system.

use thiserror::Error;

/// Errors that can occur in the deployments system.
#[derive(Debug, Error)]
pub enum Error {
    /// Database error.
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// Unsupported database backend.
    #[error("Unsupported database backend")]
    UnsupportedBackend,

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Deployment is not in the ready state and cannot be promoted.
    #[error("Deployment {id} cannot be promoted: status is not ready")]
    NotReady {
        /// The deployment ID.
        id: i64,
    },

    /// Deployment artifacts have been deleted; promotion refused.
    #[error("Deployment {id} cannot be promoted: artifact has been deleted")]
    ArtifactDeleted {
        /// The deployment ID.
        id: i64,
    },

    /// No previous deployment exists for rollback.
    #[error("No previous deployment to roll back to for owner_key '{owner_key}'")]
    NoPreviousDeployment {
        /// The owner key.
        owner_key: String,
    },

    /// Deployment not found.
    #[error("Deployment {id} not found")]
    NotFound {
        /// The deployment ID.
        id: i64,
    },

    /// Storage error.
    #[error("Storage error: {0}")]
    Storage(#[from] ferro_storage::Error),

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

impl Error {
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

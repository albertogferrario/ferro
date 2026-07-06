//! Error type for ferro-migration helpers.

use sea_orm::DbErr;
use thiserror::Error;

/// Error variants for ferro-migration backfill helpers.
#[derive(Debug, Error)]
pub enum Error {
    /// The database backend is not supported by this helper.
    #[error("unsupported backend: {0}")]
    UnsupportedBackend(String),
}

impl From<Error> for DbErr {
    fn from(e: Error) -> Self {
        DbErr::Custom(e.to_string())
    }
}

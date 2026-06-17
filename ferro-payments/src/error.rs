//! Error types for the ferro-payments data layer.

/// Errors that can occur in ferro-payments data layer operations.
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    /// The PaymentIntent or billable entity was not found.
    #[error("payment: not found")]
    NotFound,

    /// A state-transition was attempted from an invalid source status.
    /// Contains a human-readable description of the precondition that failed.
    #[error("payment: status precondition not met: {0}")]
    StatusPrecondition(String),

    /// Underlying database error.
    #[error("payment: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

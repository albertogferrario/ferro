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

    /// A Stripe API call failed.
    #[error("payment: stripe error: {0}")]
    Stripe(#[from] ferro_stripe::Error),

    /// A consumer-side `BillableLoader` failure. No `#[from]` — the consumer wraps
    /// the source error manually via `PaymentError::Loader(Box::new(err))`.
    #[error("payment: loader error: {0}")]
    Loader(Box<dyn std::error::Error + Send + Sync>),

    /// The payment was charged but could not be reconciled to a billable; an
    /// auto-refund was triggered. Defined here (D-18); only RETURNED by the webhook
    /// handlers in phase 235.
    #[error("payment: auto-refund triggered: {reason:?}")]
    AutoRefundTriggered { reason: AutoRefundReason },
}

/// Why an auto-refund was triggered. Returned inside `PaymentError::AutoRefundTriggered`
/// by the phase-235 webhook handlers; defined in phase 234 so the error type is stable
/// across both phases.
#[derive(Debug)]
pub enum AutoRefundReason {
    /// `BillableLoader::load` returned `Err`.
    LoaderError,
    /// `BillableLoader::load` returned `Ok(None)` — the billable was deleted.
    BillableVanished,
    /// The billable's side state already advanced (e.g. slot released before pay).
    SideStateConflict,
}

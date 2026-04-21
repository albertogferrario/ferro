/// Errors that can occur in ferro-stripe operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("stripe config error: {0}")]
    Config(String),

    /// Stripe API returned an error.
    #[error("stripe API error: {0}")]
    Stripe(String),

    /// No Connect account linked to this tenant.
    #[error("no Stripe Connect account linked to this tenant")]
    NoConnectAccount,

    /// Webhook signature verification failed.
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),

    /// Event was already processed (idempotency guard).
    #[error("stripe event already processed: {0}")]
    EventAlreadyProcessed(String),

    /// Idempotency key not set on CheckoutBuilder before calling create().
    #[error("idempotency key required: call .idempotency_key() before .create()")]
    MissingIdempotencyKey,
}

impl From<stripe::StripeError> for Error {
    fn from(e: stripe::StripeError) -> Self {
        Error::Stripe(e.to_string())
    }
}

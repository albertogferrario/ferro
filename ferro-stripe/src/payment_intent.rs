//! Payment intent capture and cancel operations.
//!
//! Thin capability-axis wrappers over the `stripe::PaymentIntent` API, mirroring
//! [`crate::refund`]. Used with [`crate::checkout::CheckoutBuilder::manual_capture`]
//! to charge or release a previously authorized PaymentIntent.
//!
//! The authorize/capture/cancel triple corresponds to `ferro-reservation`'s
//! hold/commit/release — a documented semantic parallel, not a compile dependency.

use crate::Error;

/// Captures a previously authorized PaymentIntent.
///
/// - `amount_cents: None` captures the full authorized amount.
/// - `amount_cents: Some(n)` captures `n` cents (partial capture); the remainder
///   is auto-released by Stripe. `n` must be positive.
///
/// NOTE: async-stripe 0.41 does not forward a per-request idempotency key to
/// `PaymentIntent::capture`. Stripe-layer deduplication is NOT guaranteed.
/// Application-layer deduplication (e.g. a DB unique constraint) is required to
/// prevent a double-capture on retry. Same caveat as [`crate::refund::create`].
pub async fn capture(
    payment_intent_id: &str,
    amount_cents: Option<i64>,
) -> Result<stripe::PaymentIntent, Error> {
    let _id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let amount_to_capture = match amount_cents {
        None => None,
        Some(n) => Some(
            u64::try_from(n)
                .map_err(|_| Error::Stripe("amount_to_capture must be positive".to_string()))?,
        ),
    };
    let client = crate::Stripe::client();
    let params = stripe::CapturePaymentIntent {
        amount_to_capture,
        ..Default::default()
    };
    let pi = stripe::PaymentIntent::capture(client, payment_intent_id, params).await?;
    Ok(pi)
}

/// Cancels (releases) a previously authorized PaymentIntent.
pub async fn cancel(payment_intent_id: &str) -> Result<stripe::PaymentIntent, Error> {
    let _id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let client = crate::Stripe::client();
    let params = stripe::CancelPaymentIntent::default();
    let pi = stripe::PaymentIntent::cancel(client, payment_intent_id, params).await?;
    Ok(pi)
}

/// Retrieves a PaymentIntent by id (e.g. to poll authorization state).
pub async fn retrieve(payment_intent_id: &str) -> Result<stripe::PaymentIntent, Error> {
    let id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let client = crate::Stripe::client();
    let pi = stripe::PaymentIntent::retrieve(client, &id, &[]).await?;
    Ok(pi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_rejects_invalid_id_before_network() {
        let result = capture("not-a-valid-pi-id", None).await;
        assert!(
            matches!(result, Err(Error::Stripe(ref m)) if m.contains("invalid payment intent id")),
            "expected invalid-id error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn capture_rejects_negative_amount() {
        let result = capture("pi_test_123", Some(-5)).await;
        assert!(
            matches!(result, Err(Error::Stripe(ref m)) if m.contains("must be positive")),
            "expected positive-amount error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn cancel_rejects_invalid_id_before_network() {
        let result = cancel("bad id with spaces").await;
        assert!(
            matches!(result, Err(Error::Stripe(ref m)) if m.contains("invalid payment intent id")),
            "expected invalid-id error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn retrieve_rejects_invalid_id_before_network() {
        let result = retrieve("").await;
        assert!(
            matches!(result, Err(Error::Stripe(ref m)) if m.contains("invalid payment intent id")),
            "expected invalid-id error, got {result:?}"
        );
    }
}

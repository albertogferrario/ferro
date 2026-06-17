//! Refund operations over Stripe charges.
//!
//! Thin capability-axis wrappers over the `stripe::Refund` API.

use crate::Error;

/// Creates a refund for the given charge.
///
/// - `amount_cents: None` issues a full refund; `Some(n)` issues a partial refund of `n` cents.
/// - `idempotency_key` SHOULD be a deterministic string per logical refund so that
///   retries do not produce duplicate refunds.
///
///   NOTE: async-stripe 0.41 does not forward this key to the Stripe API.
///   Stripe-layer deduplication is NOT guaranteed until this crate upgrades.
///   Application-layer deduplication (e.g. a DB unique constraint on charge_id)
///   is required to prevent duplicate refunds on retry.
/// - `reason` is optional; when `None`, Stripe omits the field.
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    // Note: async-stripe 0.41 does not expose a per-request idempotency-key strategy
    // on Refund::create. The key is accepted here for API surface consistency and
    // should be passed through when a future version exposes the mechanism.
    let _ = idempotency_key;
    let client = crate::Stripe::client();

    let mut params = stripe::CreateRefund::new();
    let charge: stripe::ChargeId = charge_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid charge id: {charge_id}")))?;
    params.charge = Some(charge);
    params.amount = amount_cents;
    params.reason = reason;

    let refund = stripe::Refund::create(client, params).await?;
    Ok(refund)
}

/// Creates a refund by `payment_intent_id`.
///
/// Used by the auto-refund path in `ferro-payments` when a
/// `checkout.session.completed` event carries no `charge_id` (the event exposes
/// only `payment_intent_id`). Mirrors [`create`] exactly — substitutes
/// `params.payment_intent` for `params.charge`.
///
/// - `amount_cents: None` issues a full refund; `Some(n)` issues a partial refund of `n` cents.
/// - `idempotency_key` SHOULD be a deterministic string per logical refund so that
///   retries do not produce duplicate refunds.
///
///   NOTE: same async-stripe 0.41 caveat as [`create`] — the `idempotency_key` is
///   NOT forwarded to the Stripe API, so callers must dedup at the application
///   layer (the ferro-payments auto-refund path snapshots `refund_amount_cents`
///   under a `WHERE refund_amount_cents IS NULL` guard before calling this).
/// - `reason` is optional; when `None`, Stripe omits the field.
pub async fn create_for_payment_intent(
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    // Note: async-stripe 0.41 does not expose a per-request idempotency-key strategy
    // on Refund::create. The key is accepted here for API surface consistency and
    // should be passed through when a future version exposes the mechanism.
    let _ = idempotency_key;
    let client = crate::Stripe::client();

    let mut params = stripe::CreateRefund::new();
    let pi_id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    params.payment_intent = Some(pi_id);
    params.amount = amount_cents;
    params.reason = reason;

    let refund = stripe::Refund::create(client, params).await?;
    Ok(refund)
}

/// Retrieves a refund by id.
pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error> {
    let client = crate::Stripe::client();
    let id: stripe::RefundId = refund_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid refund id: {refund_id}")))?;
    let refund = stripe::Refund::retrieve(client, &id, &[]).await?;
    Ok(refund)
}

#[cfg(test)]
mod tests {
    #[test]
    fn invalid_payment_intent_id_does_not_parse() {
        // Guards create_for_payment_intent's early-return Err path without a network call.
        assert!("not_a_pi".parse::<stripe::PaymentIntentId>().is_err());
    }
}

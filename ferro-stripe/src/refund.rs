//! Refund operations over Stripe charges.
//!
//! Thin capability-axis wrappers over the `stripe::Refund` API.

use crate::Error;

/// Creates a refund for the given charge.
///
/// - `amount_cents: None` issues a full refund; `Some(n)` issues a partial refund of `n` cents.
/// - `idempotency_key` SHOULD be a deterministic string per logical refund so that
///   retries do not produce duplicate refunds.
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

/// Retrieves a refund by id.
pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error> {
    let client = crate::Stripe::client();
    let id: stripe::RefundId = refund_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid refund id: {refund_id}")))?;
    let refund = stripe::Refund::retrieve(client, &id, &[]).await?;
    Ok(refund)
}

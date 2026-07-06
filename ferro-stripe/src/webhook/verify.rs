//! Stripe webhook signature verification.
//!
//! Verification is implemented directly against the Stripe signing protocol
//! (HMAC-SHA256 over `"{timestamp}.{body}"`) rather than delegating to
//! `async-stripe`'s `construct_event`. The latter both verifies the signature
//! *and* deserializes the payload into version-pinned object structs — so an
//! event rendered at a newer Stripe API version (a field that changed type,
//! a new required field, an unknown enum variant) fails to deserialize and is
//! rejected as if the signature were invalid. Decoupling verification from
//! object typing makes the webhook path forward-compatible across API
//! versions; the returned [`WebhookEvent`] carries `data.object` as untyped
//! JSON for handlers to read.

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::webhook::events::WebhookEvent;
use crate::Error;

/// Max age (seconds) for the signature timestamp — Stripe's recommended
/// default tolerance, protecting against replayed payloads.
const SIGNATURE_TOLERANCE_SECS: i64 = 300;

/// Verifies a Stripe webhook signature and parses the event envelope.
///
/// Uses HMAC-SHA256 as per the Stripe webhook verification protocol, then
/// parses the JSON envelope into a [`WebhookEvent`]. The event's
/// `data.object` is kept as untyped JSON — see the module docs for why.
///
/// # Errors
///
/// Returns [`Error::WebhookVerification`] when:
/// - The signature header is malformed (missing `t=` or `v1=`)
/// - The timestamp is more than 5 minutes from now
/// - No `v1` signature matches the computed HMAC
/// - The verified body is not a valid event envelope
pub fn verify_webhook(
    raw_body: &str,
    signature: &str,
    secret: &str,
) -> Result<WebhookEvent, Error> {
    let (timestamp, candidate_sigs) = parse_signature_header(signature)?;

    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > SIGNATURE_TOLERANCE_SECS {
        return Err(Error::WebhookVerification(
            "timestamp outside tolerance".into(),
        ));
    }

    let signed_payload = format!("{timestamp}.{raw_body}");
    let matched = candidate_sigs.iter().any(|sig_hex| {
        let Ok(sig_bytes) = hex::decode(sig_hex) else {
            return false;
        };
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(signed_payload.as_bytes());
        // `verify_slice` is a constant-time comparison.
        mac.verify_slice(&sig_bytes).is_ok()
    });
    if !matched {
        return Err(Error::WebhookVerification(
            "no matching v1 signature".into(),
        ));
    }

    WebhookEvent::from_json(raw_body)
}

/// Parse a Stripe `Stripe-Signature` header into `(timestamp, [v1 signatures])`.
///
/// Format: `t=1700000000,v1=abc...,v1=def...` (multiple `v1` during secret
/// rotation). Unknown scheme keys (e.g. `v0`) are ignored.
fn parse_signature_header(header: &str) -> Result<(i64, Vec<String>), Error> {
    let mut timestamp: Option<i64> = None;
    let mut v1: Vec<String> = Vec::new();
    for part in header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next().map(str::trim), kv.next().map(str::trim)) {
            (Some("t"), Some(val)) => timestamp = val.parse::<i64>().ok(),
            (Some("v1"), Some(val)) => v1.push(val.to_string()),
            _ => {}
        }
    }
    let timestamp = timestamp.ok_or_else(|| {
        Error::WebhookVerification("missing or invalid timestamp in signature header".into())
    })?;
    if v1.is_empty() {
        return Err(Error::WebhookVerification(
            "missing v1 signature in header".into(),
        ));
    }
    Ok((timestamp, v1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::signed_webhook_payload;

    const TEST_SECRET: &str = "whsec_test_secret";

    fn minimal_event_json(event_type: &str) -> String {
        serde_json::json!({
            "id": "evt_test_001",
            "object": "event",
            "api_version": "2023-10-16",
            "created": 1533204620,
            "data": {
                "object": {
                    "id": "ii_123",
                    "object": "invoiceitem",
                    "amount": 1000,
                    "currency": "usd",
                    "customer": "cus_123",
                    "date": 1533204620,
                    "description": "Test Invoice Item",
                    "discountable": false,
                    "invoice": "in_123",
                    "livemode": false,
                    "metadata": {},
                    "period": { "start": 1533204620, "end": 1533204620 },
                    "proration": false,
                    "quantity": 1
                }
            },
            "livemode": false,
            "pending_webhooks": 1,
            "request": null,
            "type": event_type
        })
        .to_string()
    }

    #[test]
    fn verify_webhook_with_valid_signature_returns_ok() {
        let payload = minimal_event_json("invoiceitem.created");
        let (sig, _) = signed_webhook_payload(&payload, TEST_SECRET);
        let result = verify_webhook(&payload, &sig, TEST_SECRET);
        assert!(result.is_ok(), "expected Ok but got: {result:?}");
    }

    #[test]
    fn verify_webhook_with_tampered_body_returns_err() {
        let payload = minimal_event_json("invoiceitem.created");
        let (sig, _) = signed_webhook_payload(&payload, TEST_SECRET);
        let tampered = payload.replace("invoiceitem.created", "invoice.paid");
        let result = verify_webhook(&tampered, &sig, TEST_SECRET);
        assert!(
            matches!(result, Err(Error::WebhookVerification(_))),
            "expected WebhookVerification error but got: {result:?}"
        );
    }

    #[test]
    fn verify_webhook_with_wrong_secret_returns_err() {
        let payload = minimal_event_json("invoiceitem.created");
        let (sig, _) = signed_webhook_payload(&payload, TEST_SECRET);
        let result = verify_webhook(&payload, &sig, "whsec_wrong_secret");
        assert!(
            matches!(result, Err(Error::WebhookVerification(_))),
            "expected WebhookVerification error but got: {result:?}"
        );
    }
}

pub mod events;
pub mod handler;

use crate::Error;

/// Verifies a Stripe webhook signature and parses the event payload.
///
/// Uses HMAC-SHA256 as per the Stripe webhook verification protocol.
/// Returns the parsed [`stripe::Event`] on success.
///
/// # Errors
///
/// Returns [`Error::WebhookVerification`] when:
/// - The signature header is malformed
/// - The HMAC does not match
/// - The timestamp is more than 5 minutes old
pub fn verify_webhook(
    raw_body: &str,
    signature: &str,
    secret: &str,
) -> Result<stripe::Event, Error> {
    stripe::Webhook::construct_event(raw_body, signature, secret)
        .map_err(|e| Error::WebhookVerification(e.to_string()))
}

/// Returns true if the given Stripe event ID has already been processed.
///
/// This is a stub implementation that always returns false.
/// Full idempotency requires a database check; implement this in your
/// event listener by checking your event log table.
///
/// # Example
///
/// ```rust
/// use ferro_stripe::webhook::is_processed;
///
/// assert!(!is_processed("evt_test_123"));
/// ```
pub fn is_processed(_event_id: &str) -> bool {
    // TODO: implement by checking a processed-events DB table in the application
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhook::events::signed_webhook_payload;

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

    #[test]
    fn is_processed_returns_false_for_unseen_ids() {
        assert!(!is_processed("evt_brand_new_id"));
        assert!(!is_processed("evt_test_123"));
        assert!(!is_processed(""));
    }
}

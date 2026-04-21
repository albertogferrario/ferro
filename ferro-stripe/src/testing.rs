//! Test helpers for ferro-stripe integration testing.
//!
//! Provides factory functions for generating signed webhook payloads and
//! typed event JSON strings compatible with the `verify_webhook` function.
//!
//! # Feature Gate
//!
//! This module is compiled only when the `test-helpers` feature is enabled or
//! when building for tests (`cfg(test)`).
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_stripe::testing::*;
//!
//! // Generate a signed webhook payload for testing
//! let event = mock_checkout_completed_event("cs_test", "cus_test");
//! let (sig, _ts) = signed_webhook_payload(&event, "whsec_test");
//! ```

use chrono::Utc;

/// Generates a `checkout.session.completed` event JSON string.
///
/// Returns a valid Stripe event envelope suitable for use with [`signed_webhook_payload`]
/// and [`crate::verify_webhook`].
pub fn mock_checkout_completed_event(session_id: &str, customer_id: &str) -> String {
    serde_json::json!({
        "id": "evt_mock_checkout_completed",
        "object": "event",
        "api_version": "2023-10-16",
        "created": Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": session_id,
                "object": "checkout.session",
                "customer": customer_id,
                "payment_status": "paid",
                "status": "complete",
                "created": 1700000000_i64,
                "expires_at": 1700086400_i64,
                "livemode": false,
                "mode": "payment",
                "payment_method_types": ["card"],
                "custom_fields": [],
                "custom_text": {
                    "after_submit": null,
                    "shipping_address": null,
                    "submit": null,
                    "terms_of_service_acceptance": null
                },
                "shipping_options": [],
                "automatic_tax": {
                    "enabled": false,
                    "status": null
                }
            }
        }
    })
    .to_string()
}

/// Generates a `customer.subscription.updated` event JSON string.
///
/// `status` should be a Stripe subscription status string such as `"active"` or `"past_due"`.
pub fn mock_subscription_updated_event(
    subscription_id: &str,
    customer_id: &str,
    status: &str,
) -> String {
    serde_json::json!({
        "id": "evt_mock_subscription_updated",
        "object": "event",
        "api_version": "2023-10-16",
        "created": Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "type": "customer.subscription.updated",
        "data": {
            "object": {
                "id": subscription_id,
                "object": "subscription",
                "customer": customer_id,
                "status": status
            }
        }
    })
    .to_string()
}

/// Generates a `customer.subscription.deleted` event JSON string.
pub fn mock_subscription_deleted_event(subscription_id: &str, customer_id: &str) -> String {
    serde_json::json!({
        "id": "evt_mock_subscription_deleted",
        "object": "event",
        "api_version": "2023-10-16",
        "created": Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "type": "customer.subscription.deleted",
        "data": {
            "object": {
                "id": subscription_id,
                "object": "subscription",
                "customer": customer_id,
                "status": "canceled"
            }
        }
    })
    .to_string()
}

/// Generates an `invoice.paid` event JSON string.
pub fn mock_invoice_paid_event(invoice_id: &str, customer_id: &str) -> String {
    serde_json::json!({
        "id": "evt_mock_invoice_paid",
        "object": "event",
        "api_version": "2023-10-16",
        "created": Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "type": "invoice.paid",
        "data": {
            "object": {
                "id": invoice_id,
                "object": "invoice",
                "customer": customer_id,
                "status": "paid",
                "amount_paid": 1000,
                "currency": "usd"
            }
        }
    })
    .to_string()
}

/// Generates a valid Stripe-signature header for testing webhook verification.
///
/// Returns `(signature_header, timestamp)` where signature_header is formatted
/// as `t={timestamp},v1={hmac_sha256}` and timestamp is the current Unix time.
///
/// # Example
///
/// ```rust,ignore
/// let (sig, ts) = signed_webhook_payload(r#"{"id":"evt_1"}"#, "whsec_secret");
/// let event = verify_webhook(r#"{"id":"evt_1"}"#, &sig, "whsec_secret");
/// assert!(event.is_ok());
/// ```
pub fn signed_webhook_payload(payload: &str, secret: &str) -> (String, i64) {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let timestamp = chrono::Utc::now().timestamp();
    let signed_payload = format!("{timestamp}.{payload}");

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signed_payload.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    let header = format!("t={timestamp},v1={signature}");
    (header, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify_webhook;

    #[test]
    fn signed_webhook_payload_round_trips_through_verify_webhook() {
        let payload = r#"{"id":"evt_test","object":"event","api_version":"2023-10-16","created":1533204620,"livemode":false,"pending_webhooks":1,"request":null,"type":"invoiceitem.created","data":{"object":{"id":"ii_123","object":"invoiceitem","amount":1000,"currency":"usd","customer":"cus_123","date":1533204620,"description":"Test","discountable":false,"invoice":"in_123","livemode":false,"metadata":{},"period":{"start":1533204620,"end":1533204620},"proration":false,"quantity":1}}}"#;
        let secret = "whsec_test_round_trip_secret";
        let (sig, _ts) = signed_webhook_payload(payload, secret);
        let result = verify_webhook(payload, &sig, secret);
        assert!(
            result.is_ok(),
            "signed_webhook_payload should produce a signature that verify_webhook accepts: {result:?}"
        );
    }

    #[test]
    fn mock_checkout_completed_event_has_correct_type_field() {
        let event = mock_checkout_completed_event("cs_test_123", "cus_test_456");
        let parsed: serde_json::Value = serde_json::from_str(&event).expect("should be valid JSON");
        assert_eq!(parsed["type"], "checkout.session.completed");
        assert_eq!(parsed["data"]["object"]["id"], "cs_test_123");
        assert_eq!(parsed["data"]["object"]["customer"], "cus_test_456");
    }

    #[test]
    fn mock_subscription_updated_event_has_correct_type_field() {
        let event = mock_subscription_updated_event("sub_123", "cus_456", "active");
        let parsed: serde_json::Value = serde_json::from_str(&event).expect("should be valid JSON");
        assert_eq!(parsed["type"], "customer.subscription.updated");
        assert_eq!(parsed["data"]["object"]["id"], "sub_123");
        assert_eq!(parsed["data"]["object"]["customer"], "cus_456");
        assert_eq!(parsed["data"]["object"]["status"], "active");
    }

    #[test]
    fn mock_subscription_deleted_event_has_correct_type_field() {
        let event = mock_subscription_deleted_event("sub_789", "cus_012");
        let parsed: serde_json::Value = serde_json::from_str(&event).expect("should be valid JSON");
        assert_eq!(parsed["type"], "customer.subscription.deleted");
        assert_eq!(parsed["data"]["object"]["id"], "sub_789");
    }

    #[test]
    fn mock_invoice_paid_event_has_correct_type_field() {
        let event = mock_invoice_paid_event("in_test_123", "cus_test_456");
        let parsed: serde_json::Value = serde_json::from_str(&event).expect("should be valid JSON");
        assert_eq!(parsed["type"], "invoice.paid");
        assert_eq!(parsed["data"]["object"]["id"], "in_test_123");
        assert_eq!(parsed["data"]["object"]["customer"], "cus_test_456");
    }
}

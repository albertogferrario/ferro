//! Test helpers for ferro-stripe integration testing.
//!
//! Provides factory functions for [`SubscriptionInfo`] in various states and
//! utilities for generating signed webhook payloads compatible with [`verify_webhook`].
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
//! // Create a mock active subscription
//! let sub = mock_subscription_active("pro");
//! assert!(sub.subscribed());
//!
//! // Generate a signed webhook payload for testing
//! let event = mock_checkout_completed_event("cs_test", "cus_test");
//! let (sig, _ts) = signed_webhook_payload(&event, "whsec_test");
//! ```

use crate::subscription::{SubscriptionInfo, SubscriptionStatus};
use chrono::{Duration, Utc};

/// Returns a [`SubscriptionInfo`] with `Active` status for the given plan.
///
/// `current_period_end` is set 30 days from now.
pub fn mock_subscription_active(plan: &str) -> SubscriptionInfo {
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::Active,
        trial_ends_at: None,
        cancel_at_period_end: false,
        current_period_end: Utc::now() + Duration::days(30),
        stripe_connect_account_id: None,
    }
}

/// Returns a [`SubscriptionInfo`] with `Trialing` status for the given plan.
///
/// `trial_ends_at` is set 14 days from now.
pub fn mock_subscription_trialing(plan: &str) -> SubscriptionInfo {
    let trial_ends = Utc::now() + Duration::days(14);
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::Trialing,
        trial_ends_at: Some(trial_ends),
        cancel_at_period_end: false,
        current_period_end: trial_ends,
        stripe_connect_account_id: None,
    }
}

/// Returns a [`SubscriptionInfo`] with `Canceled` status for the given plan.
pub fn mock_subscription_canceled(plan: &str) -> SubscriptionInfo {
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::Canceled,
        trial_ends_at: None,
        cancel_at_period_end: false,
        current_period_end: Utc::now() - Duration::days(1),
        stripe_connect_account_id: None,
    }
}

/// Returns a [`SubscriptionInfo`] with `PastDue` status for the given plan.
pub fn mock_subscription_past_due(plan: &str) -> SubscriptionInfo {
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::PastDue,
        trial_ends_at: None,
        cancel_at_period_end: false,
        current_period_end: Utc::now() + Duration::days(1),
        stripe_connect_account_id: None,
    }
}

/// Returns a [`SubscriptionInfo`] with `Active` status and `cancel_at_period_end = true`.
///
/// Represents a subscription on a grace period — still active but scheduled to cancel.
pub fn mock_subscription_on_grace(plan: &str) -> SubscriptionInfo {
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::Active,
        trial_ends_at: None,
        cancel_at_period_end: true,
        current_period_end: Utc::now() + Duration::days(15),
        stripe_connect_account_id: None,
    }
}

/// Returns an `Active` [`SubscriptionInfo`] with the given Stripe Connect account ID set.
pub fn mock_subscription_with_connect(plan: &str, connect_id: &str) -> SubscriptionInfo {
    SubscriptionInfo {
        stripe_subscription_id: format!("sub_mock_{plan}"),
        plan: plan.to_string(),
        status: SubscriptionStatus::Active,
        trial_ends_at: None,
        cancel_at_period_end: false,
        current_period_end: Utc::now() + Duration::days(30),
        stripe_connect_account_id: Some(connect_id.to_string()),
    }
}

/// Generates a `checkout.session.completed` event JSON string.
///
/// Returns a valid Stripe event envelope suitable for use with [`signed_webhook_payload`]
/// and [`crate::verify_webhook`].
pub fn mock_checkout_completed_event(session_id: &str, customer_id: &str) -> String {
    serde_json::json!({
        "id": "evt_mock_checkout_completed",
        "object": "event",
        "api_version": "2023-10-16",
        "created": 1533204620,
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
                "status": "complete"
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
        "created": 1533204620,
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
        "created": 1533204620,
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
        "created": 1533204620,
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

/// Re-exports [`signed_webhook_payload`] from the webhook module for convenience.
///
/// Generates a Stripe-compatible `t={ts},v1={hmac}` signature header.
/// The returned `(signature_header, timestamp)` pair is ready for use with
/// [`crate::verify_webhook`].
pub use crate::webhook::events::signed_webhook_payload;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify_webhook;

    #[test]
    fn mock_subscription_active_has_correct_status_and_plan() {
        let sub = mock_subscription_active("pro");
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.plan, "pro");
        assert!(!sub.cancel_at_period_end);
        assert!(sub.trial_ends_at.is_none());
        assert!(sub.stripe_connect_account_id.is_none());
    }

    #[test]
    fn mock_subscription_trialing_has_correct_status_and_trial_end() {
        let sub = mock_subscription_trialing("starter");
        assert_eq!(sub.status, SubscriptionStatus::Trialing);
        assert_eq!(sub.plan, "starter");
        assert!(sub.trial_ends_at.is_some());
        let trial_end = sub.trial_ends_at.unwrap();
        let now = Utc::now();
        assert!(trial_end > now, "trial_ends_at should be in the future");
        assert!(
            trial_end < now + Duration::days(15),
            "trial_ends_at should be ~14 days out"
        );
    }

    #[test]
    fn mock_subscription_canceled_has_correct_status() {
        let sub = mock_subscription_canceled("pro");
        assert_eq!(sub.status, SubscriptionStatus::Canceled);
        assert_eq!(sub.plan, "pro");
        assert!(!sub.subscribed());
    }

    #[test]
    fn mock_subscription_past_due_has_correct_status() {
        let sub = mock_subscription_past_due("enterprise");
        assert_eq!(sub.status, SubscriptionStatus::PastDue);
        assert_eq!(sub.plan, "enterprise");
        assert!(!sub.subscribed());
    }

    #[test]
    fn mock_subscription_on_grace_is_active_with_cancel_at_period_end() {
        let sub = mock_subscription_on_grace("pro");
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(sub.cancel_at_period_end);
        assert!(sub.on_grace_period());
        assert!(sub.subscribed());
    }

    #[test]
    fn mock_subscription_with_connect_sets_connect_account_id() {
        let sub = mock_subscription_with_connect("pro", "acct_123");
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.stripe_connect_account_id, Some("acct_123".to_string()));
    }

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

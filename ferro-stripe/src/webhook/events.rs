use std::collections::HashMap;

use crate::Error;

/// A verified Stripe webhook event, parsed from the raw JSON envelope.
///
/// This type is deliberately **independent of `async-stripe`'s versioned
/// object structs**. Webhook payloads are rendered at the Stripe account's
/// API version, which drifts ahead of any pinned client crate. Deserializing
/// the full typed object (`stripe::EventObject`) couples verification to that
/// version and makes an unrelated field change reject the whole event. Instead
/// we keep `data` as untyped JSON and let each [`StripeEvent::from_raw`] pull
/// only the fields it needs — forward-compatible across API versions.
#[derive(Debug, Clone)]
pub struct WebhookEvent {
    /// Event id (e.g. `"evt_..."`).
    pub id: String,
    /// Event type string (e.g. `"checkout.session.completed"`).
    pub type_: String,
    /// Connect account id for Connect events; `None` for platform events.
    pub account: Option<String>,
    /// API version the payload was rendered at (informational).
    pub api_version: Option<String>,
    /// Event creation time (Unix seconds).
    pub created: i64,
    /// The `data.object` payload, kept as untyped JSON.
    pub data: serde_json::Value,
}

impl WebhookEvent {
    /// Parse a Stripe event envelope from raw JSON.
    ///
    /// Does **not** verify the signature — callers handling untrusted input
    /// must go through [`crate::verify_webhook`], which verifies the HMAC and
    /// then calls this.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebhookVerification`] when the body is not valid JSON
    /// or is missing the required `type` field.
    pub fn from_json(raw_body: &str) -> Result<Self, Error> {
        let v: serde_json::Value = serde_json::from_str(raw_body)
            .map_err(|e| Error::WebhookVerification(format!("event JSON parse: {e}")))?;
        let type_ = v
            .get("type")
            .and_then(|x| x.as_str())
            .ok_or_else(|| Error::WebhookVerification("event missing 'type'".into()))?
            .to_string();
        Ok(Self {
            id: v
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            type_,
            account: v
                .get("account")
                .and_then(|x| x.as_str())
                .map(str::to_string),
            api_version: v
                .get("api_version")
                .and_then(|x| x.as_str())
                .map(str::to_string),
            created: v.get("created").and_then(|x| x.as_i64()).unwrap_or(0),
            data: v
                .get("data")
                .and_then(|d| d.get("object"))
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        })
    }
}

// --- JSON field accessors -------------------------------------------------
//
// Stripe object fields are either scalars or "expandable" references that
// appear as a bare id string OR an inline object carrying `id`. These helpers
// read both shapes and tolerate missing fields.

fn json_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

/// Read an expandable reference: a string id, or an object with an `id` field.
fn json_id(v: &serde_json::Value, key: &str) -> Option<String> {
    match v.get(key) {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Object(o)) => {
            o.get("id").and_then(|x| x.as_str()).map(str::to_string)
        }
        _ => None,
    }
}

fn json_i64(v: &serde_json::Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}

fn json_bool(v: &serde_json::Value, key: &str) -> Option<bool> {
    v.get(key).and_then(|x| x.as_bool())
}

/// Read a `metadata` object as a `String -> String` map, skipping non-string values.
fn json_metadata(v: &serde_json::Value) -> HashMap<String, String> {
    v.get("metadata")
        .and_then(|m| m.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

/// Marker trait for typed Stripe webhook event structs.
///
/// Every event struct implements this trait. `from_raw` converts a verified
/// [`WebhookEvent`] to the typed struct, or returns `None` when the event type
/// does not match (or a required identity field is absent).
pub trait StripeEvent: Send + Sync + 'static {
    fn from_raw(event: &WebhookEvent) -> Option<Self>
    where
        Self: Sized;
}

/// Stripe webhook event for `customer.subscription.updated`.
///
/// Emitted when a subscription's status, plan, or billing cycle changes.
#[derive(Debug, Clone)]
pub struct StripeSubscriptionUpdated {
    pub event_id: String,
    pub subscription_id: String,
    pub customer_id: String,
}

impl StripeEvent for StripeSubscriptionUpdated {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "customer.subscription.updated" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            subscription_id: json_str(&event.data, "id")?,
            customer_id: json_id(&event.data, "customer")?,
        })
    }
}

/// Stripe webhook event for `customer.subscription.deleted`.
///
/// Emitted when a subscription is canceled and the billing period ends.
#[derive(Debug, Clone)]
pub struct StripeSubscriptionDeleted {
    pub event_id: String,
    pub subscription_id: String,
    pub customer_id: String,
}

impl StripeEvent for StripeSubscriptionDeleted {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "customer.subscription.deleted" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            subscription_id: json_str(&event.data, "id")?,
            customer_id: json_id(&event.data, "customer")?,
        })
    }
}

/// Stripe webhook event for `checkout.session.completed`.
///
/// Emitted when a checkout session finishes successfully.
#[derive(Debug, Clone)]
pub struct StripeCheckoutCompleted {
    pub event_id: String,
    pub session_id: String,
    pub payment_intent_id: Option<String>,
    /// Total amount in cents. `0` when `amount_total` is absent from the
    /// Stripe event (free or setup-mode sessions). Callers must not use
    /// this field alone to assert that payment was received.
    pub amount_total_cents: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
    pub customer_email: Option<String>,
}

impl StripeEvent for StripeCheckoutCompleted {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "checkout.session.completed" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            session_id: json_str(&event.data, "id")?,
            payment_intent_id: json_id(&event.data, "payment_intent"),
            amount_total_cents: json_i64(&event.data, "amount_total").unwrap_or(0),
            currency: json_str(&event.data, "currency").unwrap_or_default(),
            metadata: json_metadata(&event.data),
            customer_email: json_str(&event.data, "customer_email"),
        })
    }
}

/// Stripe webhook event for `invoice.paid`.
///
/// Emitted when an invoice is paid successfully.
#[derive(Debug, Clone)]
pub struct StripeInvoicePaid {
    pub event_id: String,
    pub invoice_id: String,
    pub customer_id: String,
}

impl StripeEvent for StripeInvoicePaid {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "invoice.paid" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            invoice_id: json_str(&event.data, "id")?,
            customer_id: json_id(&event.data, "customer")?,
        })
    }
}

/// Stripe webhook event for `payment_intent.succeeded` on a Connect account.
///
/// Emitted when a payment intent succeeds on a connected Stripe account.
#[derive(Debug, Clone)]
pub struct StripeConnectPaymentSucceeded {
    pub event_id: String,
    pub payment_intent_id: String,
    pub connect_account_id: String,
}

impl StripeEvent for StripeConnectPaymentSucceeded {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "payment_intent.succeeded" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            payment_intent_id: json_str(&event.data, "id")?,
            connect_account_id: event.account.clone()?,
        })
    }
}

/// Stripe webhook event for `checkout.session.expired`.
///
/// Emitted when a checkout session expires without being completed.
#[derive(Debug, Clone)]
pub struct StripeCheckoutExpired {
    pub event_id: String,
    pub session_id: String,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripeCheckoutExpired {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "checkout.session.expired" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            session_id: json_str(&event.data, "id")?,
            metadata: json_metadata(&event.data),
        })
    }
}

/// Stripe webhook event for `payment_intent.payment_failed`.
///
/// Emitted when a payment attempt on a PaymentIntent fails.
#[derive(Debug, Clone)]
pub struct StripePaymentIntentFailed {
    pub event_id: String,
    pub payment_intent_id: String,
    pub session_id: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripePaymentIntentFailed {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "payment_intent.payment_failed" {
            return None;
        }
        let metadata = json_metadata(&event.data);
        let last_error = event.data.get("last_payment_error");
        Some(Self {
            event_id: event.id.clone(),
            payment_intent_id: json_str(&event.data, "id")?,
            session_id: metadata.get("checkout_session_id").cloned(),
            failure_code: last_error.and_then(|e| json_str(e, "code")),
            failure_message: last_error.and_then(|e| json_str(e, "message")),
            metadata,
        })
    }
}

/// Stripe webhook event for `payment_intent.amount_capturable_updated`.
///
/// Emitted when the capturable amount on a PaymentIntent changes, confirming
/// a manual-capture hold is live and ready to be captured.
#[derive(Debug, Clone)]
pub struct StripePaymentIntentAmountCapturableUpdated {
    pub event_id: String,
    pub payment_intent_id: String,
    pub amount_capturable_cents: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripePaymentIntentAmountCapturableUpdated {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "payment_intent.amount_capturable_updated" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            payment_intent_id: json_str(&event.data, "id")?,
            amount_capturable_cents: json_i64(&event.data, "amount_capturable").unwrap_or(0),
            currency: json_str(&event.data, "currency").unwrap_or_default(),
            metadata: json_metadata(&event.data),
        })
    }
}

/// Stripe webhook event for `payment_intent.canceled`.
///
/// Emitted when a PaymentIntent is canceled — either manually via
/// `payment_intent::cancel()` or automatically by Stripe after the
/// ~7-day authorization window expires.
#[derive(Debug, Clone)]
pub struct StripePaymentIntentCanceled {
    pub event_id: String,
    pub payment_intent_id: String,
    pub cancellation_reason: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripePaymentIntentCanceled {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "payment_intent.canceled" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            payment_intent_id: json_str(&event.data, "id")?,
            cancellation_reason: json_str(&event.data, "cancellation_reason"),
            metadata: json_metadata(&event.data),
        })
    }
}

/// Stripe webhook event for `charge.refunded`.
///
/// Emitted when a charge is refunded.
#[derive(Debug, Clone)]
pub struct StripeChargeRefunded {
    pub event_id: String,
    pub charge_id: String,
    pub payment_intent_id: Option<String>,
    /// The refund identifier from the charge's refunds list (`charge.refunds.data[0].id`).
    /// `None` when the event carries no refund.
    pub refund_id: Option<String>,
    pub amount_refunded_cents: i64,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripeChargeRefunded {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "charge.refunded" {
            return None;
        }
        let refund_id = event
            .data
            .get("refunds")
            .and_then(|r| r.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| json_str(first, "id"));
        Some(Self {
            event_id: event.id.clone(),
            charge_id: json_str(&event.data, "id")?,
            payment_intent_id: json_id(&event.data, "payment_intent"),
            refund_id,
            amount_refunded_cents: json_i64(&event.data, "amount_refunded").unwrap_or(0),
            metadata: json_metadata(&event.data),
        })
    }
}

/// Stripe webhook event for `charge.dispute.created`.
///
/// Emitted when a dispute is opened on a charge.
#[derive(Debug, Clone)]
pub struct StripeChargeDisputeCreated {
    pub event_id: String,
    pub charge_id: String,
    pub payment_intent_id: Option<String>,
    pub dispute_reason: String,
    pub amount_cents: i64,
}

impl StripeEvent for StripeChargeDisputeCreated {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "charge.dispute.created" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            charge_id: json_id(&event.data, "charge")?,
            payment_intent_id: json_id(&event.data, "payment_intent"),
            dispute_reason: json_str(&event.data, "reason").unwrap_or_default(),
            amount_cents: json_i64(&event.data, "amount").unwrap_or(0),
        })
    }
}

/// Stripe webhook event for `account.updated` (Connect).
///
/// Emitted when a connected account's details change.
#[derive(Debug, Clone)]
pub struct StripeConnectAccountUpdated {
    pub event_id: String,
    pub account_id: String,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub details_submitted: bool,
}

impl StripeEvent for StripeConnectAccountUpdated {
    fn from_raw(event: &WebhookEvent) -> Option<Self> {
        if event.type_ != "account.updated" {
            return None;
        }
        Some(Self {
            event_id: event.id.clone(),
            account_id: json_str(&event.data, "id")?,
            charges_enabled: json_bool(&event.data, "charges_enabled").unwrap_or(false),
            payouts_enabled: json_bool(&event.data, "payouts_enabled").unwrap_or(false),
            details_submitted: json_bool(&event.data, "details_submitted").unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_clone_send_sync<T: Clone + Send + Sync>() {}
    fn _assert_stripe_event<T: StripeEvent>() {}

    #[test]
    fn events_are_clone_send_sync() {
        _assert_clone_send_sync::<StripeSubscriptionUpdated>();
        _assert_clone_send_sync::<StripeSubscriptionDeleted>();
        _assert_clone_send_sync::<StripeCheckoutCompleted>();
        _assert_clone_send_sync::<StripeCheckoutExpired>();
        _assert_clone_send_sync::<StripeInvoicePaid>();
        _assert_clone_send_sync::<StripePaymentIntentFailed>();
        _assert_clone_send_sync::<StripePaymentIntentAmountCapturableUpdated>();
        _assert_clone_send_sync::<StripePaymentIntentCanceled>();
        _assert_clone_send_sync::<StripeChargeRefunded>();
        _assert_clone_send_sync::<StripeChargeDisputeCreated>();
        _assert_clone_send_sync::<StripeConnectAccountUpdated>();
        _assert_clone_send_sync::<StripeConnectPaymentSucceeded>();
    }

    #[test]
    fn all_event_types_implement_stripe_event() {
        _assert_stripe_event::<StripeSubscriptionUpdated>();
        _assert_stripe_event::<StripeSubscriptionDeleted>();
        _assert_stripe_event::<StripeCheckoutCompleted>();
        _assert_stripe_event::<StripeCheckoutExpired>();
        _assert_stripe_event::<StripeInvoicePaid>();
        _assert_stripe_event::<StripePaymentIntentFailed>();
        _assert_stripe_event::<StripePaymentIntentAmountCapturableUpdated>();
        _assert_stripe_event::<StripePaymentIntentCanceled>();
        _assert_stripe_event::<StripeChargeRefunded>();
        _assert_stripe_event::<StripeChargeDisputeCreated>();
        _assert_stripe_event::<StripeConnectAccountUpdated>();
        _assert_stripe_event::<StripeConnectPaymentSucceeded>();
    }

    /// Regression: a `checkout.session.completed` payload rendered at a NEWER
    /// API version (extra/unknown fields, expanded refs) must still parse and
    /// surface its metadata. This is the exact failure the JSON-native path
    /// fixes — async-stripe's typed deserializer rejected such payloads.
    #[test]
    fn checkout_completed_parses_newer_api_shape_with_metadata() {
        let raw = serde_json::json!({
            "id": "evt_1",
            "object": "event",
            "api_version": "2026-05-27.dahlia",
            "created": 1_700_000_000_i64,
            "type": "checkout.session.completed",
            "data": { "object": {
                "id": "cs_test_123",
                "object": "checkout.session",
                "payment_status": "paid",
                "status": "complete",
                "amount_total": 500,
                "currency": "eur",
                "payment_intent": "pi_abc",
                "metadata": { "order_id": "1", "tenant_id": "1" },
                // a field that does not exist in older client structs:
                "some_future_field": { "nested": true }
            }}
        })
        .to_string();

        let event = WebhookEvent::from_json(&raw).expect("envelope parses");
        let typed = StripeCheckoutCompleted::from_raw(&event).expect("typed event matches");
        assert_eq!(typed.session_id, "cs_test_123");
        assert_eq!(typed.payment_intent_id.as_deref(), Some("pi_abc"));
        assert_eq!(typed.amount_total_cents, 500);
        assert_eq!(typed.currency, "eur");
        assert_eq!(
            typed.metadata.get("order_id").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            typed.metadata.get("tenant_id").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn wrong_event_type_does_not_match() {
        let raw = serde_json::json!({
            "id": "evt_2", "type": "invoice.paid",
            "data": { "object": { "id": "in_1", "customer": "cus_1" } }
        })
        .to_string();
        let event = WebhookEvent::from_json(&raw).unwrap();
        assert!(StripeCheckoutCompleted::from_raw(&event).is_none());
        assert!(StripeInvoicePaid::from_raw(&event).is_some());
    }
}

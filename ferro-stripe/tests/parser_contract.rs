//! Parser-contract integration tests — asserts every `StripeEvent::from_raw`
//! implementation extracts fields correctly from its golden-JSON fixture,
//! and that cross-type / wrong-type events return `None`.
//!
//! Fixtures live in `tests/fixtures/stripe_events/`.

use std::collections::HashMap;

use ferro_stripe::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded, StripeEvent,
    StripeInvoicePaid, StripePaymentIntentAmountCapturableUpdated, StripePaymentIntentCanceled,
    StripePaymentIntentFailed, StripeSubscriptionDeleted, StripeSubscriptionUpdated, WebhookEvent,
};

fn parse_event(raw: &str) -> WebhookEvent {
    WebhookEvent::from_json(raw).expect("fixture should parse as WebhookEvent envelope")
}

// --- StripeCheckoutCompleted ---

const CHECKOUT_COMPLETED: &str =
    include_str!("fixtures/stripe_events/checkout_session_completed.json");
const CHECKOUT_EXPIRED: &str = include_str!("fixtures/stripe_events/checkout_session_expired.json");

#[test]
fn checkout_session_completed_parses_all_fields() {
    let event = parse_event(CHECKOUT_COMPLETED);
    let typed = StripeCheckoutCompleted::from_raw(&event)
        .expect("from_raw should return Some for checkout.session.completed");
    assert_eq!(typed.event_id, "evt_test_checkout_completed_001");
    assert_eq!(typed.session_id, "cs_test_001");
    assert_eq!(typed.payment_intent_id.as_deref(), Some("pi_test_001"));
    assert_eq!(typed.amount_total_cents, 1000);
    assert_eq!(typed.currency, "usd");
    assert_eq!(typed.customer_email.as_deref(), Some("test@example.com"));
    let mut expected = HashMap::new();
    expected.insert("order_id".to_string(), "order_42".to_string());
    assert_eq!(typed.metadata, expected);
}

#[test]
fn checkout_completed_rejects_expired_event() {
    let event = parse_event(CHECKOUT_EXPIRED);
    assert!(
        StripeCheckoutCompleted::from_raw(&event).is_none(),
        "StripeCheckoutCompleted must reject checkout.session.expired (type_ guard)"
    );
}

// --- StripeCheckoutExpired ---

#[test]
fn checkout_session_expired_parses_all_fields() {
    let event = parse_event(CHECKOUT_EXPIRED);
    let typed = StripeCheckoutExpired::from_raw(&event)
        .expect("from_raw should return Some for checkout.session.expired");
    assert_eq!(typed.event_id, "evt_test_checkout_expired_001");
    assert_eq!(typed.session_id, "cs_test_expired_001");
    let mut expected = HashMap::new();
    expected.insert("order_id".to_string(), "order_exp".to_string());
    assert_eq!(typed.metadata, expected);
}

#[test]
fn checkout_expired_rejects_completed_event() {
    let event = parse_event(CHECKOUT_COMPLETED);
    assert!(
        StripeCheckoutExpired::from_raw(&event).is_none(),
        "StripeCheckoutExpired must reject checkout.session.completed (type_ guard)"
    );
}

// --- StripePaymentIntentFailed ---

const PI_FAILED: &str = include_str!("fixtures/stripe_events/payment_intent_payment_failed.json");

#[test]
fn payment_intent_failed_parses_all_fields() {
    let event = parse_event(PI_FAILED);
    let typed = StripePaymentIntentFailed::from_raw(&event)
        .expect("from_raw should return Some for payment_intent.payment_failed");
    assert_eq!(typed.event_id, "evt_test_pi_failed_001");
    assert_eq!(typed.payment_intent_id, "pi_test_failed_001");
    assert_eq!(typed.session_id.as_deref(), Some("cs_test_related_001"));
    assert_eq!(typed.failure_code.as_deref(), Some("card_declined"));
    assert_eq!(
        typed.failure_message.as_deref(),
        Some("Your card was declined.")
    );
    assert_eq!(
        typed
            .metadata
            .get("checkout_session_id")
            .map(String::as_str),
        Some("cs_test_related_001")
    );
    assert_eq!(
        typed.metadata.get("order_id").map(String::as_str),
        Some("order_99")
    );
}

// --- StripeChargeRefunded ---

const CHARGE_REFUNDED: &str = include_str!("fixtures/stripe_events/charge_refunded.json");

#[test]
fn charge_refunded_parses_all_fields() {
    let event = parse_event(CHARGE_REFUNDED);
    let typed = StripeChargeRefunded::from_raw(&event)
        .expect("from_raw should return Some for charge.refunded");
    assert_eq!(typed.event_id, "evt_test_charge_refunded_001");
    assert_eq!(typed.charge_id, "ch_test_refunded_001");
    assert_eq!(typed.payment_intent_id.as_deref(), Some("pi_test_ref_001"));
    assert_eq!(typed.amount_refunded_cents, 2000);
    assert_eq!(typed.refund_id.as_deref(), Some("re_test_refunded_001"));
    assert_eq!(
        typed.metadata.get("order_id").map(String::as_str),
        Some("order_ref")
    );
}

// --- StripeChargeDisputeCreated ---

const DISPUTE_CREATED: &str = include_str!("fixtures/stripe_events/charge_dispute_created.json");

#[test]
fn charge_dispute_created_parses_all_fields() {
    let event = parse_event(DISPUTE_CREATED);
    let typed = StripeChargeDisputeCreated::from_raw(&event)
        .expect("from_raw should return Some for charge.dispute.created");
    assert_eq!(typed.event_id, "evt_test_dispute_created_001");
    assert_eq!(typed.charge_id, "ch_test_disputed_001");
    assert_eq!(typed.payment_intent_id.as_deref(), Some("pi_test_disp_001"));
    assert_eq!(typed.dispute_reason, "fraudulent");
    assert_eq!(typed.amount_cents, 3000);
}

// --- StripeConnectAccountUpdated ---

const ACCOUNT_UPDATED: &str = include_str!("fixtures/stripe_events/account_updated.json");

#[test]
fn account_updated_parses_all_fields() {
    let event = parse_event(ACCOUNT_UPDATED);
    let typed = StripeConnectAccountUpdated::from_raw(&event)
        .expect("from_raw should return Some for account.updated");
    assert_eq!(typed.event_id, "evt_test_account_updated_001");
    assert_eq!(typed.account_id, "acct_test_001");
    assert!(typed.charges_enabled);
    assert!(typed.payouts_enabled);
    assert!(typed.details_submitted);
}

// --- StripeSubscriptionUpdated + Deleted ---

const SUB_UPDATED: &str = include_str!("fixtures/stripe_events/customer_subscription_updated.json");
const SUB_DELETED: &str = include_str!("fixtures/stripe_events/customer_subscription_deleted.json");

#[test]
fn subscription_updated_parses_all_fields() {
    let event = parse_event(SUB_UPDATED);
    let typed = StripeSubscriptionUpdated::from_raw(&event)
        .expect("from_raw should return Some for customer.subscription.updated");
    assert_eq!(typed.event_id, "evt_test_sub_updated_001");
    assert_eq!(typed.subscription_id, "sub_test_updated_001");
    assert_eq!(typed.customer_id, "cus_test_sub_001");
}

#[test]
fn subscription_updated_rejects_deleted_event() {
    let event = parse_event(SUB_DELETED);
    assert!(StripeSubscriptionUpdated::from_raw(&event).is_none());
}

#[test]
fn subscription_deleted_parses_all_fields() {
    let event = parse_event(SUB_DELETED);
    let typed = StripeSubscriptionDeleted::from_raw(&event)
        .expect("from_raw should return Some for customer.subscription.deleted");
    assert_eq!(typed.event_id, "evt_test_sub_deleted_001");
    assert_eq!(typed.subscription_id, "sub_test_deleted_001");
    assert_eq!(typed.customer_id, "cus_test_sub_del_001");
}

#[test]
fn subscription_deleted_rejects_updated_event() {
    let event = parse_event(SUB_UPDATED);
    assert!(StripeSubscriptionDeleted::from_raw(&event).is_none());
}

// --- StripeInvoicePaid ---

const INVOICE_PAID: &str = include_str!("fixtures/stripe_events/invoice_paid.json");

#[test]
fn invoice_paid_parses_all_fields() {
    let event = parse_event(INVOICE_PAID);
    let typed =
        StripeInvoicePaid::from_raw(&event).expect("from_raw should return Some for invoice.paid");
    assert_eq!(typed.event_id, "evt_test_invoice_paid_001");
    assert_eq!(typed.invoice_id, "in_test_paid_001");
    assert_eq!(typed.customer_id, "cus_test_inv_001");
}

#[test]
fn invoice_paid_rejects_checkout_event() {
    let event = parse_event(CHECKOUT_COMPLETED);
    assert!(StripeInvoicePaid::from_raw(&event).is_none());
}

// --- StripeConnectPaymentSucceeded ---

const PI_CONNECT: &str =
    include_str!("fixtures/stripe_events/payment_intent_succeeded_connect.json");

#[test]
fn connect_payment_succeeded_parses_all_fields() {
    let event = parse_event(PI_CONNECT);
    let typed = StripeConnectPaymentSucceeded::from_raw(&event)
        .expect("from_raw should return Some for payment_intent.succeeded with account");
    assert_eq!(typed.event_id, "evt_test_pi_connect_001");
    assert_eq!(typed.payment_intent_id, "pi_test_connect_001");
    assert_eq!(typed.connect_account_id, "acct_test_connect_001");
}

// --- StripePaymentIntentAmountCapturableUpdated + StripePaymentIntentCanceled ---

const PI_AMOUNT_CAPTURABLE: &str =
    include_str!("fixtures/stripe_events/payment_intent_amount_capturable_updated.json");
const PI_CANCELED: &str = include_str!("fixtures/stripe_events/payment_intent_canceled.json");

#[test]
fn payment_intent_amount_capturable_updated_parses_all_fields() {
    let event = parse_event(PI_AMOUNT_CAPTURABLE);
    let typed = StripePaymentIntentAmountCapturableUpdated::from_raw(&event)
        .expect("from_raw should return Some for payment_intent.amount_capturable_updated");
    assert_eq!(typed.event_id, "evt_test_pi_capturable_001");
    assert_eq!(typed.payment_intent_id, "pi_test_capturable_001");
    assert_eq!(typed.amount_capturable_cents, 5000);
    assert_eq!(typed.currency, "usd");
    assert_eq!(
        typed.metadata.get("booking_id").map(String::as_str),
        Some("bk_42")
    );
}

#[test]
fn payment_intent_canceled_parses_all_fields() {
    let event = parse_event(PI_CANCELED);
    let typed = StripePaymentIntentCanceled::from_raw(&event)
        .expect("from_raw should return Some for payment_intent.canceled");
    assert_eq!(typed.event_id, "evt_test_pi_canceled_001");
    assert_eq!(typed.payment_intent_id, "pi_test_canceled_001");
    assert_eq!(
        typed.cancellation_reason.as_deref(),
        Some("requested_by_customer")
    );
    assert_eq!(
        typed.metadata.get("booking_id").map(String::as_str),
        Some("bk_43")
    );
}

#[test]
fn payment_intent_amount_capturable_updated_rejects_canceled_event() {
    let event = parse_event(PI_CANCELED);
    assert!(
        StripePaymentIntentAmountCapturableUpdated::from_raw(&event).is_none(),
        "must reject payment_intent.canceled (type_ guard)"
    );
}

#[test]
fn payment_intent_canceled_rejects_amount_capturable_event() {
    let event = parse_event(PI_AMOUNT_CAPTURABLE);
    assert!(
        StripePaymentIntentCanceled::from_raw(&event).is_none(),
        "must reject payment_intent.amount_capturable_updated (type_ guard)"
    );
}

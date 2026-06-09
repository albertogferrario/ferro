use std::collections::HashMap;

/// Marker trait for typed Stripe webhook event structs.
///
/// Every event struct implements this trait. `from_raw` converts a
/// verified [`stripe::Event`] to the typed struct, or returns `None`
/// when the event type (or data object variant) does not match.
pub trait StripeEvent: Send + Sync + 'static {
    fn from_raw(event: &stripe::Event) -> Option<Self>
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CustomerSubscriptionUpdated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Subscription(sub) => Some(Self {
                event_id: event.id.to_string(),
                subscription_id: sub.id.to_string(),
                customer_id: sub.customer.id().to_string(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CustomerSubscriptionDeleted {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Subscription(sub) => Some(Self {
                event_id: event.id.to_string(),
                subscription_id: sub.id.to_string(),
                customer_id: sub.customer.id().to_string(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CheckoutSessionCompleted {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::CheckoutSession(session) => Some(Self {
                event_id: event.id.to_string(),
                session_id: session.id.to_string(),
                payment_intent_id: session.payment_intent.as_ref().map(|e| e.id().to_string()),
                amount_total_cents: session.amount_total.unwrap_or(0),
                currency: session.currency.map(|c| c.to_string()).unwrap_or_default(),
                metadata: session.metadata.clone().unwrap_or_default(),
                customer_email: session.customer_email.clone(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::InvoicePaid {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Invoice(inv) => {
                let customer_id = inv.customer.as_ref().map(|e| e.id().to_string())?;
                Some(Self {
                    event_id: event.id.to_string(),
                    invoice_id: inv.id.to_string(),
                    customer_id,
                })
            }
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentSucceeded {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => {
                let connect_account_id = event.account.as_ref()?.to_string();
                Some(Self {
                    event_id: event.id.to_string(),
                    payment_intent_id: pi.id.to_string(),
                    connect_account_id,
                })
            }
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CheckoutSessionExpired {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::CheckoutSession(session) => Some(Self {
                event_id: event.id.to_string(),
                session_id: session.id.to_string(),
                metadata: session.metadata.clone().unwrap_or_default(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentPaymentFailed {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => {
                let failure_code = pi
                    .last_payment_error
                    .as_ref()
                    .and_then(|e| e.code.as_ref())
                    .map(|c| c.to_string());
                let failure_message = pi
                    .last_payment_error
                    .as_ref()
                    .and_then(|e| e.message.clone());
                let session_id = pi.metadata.get("checkout_session_id").cloned();
                Some(Self {
                    event_id: event.id.to_string(),
                    payment_intent_id: pi.id.to_string(),
                    session_id,
                    failure_code,
                    failure_message,
                    metadata: pi.metadata.clone(),
                })
            }
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentAmountCapturableUpdated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => Some(Self {
                event_id: event.id.to_string(),
                payment_intent_id: pi.id.to_string(),
                amount_capturable_cents: pi.amount_capturable,
                currency: pi.currency.to_string(),
                metadata: pi.metadata.clone(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentCanceled {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => Some(Self {
                event_id: event.id.to_string(),
                payment_intent_id: pi.id.to_string(),
                cancellation_reason: pi.cancellation_reason.map(|r| r.as_str().to_string()),
                metadata: pi.metadata.clone(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::ChargeRefunded {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Charge(charge) => Some(Self {
                event_id: event.id.to_string(),
                charge_id: charge.id.to_string(),
                payment_intent_id: charge.payment_intent.as_ref().map(|e| e.id().to_string()),
                refund_id: charge
                    .refunds
                    .as_ref()
                    .and_then(|list| list.data.first())
                    .map(|r| r.id.to_string()),
                amount_refunded_cents: charge.amount_refunded,
                metadata: charge.metadata.clone(),
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::ChargeDisputeCreated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Dispute(dispute) => Some(Self {
                event_id: event.id.to_string(),
                charge_id: dispute.charge.id().to_string(),
                payment_intent_id: dispute.payment_intent.as_ref().map(|e| e.id().to_string()),
                dispute_reason: dispute.reason.clone(),
                amount_cents: dispute.amount,
            }),
            _ => None,
        }
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
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::AccountUpdated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Account(acct) => Some(Self {
                event_id: event.id.to_string(),
                account_id: acct.id.to_string(),
                charges_enabled: acct.charges_enabled.unwrap_or(false),
                payouts_enabled: acct.payouts_enabled.unwrap_or(false),
                details_submitted: acct.details_submitted.unwrap_or(false),
            }),
            _ => None,
        }
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
}

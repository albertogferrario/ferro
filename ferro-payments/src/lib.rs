//! ferro-payments — polymorphic payment intent data layer for the Ferro framework.

use std::borrow::Cow;

pub mod billable;
mod error;
pub mod intent;
pub mod loader;
pub mod migration;
mod reaper;
pub mod service;
mod webhook;

pub use error::{AutoRefundReason, PaymentError};
pub use intent::entity::{ActiveModel, Column, Entity as PaymentIntentEntity, Model};
pub use intent::lifecycle::{attach_payment_intent, find_by_charge_id, find_by_payment_intent};
pub use intent::lifecycle::{
    attach_session, create_reserved, find_active_for, find_by_stripe_session, mark_paid,
    mark_refunded, mark_released,
};
pub use intent::status::PaymentIntentStatus;
pub use migration::CreatePaymentIntentsTable;
pub use reaper::{ReconcileRefundsInFlight, ReleaseExpiredPaymentIntents};
pub use webhook::wire_dispatcher;

pub use billable::Billable;
pub use loader::BillableLoader;
pub use service::{
    CheckoutRequest, CheckoutResponse, CheckoutUrl, PaymentService, ReturnUrls,
    StripeClientGateway, StripeGateway,
};

/// Open-set kind discriminator for a billable entity. Consumers declare their own
/// constants (e.g. `BillableKind::new("order")`); the crate never enumerates kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillableKind(Cow<'static, str>);

impl BillableKind {
    /// Construct from a compile-time string literal (e.g. `BillableKind::new("order")`).
    pub const fn new(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    /// Construct from a runtime `String` (e.g. the value read from
    /// `payment_intents.billable_kind` by the webhook handlers).
    pub fn from_string(s: String) -> Self {
        Self(Cow::Owned(s))
    }

    /// Borrow the kind as a string slice. Lifetime is tied to `&self`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

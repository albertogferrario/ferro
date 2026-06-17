//! ferro-payments — polymorphic payment intent data layer for the Ferro framework.

mod error;
pub mod billable;
pub mod intent;
pub mod loader;
pub mod migration;
pub mod service;

pub use error::{AutoRefundReason, PaymentError};
pub use intent::entity::{ActiveModel, Column, Entity as PaymentIntentEntity, Model};
pub use intent::lifecycle::{
    attach_session, create_reserved, find_active_for, find_by_stripe_session, mark_paid,
    mark_refunded, mark_released,
};
pub use intent::status::PaymentIntentStatus;
pub use migration::CreatePaymentIntentsTable;

pub use billable::Billable;
pub use loader::BillableLoader;
pub use service::{
    CheckoutRequest, CheckoutResponse, CheckoutUrl, PaymentService, ReturnUrls,
    StripeClientGateway, StripeGateway,
};

/// Open-set kind discriminator for a billable entity. Consumers declare their own
/// constants (e.g. `BillableKind::new("order")`); the crate never enumerates kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillableKind(&'static str);

impl BillableKind {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

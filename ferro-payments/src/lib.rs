//! ferro-payments — polymorphic payment intent data layer for the Ferro framework.

mod error;
pub mod intent;

pub use error::PaymentError;
pub use intent::status::PaymentIntentStatus;

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

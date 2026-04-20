//! Stripe webhook handling — signature verification, typed event structs,
//! and (in Phase 141) sync/queue dispatch.

pub mod events;
pub mod queue;
pub mod sync;
pub mod verify;

pub use verify::verify_webhook;
pub use events::StripeEvent;
pub use events::{
    ProcessStripeWebhook, StripeChargeDisputeCreated, StripeChargeRefunded,
    StripeCheckoutCompleted, StripeCheckoutExpired, StripeConnectAccountUpdated,
    StripeConnectPaymentSucceeded, StripeInvoicePaid, StripePaymentIntentFailed,
    StripeSubscriptionDeleted, StripeSubscriptionUpdated,
};

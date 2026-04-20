//! Stripe webhook handling — signature verification, typed event structs,
//! synchronous dispatch registry, and queue-path job.

pub mod events;
pub mod queue;
pub mod sync;
pub mod verify;

pub use verify::verify_webhook;
pub use events::StripeEvent;
pub use events::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripePaymentIntentFailed, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
};
pub use sync::SyncDispatcher;
pub use queue::ProcessStripeWebhook;

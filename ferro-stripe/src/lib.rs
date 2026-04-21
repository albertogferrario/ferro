//! # ferro-stripe
//!
//! Stripe payment integration for the Ferro framework, organized along
//! the capability axis: [`checkout`], [`refund`], [`account`],
//! [`idempotency`], [`webhook`].
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_stripe::{Stripe, StripeConfig};
//!
//! // Initialize once at app startup.
//! let config = StripeConfig::from_env().expect("Stripe config not set");
//! Stripe::init(config);
//! ```
//!
//! ## Creating a Checkout session
//!
//! ```rust,ignore
//! use ferro_stripe::{CheckoutBuilder, Mode, LineItem};
//!
//! let intent = CheckoutBuilder::new(Mode::Payment)
//!     .line_item(LineItem {
//!         name: "Widget".into(),
//!         description: None,
//!         unit_amount_cents: 1000,
//!         quantity: 1,
//!         currency: "usd".into(),
//!     })
//!     .success_url("https://example.com/ok")
//!     .cancel_url("https://example.com/cancel")
//!     .idempotency_key("order-42")
//!     .create()
//!     .await?;
//! ```
//!
//! ## Webhook idempotency
//!
//! Implement [`ProcessedEventLog`] against your database (see the module
//! docs on [`idempotency`] for the recommended SQL schema). Use
//! [`MemoryProcessedLog`] in tests and single-process development only.

pub mod account;
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
pub mod webhook;

pub use account::{billing_portal_url, create_account, create_link, retrieve_account};
pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
pub use client::Stripe;
pub use config::StripeConfig;
pub use error::Error;
pub use idempotency::{MemoryProcessedLog, ProcessedEventLog};
pub use webhook::events::StripeEvent;
pub use webhook::events::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripePaymentIntentFailed, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
};
pub use webhook::queue::ProcessStripeWebhook;
pub use webhook::sync::SyncDispatcher;
pub use webhook::verify::verify_webhook;

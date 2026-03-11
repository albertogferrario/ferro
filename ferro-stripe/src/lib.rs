//! # ferro-stripe
//!
//! Stripe payment integration for the Ferro framework.
//!
//! Provides two billing dimensions:
//! - Platform subscriptions: the Ferro application charges tenants for plan tiers (Free/Pro/Enterprise)
//! - Stripe Connect: tenants collect one-time payments from their end users via their own connected account
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_stripe::{Stripe, StripeConfig};
//!
//! // Initialize once at app startup
//! let config = StripeConfig::from_env().expect("Stripe config not set");
//! Stripe::init(config);
//!
//! // Check subscription state in handlers
//! let tenant = current_tenant()?;
//! if let Some(sub) = &tenant.subscription {
//!     if sub.subscribed() {
//!         // grant access
//!     }
//! }
//! ```

pub mod client;
pub mod config;
pub mod connect;
pub mod error;
pub mod subscription;
pub mod webhook;

pub use client::Stripe;
pub use config::StripeConfig;
pub use connect::checkout::{create_account_link, create_connect_checkout};
pub use connect::ConnectAccount;
pub use error::Error;
pub use subscription::checkout::{billing_portal_url, create_subscription_checkout};
pub use subscription::sync::{plan_from_subscription, subscription_info_from_stripe};
pub use subscription::{plan_satisfies, SubscriptionInfo, SubscriptionStatus};
pub use webhook::events::{
    ProcessStripeWebhook, StripeCheckoutCompleted, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripeSubscriptionDeleted, StripeSubscriptionUpdated,
};
pub use webhook::{is_processed, verify_webhook};

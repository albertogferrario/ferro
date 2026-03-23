//! # ferro-whatsapp
//!
//! WhatsApp Business Cloud API integration for the Ferro framework.
//!
//! Provides outbound message sending (text and template messages) via the Meta
//! Cloud API v23.0, with a static OnceLock facade matching the ferro-stripe pattern.
//! Inbound webhook verification, message deduplication, and typed event dispatch
//! are provided by the `webhook` and `dedup` modules.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_whatsapp::{WhatsApp, WhatsAppConfig, Message};
//!
//! // Initialize once at app startup
//! let config = WhatsAppConfig::from_env(Box::new(|phone| phone == "393401234567"))
//!     .expect("WhatsApp config not set");
//! WhatsApp::init(config);
//!
//! // Send a text message from any handler
//! let result = WhatsApp::send("393409999999", Message::Text {
//!     body: "Your order is ready!".into(),
//! }).await?;
//! println!("Sent wamid: {}", result.wamid);
//! ```

pub mod client;
pub mod config;
pub mod dedup;
pub mod error;
pub mod message;
pub mod webhook;

pub use client::WhatsApp;
pub use config::WhatsAppConfig;
pub use dedup::{DeduplicationStore, InMemoryDeduplicationStore};
pub use error::Error;
pub use message::{DeliveryStatus, Message, SendResult, SenderIdentity};
pub use webhook::signed_whatsapp_payload;
pub use webhook::verify_whatsapp_webhook;

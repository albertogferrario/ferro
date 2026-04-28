//! # Ferro Notifications
//!
//! Multi-channel notification system for the Ferro framework.
//!
//! Provides a Laravel-inspired notification system with support for:
//! - Mail notifications via SMTP
//! - Database notifications for in-app delivery
//! - Slack webhook notifications
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_notifications::{Notification, Notifiable, Channel, MailMessage};
//!
//! // Define a notification
//! struct OrderShipped {
//!     order_id: i64,
//!     tracking: String,
//! }
//!
//! impl Notification for OrderShipped {
//!     fn via(&self) -> Vec<Channel> {
//!         vec![Channel::Mail, Channel::Database]
//!     }
//!
//!     fn to_mail(&self) -> Option<MailMessage> {
//!         Some(MailMessage::new()
//!             .subject("Your order has shipped!")
//!             .body(format!("Tracking: {}", self.tracking)))
//!     }
//! }
//!
//! // Make User notifiable
//! struct User {
//!     email: String,
//! }
//!
//! impl Notifiable for User {
//!     fn route_notification_for(&self, channel: Channel) -> Option<String> {
//!         match channel {
//!             Channel::Mail => Some(self.email.clone()),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! // Send the notification
//! let user = User { email: "user@example.com".into() };
//! user.notify(OrderShipped {
//!     order_id: 123,
//!     tracking: "ABC123".into(),
//! }).await?;
//! ```

mod channel;
mod channels;
mod dispatcher;
mod error;
mod notifiable;
mod notification;

pub use channel::Channel;
pub use channels::{
    DatabaseMessage, InAppMessage, InAppSeverity, MailAttachment, MailMessage, PushMessage,
    SlackAttachment, SlackField, SlackMessage, SmsMessage, WhatsAppMessage,
};
pub use dispatcher::{
    InAppConfig, MailConfig, MailDriver, NotificationConfig, NotificationDispatcher, ResendConfig,
    SmtpConfig,
};
pub use error::Error;
pub use notifiable::{ChannelResult, DatabaseNotificationStore, Notifiable, StoredNotification};
pub use notification::Notification;

/// Re-export async_trait for convenience.
pub use async_trait::async_trait;

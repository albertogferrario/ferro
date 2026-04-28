//! Notification channel implementations.

mod database;
mod future;
mod in_app;
mod mail;
mod slack;
mod whatsapp;

pub use database::DatabaseMessage;
pub use future::{PushMessage, SmsMessage};
pub use in_app::{InAppMessage, InAppSeverity};
pub use mail::MailMessage;
pub use slack::{SlackAttachment, SlackField, SlackMessage};
pub use whatsapp::WhatsAppMessage;

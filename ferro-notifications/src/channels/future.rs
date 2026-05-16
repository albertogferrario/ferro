//! Placeholder message types for channels not yet implemented.
//!
//! These types exist so the [`crate::Notification`] trait can declare
//! `to_sms()` and `to_push()` with stable signatures across phases.
//! No adapter consumes these types in the current phase — the dispatcher
//! emits a structured "channel not configured" no-op for both.

use serde::{Deserialize, Serialize};

/// Placeholder SMS message. Shape will be finalized when an SMS adapter ships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SmsMessage {
    /// Message body text.
    pub body: String,
}

/// Placeholder push notification. Shape will be finalized when an APNs/FCM adapter ships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushMessage {
    /// Notification title.
    pub title: String,
    /// Notification body.
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sms_message_default() {
        let m = SmsMessage::default();
        assert_eq!(m.body, "");
    }

    #[test]
    fn test_push_message_default() {
        let m = PushMessage::default();
        assert_eq!(m.title, "");
        assert_eq!(m.body, "");
    }
}

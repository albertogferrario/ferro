//! Notification channel abstraction.

use serde::{Deserialize, Serialize};

/// Available notification channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    /// Email notifications via SMTP or Resend.
    Mail,
    /// Database-persisted in-app notifications.
    Database,
    /// Slack webhook notifications.
    Slack,
    /// WhatsApp Cloud API notifications (per CONTEXT.md D-01).
    ///
    /// Serializes as `"whatsapp"` (explicit rename for clarity; matches `lowercase` output).
    #[serde(rename = "whatsapp")]
    WhatsApp,
    /// Real-time in-app SSE notifications (per CONTEXT.md D-01).
    ///
    /// Serializes as `"in_app"` — overrides the enum-level `lowercase` rule
    /// which would otherwise produce `"inapp"`.
    #[serde(rename = "in_app")]
    InApp,
    /// SMS notifications (placeholder — no adapter ships in this phase per ARCH-FINDING-03).
    Sms,
    /// Push notifications (placeholder — no adapter ships in this phase).
    Push,
}

impl Channel {
    /// Get channel name as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Mail => "mail",
            Channel::Database => "database",
            Channel::Slack => "slack",
            Channel::WhatsApp => "whatsapp",
            Channel::InApp => "in_app",
            Channel::Sms => "sms",
            Channel::Push => "push",
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_as_str() {
        assert_eq!(Channel::Mail.as_str(), "mail");
        assert_eq!(Channel::Database.as_str(), "database");
        assert_eq!(Channel::Slack.as_str(), "slack");
        assert_eq!(Channel::WhatsApp.as_str(), "whatsapp");
        assert_eq!(Channel::InApp.as_str(), "in_app");
        assert_eq!(Channel::Sms.as_str(), "sms");
        assert_eq!(Channel::Push.as_str(), "push");
    }

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", Channel::Mail), "mail");
        assert_eq!(format!("{}", Channel::WhatsApp), "whatsapp");
        assert_eq!(format!("{}", Channel::InApp), "in_app");
    }

    #[test]
    fn test_channel_serialization() {
        // Forward
        assert_eq!(
            serde_json::to_string(&Channel::WhatsApp).unwrap(),
            "\"whatsapp\""
        );
        assert_eq!(
            serde_json::to_string(&Channel::InApp).unwrap(),
            "\"in_app\""
        );
        assert_eq!(serde_json::to_string(&Channel::Mail).unwrap(), "\"mail\"");
        assert_eq!(serde_json::to_string(&Channel::Sms).unwrap(), "\"sms\"");
        assert_eq!(serde_json::to_string(&Channel::Push).unwrap(), "\"push\"");
    }

    #[test]
    fn test_channel_deserialization() {
        // Round-trip
        assert_eq!(
            serde_json::from_str::<Channel>("\"whatsapp\"").unwrap(),
            Channel::WhatsApp
        );
        assert_eq!(
            serde_json::from_str::<Channel>("\"in_app\"").unwrap(),
            Channel::InApp
        );
        assert_eq!(
            serde_json::from_str::<Channel>("\"mail\"").unwrap(),
            Channel::Mail
        );
        assert_eq!(
            serde_json::from_str::<Channel>("\"database\"").unwrap(),
            Channel::Database
        );
        assert_eq!(
            serde_json::from_str::<Channel>("\"slack\"").unwrap(),
            Channel::Slack
        );
        // The literal "inapp" must NOT deserialize to InApp (regression guard for ARCH-FINDING-05/serde-trap)
        assert!(
            serde_json::from_str::<Channel>("\"inapp\"").is_err(),
            "InApp must not accept the unrenamed `inapp` form"
        );
    }
}

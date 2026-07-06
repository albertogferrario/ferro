//! In-app SSE notification channel.

use serde::{Deserialize, Serialize};

/// Severity hint for in-app notification rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InAppSeverity {
    /// Informational notice.
    Info,
    /// Success notice.
    Success,
    /// Warning notice.
    Warning,
    /// Error notice.
    Error,
}

/// An in-app notification message.
///
/// Persisted via `DatabaseNotificationStore` and broadcast via `Broadcaster`
/// per CONTEXT.md D-08 (DB-store leg first, broadcast leg second).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppMessage {
    /// Notification type identifier (e.g. "OrderShipped").
    pub notification_type: String,
    /// Free-form payload (broadcast as-is to the SSE channel).
    pub data: serde_json::Value,
    /// Optional severity hint for client-side rendering.
    pub severity: Option<InAppSeverity>,
}

impl InAppMessage {
    /// Create a new in-app message with the given type. Data defaults to `null`, severity to `None`.
    pub fn new(notification_type: impl Into<String>) -> Self {
        Self {
            notification_type: notification_type.into(),
            data: serde_json::Value::Null,
            severity: None,
        }
    }

    /// Set the data payload.
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set the severity hint.
    pub fn severity(mut self, level: InAppSeverity) -> Self {
        self.severity = Some(level);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_app_message_builder() {
        let msg = InAppMessage::new("OrderShipped")
            .data(serde_json::json!({"order_id": 42}))
            .severity(InAppSeverity::Success);
        assert_eq!(msg.notification_type, "OrderShipped");
        assert_eq!(msg.data, serde_json::json!({"order_id": 42}));
        assert_eq!(msg.severity, Some(InAppSeverity::Success));
    }

    #[test]
    fn test_in_app_severity_serialization() {
        for (level, expected) in &[
            (InAppSeverity::Info, "\"info\""),
            (InAppSeverity::Success, "\"success\""),
            (InAppSeverity::Warning, "\"warning\""),
            (InAppSeverity::Error, "\"error\""),
        ] {
            let json = serde_json::to_string(level).unwrap();
            assert_eq!(&json, expected);
        }
    }
}

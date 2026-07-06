//! Database notification channel.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A database message for in-app notifications.
///
/// Database notifications are stored in the database and can be
/// displayed in the application's notification center.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMessage {
    /// Notification type identifier.
    pub notification_type: String,
    /// Notification data as key-value pairs.
    pub data: HashMap<String, Value>,
}

impl DatabaseMessage {
    /// Create a new database message with a type.
    pub fn new(notification_type: impl Into<String>) -> Self {
        Self {
            notification_type: notification_type.into(),
            data: HashMap::new(),
        }
    }

    /// Add a data field to the notification.
    pub fn data<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.into(), v);
        }
        self
    }

    /// Add multiple data fields from a map.
    pub fn with_data(mut self, data: HashMap<String, Value>) -> Self {
        self.data.extend(data);
        self
    }

    /// Get a data field value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Serialize the entire message to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_message_builder() {
        let msg = DatabaseMessage::new("order_shipped")
            .data("order_id", 123)
            .data("tracking", "ABC123");

        assert_eq!(msg.notification_type, "order_shipped");
        assert_eq!(msg.get("order_id"), Some(&Value::Number(123.into())));
        assert_eq!(msg.get("tracking"), Some(&Value::String("ABC123".into())));
    }

    #[test]
    fn test_database_message_to_json() {
        let msg = DatabaseMessage::new("test").data("key", "value");

        let json = msg.to_json().unwrap();
        assert!(json.contains("\"notification_type\":\"test\""));
        assert!(json.contains("\"key\":\"value\""));
    }
}

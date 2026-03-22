use chrono::{DateTime, Utc};
use ferro_events::Event;

/// Event dispatched when a pending confirmation action expires after its TTL.
#[derive(Debug, Clone)]
pub struct ConfirmationExpired {
    /// The confirmation key that expired.
    pub key: String,
    /// When the expiry occurred.
    pub expired_at: DateTime<Utc>,
}

impl Event for ConfirmationExpired {
    fn name(&self) -> &'static str {
        "ConfirmationExpired"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_expired_event_name() {
        let event = ConfirmationExpired {
            key: "action-123".to_string(),
            expired_at: Utc::now(),
        };
        assert_eq!(event.name(), "ConfirmationExpired");
    }
}

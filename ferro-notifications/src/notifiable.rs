//! Notifiable trait for entities that can receive notifications.

use crate::channel::Channel;
use crate::channels::DatabaseMessage;
use crate::dispatcher::NotificationDispatcher;
use crate::notification::Notification;
use crate::Error;
use async_trait::async_trait;

/// Trait for entities that can receive notifications.
///
/// Implement this trait on your User model or any other entity
/// that should be able to receive notifications.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_notifications::{Notifiable, Channel};
///
/// struct User {
///     id: i64,
///     email: String,
///     slack_webhook: Option<String>,
/// }
///
/// impl Notifiable for User {
///     fn route_notification_for(&self, channel: Channel) -> Option<String> {
///         match channel {
///             Channel::Mail => Some(self.email.clone()),
///             Channel::Slack => self.slack_webhook.clone(),
///             Channel::Database => Some(self.id.to_string()),
///             _ => None,
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Notifiable: Send + Sync {
    /// Get the routing information for a specific channel.
    ///
    /// Returns the destination for the notification (email address,
    /// webhook URL, user ID, etc.) or None if the channel is not
    /// available for this entity.
    fn route_notification_for(&self, channel: Channel) -> Option<String>;

    /// Get the unique identifier for this notifiable entity.
    /// Used for database notifications.
    fn notifiable_id(&self) -> String {
        "unknown".to_string()
    }

    /// Get the type name of this notifiable entity.
    fn notifiable_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Send a notification to this entity.
    ///
    /// This is the main entry point for sending notifications.
    /// It dispatches the notification through all configured channels.
    async fn notify<N: Notification + 'static>(&self, notification: N) -> Result<(), Error> {
        NotificationDispatcher::send(self, notification).await
    }
}

/// Result of sending a notification through a channel.
#[derive(Debug)]
pub struct ChannelResult {
    /// The channel that was used.
    pub channel: Channel,
    /// Whether the send was successful.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

impl ChannelResult {
    /// Create a successful result.
    pub fn success(channel: Channel) -> Self {
        Self {
            channel,
            success: true,
            error: None,
        }
    }

    /// Create a failed result.
    pub fn failure(channel: Channel, error: impl Into<String>) -> Self {
        Self {
            channel,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Extension trait for database notification storage.
#[async_trait]
pub trait DatabaseNotificationStore: Send + Sync {
    /// Store a notification in the database.
    async fn store(
        &self,
        notifiable_id: &str,
        notifiable_type: &str,
        notification_type: &str,
        message: &DatabaseMessage,
    ) -> Result<(), Error>;

    /// Mark a notification as read.
    async fn mark_as_read(&self, notification_id: &str) -> Result<(), Error>;

    /// Get unread notifications for an entity.
    async fn unread(&self, notifiable_id: &str) -> Result<Vec<StoredNotification>, Error>;
}

/// A notification stored in the database.
#[derive(Debug, Clone)]
pub struct StoredNotification {
    /// Unique notification ID.
    pub id: String,
    /// Notifiable entity ID.
    pub notifiable_id: String,
    /// Notifiable entity type.
    pub notifiable_type: String,
    /// Notification type.
    pub notification_type: String,
    /// Notification data as JSON.
    pub data: String,
    /// When the notification was read (if at all).
    pub read_at: Option<String>,
    /// When the notification was created.
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestUser {
        id: i64,
        email: String,
    }

    impl Notifiable for TestUser {
        fn route_notification_for(&self, channel: Channel) -> Option<String> {
            match channel {
                Channel::Mail => Some(self.email.clone()),
                Channel::Database => Some(self.id.to_string()),
                _ => None,
            }
        }

        fn notifiable_id(&self) -> String {
            self.id.to_string()
        }
    }

    #[test]
    fn test_route_notification_for() {
        let user = TestUser {
            id: 42,
            email: "test@example.com".to_string(),
        };

        assert_eq!(
            user.route_notification_for(Channel::Mail),
            Some("test@example.com".to_string())
        );
        assert_eq!(
            user.route_notification_for(Channel::Database),
            Some("42".to_string())
        );
        assert_eq!(user.route_notification_for(Channel::Slack), None);
    }

    #[test]
    fn test_channel_result() {
        let success = ChannelResult::success(Channel::Mail);
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = ChannelResult::failure(Channel::Slack, "Connection failed");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Connection failed".to_string()));
    }
}

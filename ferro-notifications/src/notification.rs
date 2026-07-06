//! Core notification trait.

use crate::channel::Channel;
use crate::channels::{
    DatabaseMessage, InAppMessage, MailMessage, PushMessage, SlackMessage, SmsMessage,
    WhatsAppMessage,
};

/// A notification that can be sent through multiple channels.
///
/// Notifications encapsulate a message that should be delivered to users
/// through one or more channels (mail, database, slack, etc.).
///
/// # Example
///
/// ```rust
/// use ferro_notifications::{Notification, Channel, MailMessage, DatabaseMessage};
///
/// struct OrderShipped {
///     order_id: i64,
///     tracking_number: String,
/// }
///
/// impl Notification for OrderShipped {
///     fn via(&self) -> Vec<Channel> {
///         vec![Channel::Mail, Channel::Database]
///     }
///
///     fn to_mail(&self) -> Option<MailMessage> {
///         Some(MailMessage::new()
///             .subject("Your order has shipped!")
///             .body(format!("Tracking: {}", self.tracking_number)))
///     }
///
///     fn to_database(&self) -> Option<DatabaseMessage> {
///         Some(DatabaseMessage::new("order_shipped")
///             .data("order_id", self.order_id)
///             .data("tracking", &self.tracking_number))
///     }
/// }
/// ```
pub trait Notification: Send + Sync {
    /// The channels this notification should be sent through.
    fn via(&self) -> Vec<Channel>;

    /// Convert the notification to a mail message.
    fn to_mail(&self) -> Option<MailMessage> {
        None
    }

    /// Convert the notification to a database message.
    fn to_database(&self) -> Option<DatabaseMessage> {
        None
    }

    /// Convert the notification to a Slack message.
    fn to_slack(&self) -> Option<SlackMessage> {
        None
    }

    /// Convert the notification to a WhatsApp message (per CONTEXT.md D-02).
    fn to_whatsapp(&self) -> Option<WhatsAppMessage> {
        None
    }

    /// Convert the notification to an in-app SSE message (per CONTEXT.md D-02).
    fn to_in_app(&self) -> Option<InAppMessage> {
        None
    }

    /// Convert the notification to an SMS message (placeholder per ARCH-FINDING-03; no adapter in this phase).
    fn to_sms(&self) -> Option<SmsMessage> {
        None
    }

    /// Convert the notification to a push notification (placeholder per ARCH-FINDING-03; no adapter in this phase).
    fn to_push(&self) -> Option<PushMessage> {
        None
    }

    /// Get the notification type name for logging.
    fn notification_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNotification;

    impl Notification for TestNotification {
        fn via(&self) -> Vec<Channel> {
            vec![Channel::Mail, Channel::Database]
        }

        fn to_mail(&self) -> Option<MailMessage> {
            Some(MailMessage::new().subject("Test").body("Test body"))
        }
    }

    #[test]
    fn test_notification_via() {
        let notification = TestNotification;
        let channels = notification.via();
        assert_eq!(channels.len(), 2);
        assert!(channels.contains(&Channel::Mail));
        assert!(channels.contains(&Channel::Database));
    }

    #[test]
    fn test_notification_to_mail() {
        let notification = TestNotification;
        let mail = notification.to_mail();
        assert!(mail.is_some());
        let mail = mail.unwrap();
        assert_eq!(mail.subject, "Test");
    }

    #[test]
    fn test_notification_to_database_default() {
        let notification = TestNotification;
        assert!(notification.to_database().is_none());
    }

    #[test]
    fn test_notification_to_whatsapp_default() {
        let notification = TestNotification;
        assert!(notification.to_whatsapp().is_none());
    }

    #[test]
    fn test_notification_to_in_app_default() {
        let notification = TestNotification;
        assert!(notification.to_in_app().is_none());
    }

    #[test]
    fn test_notification_to_sms_default() {
        let notification = TestNotification;
        assert!(notification.to_sms().is_none());
    }

    #[test]
    fn test_notification_to_push_default() {
        let notification = TestNotification;
        assert!(notification.to_push().is_none());
    }
}

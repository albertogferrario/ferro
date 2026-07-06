# ferro-notifications

Multi-channel notification system for the Ferro framework.

## Features

- Mail notifications via SMTP
- Database notifications for in-app delivery
- Slack webhook notifications
- Extensible channel system

## Usage

```rust
use ferro_notifications::{Notification, Notifiable, Channel, MailMessage};

// Define a notification
struct OrderShipped {
    order_id: i64,
    tracking: String,
}

impl Notification for OrderShipped {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail, Channel::Database]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(MailMessage::new()
            .subject("Your order has shipped!")
            .body(format!("Tracking: {}", self.tracking)))
    }
}

// Make User notifiable
struct User {
    id: i64,
    email: String,
}

impl Notifiable for User {
    fn route_notification_for(&self, channel: Channel) -> Option<String> {
        match channel {
            Channel::Mail => Some(self.email.clone()),
            Channel::Database => Some(self.id.to_string()),
            _ => None,
        }
    }
}

// Send the notification
let user = User { id: 1, email: "user@example.com".into() };
user.notify(OrderShipped {
    order_id: 123,
    tracking: "ABC123".into(),
}).await?;
```

## Slack Notifications

```rust
fn to_slack(&self) -> Option<SlackMessage> {
    Some(SlackMessage::new()
        .text("New order shipped!")
        .attachment(SlackAttachment::new()
            .title("Order Details")
            .field("Order ID", &self.order_id.to_string(), true)))
}
```

## License

MIT

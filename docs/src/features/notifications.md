# Notifications

Ferro provides a Laravel-inspired multi-channel notification system. Send notifications via mail, database, Slack, and more through a unified API.

## Configuration

### Environment Variables

Configure notifications in your `.env` file:

```env
# Mail Driver: smtp (default) or resend
MAIL_DRIVER=smtp

# SMTP Configuration (when MAIL_DRIVER=smtp)
MAIL_HOST=smtp.example.com
MAIL_PORT=587
MAIL_USERNAME=your-username
MAIL_PASSWORD=your-password
MAIL_ENCRYPTION=tls

# Resend Configuration (when MAIL_DRIVER=resend)
RESEND_API_KEY=re_xxxxxxxxxxxxx

# Shared (all drivers)
MAIL_FROM_ADDRESS=noreply@example.com
MAIL_FROM_NAME="My App"

# Slack
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/xxx/yyy/zzz
```

### Bootstrap Setup

In `src/bootstrap.rs`, initialize notifications:

```rust
use ferro::{NotificationConfig, NotificationDispatcher};

pub async fn register() {
    // ... other setup ...

    // Configure notifications from environment
    let config = NotificationConfig::from_env();
    NotificationDispatcher::configure(config);
}
```

### Manual Configuration

```rust
use ferro::{NotificationConfig, MailConfig, NotificationDispatcher};

// SMTP (default)
let config = NotificationConfig::new()
    .mail(
        MailConfig::new("smtp.example.com", 587, "noreply@example.com")
            .credentials("user", "pass")
            .from_name("My App")
    )
    .slack_webhook("https://hooks.slack.com/services/...");

NotificationDispatcher::configure(config);

// Resend
let config = NotificationConfig::new()
    .mail(
        MailConfig::resend("re_xxxxxxxxxxxxx", "noreply@example.com")
            .from_name("My App")
    );

NotificationDispatcher::configure(config);
```

## Creating Notifications

### Using the CLI

Generate a new notification:

```bash
ferro make:notification OrderShipped
```

This creates `src/notifications/order_shipped.rs`:

```rust
use ferro::{Notification, Channel, MailMessage};

pub struct OrderShipped {
    pub order_id: i64,
    pub tracking_number: String,
}

impl Notification for OrderShipped {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(MailMessage::new()
            .subject("Your order has shipped!")
            .body(format!("Tracking: {}", self.tracking_number)))
    }
}
```

### Notification Trait Methods

| Method | Description | Default |
|--------|-------------|---------|
| `via()` | Channels to send through | Required |
| `to_mail()` | Mail message content | None |
| `to_database()` | Database message content | None |
| `to_slack()` | Slack message content | None |
| `notification_type()` | Type name for logging | Type name |

## Making Entities Notifiable

Implement `Notifiable` on your User model:

```rust
use ferro::{Notifiable, Channel, async_trait};

pub struct User {
    pub id: i64,
    pub email: String,
    pub slack_webhook: Option<String>,
}

impl Notifiable for User {
    fn route_notification_for(&self, channel: Channel) -> Option<String> {
        match channel {
            Channel::Mail => Some(self.email.clone()),
            Channel::Database => Some(self.id.to_string()),
            Channel::Slack => self.slack_webhook.clone(),
            _ => None,
        }
    }

    fn notifiable_id(&self) -> String {
        self.id.to_string()
    }
}
```

### Notifiable Trait Methods

| Method | Description | Default |
|--------|-------------|---------|
| `route_notification_for(channel)` | Get routing info per channel | Required |
| `notifiable_id()` | Unique identifier | "unknown" |
| `notifiable_type()` | Type name | Type name |
| `notify(notification)` | Send a notification | Provided |

## Sending Notifications

### Basic Usage

```rust
use crate::notifications::OrderShipped;

// In a controller or service
let user = User::find(user_id).await?;
user.notify(OrderShipped {
    order_id: 123,
    tracking_number: "ABC123".into(),
}).await?;
```

## Available Channels

### Mail Channel

Send emails via SMTP or Resend:

```rust
impl Notification for WelcomeEmail {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(MailMessage::new()
            .subject("Welcome to Our Platform")
            .body("Thanks for signing up!")
            .html("<h1>Welcome!</h1><p>Thanks for signing up!</p>")
            .cc("admin@example.com")
            .bcc("archive@example.com")
            .reply_to("support@example.com"))
    }
}
```

#### MailMessage Methods

| Method | Description |
|--------|-------------|
| `subject(text)` | Set email subject |
| `body(text)` | Set plain text body |
| `html(content)` | Set HTML body |
| `from(address)` | Override from address |
| `reply_to(address)` | Set reply-to address |
| `cc(address)` | Add CC recipient |
| `bcc(address)` | Add BCC recipient |
| `header(name, value)` | Add custom header |

### Database Channel

Store notifications for in-app display:

```rust
use ferro::{Notification, Channel, DatabaseMessage};

impl Notification for OrderStatusChanged {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Database]
    }

    fn to_database(&self) -> Option<DatabaseMessage> {
        Some(DatabaseMessage::new("order_status_changed")
            .data("order_id", self.order_id)
            .data("status", &self.status)
            .data("message", format!("Order #{} is now {}", self.order_id, self.status)))
    }
}
```

#### DatabaseMessage Methods

| Method | Description |
|--------|-------------|
| `new(type)` | Create with notification type |
| `data(key, value)` | Add data field |
| `with_data(map)` | Add multiple fields |
| `get(key)` | Get field value |
| `to_json()` | Serialize to JSON |

### Slack Channel

Send Slack webhook notifications:

```rust
use ferro::{Notification, Channel, SlackMessage, SlackAttachment};

impl Notification for DeploymentComplete {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Slack]
    }

    fn to_slack(&self) -> Option<SlackMessage> {
        Some(SlackMessage::new("Deployment completed successfully!")
            .channel("#deployments")
            .username("Deploy Bot")
            .icon_emoji(":rocket:")
            .attachment(
                SlackAttachment::new()
                    .color("good")
                    .title("Deployment Details")
                    .field("Environment", &self.environment, true)
                    .field("Version", &self.version, true)
                    .footer("Deployed by CI/CD")
            ))
    }
}
```

#### SlackMessage Methods

| Method | Description |
|--------|-------------|
| `new(text)` | Create with main text |
| `channel(name)` | Override channel |
| `username(name)` | Override bot name |
| `icon_emoji(emoji)` | Set emoji icon |
| `icon_url(url)` | Set image icon |
| `attachment(att)` | Add attachment |

#### SlackAttachment Methods

| Method | Description |
|--------|-------------|
| `color(hex)` | Set sidebar color |
| `title(text)` | Set attachment title |
| `title_link(url)` | Make title clickable |
| `text(content)` | Set attachment text |
| `field(title, value, short)` | Add field |
| `footer(text)` | Set footer text |
| `timestamp(unix)` | Set timestamp |

## Multi-Channel Notifications

Send to multiple channels at once:

```rust
impl Notification for OrderPlaced {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail, Channel::Database, Channel::Slack]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(MailMessage::new()
            .subject("Order Confirmation")
            .body(format!("Order #{} placed successfully", self.order_id)))
    }

    fn to_database(&self) -> Option<DatabaseMessage> {
        Some(DatabaseMessage::new("order_placed")
            .data("order_id", self.order_id)
            .data("total", self.total))
    }

    fn to_slack(&self) -> Option<SlackMessage> {
        Some(SlackMessage::new(format!("New order #{} for ${:.2}", self.order_id, self.total)))
    }
}
```

## Example: Complete Notification

```rust
// notifications/order_shipped.rs
use ferro::{Notification, Channel, MailMessage, DatabaseMessage, SlackMessage, SlackAttachment};

pub struct OrderShipped {
    pub order_id: i64,
    pub tracking_number: String,
    pub carrier: String,
    pub estimated_delivery: String,
}

impl Notification for OrderShipped {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail, Channel::Database, Channel::Slack]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(MailMessage::new()
            .subject(format!("Order #{} has shipped!", self.order_id))
            .html(format!(r#"
                <h1>Your order is on its way!</h1>
                <p>Order #{} has been shipped via {}.</p>
                <p><strong>Tracking:</strong> {}</p>
                <p><strong>Estimated Delivery:</strong> {}</p>
            "#, self.order_id, self.carrier, self.tracking_number, self.estimated_delivery)))
    }

    fn to_database(&self) -> Option<DatabaseMessage> {
        Some(DatabaseMessage::new("order_shipped")
            .data("order_id", self.order_id)
            .data("tracking_number", &self.tracking_number)
            .data("carrier", &self.carrier))
    }

    fn to_slack(&self) -> Option<SlackMessage> {
        Some(SlackMessage::new("Order shipped!")
            .attachment(
                SlackAttachment::new()
                    .color("#36a64f")
                    .title(format!("Order #{}", self.order_id))
                    .field("Carrier", &self.carrier, true)
                    .field("Tracking", &self.tracking_number, true)
                    .field("ETA", &self.estimated_delivery, false)
            ))
    }
}

// Usage in controller
let user = User::find(order.user_id).await?;
user.notify(OrderShipped {
    order_id: order.id,
    tracking_number: "1Z999AA10123456784".into(),
    carrier: "UPS".into(),
    estimated_delivery: "January 15, 2026".into(),
}).await?;
```

## Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `MAIL_DRIVER` | Mail transport driver | smtp |
| **SMTP** (when `MAIL_DRIVER=smtp`) | | |
| `MAIL_HOST` | SMTP server host | Required |
| `MAIL_PORT` | SMTP server port | 587 |
| `MAIL_USERNAME` | SMTP username | - |
| `MAIL_PASSWORD` | SMTP password | - |
| `MAIL_ENCRYPTION` | "tls" or "none" | tls |
| **Resend** (when `MAIL_DRIVER=resend`) | | |
| `RESEND_API_KEY` | Resend API key | Required |
| **Shared** | | |
| `MAIL_FROM_ADDRESS` | Default from email | Required |
| `MAIL_FROM_NAME` | Default from name | - |
| `SLACK_WEBHOOK_URL` | Slack incoming webhook | - |

## Best Practices

1. **Use descriptive notification names** - `OrderShipped` not `Notification1`
2. **Include all needed data** - Pass everything the notification needs
3. **Keep notifications focused** - One notification per event
4. **Use database for in-app** - Combine with UI notification center
5. **Handle failures gracefully** - Log errors, don't crash on send failures
6. **Test notifications** - Verify each channel works in development

## MCP Tools

Use `code_templates` with the `notifications` category to generate starter code for new notification classes without looking up the API.

### `code_templates`

Returns ready-to-use code snippets for common notification patterns. Pass `category: "notifications"` to get templates for mail, database, and Slack channels, along with the `Notifiable` trait implementation. Useful when scaffolding a new notification type quickly.

## WhatsApp Channel

Send transactional notifications via the WhatsApp Cloud API (Meta). The WhatsApp adapter delegates to [`ferro-whatsapp`](https://docs.rs/ferro-whatsapp), which owns its global state via the static `WhatsApp::init` facade.

### Setup

1. **Initialize ferro-whatsapp once at app startup** (typically in `bootstrap.rs`):

   ```rust
   use ferro_whatsapp::{WhatsApp, WhatsAppConfig};

   let wa_config = WhatsAppConfig::from_env(Box::new(|phone| {
       // phone-validation hook — your allowlist or regex check
       phone.starts_with("39")
   })).expect("WHATSAPP_* env vars not set");
   WhatsApp::init(wa_config);
   ```

2. **Enable the channel in NotificationConfig**:

   ```rust
   use ferro::{NotificationConfig, NotificationDispatcher};

   let config = NotificationConfig::from_env()    // reads WHATSAPP_ENABLED
       .with_whatsapp_enabled(true);              // or set programmatically
   NotificationDispatcher::configure(config);
   ```

   Default is `false`. When disabled, the dispatcher emits a structured "channel not configured" log and returns `Ok(())` — `ferro-whatsapp` is never touched, so `WhatsApp::init` is not required.

3. **Implement `Notifiable` to provide the recipient phone**:

   ```rust
   use ferro::{Notifiable, NotificationChannel};

   impl Notifiable for User {
       fn route_notification_for(&self, channel: NotificationChannel) -> Option<String> {
           match channel {
               NotificationChannel::WhatsApp => self.phone.clone(),
               // ... other channels ...
               _ => None,
           }
       }
   }
   ```

4. **Implement `Notification::to_whatsapp`**:

   ```rust
   use ferro::{Notification, NotificationChannel, WhatsAppMessage};

   impl Notification for OrderShipped {
       fn via(&self) -> Vec<NotificationChannel> {
           vec![NotificationChannel::WhatsApp]
       }

       fn to_whatsapp(&self) -> Option<WhatsAppMessage> {
           Some(WhatsAppMessage::text(format!(
               "Your order #{} has shipped — tracking {}",
               self.order_id, self.tracking
           )))
       }
   }
   ```

   For approved Meta templates use `WhatsAppMessage::template(name, language, parameters)`.

## In-App (SSE) Channel

Real-time in-app notifications dispatch through two legs in sequence:

1. **Persistence** — written via your `DatabaseNotificationStore` implementation
2. **Real-time fanout** — published via `ferro-broadcast` to channel `user.{notifiable_id}` with event `Notification.{notification_type}`

If either leg fails the dispatch returns an error — there is no partial-success silent fallback. The persistence-first ordering ensures the broadcaster can replay missed events from the store on client reconnect.

### Setup

```rust
use std::sync::Arc;
use ferro::{Broadcaster, InAppConfig, NotificationConfig, NotificationDispatcher};

let broker = Arc::new(Broadcaster::new());
let store: Arc<dyn ferro::DatabaseNotificationStore> = Arc::new(MyDatabaseStore::new());

let config = NotificationConfig::from_env()
    .with_in_app(InAppConfig {
        broker: broker.clone(),
        store: store.clone(),
    })
    .with_database_store(store);    // shared with Channel::Database — same Arc

NotificationDispatcher::configure(config);
```

> **Authorization note:** The InApp adapter publishes to `user.{id}` channels. You must configure ferro-broadcast's `ChannelAuthorizer` to enforce who can subscribe to those channels. See [Broadcasting](broadcasting.md) for the auth setup.

### Implementing Notifications

```rust
use ferro::{InAppMessage, InAppSeverity, Notification, NotificationChannel};

impl Notification for OrderShipped {
    fn via(&self) -> Vec<NotificationChannel> {
        vec![NotificationChannel::InApp]
    }

    fn to_in_app(&self) -> Option<InAppMessage> {
        Some(
            InAppMessage::new("OrderShipped")
                .data(serde_json::json!({
                    "order_id": self.order_id,
                    "tracking": self.tracking,
                }))
                .severity(InAppSeverity::Success),
        )
    }
}
```

### Mail Attachments

`MailMessage::attachment(filename, content_type, bytes)` adds an inline binary attachment. The builder is fallible — it returns `Err(Error::AttachmentTooLarge)` when `bytes.len()` exceeds the **25 MB per-attachment cap**. Multiple calls accumulate.

```rust
use ferro::MailMessage;

let pdf_bytes: Vec<u8> = std::fs::read("/tmp/invoice.pdf")?;

let mail = MailMessage::new()
    .subject("Your invoice is attached")
    .body("Hi, please find your invoice attached.")
    .attachment("invoice.pdf", "application/pdf", pdf_bytes)?;
```

Attachments work on **both** mail drivers:

- **SMTP (lettre):** ferro-notifications builds a `multipart/mixed` email with one `SinglePart` per attachment when `attachments` is non-empty. When empty, the existing single-part path is used (zero regression for non-attachment emails).
- **Resend (HTTP API):** attachments are base64-encoded and sent as `attachments: [{ filename, content }]` in the JSON payload. Resend has its own 40 MB total-per-email cap which the framework does NOT enforce — Resend surfaces it via API error.

The 25 MB cap is per-attachment and enforced before any allocation past 25 MB:

```rust
let huge: Vec<u8> = vec![0; 30 * 1024 * 1024];
match MailMessage::new().attachment("big.bin", "application/octet-stream", huge) {
    Err(ferro::NotificationError::AttachmentTooLarge { filename, size, limit }) => {
        eprintln!("Rejected '{filename}': {size} bytes exceeds {limit}-byte limit");
    }
    _ => unreachable!(),
}
```

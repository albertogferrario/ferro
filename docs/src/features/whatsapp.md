# WhatsApp Business Integration

Ferro integrates with the WhatsApp Business Cloud API (Meta Graph API v23.0) via the `ferro-whatsapp` crate. The integration covers outbound messaging (text and template messages), inbound webhook processing with HMAC verification, sender identity routing, and message deduplication.

## Prerequisites

- A Meta Developer account with a WhatsApp Business app
- A verified WhatsApp Business phone number
- A permanent access token (system user token)

WhatsApp Business API access requires Meta app review for production use.

## Installation

Add the `whatsapp` feature to your `ferro-rs` dependency:

```toml
[dependencies]
ferro-rs = { version = "0.1", features = ["whatsapp"] }
```

Generate the scaffold files:

```bash
ferro make:whatsapp
```

This creates three files in `src/whatsapp/`:
- `mod.rs` — initialization function
- `webhook.rs` — GET challenge verification and POST webhook handlers
- `listeners.rs` — event listener stubs for inbound events

## Configuration

Set the following environment variables:

| Variable | Source | Required |
|---|---|---|
| `WHATSAPP_APP_SECRET` | Meta Developer Dashboard → App Settings → Basic → App Secret | Yes |
| `WHATSAPP_ACCESS_TOKEN` | Meta Developer Dashboard → WhatsApp → API Setup → Permanent Token | Yes |
| `WHATSAPP_PHONE_NUMBER_ID` | Meta Developer Dashboard → WhatsApp → API Setup → Phone Number ID | Yes |
| `WHATSAPP_VERIFY_TOKEN` | A secret string you choose for webhook verification | Yes |

```env
WHATSAPP_APP_SECRET=your_app_secret
WHATSAPP_ACCESS_TOKEN=your_permanent_token
WHATSAPP_PHONE_NUMBER_ID=123456789012345
WHATSAPP_VERIFY_TOKEN=my_secret_verify_token
```

Call `init()` from `bootstrap.rs`:

```rust
crate::whatsapp::init();
```

The generated `src/whatsapp/mod.rs` uses `WhatsAppConfig::from_env()` with an `is_owner` closure:

```rust
use ferro::WhatsApp;

pub fn init() {
    let config = ferro::WhatsAppConfig::from_env(Box::new(|phone| {
        // phone is E.164 without '+', e.g. "393401234567"
        phone == std::env::var("OWNER_PHONE").as_deref().unwrap_or("")
    }))
    .expect("WhatsApp configuration missing.");
    WhatsApp::init(config);
}
```

## Sending Messages

### Text Messages

```rust
use ferro::{WhatsApp, WhatsAppMessage, WhatsAppSendResult};

let result: WhatsAppSendResult = WhatsApp::send(
    "393401234567",
    WhatsAppMessage::Text {
        body: "Hello from Ferro!".to_string(),
    },
)
.await?;

// result.wamid is the WhatsApp message ID for delivery status correlation
println!("Sent with wamid: {}", result.wamid);
```

### Template Messages

Template messages are required for outbound messages to contacts who have not messaged you in the last 24 hours. Templates must be approved by Meta before use.

```rust
use ferro::{WhatsApp, WhatsAppMessage};

WhatsApp::send(
    "393401234567",
    WhatsAppMessage::Template {
        name: "order_confirmation".to_string(),
        language: "it".to_string(),
        parameters: vec![
            serde_json::json!({"type": "text", "text": "ORD-12345"}),
            serde_json::json!({"type": "currency", "currency": {"fallback_value": "€29,90", "code": "EUR", "amount_1000": 29900}}),
        ],
    },
)
.await?;
```

Phone numbers are E.164 format without the `+` prefix (e.g., `"393401234567"` not `"+39 340 123 4567"`).

## Webhooks

### Route Registration

Register both routes in `src/routes.rs`:

```rust
use crate::whatsapp::webhook::{whatsapp_webhook, whatsapp_webhook_verify};

get!("/whatsapp/webhook", whatsapp_webhook_verify)
post!("/whatsapp/webhook", whatsapp_webhook)
```

### Webhook URL Configuration

In Meta Developer Dashboard → Your App → WhatsApp → Configuration:
1. Set Callback URL to `https://yourdomain.com/whatsapp/webhook`
2. Set Verify Token to the value of `WHATSAPP_VERIFY_TOKEN`
3. Subscribe to `messages` and `message_status` webhook fields

### Webhook Processing Flow

The generated `src/whatsapp/webhook.rs` follows this flow:

1. **GET** `/whatsapp/webhook` — Meta sends a challenge to verify the endpoint. The handler checks the verify token and responds with `hub.challenge` as plain text.

2. **POST** `/whatsapp/webhook` — Inbound messages and status updates arrive here. The handler:
   - Reads the raw body
   - Verifies the HMAC-SHA256 signature from `x-hub-signature-256`
   - Acknowledges immediately with `{"received": true}`
   - Dispatches a `ProcessWhatsAppWebhook` job to the queue for async processing

Always verify HMAC before parsing JSON — Meta signs the raw bytes, and JSON re-serialization can alter whitespace or Unicode escaping.

## Event Handling

`ProcessWhatsAppWebhook` parses the Meta webhook envelope and dispatches typed ferro-events:

### WhatsAppTextReceived

Emitted for each inbound text message:

```rust
use ferro::{async_trait, EventError, Listener, WhatsAppTextReceived};

pub struct HandleInboundMessage;

#[async_trait]
impl Listener<WhatsAppTextReceived> for HandleInboundMessage {
    async fn handle(&self, event: &WhatsAppTextReceived) -> Result<(), EventError> {
        println!("From: {:?}", event.sender_identity);
        println!("Text: {}", event.text);
        println!("Wamid: {}", event.wamid);
        // event.raw contains the full Meta JSON payload
        Ok(())
    }
}
```

`WhatsAppTextReceived` fields:
- `wamid: String` — WhatsApp message ID
- `sender_identity: SenderIdentity` — `Owner(phone)` or `Customer(phone)`
- `text: String` — message body
- `timestamp: chrono::DateTime<Utc>` — message timestamp
- `raw: serde_json::Value` — full Meta JSON payload

### WhatsAppStatusUpdate

Emitted for delivery status updates (sent, delivered, read, failed):

```rust
use ferro::{async_trait, DeliveryStatus, EventError, Listener, WhatsAppStatusUpdate};

pub struct HandleDeliveryStatus;

#[async_trait]
impl Listener<WhatsAppStatusUpdate> for HandleDeliveryStatus {
    async fn handle(&self, event: &WhatsAppStatusUpdate) -> Result<(), EventError> {
        match event.status {
            DeliveryStatus::Delivered => {
                println!("Message {} delivered", event.wamid);
            }
            DeliveryStatus::Read => {
                println!("Message {} read", event.wamid);
            }
            _ => {}
        }
        Ok(())
    }
}
```

`WhatsAppStatusUpdate` fields:
- `wamid: String` — correlates with `SendResult.wamid` from `WhatsApp::send()`
- `status: DeliveryStatus` — `Sent`, `Delivered`, `Read`, `Failed`, or `Unknown`
- `timestamp: chrono::DateTime<Utc>` — status update timestamp

Register listeners in `bootstrap.rs`:

```rust
use crate::whatsapp::listeners::{HandleDeliveryStatus, HandleInboundMessage};

ferro::register_listener::<WhatsAppTextReceived, HandleInboundMessage>();
ferro::register_listener::<WhatsAppStatusUpdate, HandleDeliveryStatus>();
```

## Sender Identity

The `is_owner` closure in `WhatsAppConfig` classifies each incoming phone number as either `SenderIdentity::Owner` or `SenderIdentity::Customer`. Identity is resolved before the event is dispatched, so listeners receive pre-classified events.

Phone numbers arrive from Meta in E.164 format **without** the `+` prefix:

```rust
WhatsAppConfig::from_env(Box::new(|phone| {
    // phone is "393401234567", not "+393401234567"
    phone == "393401234567"
}))
```

For DB-backed owner lookups:

```rust
WhatsAppConfig::from_env(Box::new(|phone| {
    // sync check against cached owner list
    OWNER_PHONES.contains(phone)
}))
```

`SenderIdentity` carries the phone number:

```rust
match event.sender_identity {
    SenderIdentity::Owner(phone) => println!("Message from owner: {phone}"),
    SenderIdentity::Customer(phone) => println!("Message from customer: {phone}"),
}
```

## Deduplication

Meta may deliver the same webhook multiple times. Use `InMemoryDeduplicationStore` to deduplicate by `wamid` before dispatching the job:

```rust
use ferro::{InMemoryDeduplicationStore, DeduplicationStore, ProcessWhatsAppWebhook, queue_dispatch};

static DEDUP: std::sync::OnceLock<InMemoryDeduplicationStore> = std::sync::OnceLock::new();

fn dedup_store() -> &'static InMemoryDeduplicationStore {
    DEDUP.get_or_init(InMemoryDeduplicationStore::new)
}

#[handler]
pub async fn whatsapp_webhook(req: Request) -> Response {
    // ... HMAC verification ...
    let payload: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| HttpResponse::text("Invalid JSON").status(400))?;

    // Deduplicate by wamid before queuing
    if let Some(wamid) = payload["entry"][0]["changes"][0]["value"]["messages"][0]["id"].as_str() {
        let is_duplicate = dedup_store()
            .check_and_insert(wamid)
            .await
            .unwrap_or(false);
        if is_duplicate {
            return Ok(HttpResponse::json(serde_json::json!({"received": true})));
        }
    }

    let job = ProcessWhatsAppWebhook { payload_json: body };
    queue_dispatch(job).await
        .map_err(|e| HttpResponse::text(format!("Queue error: {e}")).status(500))?;

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
```

`InMemoryDeduplicationStore` uses a DashMap with 5-minute TTL auto-expiry, which covers all reasonable Meta retry windows. For cross-restart deduplication, implement the `DeduplicationStore` trait with a Redis-backed store.

Deduplication is the application's responsibility — `ProcessWhatsAppWebhook` does not check it internally.

## MCP Tools

Two MCP tools are available for WhatsApp development assistance.

### `whatsapp_config_status`

Reports which environment variables are present or missing, and whether the scaffold directory (`src/whatsapp/`) exists. Use this to diagnose configuration issues before debugging webhook delivery or message sending failures.

### `whatsapp_webhook_events`

Scans `src/whatsapp/listeners.rs` for `Listener<T>` implementations and returns the event type and listener struct name for each. Use this to audit which WhatsApp events have listeners registered.

## Not Supported in v1

- Media messages (images, documents, audio)
- Interactive messages (buttons, list messages)
- Multi-phone-number support (multiple WhatsApp Business accounts)

These can be added as new `Message` enum variants in a future phase without breaking changes.

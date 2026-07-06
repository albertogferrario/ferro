# ferro-whatsapp

WhatsApp Business Cloud API integration for the [Ferro](https://ferro-rs.dev) web framework.

Send text and template messages via the Meta Cloud API v23.0, process inbound webhooks with
HMAC-SHA256 signature verification, and deduplicate messages using a pluggable store trait.

## Features

- **Outbound messaging** — send text messages and pre-approved template messages
- **Inbound webhook processing** — parse and validate incoming WhatsApp events
- **HMAC-SHA256 verification** — reject requests with invalid signatures automatically
- **Message deduplication** — pluggable `DeduplicationStore` trait to prevent duplicate processing
- **Event-driven** — emits `WhatsAppTextReceived` and `WhatsAppStatusUpdate` via ferro-events
- **Queue integration** — send messages asynchronously via ferro-queue workers

## Usage

```rust
use ferro_whatsapp::{WhatsApp, WhatsAppConfig, TextMessage};

// Configure at startup
let whatsapp = WhatsApp::new(WhatsAppConfig {
    phone_number_id: env::var("WHATSAPP_PHONE_NUMBER_ID")?,
    access_token: env::var("WHATSAPP_ACCESS_TOKEN")?,
    webhook_verify_token: env::var("WHATSAPP_WEBHOOK_TOKEN")?,
    app_secret: env::var("WHATSAPP_APP_SECRET")?,
});

// Send a text message
whatsapp
    .send_text(TextMessage {
        to: "+15550001234".into(),
        body: "Your order has been shipped.".into(),
    })
    .await?;
```

## Documentation

Full documentation at [docs.ferro-rs.dev](https://docs.ferro-rs.dev).

## License

MIT

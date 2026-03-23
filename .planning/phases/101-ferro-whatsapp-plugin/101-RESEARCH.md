# Phase 101: ferro-whatsapp Plugin - Research

**Researched:** 2026-03-23
**Domain:** WhatsApp Business Cloud API integration, Rust crate structuring, Ferro plugin patterns
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Outbound Message API**
- Message types: text + template messages in v1. Media support deferred (can be added as new enum variants without breaking changes)
- Single `WhatsApp::send(to, message)` entry point with `Message` enum (`Message::Text { body }`, `Message::Template { name, language, parameters }`)
- Returns `Result<SendResult { wamid: String }, Error>` — wamid enables correlation with delivery status webhooks
- No automatic retry — returns typed error variants (RateLimit, InvalidNumber, NetworkError, AuthError). Callers handle retry logic in their own handlers
- OnceLock facade pattern: `WhatsApp::init(config)` + `WhatsApp::send()` static methods (matching ferro-stripe)

**Sender-Identity Routing**
- Callback/closure-based identity resolution: `WhatsAppConfig` takes `is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>` — maximum flexibility for DB queries, tenant settings, etc.
- Default to customer when identity resolver can't determine: safe default prevents accidental owner privilege escalation
- Identity resolved at the webhook dispatcher level — event payloads arrive with identity pre-resolved, listeners don't re-implement phone matching
- `SenderIdentity` enum carries the phone number: `SenderIdentity::Owner(String)` / `SenderIdentity::Customer(String)`

**Deduplication Strategy**
- Pluggable `DeduplicationStore` trait with in-memory DashMap default + optional Redis for persistence across restarts (mirrors ConfirmationStore pattern from Phase 100)
- Messages only — status updates (delivered, read) are idempotent by nature
- 5-minute TTL window — covers all reasonable Meta retry windows without accumulating stale entries
- Claude's discretion on duplicate behavior (silent drop + debug log vs emit dedup event)

**Webhook Event Taxonomy**
- Per-type event structs: `WhatsAppTextReceived`, `WhatsAppStatusUpdate` — listeners subscribe to exactly what they care about via `Listener<T>` pattern
- v1 event types: message received + status update (sent/delivered/read/failed). Covers gestiscilo.it needs
- `WhatsAppTextReceived` payload: `{ wamid, sender_identity: SenderIdentity, text, timestamp, raw: serde_json::Value }`
- `WhatsAppStatusUpdate` payload: `{ wamid, status: DeliveryStatus, timestamp }`
- Webhook processing queued via ferro-queue: verify HMAC inline, ack 200 immediately, dispatch `ProcessWhatsAppWebhook` job (matching Stripe webhook pattern)

### Claude's Discretion
- Crate internal module structure (flat vs subdirectories)
- HMAC verification implementation details (Meta's x-hub-signature-256 header format)
- CLI `ferro make:whatsapp` scaffold contents (routes, listeners, env config)
- MCP introspection tool scope (config status, webhook events)
- DeliveryStatus enum variants
- Duplicate detection behavior: silent drop + debug log vs emit dedup event
- Meta Cloud API version pinning strategy
- Test helper design (mock webhook payloads, fake sender)

### Deferred Ideas (OUT OF SCOPE)
- Media message support (images, documents, audio) — add as new Message enum variants in a future phase
- Interactive messages (buttons, list messages) — separate phase
- WhatsApp Business API status/errors webhook event type — add if monitoring needs arise
- Multi-phone-number support (multiple WhatsApp Business accounts) — future phase
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WA-01 | `ferro-whatsapp` crate with outbound sender (text + template messages via Meta Cloud API) | Meta Cloud API v23.0 endpoint, reqwest client pattern, OnceLock facade from ferro-stripe/src/client.rs |
| WA-02 | Inbound webhook dispatcher with HMAC-SHA256 verification (X-Hub-Signature-256 header) | Meta webhook signing spec, hmac+sha2+hex deps already in ferro-stripe, sha256= prefix stripping |
| WA-03 | wamid-level message deduplication with pluggable DeduplicationStore trait | DashMap-backed InMemoryDeduplicationStore pattern from ferro-ai ConfirmationStore, 5-minute TTL |
| WA-04 | Sender-identity routing: `is_owner` closure classifies phone numbers as Owner/Customer | Closure-based resolver pattern, SenderIdentity enum, identity resolved before event dispatch |
| WA-05 | CLI scaffolding (`ferro make:whatsapp`), MCP introspection tools, docs, publish workflow integration | ferro-stripe CLI scaffold as direct template, stripe MCP tools as direct template, Wave 1 publish |
</phase_requirements>

## Summary

ferro-whatsapp follows the exact same structural pattern as ferro-stripe: a standalone optional crate with OnceLock facade, HMAC webhook verification, ferro-queue job dispatch, and ferro-events event structs. The primary external dependency is the Meta WhatsApp Business Cloud API (currently v23.0) accessed via raw reqwest HTTP calls — no dedicated Rust client library is needed or recommended given the narrow v1 scope (text + template messages only).

The Meta Cloud API webhook signature scheme uses HMAC-SHA256 of the raw request body, with the result placed in the `X-Hub-Signature-256` header prefixed with `sha256=`. This differs from Stripe (which uses `t={ts},v1={sig}` format) but uses the same underlying primitives (hmac/sha2/hex crates already in the workspace). The webhook payload structure is a nested JSON envelope under `entry[].changes[].value` containing either a `messages` array (inbound) or a `statuses` array (delivery receipts).

The deduplication system mirrors the `ConfirmationStore` pattern from ferro-ai (DashMap + TTL timer + abort handle) but simplified: store wamid strings, check-and-insert atomically, auto-expire after 5 minutes. The sender identity resolver is a simple closure stored in `WhatsAppConfig` called during webhook dispatch before event emission. CLI scaffolding and MCP introspection follow ferro-stripe patterns verbatim.

**Primary recommendation:** Model ferro-whatsapp directly on ferro-stripe. Reuse hmac/sha2/hex for HMAC, reqwest for outbound API calls, DashMap for dedup, and ferro-stripe's CLI scaffold and MCP tool patterns.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `reqwest` | 0.12 | HTTP client for Meta Cloud API calls | Already used in ferro-ai and ferro-notifications; workspace standard |
| `serde` + `serde_json` | 1 | JSON serialization for API payloads and webhook bodies | Workspace standard |
| `hmac` + `sha2` + `hex` | 0.12, 0.10, 0.4 | HMAC-SHA256 webhook signature verification | Already used in ferro-stripe; exact same primitives needed |
| `thiserror` | 2 | Error type derivation | New crate convention (ferro-lang, ferro-stripe, ferro-theme all use v2) |
| `async-trait` | 0.1 | Async trait methods for DeduplicationStore | Workspace standard |
| `dashmap` | 6 | Concurrent HashMap for InMemoryDeduplicationStore | Already used in ferro-ai for ConfirmationStore |
| `tokio` | 1 | Async runtime, time for TTL timers | Workspace standard |
| `tracing` | 0.1 | Debug logging for dedup drops and webhook events | Already used in ferro-ai |
| `chrono` | 0.4 (with serde) | Timestamp conversion from Unix epoch strings | Workspace standard |
| `ferro-events` | 0.1 | WhatsAppTextReceived, WhatsAppStatusUpdate event dispatch | Direct dependency |
| `ferro-queue` | 0.1 | ProcessWhatsAppWebhook job | Direct dependency |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `dotenvy` | 0.15 | Load .env in MCP tools (already used in stripe MCP) | MCP tool config status only |
| `regex` | 1 | Source scanning for MCP listener discovery | MCP tools only |
| `tempfile` | 3 | Temp dirs in CLI scaffold tests | Test helpers only |
| `console` | 0.15 | Colored CLI output | Already used in ferro-cli make_stripe |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Raw reqwest | `whatsapp-cloud-api` crate (0.3.1 on crates.io) | Crate is low-maintenance and adds a dependency for a narrow scope; raw reqwest gives full control and avoids version coupling |
| DashMap TTL | `moka` cache | moka is already a workspace dep (in framework), but DashMap + tokio::time::sleep matches the confirmed ConfirmationStore pattern exactly |

**Installation:**
```bash
# In ferro-whatsapp/Cargo.toml — inherits workspace versions
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
thiserror = "2"
async-trait = "0.1"
dashmap = "6"
tokio = { version = "1", features = ["time", "rt"] }
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
ferro-events = { path = "../ferro-events", version = "0.1" }
ferro-queue = { path = "../ferro-queue", version = "0.1" }
```

## Architecture Patterns

### Recommended Project Structure
```
ferro-whatsapp/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # Public API, pub use re-exports
    ├── config.rs       # WhatsAppConfig, from_env(), is_owner closure
    ├── client.rs       # WhatsApp facade (OnceLock, init, send)
    ├── message.rs      # Message enum, SendResult
    ├── error.rs        # Error enum (thiserror)
    ├── dedup.rs        # DeduplicationStore trait + InMemoryDeduplicationStore
    └── webhook/
        ├── mod.rs      # verify_webhook(), HMAC verification
        ├── events.rs   # WhatsAppTextReceived, WhatsAppStatusUpdate, ProcessWhatsAppWebhook
        └── handler.rs  # webhook_payload_to_job() helper (optional, matches stripe pattern)
```

Corresponding user-facing scaffold (generated by `ferro make:whatsapp`):
```
src/whatsapp/
├── mod.rs          # init(), module declarations
├── webhook.rs      # #[handler] whatsapp_webhook
└── listeners.rs    # Listener<WhatsAppTextReceived>, Listener<WhatsAppStatusUpdate>
```

### Pattern 1: OnceLock Facade (WhatsApp::init / WhatsApp::send)
**What:** Static initialization of config and reqwest client; all send calls go through `WhatsApp::send()`.
**When to use:** Application startup initialization; called once in bootstrap.rs.
**Example:**
```rust
// Source: ferro-stripe/src/client.rs — direct template
use std::sync::OnceLock;
use crate::WhatsAppConfig;

static WA_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static WA_CONFIG: OnceLock<WhatsAppConfig> = OnceLock::new();

pub struct WhatsApp;

impl WhatsApp {
    pub fn init(config: WhatsAppConfig) {
        let client = reqwest::Client::new();
        WA_CLIENT.set(client).ok();
        WA_CONFIG.set(config).ok();
    }

    pub fn config() -> &'static WhatsAppConfig {
        WA_CONFIG.get().expect("WhatsApp::init() not called")
    }

    pub async fn send(to: &str, message: Message) -> Result<SendResult, Error> {
        let config = Self::config();
        let client = WA_CLIENT.get().expect("WhatsApp::init() not called");
        send_message(client, config, to, message).await
    }
}
```

### Pattern 2: Meta Cloud API v23.0 Message Send
**What:** POST to `https://graph.facebook.com/v23.0/{phone_number_id}/messages` with Bearer token.
**When to use:** All outbound message sends.
**Example:**
```rust
// Source: Meta official docs, verified via web search 2026-03-23
// Text message payload
let body = serde_json::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": to,           // E.164 format without +, e.g. "14155551234"
    "type": "text",
    "text": { "body": text_body }
});

// Template message payload
let body = serde_json::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": to,
    "type": "template",
    "template": {
        "name": template_name,
        "language": { "code": language },  // e.g. "en_US", "it"
        "components": parameters            // Vec<serde_json::Value>
    }
});

let resp = client
    .post(format!("https://graph.facebook.com/v23.0/{}/messages", config.phone_number_id))
    .bearer_auth(&config.access_token)
    .json(&body)
    .send()
    .await?;

// Success response contains: { "messages": [{ "id": "wamid.xxx" }] }
let wamid = resp.json::<serde_json::Value>().await?
    ["messages"][0]["id"].as_str()?.to_string();
```

### Pattern 3: Meta Webhook HMAC-SHA256 Verification
**What:** Compute HMAC-SHA256 of raw body using app secret; compare to `X-Hub-Signature-256` header (strip `sha256=` prefix). Use constant-time comparison.
**When to use:** Every inbound POST to the webhook endpoint before any processing.
**Example:**
```rust
// Source: Meta official docs, ferro-stripe/src/webhook/events.rs (same hmac/sha2/hex deps)
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn verify_webhook(raw_body: &[u8], signature_header: &str, app_secret: &str) -> Result<(), Error> {
    // Header format: "sha256=<hex_digest>"
    let expected_sig = signature_header
        .strip_prefix("sha256=")
        .ok_or(Error::WebhookVerification("Missing sha256= prefix".into()))?;

    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(raw_body);
    let computed = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison prevents timing attacks
    if !constant_time_eq(computed.as_bytes(), expected_sig.as_bytes()) {
        return Err(Error::WebhookVerification("Signature mismatch".into()));
    }
    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
```

### Pattern 4: Webhook Payload Parsing
**What:** Navigate the Meta webhook JSON envelope to extract message or status data.
**When to use:** Inside `ProcessWhatsAppWebhook::handle()` to build ferro-events events.
**Example:**
```rust
// Source: Meta official docs, verified 2026-03-23
// Inbound message path: entry[0].changes[0].value.messages[0]
let v: serde_json::Value = serde_json::from_str(&self.payload_json)?;
let value = &v["entry"][0]["changes"][0]["value"];

if let Some(messages) = value["messages"].as_array() {
    for msg in messages {
        let wamid = msg["id"].as_str()?.to_string();
        let from_phone = msg["from"].as_str()?.to_string(); // E.164 without +
        let timestamp: i64 = msg["timestamp"].as_str()?.parse()?;
        let text_body = msg["text"]["body"].as_str()?.to_string();
        // msg["type"] == "text" for text messages
    }
}

// Status update path: entry[0].changes[0].value.statuses[0]
if let Some(statuses) = value["statuses"].as_array() {
    for status in statuses {
        let wamid = status["id"].as_str()?.to_string();
        let status_str = status["status"].as_str()?; // "sent", "delivered", "read", "failed"
    }
}
```

### Pattern 5: DeduplicationStore Trait (mirrors ConfirmationStore)
**What:** Trait for wamid-based message deduplication; InMemoryDeduplicationStore uses DashMap + TTL timers.
**When to use:** At start of webhook job processing before dispatching ferro-events.
**Example:**
```rust
// Source: ferro-ai/src/confirmation/mod.rs and store.rs — direct template
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait DeduplicationStore: Send + Sync {
    /// Returns true if wamid was already seen (duplicate). Inserts if new.
    async fn check_and_insert(&self, wamid: &str) -> Result<bool, Error>;
}

// InMemoryDeduplicationStore: DashMap<String, ()> with TTL expiry
// TTL = Duration::from_secs(300) (5 minutes, covers Meta retry windows)
// On duplicate: silent drop + tracing::debug! log
```

### Pattern 6: Webhook Flow (matches ferro-stripe)
**What:** Handler verifies HMAC inline, acks 200 immediately, dispatches job to queue.
**When to use:** Always — prevents Meta from timing out and retrying.
**Example:**
```rust
// Source: ferro-stripe/src/commands/make_stripe.rs webhook template
#[handler]
pub async fn whatsapp_webhook(req: Request) -> Response {
    let sig = req.header("x-hub-signature-256")
        .ok_or_else(|| HttpResponse::text("Missing signature").status(400))?;
    let body = req.body_string().await
        .map_err(|_| HttpResponse::text("Failed to read body").status(400))?;

    ferro::verify_whatsapp_webhook(body.as_bytes(), &sig, &WhatsApp::config().app_secret)
        .map_err(|_| HttpResponse::text("Invalid signature").status(400))?;

    let job = ProcessWhatsAppWebhook { payload_json: body };
    ferro::queue_dispatch(job).await
        .map_err(|e| HttpResponse::text(format!("Queue error: {e}")).status(500))?;

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
```

### Pattern 7: Webhook GET Verification (Meta challenge)
**What:** Meta sends a GET request to verify the webhook endpoint before enabling it. Must respond with hub.challenge.
**When to use:** Route registration alongside the POST handler.
**Example:**
```rust
// Source: Meta official docs
#[handler]
pub async fn whatsapp_webhook_verify(req: Request) -> Response {
    let mode = req.query("hub.mode").unwrap_or_default();
    let token = req.query("hub.verify_token").unwrap_or_default();
    let challenge = req.query("hub.challenge").unwrap_or_default();

    if mode == "subscribe" && token == WhatsApp::config().verify_token {
        Ok(HttpResponse::text(challenge))
    } else {
        Err(HttpResponse::text("Forbidden").status(403))
    }
}
```

### Anti-Patterns to Avoid
- **Process webhook synchronously:** Meta requires 200 within 5 seconds. Always queue via ferro-queue.
- **Parse JSON before HMAC verification:** Verify the raw body first; JSON parsing may alter byte representation.
- **Skip constant-time comparison:** Use XOR accumulation, not `==` on strings, to prevent timing attacks.
- **Store `is_owner` as a plain fn pointer:** Use `Box<dyn Fn(&str) -> bool + Send + Sync>` to allow closures capturing DB connections.
- **Hard-code API version in multiple places:** Define `const META_API_VERSION: &str = "v23.0"` once in config.rs.
- **Panic on duplicate wamid:** Silent drop + debug log is the correct behavior; panicking crashes the job.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HMAC-SHA256 computation | Custom SHA256 implementation | `hmac` + `sha2` + `hex` crates | Already in workspace via ferro-stripe; constant-time safety is non-trivial |
| Concurrent map with expiry | Custom HashMap + Mutex | `dashmap` + `tokio::time::sleep` + `AbortHandle` | Exact pattern from ferro-ai ConfirmationStore — proven, tested |
| HTTP client for API calls | Custom TCP connection | `reqwest` 0.12 with `json` feature | Workspace standard, already in ferro-ai and ferro-notifications |
| JSON parsing of Meta payload | Custom parser | `serde_json::Value` path navigation | Meta's nested envelope structure handled cleanly with Value indexing |

**Key insight:** The webhook verification and dedup patterns are subtle (timing attacks, TTL cancellation, DashMap guard lifetime). Use established patterns from ferro-stripe and ferro-ai rather than reimplementing.

## Common Pitfalls

### Pitfall 1: Stripe vs Meta HMAC Header Format
**What goes wrong:** The Stripe signature header format is `t={timestamp},v1={sig}` but Meta's is `sha256={hex_digest}`. Using the Stripe verify_webhook function for Meta webhooks will always fail.
**Why it happens:** Both use HMAC-SHA256 but different envelope formats.
**How to avoid:** Implement a separate `verify_whatsapp_webhook(body: &[u8], sig: &str, secret: &str)` that strips `sha256=` prefix — do not share code with Stripe's verifier.
**Warning signs:** Signature verification always failing even with correct secret.

### Pitfall 2: Raw Body Required for HMAC
**What goes wrong:** If the framework or middleware has already parsed/re-serialized the JSON body before HMAC verification, the byte representation may differ from what Meta signed, causing HMAC failures.
**Why it happens:** JSON re-serialization can change whitespace, key order, or Unicode escaping.
**How to avoid:** Read raw body string first, verify HMAC against raw bytes, then parse JSON. Use `req.body_string().await` which gives the raw body.
**Warning signs:** Intermittent HMAC failures, especially for messages with non-ASCII characters.

### Pitfall 3: Meta Webhook GET Challenge Must Return Plain Text
**What goes wrong:** Returning JSON `{"challenge": "..."}` for the GET verification fails. Meta expects the raw challenge string as plain text body.
**Why it happens:** Meta documentation is explicit: respond with the hub.challenge value as plain text.
**How to avoid:** Use `HttpResponse::text(challenge)` not `HttpResponse::json(...)`.
**Warning signs:** Webhook endpoint never activates in Meta developer dashboard.

### Pitfall 4: Phone Number Format Inconsistency
**What goes wrong:** Meta delivers phone numbers without `+` prefix (E.164 without leading +, e.g. `393401234567`). If `is_owner` closure compares with `+393401234567`, matching fails.
**Why it happens:** Meta's internal format omits the `+`.
**How to avoid:** Document in `WhatsAppConfig` that `is_owner` receives numbers without `+` prefix. Normalize consistently.
**Warning signs:** Owner identity never resolves even for known owner numbers.

### Pitfall 5: DashMap Guard Held Across Await
**What goes wrong:** Holding a DashMap `Ref` or `RefMut` guard across an `.await` point causes a compilation error (`DashMap` guards are not `Send`).
**Why it happens:** DashMap shard locks are not Send.
**How to avoid:** Clone the value out of the guard before any `.await`. Pattern: `let val = map.get(key).map(|r| r.clone());` — guard drops at end of statement.
**Warning signs:** Compiler error: "future is not Send ... because DashMap reference is not Send".

### Pitfall 6: Meta API Version Drift
**What goes wrong:** Pinning to `v21.0` when Meta has released `v23.0` causes deprecation warnings and eventually API unavailability.
**Why it happens:** Meta releases new Graph API versions every few months and deprecates old ones.
**How to avoid:** Define `const META_API_VERSION: &str = "v23.0"` in config.rs, document the version in README, plan to bump periodically.
**Warning signs:** Meta API returns deprecation warnings in response headers.

### Pitfall 7: Template Message Parameters as Vec<Value>
**What goes wrong:** `Message::Template { parameters: Vec<String> }` is too simple — Meta's template API accepts typed parameter objects (`{"type": "text", "text": "..."}`, `{"type": "currency", ...}`).
**Why it happens:** The locked decision uses `parameters` as `Vec<serde_json::Value>` for flexibility.
**How to avoid:** Keep `parameters` as `Vec<serde_json::Value>` in the `Message::Template` variant. Document that each element must be a typed parameter object per Meta spec.
**Warning signs:** Template messages rejected with "invalid parameter type" errors.

## Code Examples

Verified patterns from official sources and established codebase:

### WhatsAppConfig struct
```rust
// Based on ferro-stripe/src/config.rs pattern
pub struct WhatsAppConfig {
    /// Meta app secret for HMAC verification (not the access token).
    pub app_secret: String,
    /// System user access token for sending messages.
    pub access_token: String,
    /// Phone Number ID from Meta developer dashboard.
    pub phone_number_id: String,
    /// Verify token for GET endpoint challenge verification.
    pub verify_token: String,
    /// Closure for classifying a phone number as owner or customer.
    /// Receives phone number in E.164 format without '+' (as Meta delivers it).
    pub is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl WhatsAppConfig {
    pub fn from_env(is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Result<Self, Error> {
        Ok(Self {
            app_secret: std::env::var("WHATSAPP_APP_SECRET")
                .map_err(|_| Error::Config("WHATSAPP_APP_SECRET not set".into()))?,
            access_token: std::env::var("WHATSAPP_ACCESS_TOKEN")
                .map_err(|_| Error::Config("WHATSAPP_ACCESS_TOKEN not set".into()))?,
            phone_number_id: std::env::var("WHATSAPP_PHONE_NUMBER_ID")
                .map_err(|_| Error::Config("WHATSAPP_PHONE_NUMBER_ID not set".into()))?,
            verify_token: std::env::var("WHATSAPP_VERIFY_TOKEN")
                .map_err(|_| Error::Config("WHATSAPP_VERIFY_TOKEN not set".into()))?,
            is_owner,
        })
    }
}
```

### Message enum and SendResult
```rust
pub enum Message {
    Text { body: String },
    Template {
        name: String,
        language: String,
        /// Typed parameter objects: [{"type": "text", "text": "..."}, ...]
        parameters: Vec<serde_json::Value>,
    },
}

pub struct SendResult {
    /// WhatsApp message ID returned by Meta API. Use for delivery status correlation.
    pub wamid: String,
}
```

### Error enum
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),
    #[error("rate limit exceeded")]
    RateLimit,
    #[error("invalid phone number")]
    InvalidNumber,
    #[error("authentication error")]
    AuthError,
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("api error {status}: {message}")]
    ApiError { status: u16, message: String },
}
```

### DeliveryStatus enum
```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
    /// Catch-all for unknown status strings from Meta
    #[serde(other)]
    Unknown,
}
```

### Test helper (signed webhook payload for Meta format)
```rust
// Analogous to ferro-stripe/src/webhook/events.rs signed_webhook_payload
// but using Meta's sha256= prefix format
#[cfg(any(test, feature = "test-helpers"))]
pub fn signed_whatsapp_payload(raw_body: &[u8], app_secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(raw_body);
    let digest = hex::encode(mac.finalize().into_bytes());
    format!("sha256={digest}")
}
```

### MCP whatsapp_config_status tool pattern
```rust
// Source: ferro-mcp/src/tools/stripe.rs — direct template
pub struct WhatsAppConfigStatus {
    pub configured: bool,
    pub keys_present: Vec<String>,
    pub keys_missing: Vec<String>,
    pub scaffold_exists: bool,
    pub scaffold_files: Vec<String>,
}

pub fn whatsapp_config_status(project_root: &Path) -> WhatsAppConfigStatus {
    let required_keys = [
        "WHATSAPP_APP_SECRET",
        "WHATSAPP_ACCESS_TOKEN",
        "WHATSAPP_PHONE_NUMBER_ID",
        "WHATSAPP_VERIFY_TOKEN",
    ];
    // ... same pattern as stripe_config_status
    // scaffold_dir = project_root.join("src/whatsapp")
}
```

### CLI scaffold make:whatsapp files
Files generated by `ferro make:whatsapp`:
- `src/whatsapp/mod.rs` — `init()` function calling `WhatsApp::init(config)`
- `src/whatsapp/webhook.rs` — POST and GET handlers
- `src/whatsapp/listeners.rs` — `Listener<WhatsAppTextReceived>` and `Listener<WhatsAppStatusUpdate>` stubs

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WhatsApp On-Premises API | Cloud API only | Oct 23, 2025 (sunset) | On-premises no longer works; Cloud API is the only option |
| Graph API v21.0 | v23.0 current | 2026-03 | Pin to v23.0; v21.0 still works but deprecating |
| Per-connection reqwest::Client | Shared client via OnceLock | N/A — this is the established pattern | Single client reuse across all sends |

**Deprecated/outdated:**
- WhatsApp On-Premises API: sunset October 23, 2025 — do not reference in docs.
- `whatsapp-cloud-api` crate on crates.io (0.3.1): low maintenance, narrow coverage, avoid dependency.

## Open Questions

1. **Redis DeduplicationStore implementation**
   - What we know: locked decision allows optional Redis backend (mirrors ConfirmationStore)
   - What's unclear: Whether to implement Redis variant in Phase 101 or defer to a follow-up
   - Recommendation: Include the trait, implement InMemoryDeduplicationStore in v1. Add Redis implementation if gestiscilo.it explicitly needs cross-restart dedup.

2. **Duplicate behavior: silent drop vs dedup event**
   - What we know: Claude's discretion (from CONTEXT.md)
   - What's unclear: gestiscilo.it's operational need — do operators need to audit duplicates?
   - Recommendation: Silent drop + `tracing::debug!("duplicate wamid: {wamid}")` for v1. No dedup event — keeps the event taxonomy clean and reduces listener surface area.

3. **Meta API version pinning strategy**
   - What we know: Current version is v23.0 (verified March 2026); Meta releases new versions every ~6 months and deprecates old ones after ~2 years
   - What's unclear: Whether to make version configurable via config or hard-code
   - Recommendation: Hard-code `const META_API_VERSION: &str = "v23.0"` in a single location (config.rs or client.rs). Document the version in README. Version overriding is not needed for v1.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | None (workspace-level) |
| Quick run command | `cargo test -p ferro-whatsapp` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WA-01 | WhatsApp::send returns SendResult with wamid | unit (mock HTTP) | `cargo test -p ferro-whatsapp test_send_text` | Wave 0 |
| WA-01 | WhatsApp::send template message builds correct payload | unit | `cargo test -p ferro-whatsapp test_send_template_payload` | Wave 0 |
| WA-01 | Error variants map correctly from HTTP status codes | unit | `cargo test -p ferro-whatsapp test_error_mapping` | Wave 0 |
| WA-02 | verify_webhook accepts valid sha256= signature | unit | `cargo test -p ferro-whatsapp verify_webhook_valid` | Wave 0 |
| WA-02 | verify_webhook rejects tampered body | unit | `cargo test -p ferro-whatsapp verify_webhook_tampered` | Wave 0 |
| WA-02 | verify_webhook rejects wrong secret | unit | `cargo test -p ferro-whatsapp verify_webhook_wrong_secret` | Wave 0 |
| WA-02 | verify_webhook rejects missing sha256= prefix | unit | `cargo test -p ferro-whatsapp verify_webhook_bad_prefix` | Wave 0 |
| WA-03 | InMemoryDeduplicationStore returns false first time | unit | `cargo test -p ferro-whatsapp dedup_first_insert` | Wave 0 |
| WA-03 | InMemoryDeduplicationStore returns true on duplicate | unit | `cargo test -p ferro-whatsapp dedup_duplicate` | Wave 0 |
| WA-03 | InMemoryDeduplicationStore entries expire after 5 min | unit (paused clock) | `cargo test -p ferro-whatsapp dedup_ttl_expiry` | Wave 0 |
| WA-04 | SenderIdentity::Owner when is_owner returns true | unit | `cargo test -p ferro-whatsapp sender_identity_owner` | Wave 0 |
| WA-04 | SenderIdentity::Customer when is_owner returns false | unit | `cargo test -p ferro-whatsapp sender_identity_customer` | Wave 0 |
| WA-05 | make:whatsapp generates mod.rs, webhook.rs, listeners.rs | unit | `cargo test -p ferro-cli make_whatsapp_generates_files` | Wave 0 |
| WA-05 | make:whatsapp does not overwrite existing files | unit | `cargo test -p ferro-cli make_whatsapp_no_overwrite` | Wave 0 |
| WA-05 | whatsapp_config_status reports missing env vars | unit | `cargo test -p ferro-mcp whatsapp_config_status_missing` | Wave 0 |
| WA-05 | whatsapp_webhook_events scans listeners.rs | unit | `cargo test -p ferro-mcp whatsapp_webhook_events_parsed` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-whatsapp`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `ferro-whatsapp/src/` — entire new crate, create with `Cargo.toml`, `src/lib.rs`, all modules
- [ ] `ferro-whatsapp/Cargo.toml` — workspace member entry
- [ ] Update `/Cargo.toml` `members` array to include `"ferro-whatsapp"`
- [ ] Update `framework/Cargo.toml` to add optional `ferro-whatsapp` dependency and `whatsapp` feature flag
- [ ] Update `.github/workflows/publish.yml` Wave 1 CRATES list to include `ferro-whatsapp`

## Sources

### Primary (HIGH confidence)
- `ferro-stripe/src/client.rs` — OnceLock facade pattern (direct template)
- `ferro-stripe/src/webhook/mod.rs` — HMAC verification structure (direct template)
- `ferro-stripe/src/webhook/events.rs` — ProcessJob + event structs (direct template)
- `ferro-stripe/src/config.rs` — Config + from_env() pattern (direct template)
- `ferro-cli/src/commands/make_stripe.rs` — CLI scaffold pattern (direct template)
- `ferro-mcp/src/tools/stripe.rs` — MCP introspection tools (direct template)
- `ferro-ai/src/confirmation/store.rs` — DashMap + TTL dedup pattern (direct template)
- `ferro-ai/src/confirmation/mod.rs` — ConfirmationStore trait (dedup trait template)
- `framework/Cargo.toml` — Feature flag conventions (direct reference)
- `.github/workflows/publish.yml` — Wave 1 publish pattern (direct reference)

### Secondary (MEDIUM confidence)
- Meta official docs via web search (2026-03-23): v23.0 API endpoint, webhook payload structure, X-Hub-Signature-256 format, GET challenge verification
- Web search 2026-03-23: `sha256=` prefix confirmed, constant-time comparison requirement confirmed, E.164 without `+` confirmed

### Tertiary (LOW confidence)
- None — all critical findings verified against official sources or existing codebase patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps already in workspace or ferro-stripe/ferro-ai
- Architecture: HIGH — direct 1:1 template from ferro-stripe
- Meta API details: MEDIUM — verified via web search against official Meta docs; no Context7 source for Meta API
- Pitfalls: HIGH — combination of official docs (timing attack, raw body) and codebase experience (DashMap guards, phone format)

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (Meta API version may change; stable library stack otherwise)

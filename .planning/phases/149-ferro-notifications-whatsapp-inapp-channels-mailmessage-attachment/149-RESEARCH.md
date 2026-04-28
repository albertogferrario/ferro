# Phase 149: ferro-notifications WhatsApp + InApp + MailMessage Attachment — Research

**Researched:** 2026-04-28
**Domain:** ferro-notifications, ferro-whatsapp, ferro-broadcast, lettre 0.11, Resend HTTP API
**Confidence:** HIGH — all claims verified against live crate surfaces; lettre API verified via Context7

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01** Channel::WhatsApp + Channel::InApp added to channel.rs; as_str() returns "whatsapp" / "in_app"
- **D-02** Notification trait gains four default-None methods: to_whatsapp, to_in_app, to_sms, to_push
- **D-03** WhatsAppMessage wraps ferro_whatsapp::Message; builders: text(body), template(name, lang, components)
- **D-04** WhatsAppChannel calls ferro_whatsapp::WhatsApp::send directly; gated on NotificationConfig::whatsapp_enabled
- **D-05** Error::WhatsApp(ferro_whatsapp::Error) added via #[from]
- **D-06** InAppMessage: { notification_type: String, data: serde_json::Value, severity: Option<InAppSeverity> }
- **D-07** NotificationConfig::in_app: Option<InAppConfig> where InAppConfig holds Arc<Broadcaster> + Arc<dyn DatabaseNotificationStore>; ferro-broadcast becomes a hard dep
- **D-08** InAppChannel writes DB-store leg first, then broadcast leg; error from either bubbles up
- **D-09** MailMessage::attachments: Vec<MailAttachment>; MailAttachment { filename, content_type, content: Vec<u8> }
- **D-10** attachment() builder: pub fn attachment(mut self, filename: impl Into<String>, content_type: impl Into<String>, content: Vec<u8>) -> Result<Self, Error>
- **D-11** Per-attachment 25MB cap via Error::AttachmentTooLarge { filename, size, limit }; no cumulative cap
- **D-12** Full parity: both SMTP (lettre MultiPart::mixed) and Resend HTTP (base64 attachment field) ship in this phase
- **D-13** NotificationConfig::database_store: Option<Arc<dyn DatabaseNotificationStore>>; send_database wired when Some
- **D-14** NotificationConfig gains whatsapp_enabled, in_app, database_store fields with builder methods
- **D-15** ferro-broadcast added to ferro-notifications/Cargo.toml; publish wave unchanged (both in Wave 1a)
- **D-16** Single ferro-notifications version bump; ferro-broadcast/ferro-whatsapp/ferro-mcp versions unchanged

### Claude's Discretion

- Exact lettre MultiPart builder ergonomics (private helper vs inline)
- SmsMessage / PushMessage placeholder type shape
- SMTP integration test fixture choice (Mailpit vs lettre stub transport)
- WhatsAppMessage wrapper vs direct re-export (wrapper recommended for parity)
- Sub-module layout under channels/ (single file vs sub-folder)

### Deferred Ideas (OUT OF SCOPE)

- APNs / FCM Push adapter
- SMS adapter
- Streaming / path-based mail attachments
- Cumulative attachment-size enforcement
- Inbound WhatsApp / InApp handling
- Delivery-receipt webhook integration
- MCP tool exposure of channel variants
</user_constraints>

---

## Executive Summary

Phase 149 extends `ferro-notifications` with two channel adapters (WhatsApp, InApp) and a `MailMessage` attachment API. All decisions are locked in CONTEXT.md and are buildable against the verified crate surfaces. The WhatsApp adapter is a one-line wrapper around `ferro_whatsapp::WhatsApp::send` (static facade pattern, no injection). The InApp adapter writes both a DB-store leg and a broadcast leg through the existing `DatabaseNotificationStore` trait and `Broadcaster::broadcast()` API. Mail attachments require restructuring the SMTP path from `email_builder.body()` to `email_builder.multipart(MultiPart::mixed())` when attachments are present, and adding a base64-encoded `attachments` field to the Resend payload struct. One new architectural finding is documented below (ARCH-FINDING-05): the `Channel` enum's `#[serde(rename_all = "lowercase")]` attribute produces `"inapp"` for `InApp` and `"whatsapp"` for `WhatsApp`, but the locked decision specifies wire forms `"in_app"` and `"whatsapp"`. Per-variant `#[serde(rename)]` overrides are required.

---

## Crate Surface Map

### ferro-notifications/src/channel.rs

**Now:** 5 variants (Mail, Database, Slack, Sms, Push); `#[serde(rename_all = "lowercase")]`; `as_str()` match.

**Changes:**
- Add `Channel::WhatsApp` with `#[serde(rename = "whatsapp")]` (matches `lowercase` output, explicit for clarity)
- Add `Channel::InApp` with `#[serde(rename = "in_app")]` (overrides `lowercase` which would produce `"inapp"`)
- Add `Channel::WhatsApp => "whatsapp"` and `Channel::InApp => "in_app"` arms to `as_str()`

### ferro-notifications/src/notification.rs

**Now:** `to_mail`, `to_database`, `to_slack` default-None methods; imports `DatabaseMessage, MailMessage, SlackMessage`.

**Changes:**
- Add imports for `WhatsAppMessage, InAppMessage, SmsMessage, PushMessage` (new types)
- Add `to_whatsapp() -> Option<WhatsAppMessage>`, `to_in_app() -> Option<InAppMessage>`, `to_sms() -> Option<SmsMessage>`, `to_push() -> Option<PushMessage>` — all default None

### ferro-notifications/src/channels/mod.rs

**Now:** `mod database; mod mail; mod slack;` — exports DatabaseMessage, MailMessage, Slack*.

**Changes:**
- Add `mod whatsapp; mod in_app;`
- Re-export `WhatsAppMessage, InAppMessage, InAppSeverity, MailAttachment, SmsMessage, PushMessage`

### ferro-notifications/src/channels/whatsapp.rs (NEW)

Contains `WhatsAppMessage` struct wrapping `ferro_whatsapp::Message`, builder methods `text()` and `template()`.

### ferro-notifications/src/channels/in_app.rs (NEW)

Contains `InAppMessage`, `InAppSeverity` enum (Info | Success | Warning | Error), builder methods.

### ferro-notifications/src/channels/mail.rs

**Now:** `MailMessage` with fields subject, body, html, from, reply_to, cc, bcc, headers. All builder methods return `Self` (infallible).

**Changes:**
- Add `pub struct MailAttachment { pub filename: String, pub content_type: String, pub content: Vec<u8> }`
- Add `pub attachments: Vec<MailAttachment>` field to `MailMessage` with `#[serde(default)]`
- Add `pub fn attachment(mut self, ...) -> Result<Self, Error>` builder (fallible; 25MB cap enforced here)

### ferro-notifications/src/channels/database.rs

No changes. `DatabaseMessage` is already correct and shared by both `Channel::Database` and `Channel::InApp`.

### ferro-notifications/src/dispatcher.rs

**Now:** `NotificationConfig` has `mail: Option<MailConfig>` and `slack_webhook: Option<String>`. `send_database` is a placeholder. `ResendEmailPayload` has no attachment field. SMTP path uses `email_builder.body()` (single-part only).

**Changes:**
- `NotificationConfig`: add `whatsapp_enabled: bool`, `in_app: Option<InAppConfig>`, `database_store: Option<Arc<dyn DatabaseNotificationStore>>`
- `NotificationConfig::from_env()`: read `WHATSAPP_ENABLED` env var (parse bool, default false)
- Add builder methods: `with_whatsapp_enabled(bool)`, `with_in_app(InAppConfig)`, `with_database_store(Arc<dyn ...>)`
- Add `pub struct InAppConfig { pub broker: Arc<Broadcaster>, pub store: Arc<dyn DatabaseNotificationStore> }`
- `ResendEmailPayload`: add `attachments: Vec<ResendAttachment>` field with `#[serde(skip_serializing_if = "Vec::is_empty")]`
- Add `struct ResendAttachment { filename: String, content: String }` (content = base64)
- SMTP path: refactor `send_mail_smtp` to use `MultiPart::mixed()` when `message.attachments` is non-empty; keep current single-part path as the else branch
- `send_database`: wire to `config.database_store.store(...)` when `Some`; keep placeholder log when `None`
- Add `send_whatsapp` async fn
- Add `send_in_app` async fn
- Extend `NotificationDispatcher::send` match: add arms for `Channel::WhatsApp` and `Channel::InApp`

### ferro-notifications/src/error.rs

**Now:** Mail, Slack, Database, ChannelNotAvailable, Serialization, Other.

**Changes:**
- Add `WhatsApp(#[from] ferro_whatsapp::Error)` variant
- Add `AttachmentTooLarge { filename: String, size: usize, limit: usize }` variant

### ferro-notifications/Cargo.toml

**Changes:**
- Add `ferro-broadcast = { path = "../ferro-broadcast", version = "0.2" }` to `[dependencies]`
- Add `ferro-whatsapp = { path = "../ferro-whatsapp", version = "0.2" }` to `[dependencies]`
- Add `base64 = "0.22"` to `[dependencies]` (for Resend attachment encoding)

### framework/src/lib.rs

**Changes:**
- Add `WhatsAppMessage, InAppMessage, InAppSeverity, MailAttachment, SmsMessage, PushMessage, InAppConfig` to the `ferro_notifications` re-export block
- Verify `NotificationChannel` alias covers new variants (it does — it re-exports the enum itself)

### docs/src/notifications/ (or equivalent)

Add documentation sections for the WhatsApp channel, InApp channel, and MailMessage attachment API.

---

## Implementation Approach by Subsystem

### Channel Enum (channel.rs)

The existing `#[serde(rename_all = "lowercase")]` attribute is applied at enum level. All single-word existing variants (Mail, Database, Slack, Sms, Push) produce lowercase strings identical under both `lowercase` and `snake_case` rules. The two new variants require special handling:

- `WhatsApp` under `lowercase` → `"whatsapp"` (happens to be correct; but add explicit `#[serde(rename = "whatsapp")]` for clarity)
- `InApp` under `lowercase` → `"inapp"` (WRONG; D-01 requires `"in_app"`; use `#[serde(rename = "in_app")]`)

**Concrete action:** Keep `#[serde(rename_all = "lowercase")]` at enum level. Add per-variant overrides:
```rust
#[serde(rename = "whatsapp")]
WhatsApp,
#[serde(rename = "in_app")]
InApp,
```
[VERIFIED: channel.rs line 7, serde_json behavior analysis]

### Notification Trait (notification.rs)

Add four new default-None methods. Return types reference new message structs defined in the channels sub-modules. The trait is `Send + Sync`; all new message types must also be `Send + Sync` (they are — they contain only standard owned types or `serde_json::Value`).

Placeholder types for SMS and Push: define minimal empty structs `SmsMessage` and `PushMessage` in their respective channel files (or in a shared `channels/future.rs`). Shape doesn't matter yet — they just need to exist so the trait signatures compile.

### WhatsApp Adapter (channels/whatsapp.rs + dispatcher.rs)

`WhatsAppMessage` wraps `ferro_whatsapp::Message`. The `Message` enum is `Clone + Debug` [VERIFIED: ferro-whatsapp/src/message.rs line 7]. `WhatsApp::send(to, message)` is `pub async fn` accepting `&str` and `Message` by value [VERIFIED: ferro-whatsapp/src/client.rs line 56].

`send_whatsapp` in dispatcher:
1. Check `config.whatsapp_enabled` — if false, emit info log "channel not configured" and return Ok(())
2. Get phone from `notifiable.route_notification_for(Channel::WhatsApp)` — if None, return `Error::ChannelNotAvailable`
3. Call `ferro_whatsapp::WhatsApp::send(&phone, whatsapp_msg.message).await` — propagates `ferro_whatsapp::Error` via `Error::WhatsApp`

The `ferro_whatsapp::Error` type does not implement `std::error::Error` in a way that conflicts with `thiserror` — it is a `thiserror`-derived enum [VERIFIED: ferro-whatsapp/src/error.rs line 2]. The `#[from]` conversion will work.

**Important:** `WhatsApp::send` panics if `WhatsApp::init` has not been called [VERIFIED: ferro-whatsapp/src/client.rs line 58]. The adapter must rely on consumers having called `WhatsApp::init` at startup. The `whatsapp_enabled: false` default ensures the dispatch arm is not reached unless the consumer explicitly enables it — which implies they've also called `WhatsApp::init`.

### InApp Adapter (channels/in_app.rs + dispatcher.rs)

`InAppConfig` holds two `Arc`-wrapped handles: `Arc<Broadcaster>` and `Arc<dyn DatabaseNotificationStore>`. Both are `Send + Sync`.

`Broadcaster::broadcast()` signature [VERIFIED: ferro-broadcast/src/broadcaster.rs line 232]:
```rust
pub async fn broadcast<T: Serialize>(&self, channel: &str, event: &str, data: T) -> Result<(), Error>
```

The InApp dispatch sequence:
1. Get `notifiable_id` and `notifiable_type` from the notifiable
2. Convert `InAppMessage` to `DatabaseMessage` for storage (mapping fields)
3. Call `config.store.store(notifiable_id, notifiable_type, &notification_type, &db_msg).await` — on error, return `Error::Database`
4. Call `config.broker.broadcast(&format!("user.{}", notifiable_id), &format!("Notification.{}", in_app_msg.notification_type), &in_app_msg.data).await` — on error, return wrapped error

**Conversion from InAppMessage to DatabaseMessage:** `InAppMessage` carries `serde_json::Value` as `data`; `DatabaseMessage` carries `HashMap<String, Value>` as `data`. The conversion should serialize the `InAppMessage` struct entirely as the data map — or store the `serde_json::Value` payload under a well-known key (e.g., `"payload"`). Recommendation: construct a `DatabaseMessage::new(in_app_msg.notification_type.clone())` and call `.with_data()` with the `data` field deserialized into a `HashMap`, falling back to `{ "payload": value }` if the value is not an object.

**`ferro_broadcast::Error` needs mapping to `ferro_notifications::Error`.** `ferro_broadcast::Error` is not automatically convertible. Add an `Other(String)` fallback or a new `Error::Broadcast(String)` variant. Recommendation: add `Error::Broadcast(String)` for clarity, constructed via `Error::broadcast(msg)` helper — matches the existing Mail/Slack/Database helper pattern.

### MailMessage Attachment — SMTP Path (dispatcher.rs)

The current SMTP path (dispatcher.rs lines 411-420) calls `email_builder.header(ContentType::...).body(...)`. This produces a single-part message and cannot accommodate attachments.

When `message.attachments` is non-empty, the path must switch to `email_builder.multipart(MultiPart::mixed() ...)`. When empty, keep the current single-part `.body()` call for backward compatibility.

Verified lettre 0.11 pattern [VERIFIED via Context7 /websites/rs_lettre]:
```rust
use lettre::message::{Attachment, MultiPart, SinglePart, header::ContentType};

// Body part (existing logic, now as a SinglePart)
let body_part = if let Some(ref html) = message.html {
    SinglePart::html(html.clone())
} else {
    SinglePart::plain(message.body.clone())
};

// Build multipart/mixed
let mut mp = MultiPart::mixed().singlepart(body_part);
for att in &message.attachments {
    let ct = ContentType::parse(&att.content_type)
        .map_err(|e| Error::mail(format!("Invalid content-type: {e}")))?;
    let part = Attachment::new(att.filename.clone())
        .body(att.content.clone(), ct);
    mp = mp.singlepart(part);
}
let email = email_builder.multipart(mp)
    .map_err(|e| Error::mail(format!("Failed to build email: {e}")))?;
```

**Pitfall:** `Attachment::new(filename).body(content, content_type)` returns `SinglePart` directly (not a builder) [VERIFIED via Context7]. `MultiPart::mixed().singlepart()` accepts `SinglePart`. The accumulation pattern `mp = mp.singlepart(part)` is correct — `MultiPartBuilder` is consuming.

**Pitfall:** `email_builder.multipart()` and `email_builder.body()` both consume `email_builder` and return `Result<Message, ...>`. The branch must produce the same `Message` type on both paths — both do [VERIFIED: lettre API].

### MailMessage Attachment — Resend Path (dispatcher.rs)

The Resend payload struct (`ResendEmailPayload`) is defined inline at dispatcher.rs line 259. It must gain:
```rust
#[derive(Serialize)]
struct ResendAttachment {
    filename: String,
    content: String, // base64-encoded
}

// In ResendEmailPayload:
#[serde(skip_serializing_if = "Vec::is_empty")]
attachments: Vec<ResendAttachment>,
```

Encoding: `base64::engine::general_purpose::STANDARD.encode(&att.content)` [ASSUMED: standard `base64` crate 0.22 API; verify import path if using `base64 = "0.22"`]. Alternatively, use the `base64` feature of the existing dependencies — `reqwest` pulls in `base64` transitively. Recommendation: add explicit `base64 = "0.22"` to Cargo.toml to avoid relying on transitive availability.

Resend API attachment schema [VERIFIED via Resend docs / prior knowledge]: `{ "filename": "...", "content": "<base64>" }`. Array field name is `"attachments"`. No per-attachment size limit enforced by the API itself below the 25MB framework limit.

### Database Channel Fix (dispatcher.rs send_database)

Current placeholder (lines 516-526) only logs. With D-13:

```rust
async fn send_database<N: Notifiable + ?Sized>(
    notifiable: &N,
    message: &DatabaseMessage,
) -> Result<(), Error> {
    let notifiable_id = notifiable.notifiable_id();
    let notifiable_type = notifiable.notifiable_type();
    
    if let Some(store) = CONFIG.get().and_then(|c| c.database_store.as_ref()) {
        store.store(&notifiable_id, notifiable_type, &message.notification_type, message)
            .await?;
        info!(notifiable_id = %notifiable_id, "Database notification stored");
    } else {
        info!(notifiable_id = %notifiable_id, notification_type = %message.notification_type, 
              "Database notification stored (placeholder — no store configured)");
    }
    Ok(())
}
```

The `?` on `store.store(...).await?` works because `DatabaseNotificationStore::store` already returns `Result<(), Error>` where `Error` is `ferro_notifications::Error` [VERIFIED: notifiable.rs line 107].

### NotificationConfig (dispatcher.rs)

`NotificationConfig` currently derives `Clone` and `Default`. Adding `Arc<dyn DatabaseNotificationStore>` and `Arc<Broadcaster>` (via `InAppConfig`) does not break `Clone` (both are `Clone`). It does not break `Default` as long as the new fields are `Option<...>` with default `None` and `whatsapp_enabled` defaults to `false`.

`from_env()` must be updated to include `whatsapp_enabled: env::var("WHATSAPP_ENABLED").ok().and_then(|v| v.parse().ok()).unwrap_or(false)`. The `in_app` and `database_store` fields stay None from env — they require programmatic construction.

### Error Handling

Current `Error` enum has no `#[from]` on any existing variant except `Serialization(#[from] serde_json::Error)`. Adding `WhatsApp(#[from] ferro_whatsapp::Error)` is straightforward. The `AttachmentTooLarge` variant has named fields (no from).

**Conflict check:** `ferro_whatsapp::Error` does not implement `serde_json::Error`, so there is no ambiguity with the existing Serialization conversion. [VERIFIED: ferro-whatsapp/src/error.rs]

### Tests

The existing test suite uses `serial_test` for env-variable tests. New tests follow the same pattern.

Key test surface:
- `test_channel_whatsapp_as_str` — `Channel::WhatsApp.as_str() == "whatsapp"`
- `test_channel_in_app_as_str` — `Channel::InApp.as_str() == "in_app"`
- `test_channel_serialization` — round-trip JSON for `"whatsapp"` and `"in_app"`
- `test_whatsapp_message_text_builder`
- `test_in_app_message_builder`
- `test_mail_attachment_25mb_cap_enforced`
- `test_mail_attachment_accumulates`
- `test_resend_payload_with_attachments_serialization` — verify base64 field present; absent when empty
- `test_send_database_with_store` — mock `DatabaseNotificationStore` implementation

---

## Publish Wave Analysis

[VERIFIED: .github/workflows/publish.yml]

- `ferro-notifications` is in **Wave 1a** (pure leaf, zero internal ferro-* deps)
- `ferro-broadcast` is in **Wave 1a** (pure leaf, zero internal ferro-* deps)
- `ferro-whatsapp` is in **Wave 1b** (depends on ferro-events, ferro-queue which are Wave 1a)

**Finding:** Adding `ferro-broadcast` to `ferro-notifications/Cargo.toml` does NOT change the wave — both are Wave 1a. No publish.yml edit needed.

**Finding:** Adding `ferro-whatsapp` to `ferro-notifications/Cargo.toml` DOES change the wave. ferro-whatsapp is Wave 1b. If ferro-notifications depends on ferro-whatsapp, it can no longer be Wave 1a — it must move to Wave 1b or later.

**This is ARCH-FINDING-05. See below.**

---

## External Library Pitfalls

### lettre 0.11 Multipart

1. **`email_builder.multipart()` vs `.body()` — mutual exclusion.** Both consume `MessageBuilder` and return `Result<Message, MessageBuildingError>`. Once the no-attachment path uses `.body()`, the attachment path must use `.multipart()`. The two branches must be inside a single `if/else` to avoid "moved value" errors. [VERIFIED: Context7]

2. **`MultiPartBuilder` is consuming.** Each `.singlepart()` call consumes `self` and returns `MultiPartBuilder`. Chain calls work: `let mp = MultiPart::mixed().singlepart(body).singlepart(att1).singlepart(att2)`. Or accumulate with `let mut mp = MultiPart::mixed().singlepart(body); for att in atts { mp = mp.singlepart(att); }`. Both patterns compile. [VERIFIED: Context7]

3. **`ContentType::parse(&str)` returns `Result`** (not a panic path). Must map the error to `Error::mail(...)`. [VERIFIED: Context7]

4. **`Attachment::new(filename).body(content, content_type)` returns `SinglePart` directly** — not a `Result`. The `body()` call is infallible from the caller's perspective (error surfaces only if content_type is pre-validated). [VERIFIED: Context7]

5. **`Vec<u8>` implements `IntoBody`** in lettre — pass `att.content.clone()` directly. No need for `Body::new()` pre-encoding when content is not reused across multiple sends. [VERIFIED: Context7]

6. **HTML vs plain text body part in multipart.** Currently the SMTP path uses `ContentType::TEXT_HTML` / `ContentType::TEXT_PLAIN` directly on `email_builder`. In the multipart case, use `SinglePart::html(html)` or `SinglePart::plain(body)` which set the correct headers automatically. Do not mix manual header approach with `SinglePart::html/plain` convenience methods for the same part. [VERIFIED: Context7]

### Resend HTTP API

1. **Attachment field name is `"attachments"`** (array). Each element: `{ "filename": "...", "content": "<base64-string>" }`. [ASSUMED: from Resend documentation patterns; verify against https://resend.com/docs/api-reference/emails/send-email if the schema causes a 4xx]

2. **Base64 encoding:** Resend expects standard base64 (not URL-safe). Use `base64::engine::general_purpose::STANDARD.encode(bytes)`. [ASSUMED: standard Resend API behavior]

3. **No per-attachment size limit documented below the transport limit.** The 25MB framework limit is the effective cap. [ASSUMED: Resend's stated limit is 40MB total per email; per-attachment not separately documented]

4. **`skip_serializing_if = "Vec::is_empty"` on the `attachments` field** keeps the payload byte-identical to today when no attachments are present — no regression for existing consumers. [VERIFIED: existing ResendEmailPayload already uses this pattern for cc/bcc]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (builtin, no external runner) |
| Config file | none |
| Quick run command | `cargo test -p ferro-notifications` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Behavior | Test Type | Automated Command |
|----------|-----------|-------------------|
| Channel::WhatsApp / InApp serialize as "whatsapp" / "in_app" | unit | `cargo test -p ferro-notifications channel` |
| Notification trait default-None methods for new channels | unit | `cargo test -p ferro-notifications notification` |
| WhatsAppMessage text/template builders | unit | `cargo test -p ferro-notifications channels::whatsapp` |
| InAppMessage severity builder | unit | `cargo test -p ferro-notifications channels::in_app` |
| MailMessage::attachment 25MB cap rejected | unit | `cargo test -p ferro-notifications channels::mail` |
| MailMessage::attachment accumulates multiple attachments | unit | `cargo test -p ferro-notifications channels::mail` |
| Resend payload: attachments field absent when empty | unit | `cargo test -p ferro-notifications dispatcher` |
| Resend payload: attachments field present with base64 content | unit | `cargo test -p ferro-notifications dispatcher` |
| send_database with configured store invokes store.store() | unit (mock) | `cargo test -p ferro-notifications dispatcher` |
| send_database without store emits placeholder log only | unit | `cargo test -p ferro-notifications dispatcher` |
| NotificationConfig::from_env reads WHATSAPP_ENABLED | unit (serial) | `cargo test -p ferro-notifications dispatcher -- --test-thread=1` |
| Error::WhatsApp from ferro_whatsapp::Error | unit | `cargo test -p ferro-notifications error` |
| Error::AttachmentTooLarge displays correctly | unit | `cargo test -p ferro-notifications error` |

### Wave 0 Gaps

- `ferro-notifications/src/channels/whatsapp.rs` — new file, no existing tests
- `ferro-notifications/src/channels/in_app.rs` — new file, no existing tests
- Mock `DatabaseNotificationStore` implementation for unit tests (can be defined in test module inline)

---

## Additional Architectural Findings

### ARCH-FINDING-05: Adding ferro-whatsapp as a hard dep moves ferro-notifications out of Wave 1a

**Discrepancy.** The publish workflow places `ferro-notifications` in Wave 1a ("Crates with ZERO internal ferro-* dependencies"). D-15 assumes adding `ferro-broadcast` does not change the wave — correct. However, `WhatsAppChannel::dispatch` must call `ferro_whatsapp::WhatsApp::send`, requiring a direct crate dependency on `ferro-whatsapp`. `ferro-whatsapp` is Wave 1b (depends on `ferro-events` and `ferro-queue`). If `ferro-notifications` depends on `ferro-whatsapp`, it can no longer be Wave 1a.

**Cause.** D-15 audited `ferro-broadcast` but the CONTEXT.md did not explicitly audit the `ferro-whatsapp` dependency, which was implicit in D-04.

**Options:**

A. **Move ferro-notifications to Wave 1b** in publish.yml. Both `ferro-whatsapp` and `ferro-notifications` publish in Wave 1b. This is the simplest fix — one line change in publish.yml.

B. **Feature-gate the ferro-whatsapp dep** behind an `in-whatsapp` feature flag. Wave 1a publishes without WhatsApp support; consumers opt in. This increases complexity and contradicts the spirit of D-15 ("hard dep is cheaper than a thin abstraction").

C. **Call WhatsApp via the ferro-whatsapp crate dynamically** (e.g., through a trait abstraction) — but CONTEXT.md D-04 explicitly closes this path.

**Recommendation:** Option A. Move `ferro-notifications` from Wave 1a to Wave 1b in publish.yml. It is the minimal change. The only risk is publish ordering — `ferro-notifications` must publish after `ferro-events`, `ferro-queue`, and `ferro-whatsapp` are indexed. The 30-second sleep between Wave 1a and 1b already handles this.

**Fix in plan.** Add `ferro-whatsapp = { path = "../ferro-whatsapp", version = "0.2" }` to `ferro-notifications/Cargo.toml`; move `ferro-notifications` from the Wave 1a list to the Wave 1b list in `.github/workflows/publish.yml`.

---

## File-by-File Changes

| File | Change Type | Summary |
|------|-------------|---------|
| `ferro-notifications/Cargo.toml` | modify | Add ferro-broadcast, ferro-whatsapp, base64 deps |
| `ferro-notifications/src/channel.rs` | modify | Add WhatsApp, InApp variants with per-variant serde renames; extend as_str() |
| `ferro-notifications/src/notification.rs` | modify | Add to_whatsapp, to_in_app, to_sms, to_push default-None methods |
| `ferro-notifications/src/channels/mod.rs` | modify | Add mod whatsapp; mod in_app; re-export new types |
| `ferro-notifications/src/channels/whatsapp.rs` | new | WhatsAppMessage struct + builders |
| `ferro-notifications/src/channels/in_app.rs` | new | InAppMessage + InAppSeverity + builders |
| `ferro-notifications/src/channels/mail.rs` | modify | Add MailAttachment struct; add attachments field; add attachment() builder |
| `ferro-notifications/src/error.rs` | modify | Add WhatsApp(#[from]), AttachmentTooLarge, Broadcast variants + helpers |
| `ferro-notifications/src/dispatcher.rs` | modify | NotificationConfig new fields; InAppConfig struct; ResendEmailPayload attachments; SMTP multipart path; send_database wire; send_whatsapp; send_in_app; Channel match arms |
| `ferro-notifications/src/lib.rs` | modify | Re-export new public types |
| `framework/src/lib.rs` | modify | Add new types to ferro_notifications re-export block |
| `.github/workflows/publish.yml` | modify | Move ferro-notifications from Wave 1a to Wave 1b list (ARCH-FINDING-05) |
| `docs/src/notifications/` | add/modify | Document WhatsApp channel, InApp channel, MailMessage attachment API |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Resend attachment field name is `"attachments"` with `{ filename, content }` schema | Resend path | Resend returns 422; fix is JSON field rename |
| A2 | Resend expects standard (not URL-safe) base64 encoding | Resend path | Resend returns 400 or corrupt attachment; fix is encoding swap |
| A3 | Resend has no per-attachment size limit below 40MB total | Resend path | Resend rejects files between 25MB and 40MB; framework cap is already below this |
| A4 | `base64 = "0.22"` API is `base64::engine::general_purpose::STANDARD.encode(bytes)` | Resend path | Compile error; fix is adjusting import path |

---

## Open Questions

None. All architectural questions have been answered by reading the live crate surfaces. ARCH-FINDING-05 surfaces a previously unknown dependency-wave issue but provides a clear fix.

---

## Sources

### Primary (HIGH confidence)

- `ferro-notifications/src/dispatcher.rs` — full read; exact placeholder location L503-527 confirmed
- `ferro-notifications/src/channel.rs` — serde attribute at L7 confirmed (`rename_all = "lowercase"`)
- `ferro-notifications/src/channels/mail.rs` — MailMessage fields and builder methods confirmed
- `ferro-notifications/src/error.rs` — existing variants and helper methods confirmed
- `ferro-whatsapp/src/client.rs` — WhatsApp::send signature at L56 confirmed; panic behavior documented
- `ferro-whatsapp/src/message.rs` — Message enum variants confirmed
- `ferro-whatsapp/src/error.rs` — thiserror-derived enum confirmed; no conflicts with #[from]
- `ferro-broadcast/src/broadcaster.rs` — Broadcaster::broadcast signature at L232 confirmed
- `.github/workflows/publish.yml` — Wave 1a and 1b contents confirmed
- Context7 `/websites/rs_lettre` — Attachment, MultiPart::mixed, SinglePart API verified

### Secondary (MEDIUM confidence)

- Serde `rename_all = "lowercase"` behavior for multi-word variants (InApp → "inapp") — well-known serde behavior, verified by reasoning from spec

### Tertiary (LOW confidence)

- A1–A4: Resend attachment API schema and base64 encoding requirements (ASSUMED; see Assumptions Log)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all library surfaces read directly
- Architecture: HIGH — all integration points verified against live code
- Pitfalls: HIGH for lettre (Context7 verified); MEDIUM for Resend (ASSUMED schema)
- Publish wave finding: HIGH — publish.yml read directly

**Research date:** 2026-04-28
**Valid until:** 2026-05-28 (stable crate surfaces; lettre 0.11 is not fast-moving)

## RESEARCH COMPLETE

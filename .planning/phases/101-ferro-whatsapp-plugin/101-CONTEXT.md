# Phase 101: ferro-whatsapp Plugin - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Create `ferro-whatsapp` plugin crate providing WhatsApp Business Cloud API integration: outbound message sender (text + templates), inbound webhook dispatcher with HMAC verification, wamid-level message deduplication, and sender-identity routing (owner vs customer classification). Includes CLI scaffolding, MCP introspection tools, documentation, and publish workflow integration. Driven by gestiscilo.it v2.4 but extracted as a reusable framework plugin.

</domain>

<decisions>
## Implementation Decisions

### Outbound Message API
- Message types: text + template messages in v1. Media support deferred (can be added as new enum variants without breaking changes)
- Single `WhatsApp::send(to, message)` entry point with `Message` enum (`Message::Text { body }`, `Message::Template { name, language, parameters }`)
- Returns `Result<SendResult { wamid: String }, Error>` — wamid enables correlation with delivery status webhooks
- No automatic retry — returns typed error variants (RateLimit, InvalidNumber, NetworkError, AuthError). Callers handle retry logic in their own handlers
- OnceLock facade pattern: `WhatsApp::init(config)` + `WhatsApp::send()` static methods (matching ferro-stripe)

### Sender-Identity Routing
- Callback/closure-based identity resolution: `WhatsAppConfig` takes `is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>` — maximum flexibility for DB queries, tenant settings, etc.
- Default to customer when identity resolver can't determine: safe default prevents accidental owner privilege escalation
- Identity resolved at the webhook dispatcher level — event payloads arrive with identity pre-resolved, listeners don't re-implement phone matching
- `SenderIdentity` enum carries the phone number: `SenderIdentity::Owner(String)` / `SenderIdentity::Customer(String)`

### Deduplication Strategy
- Pluggable `DeduplicationStore` trait with in-memory DashMap default + optional Redis for persistence across restarts (mirrors ConfirmationStore pattern from Phase 100)
- Messages only — status updates (delivered, read) are idempotent by nature
- 5-minute TTL window — covers all reasonable Meta retry windows without accumulating stale entries
- Claude's discretion on duplicate behavior (silent drop + debug log vs emit dedup event)

### Webhook Event Taxonomy
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

</decisions>

<specifics>
## Specific Ideas

- Extracted from gestiscilo.it's existing WhatsApp webhook code into a reusable plugin
- Primary use case: receive customer/owner messages, classify sender, dispatch to appropriate handlers, send replies/confirmations
- Template messages needed for proactive messaging (booking confirmations, reminders) — Meta requires templates outside the 24h conversation window
- Integrates with ferro-ai (Phase 100) for message classification (e.g., owner command interpretation)
- Confirmation system (Phase 100) can be used for destructive WhatsApp commands (e.g., "confirm delete within 30s")

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-stripe/src/client.rs`: OnceLock facade pattern — direct template for WhatsApp::init/send
- `ferro-stripe/src/webhook/mod.rs`: HMAC-SHA256 verification — reference for Meta webhook signature validation
- `ferro-stripe/src/webhook/events.rs`: ProcessStripeWebhook job struct — template for ProcessWhatsAppWebhook
- `ferro-cli/src/commands/make_stripe.rs`: CLI scaffold with write_if_not_exists — template for make_whatsapp
- `ferro-mcp/src/tools/stripe.rs`: MCP introspection tools — template for WhatsApp config status
- `ferro-notifications/src/dispatcher.rs`: reqwest HTTP client usage (lines 482, 541) — reference for Meta Cloud API calls
- `ferro-ai`: Phase 100 classification and confirmation primitives — used by gestiscilo.it for WhatsApp command interpretation

### Established Patterns
- Feature-gated crates: `#[cfg(feature = "whatsapp")]` with re-exports in `framework/src/lib.rs`
- New crate conventions: thiserror Error enum, workspace Cargo.toml inheritance, kebab-case crate name
- Webhook flow: verify signature inline → ack 200 → queue job → job dispatches ferro-events → listeners handle
- OnceLock + Config + from_env() pattern for external service configuration
- `async_trait` for async trait methods, `DashMap` for concurrent shared state

### Integration Points
- `framework/src/lib.rs` — Feature-gated re-exports behind `#[cfg(feature = "whatsapp")]`
- `framework/Cargo.toml` — Optional dependency with `"whatsapp"` feature flag
- `ferro-cli/src/commands/` — New `make_whatsapp.rs` command
- `ferro-mcp/src/tools/` — New WhatsApp introspection tools
- `ferro-events` — WhatsAppTextReceived, WhatsAppStatusUpdate event types
- `ferro-queue` — ProcessWhatsAppWebhook job
- `.github/workflows/publish.yml` — Add ferro-whatsapp to Wave 1
- `docs/src/features/` — whatsapp.md documentation

</code_context>

<deferred>
## Deferred Ideas

- Media message support (images, documents, audio) — add as new Message enum variants in a future phase
- Interactive messages (buttons, list messages) — separate phase
- WhatsApp Business API status/errors webhook event type — add if monitoring needs arise
- Multi-phone-number support (multiple WhatsApp Business accounts) — future phase

</deferred>

---

*Phase: 101-ferro-whatsapp-plugin*
*Context gathered: 2026-03-23*

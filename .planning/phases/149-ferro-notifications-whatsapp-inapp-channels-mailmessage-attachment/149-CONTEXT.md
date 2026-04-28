# Phase 149: ferro-notifications WhatsApp + InApp + MailMessage Attachment - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning
**Mode:** `--auto` (decisions auto-selected from recommended options; logged inline)

<domain>
## Phase Boundary

Extend `ferro-notifications` with two new channel adapters and a Mail attachment builder. Additive, non-breaking to existing `Notification` impls.

**In scope:**
- `Channel::WhatsApp` and `Channel::InApp` enum variants + adapters
- `Notification::to_whatsapp()` / `to_in_app()` default-None trait methods
- `WhatsAppChannel` adapter routed through the existing `ferro-whatsapp` static facade
- `InAppChannel` adapter that writes both legs (SSE broker + DB store) on dispatch
- `MailMessage::attachment(filename, content_type, bytes)` builder + 25MB per-attachment guard + multipart wiring for **both** SMTP (lettre) and Resend drivers
- Architectural fixes surfaced during scout (see Architectural Findings) that are load-bearing for the new channels

**Out of scope (explicit):**
- APNs / FCM Push adapter — `Channel::Push` remains enum-only stub
- Inbound notification handling — outbound only
- New WhatsApp template-message authoring helpers — adapter forwards what `Notification::to_whatsapp` returns
- Stripe-style webhook wiring for delivery receipts — not in v11.9

</domain>

<architectural_findings>
## Architectural Findings (from scout audit)

These findings were surfaced by reading the existing crate surfaces during the discuss-phase scout. Each is load-bearing for the new channels — left unfixed they would force the planner into workarounds. Per the project rule that discrepancies must be audited, reported, and fixed (not silently routed around), each finding has an explicit fix-in-plan directive.

### ARCH-FINDING-01: ROADMAP success criterion #3 misnames the WhatsApp integration shape

**Discrepancy.** ROADMAP Phase 149 success criterion #3 reads:
> `WhatsAppChannel` adapter accepts a `ferro_whatsapp::Client` injected via `NotificationConfig::whatsapp` and dispatches via that client's existing send API

`ferro-whatsapp/src/lib.rs:34` exports `WhatsApp` (a static `OnceLock` facade matching the ferro-stripe pattern); `ferro-whatsapp/src/client.rs:25` is `pub struct WhatsApp;` with associated functions. There is no public `Client` type to inject.

**Cause.** The criterion was drafted from the gestiscilo-it consumer's mental model rather than from `ferro-whatsapp`'s actual surface, which uses an init-once global facade.

**Fix in plan.** `WhatsAppChannel` calls `ferro_whatsapp::WhatsApp::send(phone, message)` directly. `NotificationConfig::whatsapp_enabled: bool` (default `false`) is the opt-in flag — there is no client object to inject because `ferro-whatsapp` already owns its global state. Update ROADMAP success criterion #3 wording in the same commit chain that ships the adapter. See D-04.

### ARCH-FINDING-02: Database channel currently logs a placeholder instead of calling `DatabaseNotificationStore`

**Discrepancy.** `DatabaseNotificationStore` is exported from `ferro-notifications/src/lib.rs:68` and defined at `notifiable.rs:99`, but `dispatcher.rs:516-526` (`send_database`) only calls `tracing::info!("Database notification stored (placeholder)")`. The trait has never been wired into the dispatch path. `ChannelResult` (`notifiable.rs:67-95`) is also defined but unused.

**Cause.** Pre-existing TODO carried since the channel was added — never closed because no consumer required real persistence.

**Fix in plan.** Wire `DatabaseNotificationStore` through `NotificationConfig::database_store: Option<Arc<dyn DatabaseNotificationStore>>`. When configured, `send_database` calls `store.store(...)`. When unconfigured, retain the current placeholder log (backward-compatible). This is in-scope for Phase 149 because `InAppChannel` requires the same wiring (it writes both an SSE leg and a DB-store leg) — fixing the database channel and adding the InApp channel share one persistence path. Doing them separately would duplicate the integration. See D-08.

### ARCH-FINDING-03: `Channel::Sms` is treated inconsistently with `Channel::Push`

**Discrepancy.** `channel.rs:15-18` lists both `Sms` and `Push` as "future" stubs. The ROADMAP only addresses `Push` ("remains enum-only stub"); `Sms` is not mentioned. Asymmetric treatment leaves `Sms` in an undefined intermediate state.

**Cause.** Authoring oversight — both variants pre-date this phase and have always been future stubs.

**Fix in plan.** Treat `Sms` exactly like `Push`. Add `Notification::to_sms()` and `Notification::to_push()` as default-`None` trait methods alongside the new `to_whatsapp()` / `to_in_app()` methods. Dispatcher emits the same structured "channel not configured" no-op for both. This keeps the trait API symmetric and prevents the next consumer from re-discovering this gap. See D-06.

</architectural_findings>

<decisions>
## Implementation Decisions

### Channel Enum & Trait Surface

- **D-01:** Add `Channel::WhatsApp` and `Channel::InApp` to the `Channel` enum in `channel.rs`. Both variants serialize as snake_case lowercase via the existing `#[serde(rename_all = "lowercase")]` attribute. `Channel::WhatsApp.as_str()` returns `"whatsapp"`; `Channel::InApp.as_str()` returns `"in_app"`. `Channel::Mail`, `Channel::Database`, `Channel::Slack`, `Channel::Sms`, `Channel::Push` variants and their string forms are unchanged. *(auto: only viable shape given existing enum derive set)*

- **D-02:** Extend `Notification` trait (`notification.rs:39`) with four default-`None` methods so existing impls compile unchanged:
  ```rust
  fn to_whatsapp(&self) -> Option<WhatsAppMessage> { None }
  fn to_in_app(&self) -> Option<InAppMessage> { None }
  fn to_sms(&self) -> Option<SmsMessage> { None }   // Forward-compat — see ARCH-FINDING-03
  fn to_push(&self) -> Option<PushMessage> { None } // Forward-compat — see ARCH-FINDING-03
  ```
  The forward-compat `to_sms` / `to_push` methods do not get an adapter implementation in this phase — the dispatcher logs "channel not configured" for them (existing behavior at `dispatcher.rs:322`). *(auto: symmetric trait surface; chosen to close ARCH-FINDING-03 in the same change)*

### WhatsApp Channel

- **D-03:** Define `WhatsAppMessage` in `channels/whatsapp.rs` as a thin wrapper over `ferro_whatsapp::Message` (the existing `Text { body }` / `Template { ... }` enum). Initial shape:
  ```rust
  pub struct WhatsAppMessage {
      pub message: ferro_whatsapp::Message,
  }
  ```
  Builder methods: `WhatsAppMessage::text(body)`, `WhatsAppMessage::template(name, lang, components)`. *(auto: re-export the existing typed shape rather than re-invent — matches D-04)*

- **D-04:** `WhatsAppChannel` adapter calls `ferro_whatsapp::WhatsApp::send(phone, message)` directly via the static facade. **Closes ARCH-FINDING-01.** The recipient phone is obtained from `Notifiable::route_notification_for(Channel::WhatsApp)`. The dispatcher gates the call on `NotificationConfig::whatsapp_enabled` (default `false`); when disabled it emits the same structured "channel not configured" no-op as Sms / Push. The adapter does not own a client and does not store credentials — both live in `ferro_whatsapp::WhatsApp`'s global state, initialized once at app startup (matches the ferro-stripe pattern).

- **D-05:** `Error::WhatsApp(ferro_whatsapp::Error)` variant added to `ferro-notifications`'s `Error` enum via `#[from]`. Errors propagate up the dispatch chain — no swallow, no retry. *(auto: matches existing `Error::Slack` / `Error::Mail` pattern)*

### InApp Channel

- **D-06:** `InAppMessage` struct in `channels/in_app.rs` carries a notification type, a `serde_json::Value` payload, and an optional severity hint:
  ```rust
  pub struct InAppMessage {
      pub notification_type: String,
      pub data: serde_json::Value,
      pub severity: Option<InAppSeverity>, // Info | Success | Warning | Error
  }
  ```
  Builder methods: `InAppMessage::new(notification_type)`, `.data(value)`, `.severity(level)`. *(auto: matches `DatabaseMessage` shape)*

- **D-07:** `NotificationConfig::in_app: Option<InAppConfig>` where:
  ```rust
  pub struct InAppConfig {
      pub broker: Arc<ferro_broadcast::Broadcaster>,
      pub store: Arc<dyn DatabaseNotificationStore>,
  }
  ```
  `ferro-notifications` adds a **hard dependency** on `ferro-broadcast`. The `Broadcaster` type already exists at `ferro-broadcast/src/broadcaster.rs:37`. *(auto: hard dep is cheaper than a thin abstraction trait — both crates are workspace-internal and ship together)*

- **D-08:** `InAppChannel::dispatch` writes **both legs** atomically-from-the-caller's-POV: it persists via `DatabaseNotificationStore::store()` first, then publishes via `Broadcaster.broadcast()` to channel `format!("user.{}", notifiable_id)` with event name `format!("Notification.{}", notification_type)`. **Closes ARCH-FINDING-02** by routing both `Channel::Database` and `Channel::InApp` through the same store. If either leg fails the dispatch returns an error; no partial-success silent fallback. *(auto: persistence-first is the correct order — broker can replay on reconnect from the store; the inverse order would risk silent loss if the broker call succeeded but persistence failed)*

### MailMessage Attachment

- **D-09:** Add `MailMessage::attachments: Vec<MailAttachment>` field where:
  ```rust
  pub struct MailAttachment {
      pub filename: String,
      pub content_type: String, // e.g. "application/pdf"
      pub content: Vec<u8>,
  }
  ```
  Inline `Vec<u8>` — matches the existing all-in-memory `MailMessage` shape. No streaming, no path-based attachments. *(auto: simplest representation that works; streaming is a v2 concern if/when 25MB feels small)*

- **D-10:** Builder method signature:
  ```rust
  pub fn attachment(mut self, filename: impl Into<String>, content_type: impl Into<String>, content: Vec<u8>) -> Self
  ```
  Multiple calls accumulate. Returns `self` (consuming builder, matches `with_*`-equivalent pattern in `MailMessage::cc` / `bcc`). *(auto: matches existing builder convention)*

- **D-11:** Per-attachment 25MB cap enforced in the builder via a typed error variant: `Error::AttachmentTooLarge { filename: String, size: usize, limit: usize }` (limit = 25 \* 1024 \* 1024 = 26_214_400 bytes). Builder returns `Result<Self, Error>` — call sites unwrap or propagate. **Cumulative cap is NOT enforced** — Resend's per-email cap is 40MB total but is the carrier's responsibility to surface; we do not duplicate provider-specific caps in the framework layer. *(auto: per-attachment cap matches the success-criteria language exactly; cumulative-cap enforcement would couple the framework to a specific provider)*

- **D-12:** **Both** SMTP (lettre) and Resend HTTP API drivers ship with attachment support in this phase — full parity. The success criteria did not require parity, but a lopsided implementation would create a runtime trap (consumer attaches a PDF, receives `Error::AttachmentNotSupported` only when their MAIL_DRIVER happens to be Resend). **Fix:**
  - SMTP (lettre): `MultiPart::mixed()` with the existing body part + one `SinglePart` per attachment using `Attachment::new(filename).body(content, ContentType::parse(...))`.
  - Resend: extend `ResendEmailPayload` with `attachments: Vec<ResendAttachment>` where `ResendAttachment { filename: String, content: String }` (content = base64-encoded). Use `serde(skip_serializing_if = "Vec::is_empty")` to keep the no-attachment payload byte-identical to today.
  *(auto: full parity is the right call given the runtime-trap risk; doubling wiring is a one-time cost)*

### Database Channel Fix (in-scope per ARCH-FINDING-02)

- **D-13:** Add `NotificationConfig::database_store: Option<Arc<dyn DatabaseNotificationStore>>`. Update `dispatcher.rs::send_database` to call `store.store(notifiable_id, notifiable_type, &message.notification_type, message)` when configured. When unconfigured, retain the current placeholder log (backward-compatible — no consumer of the current placeholder is broken). Logging on the success path is unchanged in shape; only the side effect is added. *(auto: closes ARCH-FINDING-02 with the minimum surface change)*

### Configuration & Env Wiring

- **D-14:** `NotificationConfig` gains:
  ```rust
  pub whatsapp_enabled: bool,                                          // D-04
  pub in_app: Option<InAppConfig>,                                     // D-07
  pub database_store: Option<Arc<dyn DatabaseNotificationStore>>,      // D-13
  ```
  Existing `mail` and `slack_webhook` fields unchanged. Builder methods `with_whatsapp_enabled(bool)`, `with_in_app(InAppConfig)`, `with_database_store(Arc<dyn ...>)` added. `from_env()` reads `WHATSAPP_ENABLED` (parsed as bool, default `false`). `in_app` and `database_store` are **not** env-driven — they require typed handles consumers must construct in code. *(auto: matches existing `mail()` / `slack_webhook()` builder pattern)*

### Workspace & Publish

- **D-15:** Add `ferro-broadcast` to `ferro-notifications/Cargo.toml` `[dependencies]`. Update `.github/workflows/publish.yml` only if this changes ferro-notifications' wave (it does not — `ferro-broadcast` already ships in the same wave). *(auto: per the conventions memory, every new internal dep needs the publish-wave audit)*

- **D-16:** This phase publishes a single new `ferro-notifications` version on merge to master via the existing GH Actions wave-based publish flow (ARCH-FINDING-01's ROADMAP wording fix is doc-only and ships in the same commit). `ferro-broadcast`, `ferro-whatsapp`, and `ferro-mcp` versions are unchanged. *(auto: one crate touched, one version bump)*

### Claude's Discretion

The following are left for the planner / executor to resolve from existing code patterns — no user input needed:

- Exact lettre `MultiPart` builder ergonomics (whether to introduce a private helper or inline the multipart construction in `send_mail_smtp`).
- Whether `to_sms` / `to_push` get placeholder `SmsMessage` / `PushMessage` types or stay as future-only (recommendation: define empty placeholder types so trait signatures are stable across phases — but defer the full message shape to whichever phase ships the adapter).
- Test-fixture choice for the SMTP attachment integration test (Mailpit vs an in-memory `lettre::transport::stub` — ROADMAP mentions Mailpit; planner can choose either).
- Whether `WhatsAppMessage` lives at `ferro-notifications/src/channels/whatsapp.rs` or directly re-exports `ferro_whatsapp::Message` with no wrapper (recommendation: thin wrapper for parity with the other channel message types).
- Sub-module layout under `channels/` — single file vs sub-folder per channel.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition

- `.planning/ROADMAP.md` §"Phase 149: ferro-notifications WhatsApp + InApp channels + MailMessage attachment" (line 1344) — goal, success criteria, dependencies. Note: success criterion #3 must be reworded per ARCH-FINDING-01.

### Existing crate surfaces (read before touching)

- `ferro-notifications/src/lib.rs` — public re-exports (line 60-71)
- `ferro-notifications/src/channel.rs` — `Channel` enum (line 8); add WhatsApp, InApp variants here
- `ferro-notifications/src/notification.rs` — `Notification` trait (line 39); add `to_whatsapp`, `to_in_app`, `to_sms`, `to_push` here
- `ferro-notifications/src/dispatcher.rs` — `NotificationDispatcher` (line 277), `NotificationConfig` (line 18), `send_database` placeholder fix point (line 503-527)
- `ferro-notifications/src/notifiable.rs` — `DatabaseNotificationStore` trait (line 99), already exported
- `ferro-notifications/src/channels/mail.rs` — `MailMessage` struct (line 7); attachment field added here
- `ferro-whatsapp/src/lib.rs` — `WhatsApp` static facade (line 34)
- `ferro-whatsapp/src/client.rs` — `pub struct WhatsApp;` and its `impl` (line 25-27); the `send` method to call from the adapter
- `ferro-whatsapp/src/message.rs` — `Message` enum that `WhatsAppMessage` wraps
- `ferro-broadcast/src/lib.rs` — public exports including `Broadcaster` (line 60-65)
- `ferro-broadcast/src/broadcaster.rs` — `Broadcaster` type (line 37); the `broadcast` API the InApp adapter calls

### External (researcher to verify; not present in this repo)

- `gestiscilo-it/app/.planning/REQUIREMENTS.md` — FERRO-01, FERRO-02, FERRO-03 (consumer-side requirements; in a separate repo)
- `gestiscilo-it/app/.planning/research/v6.4-DOCUMENTS-NOTIFICATIONS-STACK.md` — full integration design (the ROADMAP cites this path as `.planning/research/v6.4-DOCUMENTS-NOTIFICATIONS-STACK.md` but it is **not present in this repo**; researcher should locate it in the gestiscilo-it consumer repo before planning, or escalate if missing)

### Conventions

- `.planning/codebase/CONVENTIONS.md` — naming, builder pattern (`with_*` consuming `mut self`), error handling (`thiserror`, one `Error` per crate)
- `~/.claude/projects/-Users-alberto-repositories-albertogferrario-ferro/memory/MEMORY.md` (private) — Key Conventions section: when adding internal deps, audit `.github/workflows/publish.yml` waves

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `ferro_broadcast::Broadcaster` (`ferro-broadcast/src/broadcaster.rs:37`) — the SSE / WebSocket broker the InApp adapter publishes through. Already provides `broadcast(channel, event, data)` semantics; no new abstraction needed.
- `ferro_notifications::DatabaseNotificationStore` (`notifiable.rs:99`) — already-exported trait. Both `Channel::Database` (when fixed per D-13) and `Channel::InApp` (per D-08) share this storage path. No new trait.
- `ferro_whatsapp::WhatsApp::send` (`ferro-whatsapp/src/client.rs:27`) — static-facade send API. The WhatsApp adapter is a one-line wrapper around this.
- `ferro_whatsapp::Message` enum (`ferro-whatsapp/src/message.rs`) — the typed message shape. `WhatsAppMessage` wraps this.
- `lettre::message::{MultiPart, SinglePart, Attachment}` — already-pulled-in dependency (used by SMTP path). Multipart attachment construction is a built-in.
- `serde_json::Value` — payload type for `InAppMessage::data`, matches `DatabaseMessage` (`channels/database.rs`).

### Established Patterns

- **Static-facade integration crates.** `ferro-whatsapp` and `ferro-stripe` both use `OnceLock`-backed init-once singletons (e.g. `WhatsApp::init`, `WhatsApp::send`). The framework gates feature availability via flags rather than by injecting client objects (see ARCH-FINDING-01).
- **Default-`None` trait methods for channel converters.** `Notification::to_mail`, `to_database`, `to_slack` all return `None` by default. The new `to_whatsapp`, `to_in_app`, `to_sms`, `to_push` follow the same pattern.
- **Adapter-pattern dispatcher.** `NotificationDispatcher::send` matches on `Channel`, calls the corresponding `to_*` method, dispatches if `Some`. No new dispatch primitive — extend the match.
- **Single-driver / multi-driver mail.** The Mail channel already supports SMTP (lettre) and Resend (HTTP). Attachments must work on both — see D-12.
- **Typed `Error` enum per crate via `thiserror`.** Add `Error::WhatsApp(#[from] ferro_whatsapp::Error)` and `Error::AttachmentTooLarge { filename, size, limit }` to `ferro-notifications/src/error.rs`.
- **Consuming builder methods (`mut self -> Self`).** All `MailMessage` builder methods consume `self`; new `attachment` method follows suit (but returns `Result<Self, Error>` per D-11).
- **`#[serde(rename_all = "lowercase")]` on the Channel enum.** New variants automatically serialize as `"whatsapp"` and `"in_app"` (note: `InApp` snake-cases to `inapp` under the existing `lowercase` rule — verify whether the wire form should be `inapp` or `in_app`; if `in_app`, switch the attribute to `snake_case` and check existing-variant compatibility — `Mail` / `Database` / `Slack` / `Sms` / `Push` round-trip identically under both rules).

### Integration Points

- `ferro-notifications/Cargo.toml` — add `ferro-broadcast = { path = "../ferro-broadcast", version = "<workspace>" }` to `[dependencies]`. Optionally feature-gate behind `in-app` if we want to avoid a hard dep; default-on per D-07.
- `framework/src/lib.rs` — re-export the new types (`Channel::WhatsApp` / `InApp`, `WhatsAppMessage`, `InAppMessage`, `MailAttachment`) if user-facing — verify what is currently re-exported from `ferro-notifications` at `framework/src/lib.rs` and add symmetrically.
- `ferro-mcp/src/tools/` — `application_info` should not need updates (channels are not introspected today). Verify no MCP tool currently lists Channel variants; if any do, update them.
- `docs/src/notifications/` (if present) — consumer-facing docs for the two new channels and the attachment API.
- `.github/workflows/publish.yml` — verify ferro-notifications' wave does not change (it should not — ferro-broadcast already ships in or before its wave; planner to confirm).

</code_context>

<specifics>
## Specific Ideas

- The attachment 25MB cap in success criterion #5 is per-attachment (matches D-11). It is enforced by the framework — the consumer never has to worry about lettre's or Resend's own size limits below 25MB.
- The ROADMAP's "consumer-side smoke test in gestiscilo-it" (criterion #7) is **not** part of this phase's deliverables — it is a downstream verification that gestiscilo-it does in its Phase 120. The auto-publish on merge unblocks that.
- v11.9 publishes a single new `ferro-notifications` version. No coordinated multi-crate version bump.

</specifics>

<deferred>
## Deferred Ideas

- **Push channel adapter (APNs / FCM).** Out of scope — `Channel::Push` stays enum-only stub. Future v12.x phase candidate.
- **SMS adapter (Twilio / Vonage / equivalent).** Same status as Push. Future v12.x phase candidate (the symmetric `to_sms()` method shipped in this phase keeps the trait stable when this lands).
- **Streaming / path-based mail attachments.** Inline `Vec<u8>` for now per D-09. If 25MB feels small, revisit with a streaming `Read` source in a future phase — would also enable lazy-loaded attachments.
- **Cumulative attachment-size enforcement.** Per-provider concern. Defer until a provider-cap escalation actually bites.
- **Inbound WhatsApp / InApp event handling.** Outbound only in v11.9. ferro-whatsapp already has `webhook` and `dedup` modules for inbound; integrating those with `Notification` is a future concern.
- **Delivery-receipt webhook integration (Stripe-style).** Future phase. The current dispatch path returns `Result<(), Error>` only — no async delivery confirmation.
- **MCP tool exposure of channel variants.** No MCP tool currently introspects Channel variants. If a future agent wants to enumerate available channels for a project, that is its own MCP-tooling phase.

</deferred>

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Context gathered: 2026-04-28 (auto mode)*

# Phase 149: ferro-notifications WhatsApp + InApp + MailMessage Attachment - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered and the auto-mode rationale.

**Date:** 2026-04-28
**Phase:** 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
**Mode:** `--auto` (recommended option auto-selected for each gray area; logged here for review)
**Areas discussed:** InApp adapter wiring, Mail attachment shape & cap, Resend driver attachment parity, WhatsApp client lifecycle, Trait-method symmetry (`to_sms` / `to_push`), Database channel placeholder fix

---

## Pre-discussion: Scout-driven Architectural Findings

Before any decisions were locked, the scout audit surfaced three discrepancies that shape the rest of the discussion. These are not gray areas — they are bugs in either the existing code or the ROADMAP wording.

| # | Finding | Source | Resolution |
|---|---------|--------|------------|
| ARCH-FINDING-01 | ROADMAP success criterion #3 references a `ferro_whatsapp::Client` injection that does not exist in `ferro-whatsapp` | `ferro-whatsapp/src/lib.rs:34`, `client.rs:25` | Reword criterion + adapt approach: call `WhatsApp::send` static facade directly (D-04) |
| ARCH-FINDING-02 | `Channel::Database` dispatch is a placeholder log; `DatabaseNotificationStore` is exported but never wired | `dispatcher.rs:503-527`, `notifiable.rs:99` | Wire `DatabaseNotificationStore` through `NotificationConfig::database_store` in this phase (D-08, D-13) |
| ARCH-FINDING-03 | `Channel::Sms` is a "future" variant treated inconsistently with `Channel::Push` in the ROADMAP | `channel.rs:15-18` | Symmetric forward-compat trait methods `to_sms` / `to_push` both ship as default-`None` (D-02, D-06) |

---

## InApp adapter wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Single `InAppConfig { broker: Arc<Broadcaster>, store: Arc<dyn DatabaseNotificationStore> }` injected via `NotificationConfig::in_app`, hard dep on `ferro-broadcast` | Minimal abstraction; broker and store are typed and bundled. `ferro-notifications` takes a hard dep on `ferro-broadcast`. | ✓ |
| Two separate `Arc<dyn Trait>` handles on `NotificationConfig` (`InAppBroker` + `InAppStore` traits) | Crate-decoupled — `ferro-notifications` defines a thin `InAppBroker` trait that `ferro-broadcast::Broadcaster` implements externally. | |
| Lazy lookup from a global registry | Defer concrete handles until first use. Simplest call site, harder to reason about lifecycle. | |

**Auto-selected:** Option 1 (single typed config struct, hard dep on `ferro-broadcast`).
**Rationale:** Both crates ship together in the workspace, both publish in the same wave. A thin trait would be ceremony for no benefit — there is exactly one `Broadcaster` implementation. Captured as **D-07**.

---

## Mail attachment shape & 25MB cap semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Inline `Vec<u8>` field on `MailMessage`; per-attachment cap; typed `Error::AttachmentTooLarge` | Matches existing all-in-memory `MailMessage` shape. Builder returns `Result<Self, Error>`. | ✓ |
| Streaming `Box<dyn Read>` source; per-attachment cap | Lower memory ceiling. Forces async-read complexity into the SMTP and Resend send paths. | |
| Path-based `PathBuf` source; lazy file read at send time | Lowest memory ceiling. Forces I/O at send time, complicates error semantics, ties API to local filesystem. | |
| Cap enforced cumulatively (per-message) | Total-message cap matches some providers' limits but not others. Would force consumer to track running total. | |

**Auto-selected:** Option 1, per-attachment cap (matches success criteria language).
**Rationale:** `MailMessage` is already entirely in-memory; introducing a streaming source is an architecture shift unrelated to this phase. The 25MB cap is a guard against accidental large attachments, not a memory-pressure mitigation. Per-attachment matches the success-criteria wording exactly. Cumulative caps are provider-specific and belong in the provider, not the framework. Captured as **D-09, D-10, D-11**.

---

## Resend driver attachment parity

| Option | Description | Selected |
|--------|-------------|----------|
| Both SMTP (lettre multipart) AND Resend (base64 in JSON payload) support attachments end-to-end | Full parity. Doubles the wiring once but eliminates the runtime-trap risk. | ✓ |
| SMTP-only attachment support; Resend driver returns `Error::AttachmentNotSupported` at send time | Smaller initial surface. Risk: consumer attaches a PDF, runs into the error only when MAIL_DRIVER happens to be Resend. | |
| Disable attachments unless driver supports them (compile-time check) | Forces driver choice into the type system. Overkill for a runtime-config dimension. | |

**Auto-selected:** Option 1 (full parity).
**Rationale:** Resend's HTTP API supports `attachments: [{filename, content (base64)}]` natively. Partial parity creates exactly the kind of "works on my machine" trap that Alberto's maximum-quality discipline rules out. The doubling cost is one-time. Captured as **D-12**.

---

## WhatsApp client lifecycle & multi-tenant routing

| Option | Description | Selected |
|--------|-------------|----------|
| Direct call to `ferro_whatsapp::WhatsApp::send` static facade; `NotificationConfig::whatsapp_enabled: bool` opt-in flag; multi-tenant routing handled inside `ferro-whatsapp` via its phone-validator config | Matches the existing `ferro-whatsapp` surface (and ferro-stripe pattern). Closes ARCH-FINDING-01. | ✓ |
| Inject `Arc<ferro_whatsapp::Client>` via `NotificationConfig::whatsapp` | Matches the original ROADMAP success criterion wording — but `ferro-whatsapp` does not export a `Client` type. Would require reshaping `ferro-whatsapp` first. | |
| Per-`Notifiable` lookup (multi-tenant routes from different WhatsApp Business numbers) | Multi-tenant lives inside `ferro-whatsapp` via its existing phone-validator hook — re-implementing it at the dispatcher layer would duplicate a concern. | |

**Auto-selected:** Option 1.
**Rationale:** ARCH-FINDING-01 makes the original ROADMAP wording untenable without first reshaping `ferro-whatsapp`. The static-facade pattern is already in use by `ferro-stripe` and is the framework convention. Captured as **D-04**, **D-05**, **D-14**. ROADMAP success criterion #3 will be reworded in the same commit chain.

---

## Trait-method symmetry: `to_sms` / `to_push`

| Option | Description | Selected |
|--------|-------------|----------|
| Add both `to_sms()` and `to_push()` as default-`None` trait methods alongside `to_whatsapp` / `to_in_app` | Symmetric trait surface. Future phases ship adapters without a breaking trait change. Closes ARCH-FINDING-03. | ✓ |
| Add only `to_push()` (matching the ROADMAP's explicit mention) | Asymmetric — leaves `to_sms` to a future trait change. ROADMAP's silence on `Sms` is an oversight, not a deliberate exclusion. | |
| Add neither — defer both until their adapters ship | Smallest surface today, biggest churn when adapters land. Trait change is harder than a default-method addition. | |

**Auto-selected:** Option 1.
**Rationale:** Cheapest closure of ARCH-FINDING-03 and the smallest adapter-introduction cost when those phases land. Captured as **D-02, D-06**.

---

## Database channel placeholder fix

| Option | Description | Selected |
|--------|-------------|----------|
| Wire `DatabaseNotificationStore` through `NotificationConfig::database_store: Option<Arc<dyn ...>>` in this phase; placeholder log preserved when unconfigured | InApp already needs this wiring; doing both at once avoids duplicate persistence paths. Backward-compatible (unconfigured = current behavior). | ✓ |
| Leave the placeholder; have InApp own its own store wiring | Two persistence paths into the same trait; later refactor to merge them. | |
| Defer to a separate dedicated phase | Splits a unit of work that naturally belongs together. Burns a phase number on a five-line dispatcher fix. | |

**Auto-selected:** Option 1.
**Rationale:** ARCH-FINDING-02 plus D-08 (InApp dispatch writes both legs through the store) collapse to one wiring change. Splitting them would force the InApp adapter to ship its own ad-hoc persistence path that the Database channel would later replace. Captured as **D-13**.

---

## Claude's Discretion

The following were left for the planner / executor to resolve from existing patterns — no auto-mode pick was needed because the alternatives have no architectural consequence:

- Exact lettre `MultiPart` builder ergonomics (helper vs inline)
- `to_sms` / `to_push` placeholder message types (`SmsMessage` / `PushMessage` empty structs vs deferred)
- Test-fixture choice for SMTP attachment integration (Mailpit vs `lettre::transport::stub`)
- Whether `WhatsAppMessage` is a wrapper or a direct re-export of `ferro_whatsapp::Message`
- Sub-module layout under `channels/` (single file vs sub-folder per channel)

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section. Summary: APNs / FCM Push adapter, SMS adapter, streaming / path-based attachments, cumulative attachment-size enforcement, inbound webhook integration, delivery-receipt webhooks, MCP exposure of Channel variants.

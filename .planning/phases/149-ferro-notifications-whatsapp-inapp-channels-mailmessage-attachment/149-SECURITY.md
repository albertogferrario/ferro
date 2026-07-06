---
phase: 149
slug: ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-01
---

# Phase 149 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| public crate API → consumer | New types exposed via `pub use` in `channels/mod.rs` are part of the crate's public surface | Type signatures and channel definitions |
| ferro-whatsapp Error → ferro-notifications Error | `#[from]` conversion preserves the underlying typed error chain (no message-flattening) | Typed error variants |
| consumer code → MailMessage::attachment | `filename` and `content_type` strings are caller-supplied; later passed to lettre and Resend | File metadata + bytes up to 25 MB |
| in-memory bytes → SMTP/Resend transport | Attachment content held in `Vec<u8>` then forwarded; 25 MB cap protects the transport | Binary attachment content |
| `MailAttachment.filename / content_type` → SMTP wire format | Caller-supplied strings; lettre owns header escaping | Caller-supplied strings |
| `MailAttachment.content` → Resend HTTP API | Bytes are base64-encoded then JSON-serialized via standard alphabet | Binary attachment content |
| `Notifiable::route_notification_for(Channel::WhatsApp)` → `WhatsApp::send` | Phone number is consumer-supplied; ferro-whatsapp `WhatsAppConfig` validator owns sanitization | Recipient phone number |
| `Notifiable::notifiable_id` → broadcast channel name | Adapter constructs `user.{id}`; ferro-broadcast `ChannelAuthorizer` enforces subscriptions | Notifiable ID |
| `Arc<dyn DatabaseNotificationStore>` → database | Consumer-supplied trait implementation; ferro-notifications does not store credentials | Notification record |
| `ferro-notifications` crate publish → crates.io | Wave 1b ordering ensures `ferro-whatsapp` is indexed before `ferro-notifications` references it | Public API surface |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-149-W0-01 | Information Disclosure | `InAppMessage::data` (serde_json::Value) | accept | Caller-supplied; framework does not log payloads. Documented as "passed through to broadcast channel". | closed |
| T-149-W0-02 | Tampering | `InAppSeverity` serde `rename_all="lowercase"` | mitigate | `test_in_app_severity_serialization` locks wire form. Future variants require explicit `#[serde(rename)]` review. | closed |
| T-149-W0-03 | Information Disclosure | Placeholder `SmsMessage` / `PushMessage` | accept | No carrier integration in phase; types are passive data structures with no transport path. | closed |
| T-149-W1A-01 | Tampering | `Channel` serde wire form | mitigate | `test_channel_deserialization` includes regression guard: `"inapp"` is rejected, catching any future refactor that drops the explicit `rename = "in_app"` override. | closed |
| T-149-W1A-02 | Information Disclosure | `Error::AttachmentTooLarge` displays size and filename | accept | Filename is consumer-supplied; size leak is bounded (single integer). Display string is for trusted server-side error logs only. | closed |
| T-149-W1A-03 | Repudiation | `#[from] ferro_whatsapp::Error` error chain | mitigate | Uses `#[from]` (preserves source via `Error::source()`) rather than a string flattening variant. Verified by `test_error_whatsapp_from_impl`. | closed |
| T-149-W1B-01 | Denial of Service | Resource exhaustion via large attachment | mitigate | 25 MB hard cap enforced before push to `Vec<MailAttachment>`. Inclusive boundary. Verified by `test_mail_attachment_at_exact_limit_succeeds` and `test_mail_attachment_over_limit_returns_typed_error`. | closed |
| T-149-W1B-02 | Tampering / Injection | Filename and content_type passed to lettre / Resend | accept | Plan 04 delegates to `lettre::Attachment::new(filename).body(content, ContentType::parse(content_type))` — lettre owns header escaping; `ContentType::parse` is fallible and maps to `Error::Mail`. No second sanitization layer added. | closed |
| T-149-W1B-03 | Information Disclosure | `Vec<u8>` attachment content in process memory | accept | Same trust boundary as the existing `MailMessage::body: String`. No new exposure surface. | closed |
| T-149-W2-01 | Tampering / Header Injection | Filename and content_type passed to lettre | mitigate | Uses `Attachment::new(filename).body(content, ContentType::parse(content_type)?)` — lettre handles all header escaping. `ContentType::parse` is fallible and propagated as `Error::Mail`. | closed |
| T-149-W2-02 | Information Disclosure | Resend payload includes attachment content as base64 in plaintext JSON | accept | TLS-only transport via reqwest default. Same trust as existing `body` and `subject` fields in payload. | closed |
| T-149-W2-03 | Tampering | Base64 alphabet must match Resend's expectation | mitigate | `test_base64_encoding_uses_standard_alphabet` locks the standard alphabet. URL-safe substitution would corrupt binary content. | closed |
| T-149-W2-04 | Resource Exhaustion (Memory) | Base64 encoding allocates ~4/3× the size of original bytes | accept | Bounded by the 25 MB per-attachment cap (T-149-W1B-01). Worst case ~33 MB transient string per attachment — within standard server budgets. | closed |
| T-149-W3-01 | Spoofing / SSRF | Phone number from `route_notification_for(Channel::WhatsApp)` | mitigate (delegated) | ferro-whatsapp's existing phone-validator hook is the enforcement point. Adapter does not add a second validation layer per CONTEXT.md D-03. | closed |
| T-149-W3-02 | Denial of Service | `WhatsApp::send` panic if `init` not called | mitigate | `whatsapp_enabled: false` default + early-return when disabled means panic path is unreachable for default configurations. Documented in `send_whatsapp` rustdoc. | closed |
| T-149-W3-03 | Information Disclosure | `info!(to = %phone, ...)` logs recipient phone | accept | Same trust as existing `to = %to` logging in `send_mail` and `send_slack`. Server-side log retention is consumer policy. | closed |
| T-149-W3-04 | Repudiation | ferro_whatsapp::Error chain via `#[from]` | mitigate | Uses `#[from]` (preserves `source()` chain). Logs `wamid` on success for outbound message correlation. Verified by `test_error_whatsapp_from_impl` (landed Plan 02). | closed |
| T-149-W4-01 | Authorization Bypass | InApp publishes to `user.{id}` channel | mitigate (delegated) | ferro-broadcast `ChannelAuthorizer` is the enforcement point. Plan 07 docs require InApp consumers to configure `ChannelAuthorizer` for `user.*` channels. | closed |
| T-149-W4-02 | Tampering / Privilege Escalation | `Arc<dyn DatabaseNotificationStore>` is consumer-supplied | accept | Same trust model as existing Slack webhook URL injection. Typed boundary; no new attack surface. | closed |
| T-149-W4-03 | Repudiation | InApp dispatch failure mid-leg (DB succeeded, broadcast failed) | accept | Per CONTEXT.md D-08 the error bubbles up; the store retains the persistence record. Broker can replay on client reconnect. Documented in `send_in_app` rustdoc. | closed |
| T-149-W4-04 | Information Disclosure | `info!(notifiable_id = %id, ...)` logs | accept | Same trust as existing `send_mail` and `send_slack` recipient logging. Server-side log retention is consumer policy. | closed |
| T-149-W4-05 | Information Disclosure | `inapp_to_database_message` clones entire `data` payload | accept | Caller-supplied; framework does not introspect or log payload values (only keys via debug format). | closed |
| T-149-W5-01 | Tampering | publish.yml wave ordering | mitigate | Existing 30-second sleep between waves handles indexing. Retry-tolerant `if echo "$OUTPUT" | grep -q "already exists"` block handles double-publish. `ferro-notifications` placed in Wave 1b (ARCH-FINDING-05 closed). | closed |
| T-149-W5-02 | Information Disclosure | Integration test logs recipient address | accept | Localhost-only fixture (`test-recipient@example.com` + Mailpit at localhost:1025); no real data. | closed |
| T-149-W5-03 | Denial of Service | Integration test polls Mailpit API for 5 seconds | accept | Bounded; skip path exits immediately when `MAILPIT_SMTP_HOST` is unset (default CI behavior). | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-149-01 | T-149-W0-01 | InAppMessage::data is caller-supplied and not logged; documented as pass-through. | phase plan | 2026-05-01 |
| AR-149-02 | T-149-W0-03 | Placeholder types have no transport path in this phase; no real carrier risk. | phase plan | 2026-05-01 |
| AR-149-03 | T-149-W1A-02 | AttachmentTooLarge error exposes only a bounded integer and caller-supplied filename to trusted server logs. | phase plan | 2026-05-01 |
| AR-149-04 | T-149-W1B-02 | Second sanitization layer omitted; lettre's `ContentType::parse` is the single enforcement point by design. | phase plan | 2026-05-01 |
| AR-149-05 | T-149-W1B-03 | Attachment bytes in process memory are at the same trust level as existing MailMessage::body. | phase plan | 2026-05-01 |
| AR-149-06 | T-149-W2-02 | Resend attachment payload in base64 JSON is protected by TLS; same trust as existing fields. | phase plan | 2026-05-01 |
| AR-149-07 | T-149-W2-04 | ~33 MB transient allocation per attachment is bounded by the 25 MB cap and within normal server budgets. | phase plan | 2026-05-01 |
| AR-149-08 | T-149-W3-03 | Recipient phone logged at info level; same pattern as existing send_mail / send_slack. Log retention is consumer policy. | phase plan | 2026-05-01 |
| AR-149-09 | T-149-W4-02 | Consumer-supplied DatabaseNotificationStore follows existing Slack webhook trust model; typed boundary. | phase plan | 2026-05-01 |
| AR-149-10 | T-149-W4-03 | Mid-leg InApp failure is documented; DB-first ordering enables broker replay from persisted record. | phase plan | 2026-05-01 |
| AR-149-11 | T-149-W4-04 | notifiable_id logged at info level; same pattern as existing channels. Log retention is consumer policy. | phase plan | 2026-05-01 |
| AR-149-12 | T-149-W4-05 | data payload not logged by values; only debug-format keys are emitted. | phase plan | 2026-05-01 |
| AR-149-13 | T-149-W5-02 | Integration test uses localhost Mailpit with fixture addresses; no real recipient data. | phase plan | 2026-05-01 |
| AR-149-14 | T-149-W5-03 | Poll loop is bounded (5 s max) and exits immediately when MAILPIT_SMTP_HOST is unset. | phase plan | 2026-05-01 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-01 | 25 | 25 | 0 | gsd-secure-phase (static — all summaries confirmed threats_open: 0) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-01

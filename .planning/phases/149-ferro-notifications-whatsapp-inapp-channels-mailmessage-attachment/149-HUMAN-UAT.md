---
status: partial
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
source: ["149-VERIFICATION.md"]
started: "2026-04-29T00:00:00Z"
updated: "2026-04-29T00:00:00Z"
---

## Current Test

[awaiting human testing]

## Tests

### 1. Consumer smoke test in gestiscilo-it (SC #7 — Phase 120 environment)
expected: `use ferro_notifications::{Channel, WhatsAppMessage, InAppMessage, MailAttachment, MailMessage, InAppConfig};` resolves and compiles in the gestiscilo-it consumer crate. `MailMessage::new().attachment(filename, mime, bytes)` compiles and dispatches via the live ferro_whatsapp::WhatsApp::send static facade plus the SSE broker. Real Meta WhatsApp Business API token + a running SSE consumer required — only exercisable from the gestiscilo-it Phase 120 environment.
result: [pending]

### 2. Mailpit live SMTP attachment round-trip (SC #5)
expected: With Mailpit running locally (`docker run -p 1025:1025 -p 8025:8025 axllent/mailpit`), running `MAILPIT_SMTP_HOST=localhost cargo test -p ferro-notifications --features integration-tests --test smtp_attachment_integration` exercises the live SMTP → Mailpit round-trip and asserts byte-equality of the delivered attachment. The integration test skip-path is already verified in default CI (1 passed, 0 failed); the live exercise is the remaining deliverable named in SC #5.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps

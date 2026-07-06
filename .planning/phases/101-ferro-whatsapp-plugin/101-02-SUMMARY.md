---
phase: 101-ferro-whatsapp-plugin
plan: "02"
subsystem: ferro-whatsapp
tags: [whatsapp, messaging, hmac, webhook, deduplication, ferro-events, ferro-queue]

requires:
  - phase: 101-01
    provides: "WhatsApp OnceLock facade, WhatsAppConfig with is_owner closure, Error enum, SenderIdentity, DeliveryStatus types"

provides:
  - "verify_whatsapp_webhook: HMAC-SHA256 webhook verification with constant-time comparison"
  - "signed_whatsapp_payload: test helper for generating sha256= prefixed signatures"
  - "InMemoryDeduplicationStore: DashMap-backed wamid dedup with 5-minute TTL auto-expiry"
  - "DeduplicationStore trait: pluggable dedup backend"
  - "WhatsAppTextReceived: ferro_events::Event for inbound text messages"
  - "WhatsAppStatusUpdate: ferro_events::Event for delivery status updates"
  - "ProcessWhatsAppWebhook: ferro_queue::Job for async Meta envelope parsing and event dispatch"

affects: [101-03, ferro-mcp, ferro-cli]

tech-stack:
  added: []
  patterns:
    - "HMAC-SHA256 webhook verification with sha256= prefix stripping and XOR constant-time comparison"
    - "DashMap + AbortHandle TTL pattern for in-memory dedup (mirrors ferro-ai ConfirmationStore)"
    - "paused-clock TDD for TTL tests: yield_now() before tokio::time::advance()"
    - "ferro_queue::Error::custom() for job parsing errors (not tuple variant)"
    - "dispatch_sync() on ferro-events Event trait for fire-and-forget event emission in queue jobs"

key-files:
  created:
    - ferro-whatsapp/src/webhook/mod.rs
    - ferro-whatsapp/src/webhook/events.rs
    - ferro-whatsapp/src/dedup.rs
  modified:
    - ferro-whatsapp/src/lib.rs

key-decisions:
  - "signed_whatsapp_payload is a regular pub fn (not feature-gated) matching ferro-stripe Phase 96-03 decision — needed in production test suites"
  - "ferro_queue::Error::custom() used for JSON parse errors in ProcessWhatsAppWebhook::handle() — ferro_queue::Error has no tuple variants"
  - "Deduplication deferred to application layer: ProcessWhatsAppWebhook does not check dedup internally; users wire InMemoryDeduplicationStore in their webhook handler before dispatching the job"
  - "parse_text_messages and parse_status_updates are private helpers; ProcessWhatsAppWebhook::handle() calls them directly for clean separation of concerns"

requirements-completed: [WA-02, WA-03, WA-04]

duration: ~10min
completed: 2026-03-23
---

# Phase 101 Plan 02: Webhook Verification, Deduplication, and Event Dispatch Summary

**HMAC-SHA256 webhook verification with constant-time comparison, DashMap-backed wamid dedup with 5-minute TTL, and typed ferro-events dispatch via ProcessWhatsAppWebhook job**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-23
- **Completed:** 2026-03-23
- **Tasks:** 2
- **Files modified:** 4 (+ 2 ferro-json-ui clippy fixes)

## Accomplishments

- HMAC-SHA256 webhook verification with sha256= prefix format and XOR constant-time comparison
- InMemoryDeduplicationStore: DashMap + AbortHandle TTL, entries auto-expire after 5 minutes
- WhatsAppTextReceived and WhatsAppStatusUpdate as typed ferro-events Event structs
- ProcessWhatsAppWebhook as ferro-queue Job that parses Meta's nested JSON envelope and dispatches events
- Sender identity resolution via is_owner closure from WhatsAppConfig before event dispatch
- 40 total tests (23 from Plan 01 + 9 webhook/dedup + 8 events), all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Webhook HMAC verification and InMemoryDeduplicationStore** - `9d307e0` (feat)
2. **Task 2: WhatsAppTextReceived, WhatsAppStatusUpdate, ProcessWhatsAppWebhook** - `ff225cd` (feat)

## Files Created/Modified

- `ferro-whatsapp/src/webhook/mod.rs` - HMAC-SHA256 verification, signed_whatsapp_payload, constant_time_eq
- `ferro-whatsapp/src/webhook/events.rs` - Event structs, ProcessWhatsAppWebhook job, parse helpers
- `ferro-whatsapp/src/dedup.rs` - DeduplicationStore trait, InMemoryDeduplicationStore with TTL
- `ferro-whatsapp/src/lib.rs` - Re-exports for all new public types

## Decisions Made

1. **`signed_whatsapp_payload` is a regular pub fn** — not feature-gated, matching the ferro-stripe Phase 96-03 decision. Needed in production test suites without the `test-helpers` feature flag.
2. **`ferro_queue::Error::custom()`** — used for JSON parse errors in job handle(). The ferro_queue::Error enum uses constructor methods (not tuple variants like `Error::Job`).
3. **Deduplication at the application layer** — ProcessWhatsAppWebhook does not internally check dedup. Users wire an InMemoryDeduplicationStore in their webhook handler before dispatching the job. This keeps the job's responsibility focused on parsing/dispatching and avoids global state in the crate.
4. **Private parse helpers** — `parse_text_messages`, `parse_status_updates`, `resolve_identity` are private functions; the Job trait impl calls them. Tests exercise the helpers directly for isolation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed ferro_queue::Error::Job variant reference**
- **Found during:** Task 2 compilation
- **Issue:** Plan's code snippet used `ferro_queue::Error::Job(...)` but the actual ferro-queue Error enum uses constructor methods, not tuple variants
- **Fix:** Changed to `ferro_queue::Error::custom(format!("invalid webhook JSON: {e}"))`
- **Files modified:** ferro-whatsapp/src/webhook/events.rs
- **Verification:** Compiles and all tests pass
- **Committed in:** ff225cd (Task 2 commit)

**2. [Rule 3 - Blocking] Fixed pre-existing clippy uninlined_format_args in ferro-json-ui**
- **Found during:** Task 2 final verification (`cargo clippy --all --all-targets`)
- **Issue:** ferro-json-ui/src/layout.rs:156 and render.rs:1469+1721 had `format!("...{}", icon)` patterns that clippy -D warnings rejects. These were introduced by the cargo fmt run in Plan 01 (which reformatted the files but didn't touch the underlying code).
- **Fix:** Inlined format args: `format!("..{icon}..")` pattern
- **Files modified:** ferro-json-ui/src/layout.rs, ferro-json-ui/src/render.rs
- **Verification:** `cargo clippy --all --all-targets -- -D warnings` passes clean
- **Committed in:** ff225cd (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2x Rule 3 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation and CI compliance. No scope creep.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None - no external service configuration required for this plan.

## Next Phase Readiness

- Inbound processing pipeline complete: HMAC verification → dedup → identity resolution → typed event dispatch
- Plan 03 (CLI scaffolding, MCP tools, docs) can proceed
- ferro-whatsapp is now fully bidirectional: outbound (Plan 01) + inbound (this plan)

---
*Phase: 101-ferro-whatsapp-plugin*
*Completed: 2026-03-23*

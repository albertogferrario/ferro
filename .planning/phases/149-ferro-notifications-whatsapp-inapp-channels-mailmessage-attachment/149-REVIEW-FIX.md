---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
fixed_at: 2026-04-29T00:00:00Z
review_path: .planning/phases/149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment/149-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 149: Code Review Fix Report

**Fixed at:** 2026-04-29
**Source review:** .planning/phases/149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment/149-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

Scope was `critical_warning` (WR-01 through WR-05). The 6 Info findings (IN-01 through IN-06) were intentionally out of scope and are tracked for future hardening phases.

## Fixed Issues

### WR-01: `MailConfig::credentials` silently mutates Resend config into SMTP shape

**Files modified:** `ferro-notifications/src/dispatcher.rs`
**Commit:** d49750cb
**Applied fix:** Added a `matches!(self.driver, MailDriver::Smtp)` guard at the top of `MailConfig::credentials`. When the driver is not SMTP (i.e. Resend), the method now logs at `warn!` and returns `self` unchanged instead of inserting a phantom `SmtpConfig` via `get_or_insert`. Switched the SMTP-path insert to `get_or_insert_with` for consistency. Also added a `tracing::warn` import (extended the existing `tracing::{error, info}` use). Documented the new contract in the rustdoc.

### WR-02: In-app dispatch is not atomic; docstring overstates the guarantee

**Files modified:** `ferro-notifications/src/dispatcher.rs`
**Commit:** afddafa3
**Applied fix:** Replaced the line "Either failure aborts the dispatch (no partial-success silent fallback)" with a multi-line note that explicitly states the legs are NOT transactional, that a DB-success / broadcast-failure path leaves a persisted row alongside an `Err` return, and that callers performing manual retries must dedupe on `(notifiable_id, notification_type, idempotency-key)`. Cross-references CONTEXT.md D-08 for the design rationale. Documentation-only change.

### WR-03: Resend driver does not validate response body shape; no message id logged

**Files modified:** `ferro-notifications/src/dispatcher.rs`
**Commit:** 07abfa57
**Applied fix:** After the 2xx success branch, parse the response body as `serde_json::Value`, extract `id` as a string, and include it as `resend_id = %resend_id` in the success `info!` log. Parse failure is non-fatal (the send already succeeded per the status code) and is logged at `warn!` with `<unparseable>` as the placeholder id. Missing `id` field falls back to `<no-id>`.

### WR-04: WhatsApp channel does no rate-limit/retry handling

**Files modified:** `ferro-notifications/src/dispatcher.rs`
**Commit:** 986ab164
**Applied fix:** Added a 5-line rustdoc paragraph to `send_whatsapp` that (a) states retry is the caller's responsibility, (b) names the retryable variants by example (`RateLimit`, `NetworkError`) vs terminal (invalid phone number), (c) points at `Error::WhatsApp(inner)` as the public match surface, and (d) recommends `ferro-queue` as the retry path. Documentation-only change — no behavior change to the dispatch.

### WR-05: Database channel is silent no-op when configured store is absent

**Files modified:** `ferro-notifications/src/dispatcher.rs`
**Commit:** 3391fed8
**Applied fix:** Promoted the unconfigured-store branch from `info!` to `warn!` and rephrased the message from "Database notification stored (placeholder — no store configured)" to "Database notification dropped — no store configured. Call NotificationConfig::with_database_store() at startup." This is the minimal-churn option (#2) from the review — preserves the silent-success return value to avoid breaking existing placeholder-path consumers, but raises the log level so the drop is visible in default-configured operator dashboards.

---

_Fixed: 2026-04-29_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

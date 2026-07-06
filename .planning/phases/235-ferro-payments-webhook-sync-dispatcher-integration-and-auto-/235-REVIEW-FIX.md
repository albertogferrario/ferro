---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-
fixed_at: 2026-06-17T00:00:00Z
review_path: .planning/phases/235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-/235-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 235: Code Review Fix Report

**Fixed at:** 2026-06-17
**Source review:** .planning/phases/235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-/235-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01 through WR-04; IN-01/IN-02/IN-03 out of scope per fix_scope=critical_warning)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: `attach_session` return value silently discarded

**Files modified:** `ferro-payments/src/service.rs`
**Commit:** 60a6ffb5
**Applied fix:** Captured the `bool` return from `lifecycle::attach_session` into `attached`; added `tracing::warn!(row_id = row.id, "attach_session no-op: session already attached")` when `!attached`. The `?` was already present for the error path; the branch now makes the unexpected-but-harmless no-op observable in logs.

---

### WR-02: `handle_session_completed` — `on_paid` error swallowed and re-classified as `SideStateConflict`

**Files modified:** `ferro-payments/src/webhook.rs`
**Commit:** 9c5e3a6c
**Applied fix:** Added a private `is_transient(e: &PaymentError) -> bool` helper that returns `true` for `PaymentError::Db(_) | PaymentError::Stripe(_)`. In the `on_paid` error arm: rollback the txn, then branch — transient errors propagate via `return Err(e)` (Stripe retries; `mark_paid` is idempotent via guards), permanent errors (`StatusPrecondition`, `Loader`, `NotFound`, `AutoRefundTriggered`) trigger `trigger_auto_refund` with `SideStateConflict`. This prevents a transient DB deadlock from issuing an irreversible refund.

**Status:** fixed: requires human verification (logic classification — `is_transient` predicate covers Db/Stripe as transient, all others as terminal)

---

### WR-03: `handle_session_expired` / `handle_charge_refunded` — `StatusPrecondition` reaches the retry bridge

**Files modified:** `ferro-payments/src/webhook.rs`
**Commit:** 9c5e3a6c
**Applied fix:** Both handlers now use the shared `is_transient` helper on `on_released`/`on_refunded` errors. Transient errors propagate (Stripe retries). Terminal errors (`StatusPrecondition` and all other non-infrastructure variants) are absorbed — txn is rolled back and the handler returns `Ok(())`, stopping the Stripe retry loop. This mirrors the policy stated in the `payment_to_stripe_error` doc-comment ("terminal outcomes must never reach this bridge").

**Status:** fixed: requires human verification (logic classification — same `is_transient` predicate applied consistently)

---

### WR-04: `request_refund` Stripe failure leaves row silently stuck as refund-in-flight

**Files modified:** `ferro-payments/src/service.rs`
**Commit:** 9c5e3a6c
**Applied fix:** The `create_refund` `.map_err` closure now calls `tracing::error!(intent_id, %charge_id, err = %e, "request_refund Stripe call failed; row is refund-in-flight (refund_amount_cents set, refunded_at NULL) — phase-236 reaper recovers")` before wrapping into `PaymentError::Stripe(e)`. Mirrors the logging in `trigger_auto_refund`'s Stripe failure branch.

---

## Skipped Issues

None — all in-scope findings were fixed.

---

## Out-of-scope Info Findings (not fixed)

- **IN-01:** `idempotency_key` suppressed in `ferro-stripe/refund.rs` — async-stripe 0.41 limitation, `// TODO` comment suggestion. Out of scope per fix_scope=critical_warning.
- **IN-02:** Missing test for `trigger_auto_refund` when `payment_intent_id` is `None`. Out of scope per fix_scope=critical_warning.
- **IN-03:** `PaymentError::AutoRefundTriggered` is defined but never returned. Out of scope per fix_scope=critical_warning.

---

## Verification

- `cargo fmt --all`: clean
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: clean
- `cargo test -p ferro-payments`: 39 passed, 0 failed

_Fixed: 2026-06-17_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

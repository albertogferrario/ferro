---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: "03"
subsystem: ferro-payments
tags: [payments, reaper, lifecycle, stripe, testing]
dependency_graph:
  requires: [236-01, 236-02]
  provides: [PAY-POLY-REAP-01, PAY-POLY-REAP-02]
  affects: [ferro-payments/src/service.rs]
tech_stack:
  added: []
  patterns:
    - per-intent async block returning Result<bool> for Succeeded/skip discrimination
    - D-06 benign loader-vanished skip (warn, no auto-refund, no Err propagation)
    - D-09 double-refund guard (Failed → warn only, zero Stripe calls)
    - D-05 failure isolation (on_released/on_refunded Err → log, loop continues)
    - clock injection via _at(now) inner method for deterministic offline tests
key_files:
  modified:
    - ferro-payments/src/service.rs
decisions:
  - "reconcile_refunds_in_flight_at uses Result<bool> (not Result<()>) as the per-intent block return so Pending/Failed/race-no-op returning Ok(false) are structurally distinct from Succeeded Ok(true) — prevents accidental count increment"
  - "reaper_continues_on_error tests on_released returning Err (not loader returning Err): loader Err hits D-06 benign path (Ok(false)/warn), not D-05 failure isolation; the test targets the correct isolation boundary"
metrics:
  duration_minutes: 6
  completed_date: "2026-06-21"
  tasks_completed: 2
  files_modified: 1
---

# Phase 236 Plan 03: Reaper Methods (release_expired + reconcile_refunds_in_flight) Summary

Implements the two reaper methods on `PaymentService<L>` that self-heal money-stuck edge cases: expired reserved intents and refund-in-flight intents that Stripe resolved without a webhook delivery.

## What Was Built

**`release_expired_at(now)` / `release_expired()`** (PAY-POLY-REAP-01):

- `find_expired(now)` selects `status=reserved AND expires_at < now`.
- Per-intent loop: `mark_released` (GuardedUpdate `reserved→released`) → loader → `on_released` txn.
- `mark_released` returns `Ok(false)` (racing webhook) → no-op skip, no `on_released`.
- Loader `Ok(None)/Err` → D-06 benign skip: `tracing::warn!`, no auto-refund (no money was captured — status was `reserved`).
- `on_released` returns `Err` → D-05 isolation: `tracing::error!`, loop continues, other rows still release.
- Returns count of rows whose per-intent block returned `Ok(true)` (i.e. `on_released` completed).

**`reconcile_refunds_in_flight_at(now)` / `reconcile_refunds_in_flight()`** (PAY-POLY-REAP-02):

- Age anchor: `older_than = now - 1h`; `find_refunds_in_flight(older_than)` selects `status=paid AND refund_amount_cents IS NOT NULL AND refunded_at IS NULL AND paid_at < older_than`.
- Per-intent loop: poll `fetch_refund_status_for_payment_intent` → match `RefundStatus`:
  - `Succeeded { amount_cents }` → `mark_refunded` → `on_refunded` txn → `Ok(true)` (counted).
  - `Pending` → `Ok(false)` (skip, left for next tick).
  - `Failed` → D-09 double-refund guard: `tracing::warn!`, **no Stripe call**, **no `mark_refunded`` → `Ok(false)`.
- `mark_refunded` returning `Ok(false)` (webhook race) → `Ok(false)` no-op.
- Loader `Ok(None)/Err` on Succeeded path → rollback + `Ok(false)` skip (mirrors webhook `_` arm).
- D-05: `on_refunded` Err → `tracing::error!`, loop continues.

## Tests Added

| Test | Covers |
|------|--------|
| `release_expired` | Happy path: expired row → released, count=1 |
| `release_expired_excludes_non_expired_row` | Future-expires row untouched, count=0 |
| `reaper_skips_already_released` | Pre-released row excluded by find_expired (status != reserved) |
| `reaper_continues_on_error` | `on_released` Err → logged, loop continues, other row counted (D-05) |
| `reconcile_succeeded` | Stripe Succeeded → mark_refunded + on_refunded, count=1 |
| `reconcile_pending_noop` | Stripe Pending → row untouched, count=0 |
| `reconcile_failed_no_retry` | Stripe Failed → warn only, zero create_refund calls, count=0 (D-09) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] reconcile_pending_noop counted Pending as resolved**
- **Found during:** Task 2 test run
- **Issue:** Per-intent block used `Result<(), PaymentError>`. All early-return paths (`Pending`, `Failed`, race no-op) returned `Ok(())`, which the outer `match { Ok(()) => resolved += 1 }` incremented unconditionally.
- **Fix:** Changed block return type to `Result<bool, PaymentError>`. `Succeeded` path returns `Ok(true)` (counted); all skip paths return `Ok(false)` (not counted). Outer match: `Ok(true) => resolved += 1`, `Ok(false) => {}`.
- **Files modified:** `ferro-payments/src/service.rs`
- **Commit:** 9df8350e

**2. [Rule 1 - Bug] reaper_continues_on_error tested wrong failure path**
- **Found during:** Task 1 test run
- **Issue:** Initial test made the loader return `Err`, but D-06 (loader-vanished benign skip) treats loader `Err` as `Ok(false)` (warn + skip) → block returns `Ok(())` (original design) → count was 2, not 1.
- **Fix:** Changed test to make `on_released` return `Err` instead — that is the D-05 isolation path. The block returns `Err(e)` → outer match logs and skips the increment → count=1 as expected.
- **Files modified:** `ferro-payments/src/service.rs`
- **Commit:** 9df8350e

**3. [Rule 2 - Missing critical functionality] Missing blank lines in doc bullet lists**
- **Found during:** clippy pass
- **Fix:** Added blank line after the final bullet in both `release_expired_at` and `reconcile_refunds_in_flight_at` doc comments to satisfy `clippy::doc_lazy_continuation`.
- **Commit:** 9df8350e

**4. [Rule 2 - Missing critical functionality] MutexGuard held across await**
- **Found during:** clippy pass
- **Fix:** Wrapped Mutex lock assertions in explicit `{ }` scopes in `reconcile_succeeded` and `reconcile_failed_no_retry` tests so guards are dropped before the subsequent `.await` calls.
- **Commit:** 9df8350e

## Self-Check

- `pub async fn release_expired` in service.rs: FOUND
- `release_expired_at` in service.rs: FOUND
- `find_expired(` in service.rs: FOUND
- `no money captured` in service.rs: FOUND
- `pub async fn reconcile_refunds_in_flight` in service.rs: FOUND
- `reconcile_refunds_in_flight_at` in service.rs: FOUND
- `fetch_refund_status_for_payment_intent` in service.rs: FOUND
- `double-refund guard` in service.rs: FOUND
- Commit 9df8350e: FOUND

## Self-Check: PASSED

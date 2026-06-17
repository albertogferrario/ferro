---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
plan: "05"
subsystem: ferro-payments
tags: [payments, stripe, webhook, dispatcher, idempotency, auto-refund, race-safety]
dependency_graph:
  requires:
    - ferro_stripe::SyncDispatcher::on (ferro-stripe plan 01)
    - ferro_stripe::MemoryProcessedLog + ProcessedEventLog (ferro-stripe plan 01)
    - ferro_stripe::{StripeCheckoutCompleted, StripeCheckoutExpired, StripeChargeRefunded} (ferro-stripe plan 01)
    - lifecycle::find_by_payment_intent, find_by_charge_id, attach_payment_intent (235-03)
    - PaymentService::processed_log field + new() with processed_log (235-04)
    - StripeGateway::create_refund_for_payment_intent (235-04)
  provides:
    - ferro_payments::wire_dispatcher (pub)
    - PaymentService::handle_session_completed (pub(crate))
    - PaymentService::handle_session_expired (pub(crate))
    - PaymentService::handle_charge_refunded (pub(crate))
    - PaymentService::trigger_auto_refund (private)
    - payment_to_stripe_error (private error bridge)
    - lib.rs re-exports: wire_dispatcher, find_by_payment_intent, find_by_charge_id, attach_payment_intent
  affects: [ferro-payments phase 236 reapers, gestiscilo consumer adoption]
tech_stack:
  added:
    - tracing = "0.1" (ferro-payments/Cargo.toml)
  patterns:
    - processedEventLog-idempotency-fastpath
    - sea-orm-transaction-begin-commit-rollback
    - guarded-update-is-null-dedup
    - arc-two-level-clone-for-fn-closures
    - auto-refund-on-unhonorable-capture
    - charge-id-fallback-lookup
key_files:
  created:
    - ferro-payments/src/webhook.rs
  modified:
    - ferro-payments/src/lib.rs
    - ferro-payments/src/service.rs
    - ferro-payments/Cargo.toml
decisions:
  - "PaymentService fields promoted to pub(crate) (db, stripe, processed_log, loader) — webhook.rs is a sibling module and cannot access private fields; no accessor method added to keep the impl clean"
  - "payment_to_stripe_error simplified: PaymentError::Stripe(s) => s (move, not clone); all other variants => Error::Stripe(format!(...)); ferro_stripe::Error does not implement Clone"
  - "LoaderBehavior enum variants renamed to Billable/None/Error (not ReturnBillable/ReturnNone/ReturnError) — clippy enum_variant_names lint fires on shared prefix"
  - "tracing added as explicit dependency — was used via transitive dep before; explicit makes it stable"
  - "Task 1 and Task 2 committed together since tests live in the same webhook.rs file; no separate test-only commit was possible without splitting the file"
metrics:
  duration_seconds: 559
  completed_date: "2026-06-17"
  tasks_completed: 2
  files_modified: 4
  files_created: 1
---

# Phase 235 Plan 05: wire_dispatcher + webhook handlers + auto-refund + 12-test suite Summary

**One-liner:** `wire_dispatcher` registers three idempotency-guarded, transactional Stripe webhook handlers (`checkout.session.completed`, `checkout.session.expired`, `charge.refunded`) with auto-refund fallback (`trigger_auto_refund`) that proves money-loss correctness via 12 offline tests covering replay, race, and every auto-refund path.

## What Was Built

### Task 1: `ferro-payments/src/webhook.rs` (new file, 1233 lines with tests)

**Production code:**

- `wire_dispatcher<L: BillableLoader + 'static>(dispatcher: SyncDispatcher, service: Arc<PaymentService<L>>) -> SyncDispatcher` — consuming builder registering three typed handlers via the two-level Arc-clone Fn-closure pattern (pre-clone svc1/svc2/svc3 outside; re-clone inside each `async move`). Ensures closures are `Fn` not `FnOnce` so Stripe retries work.
- `payment_to_stripe_error(e: PaymentError) -> ferro_stripe::Error` — error bridge: `PaymentError::Stripe(s) => s` (move); all other variants → `Error::Stripe(format!("payment: {e}"))`. Terminal outcomes never reach this bridge; handlers absorb them as `Ok(())`.
- `handle_session_completed` — idempotency fast-path → `find_by_stripe_session` → `mark_paid` (GuardedUpdate) → `attach_payment_intent` → `db.begin()` → `loader.load()` → `on_paid(&txn)`. Any failure path routes to `trigger_auto_refund` with `SideStateConflict`/`LoaderError`/`BillableVanished`.
- `handle_session_expired` — idempotency fast-path → `find_by_stripe_session` → `mark_released` (GuardedUpdate, `Ok(false)` = no-op) → `db.begin()` → `loader.load()` → `on_released(&txn)`.
- `handle_charge_refunded` — idempotency fast-path → `find_by_payment_intent` (primary) / `find_by_charge_id` (fallback when `payment_intent_id: None`) → `mark_refunded` → `db.begin()` → `loader.load()` → `on_refunded(&txn, amount_cents)`.
- `trigger_auto_refund` — `None` `payment_intent_id` → debug log + `Ok(())` (free/setup sessions). Else: `GuardedUpdate WHERE refund_amount_cents IS NULL` snapshot (exactly-once dedup); `Ok(false)` = already snapshotted → no-op. Then `stripe.create_refund_for_payment_intent`: on `Err` logs "refund-in-flight (phase-236 reaper recovers)" and returns `Ok(())` — never compensate-reset (D-11).

**`ferro-payments/src/service.rs`:** Fields `db`, `stripe`, `processed_log`, `loader` promoted to `pub(crate)`; `#[allow(dead_code)]` annotations removed.

**`ferro-payments/src/lib.rs`:** `mod webhook;` added; `pub use webhook::wire_dispatcher;`; `pub use intent::lifecycle::{attach_payment_intent, find_by_charge_id, find_by_payment_intent};`.

**`ferro-payments/Cargo.toml`:** `tracing = "0.1"` added.

### Task 2: 12 named tests in `webhook.rs` `#[cfg(test)] mod tests`

All 12 tests pass offline (MockStripeGateway + MemoryProcessedLog + in-memory SQLite):

| Test | Requirement | Assertion |
|------|-------------|-----------|
| `handle_session_completed` | WH-02 | status=paid, pi attached, on_paid count=1, no auto-refund |
| `handle_session_completed_replay` | WH-02 (T-235-09) | on_paid count=1 across two dispatches |
| `handle_session_completed_side_state_conflict` | WH-02/06 | pi_refund_calls len=1, (pi_id, Some(1000)) |
| `handle_session_expired` | WH-03 | status=released, on_released count=1 |
| `handle_session_expired_noop` | WH-03 | on_released count=0 |
| `handle_charge_refunded` | WH-04 | status=refunded, on_refunded([750]) |
| `handle_charge_refunded_charge_id_fallback` | WH-04 (D-07) | status=refunded via charge_id fallback |
| `auto_refund_billable_vanished` | WH-05 (T-235-11) | pi_refund_calls len=1, refund_amount_cents snapshotted |
| `auto_refund_loader_error` | WH-05 (T-235-11) | pi_refund_calls len=1 |
| `webhook_reaper_race` | WH-06 (T-235-10) | on_paid=0, pi_refund_calls len=1 |
| `paid_after_released` | WH-06 | pi_refund_calls len=1 |
| `wire_dispatcher_registers_three_handlers` | WH-01 | dispatch routes to handle_session_completed; status=paid |

## Verification

- `cargo build -p ferro-payments`: exit 0
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: exit 0
- `cargo test -p ferro-payments`: 39 passed, 0 failed (includes all 12 new webhook tests)
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | wire_dispatcher + 3 webhook handlers + auto-refund + lib.rs re-exports | 30b4e317 | ferro-payments/src/webhook.rs (new), ferro-payments/src/lib.rs, ferro-payments/src/service.rs, ferro-payments/Cargo.toml, Cargo.lock |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] `pub(crate)` field visibility for `PaymentService`**
- **Found during:** Task 1 (first build attempt)
- **Issue:** `webhook.rs` is a sibling module and cannot access private fields of `PaymentService` directly. The plan mentioned "accessor (`self.loader()`) OR direct `self.loader`" but no accessor existed in service.rs from plan 04.
- **Fix:** Promoted `db`, `stripe`, `processed_log`, `loader` to `pub(crate)`. Removed `#[allow(dead_code)]` annotations (now used). `return_url_builder` kept private (not needed in webhook.rs).
- **Files modified:** ferro-payments/src/service.rs
- **Commit:** 30b4e317

**2. [Rule 1 - Bug] `ferro_stripe::Error` not `Clone` — `payment_to_stripe_error` fix**
- **Found during:** Task 1 (first build attempt)
- **Issue:** PATTERNS.md showed `PaymentError::Stripe(ref s) => s.clone()` but `ferro_stripe::Error` does not derive `Clone`.
- **Fix:** Changed to `PaymentError::Stripe(s) => s` (move) and collapsed the match to two arms.
- **Files modified:** ferro-payments/src/webhook.rs
- **Commit:** 30b4e317

**3. [Rule 1 - Bug] `LoaderBehavior` enum clippy `enum_variant_names` lint**
- **Found during:** Task 2 (clippy gate)
- **Issue:** `ReturnBillable`/`ReturnNone`/`ReturnError` share the `Return` prefix — clippy fires `-D warnings`.
- **Fix:** Renamed to `Billable`/`None`/`Error`; updated `Ok(Option::None)` to avoid shadowing.
- **Files modified:** ferro-payments/src/webhook.rs
- **Commit:** 30b4e317

**4. [Rule 2 - Missing dependency] `tracing` not in `ferro-payments/Cargo.toml`**
- **Found during:** Task 1 (first build — `error[E0433]: use of unresolved module tracing`)
- **Issue:** `tracing::warn!` / `tracing::error!` / `tracing::debug!` called in webhook.rs but `tracing` was not an explicit dependency.
- **Fix:** Added `tracing = "0.1"` to `[dependencies]`.
- **Files modified:** ferro-payments/Cargo.toml
- **Commit:** 30b4e317

**5. [Rule 2 - Missing import] `EntityTrait` in test module**
- **Found during:** Task 2 (clippy --all-targets)
- **Issue:** `Entity::find_by_id` requires `EntityTrait` in scope; test imports used `use crate::intent::entity::Entity` but not the trait.
- **Fix:** Added `use sea_orm::EntityTrait as _;` to test imports.
- **Files modified:** ferro-payments/src/webhook.rs
- **Commit:** 30b4e317

**6. [Rule 1 - Bug] `Arc<MemoryProcessedLog>` coercion to `Arc<dyn ProcessedEventLog>`**
- **Found during:** Task 2 (`handle_session_completed_replay` test construction)
- **Issue:** `Arc::clone(&log)` where `log: Arc<MemoryProcessedLog>` could not coerce to `Arc<dyn ProcessedEventLog>` without an explicit type annotation.
- **Fix:** Added explicit type: `let log: Arc<dyn ProcessedEventLog> = Arc::new(MemoryProcessedLog::new());`
- **Files modified:** ferro-payments/src/webhook.rs
- **Commit:** 30b4e317

## Known Stubs

None. All handlers are fully implemented and tested. The `trigger_auto_refund` "refund-in-flight" path (Stripe call failure, D-11) is implemented with a `tracing::error!` log and `Ok(())` return — the phase-236 reaper is the intended recovery, not a stub.

## Threat Flags

None beyond the plan's threat model. All T-235-09 through T-235-13 mitigations are implemented and proven by tests.

## Self-Check: PASSED

- ferro-payments/src/webhook.rs — FOUND (created)
- ferro-payments/src/lib.rs — FOUND (modified, contains `pub use webhook::wire_dispatcher`)
- ferro-payments/src/service.rs — FOUND (modified, fields pub(crate))
- ferro-payments/Cargo.toml — FOUND (modified, tracing added)
- Commit 30b4e317 — FOUND
- All 12 test functions present and green

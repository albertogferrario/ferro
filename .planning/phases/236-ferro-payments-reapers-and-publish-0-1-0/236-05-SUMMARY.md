---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: "05"
subsystem: ferro-payments
tags: [testing, integration-test, payments, stripe, billable]
dependency_graph:
  requires: [236-04]
  provides: [PAY-POLY-REAP-04]
  affects: [ferro-payments]
tech_stack:
  added: []
  patterns:
    - "#[ignore]-gated integration test with env-var early-return skip"
    - "in-memory SQLite harness (TestMigrator + fresh_db) in test binary"
    - "example Billable + BillableLoader defined in integration test file"
key_files:
  created:
    - ferro-payments/tests/integration.rs
  modified:
    - ferro-payments/src/intent/lifecycle.rs  # cargo fmt only
    - ferro-payments/src/lib.rs               # cargo fmt only
    - ferro-payments/src/reaper.rs            # cargo fmt only
    - ferro-payments/src/service.rs           # cargo fmt only
decisions:
  - "Integration test lives in tests/ (not src/) so it compiles against the public API only"
  - "ReservationBillable named 'reservation' (not 'booking') to distinguish from service.rs test fixtures"
  - "Negative TTL (-1s) used to make rows immediately expired at creation time — avoids time.Sleep, deterministic"
  - "cargo fmt applied to all ferro-payments/src files in same commit (no logic changes)"
metrics:
  duration_minutes: 5
  completed_date: "2026-06-20"
  tasks_completed: 1
  files_changed: 5
---

# Phase 236 Plan 05: Integration Test (gated e2e) Summary

`#[ignore]`-gated end-to-end integration test that drives the full ferro-payments consumer path against Stripe test mode, skipping cleanly when the key is absent.

## Objective

Satisfy the spec's "Integration (workspace test bin)" row (D-10) without adding a new publishable crate. Single `#[ignore]`-gated test in `ferro-payments/tests/integration.rs` drives `start_checkout → release_expired` via the public API and skips cleanly (early return, no panic) when `STRIPE_TEST_SECRET_KEY` is absent.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create `#[ignore]`-gated e2e integration test | 2582ae05 | ferro-payments/tests/integration.rs (created), 4 src files (fmt) |

## What Was Built

`ferro-payments/tests/integration.rs` contains:

1. **`ReservationBillable`** — a minimal example `Billable` impl with stable values (kind=`"reservation"`, 500 EUR cents, no Connect). All three `on_*` hooks are no-ops — the test asserts row state, not consumer side effects.
2. **`ReservationLoader`** — a minimal `BillableLoader` returning a boxed `ReservationBillable` for any `(kind, id)`.
3. **`fresh_db()` + `TestMigrator`** — in-memory SQLite harness copying the pattern from `service.rs` tests; uses the public `CreatePaymentIntentsTable` migration.
4. **`e2e_checkout_and_release`** — the gated test:
   - `#[ignore]` so `cargo test --all-features` skips it by default.
   - Reads `STRIPE_TEST_SECRET_KEY`; early-returns (`return;`) when absent — no panic.
   - When key present: init `ferro_stripe::Stripe`, call `start_checkout` with a negative TTL (row immediately expired), then `release_expired()`, assert count=1 and row `status=released`.

## Verification Results

All acceptance criteria met:

```
cargo test -p ferro-payments --test integration
  → 0 passed, 0 failed, 1 ignored  (test skipped by default) ✓

cargo test -p ferro-payments --test integration -- --ignored
  → 1 passed, 0 failed, 0 ignored  (early-return skip without key) ✓

cargo clippy -p ferro-payments --all-targets -- -D warnings
  → Finished (exit 0) ✓

cargo fmt -p ferro-payments -- --check
  → (no output, exit 0) ✓
```

Grep checks:
- `#[ignore` present ✓
- `STRIPE_TEST_SECRET_KEY` present ✓
- `return;` (early-return skip) present ✓
- `impl Billable for` present ✓
- `release_expired` present ✓
- File is 239 lines (>= 40-line minimum) ✓

## Deviations from Plan

**1. [Rule 1 - Bug] cargo fmt applied to pre-existing src files**
- **Found during:** Task 1 — `cargo fmt --check` showed formatting diffs across 4 pre-existing `ferro-payments/src/` files from prior phases.
- **Fix:** Applied `cargo fmt -p ferro-payments` to fix formatting (no logic changes). All 4 files had pure whitespace/alignment reformatting.
- **Files modified:** `ferro-payments/src/intent/lifecycle.rs`, `ferro-payments/src/lib.rs`, `ferro-payments/src/reaper.rs`, `ferro-payments/src/service.rs`
- **Commit:** 2582ae05 (included in task commit)

## Known Stubs

None. The test is intentionally minimal — the `on_*` no-ops are by design (the test asserts payment-intent row state, not consumer side effects). This is documented inline.

## Threat Flags

None. The integration test file introduces no new network endpoints, auth paths, or schema changes. It is test-only (`tests/` directory, not compiled into the library).

## Self-Check: PASSED

- File exists: `/Users/alberto/repositories/albertogferrario/ferro/ferro-payments/tests/integration.rs` ✓
- Commit 2582ae05 exists in git log ✓
- `cargo test -p ferro-payments --test integration` exits 0 ✓
- `cargo test -p ferro-payments --test integration -- --ignored` exits 0 ✓

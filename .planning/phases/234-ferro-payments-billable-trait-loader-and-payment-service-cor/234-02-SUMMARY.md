---
phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor
plan: "02"
subsystem: ferro-payments
tags: [rust, payments, async-trait, object-safe, lifecycle, guarded-update]
dependency_graph:
  requires: [233-03]
  provides: [Billable trait, BillableLoader trait, attach_session lifecycle fn]
  affects: [ferro-payments/src/billable.rs, ferro-payments/src/loader.rs, ferro-payments/src/intent/lifecycle.rs]
tech_stack:
  added: []
  patterns: [async-trait object-safe trait, GuardedUpdate IS NULL guard, TDD per task]
key_files:
  created:
    - ferro-payments/src/billable.rs
    - ferro-payments/src/loader.rs
  modified:
    - ferro-payments/src/intent/lifecycle.rs
    - ferro-payments/src/lib.rs
decisions:
  - "D-05: connect_account_id defaults to None — non-Connect billables need no override"
  - "D-06: Billable is NOT Clone — everything passes &dyn Billable"
  - "D-07/08: BillableLoader::load takes (kind, id) with no tenant_id — loader owns scoping"
  - "T-234-03: attach_session guarded by StripeSessionId IS NULL — second write is Ok(false) no-op"
metrics:
  duration: "~8 minutes"
  completed: "2026-06-17T03:31:31Z"
  tasks: 3
  files: 4
requirements: [PAY-POLY-SVC-01, PAY-POLY-SVC-02, PAY-POLY-SVC-03]
---

# Phase 234 Plan 02: Billable Trait + BillableLoader Trait + attach_session Summary

Object-safe `#[async_trait]` `Billable` and `BillableLoader` traits plus a guarded `attach_session` lifecycle function, providing the domain abstraction surface that Wave 2's `PaymentService` orchestrates.

## What Was Built

### Task 1: Billable trait (`ferro-payments/src/billable.rs`)

`Billable: Send + Sync` with six sync accessors (`kind`, `id`, `tenant_id`, `amount_cents`, `currency`, `checkout_line_description`), a defaulted `connect_account_id() -> None` (D-05), and three async side effects (`on_paid`, `on_released`, `on_refunded`) all taking `&DatabaseTransaction`. NOT `Clone` (D-06). `Box<dyn Billable>` is constructible. Tests confirm object-safety and the `None` default.

### Task 2: BillableLoader trait (`ferro-payments/src/loader.rs`)

Single-method trait: `async fn load(kind: BillableKind, id: i64) -> Result<Option<Box<dyn Billable>>, PaymentError>`. No `tenant_id` arg (D-08). `MockLoader` returning `Ok(None)` confirms object-safety (`&dyn BillableLoader` usable).

### Task 3: attach_session lifecycle function (`ferro-payments/src/intent/lifecycle.rs`)

Added after `mark_refunded`, before query functions. `GuardedUpdate WHERE stripe_session_id IS NULL` atomically sets `stripe_session_id` (Value::String) and `application_fee_cents` (Value::BigInt — `None` maps to `Value::BigInt(None)` for SQL NULL, not 0). Returns `Ok(true)` on first attach, `Ok(false)` on retry (guard excluded row). Two tests: happy-path value assertion and idempotent-noop assertion.

### lib.rs

Added `pub mod billable;` and `pub mod loader;`. Re-exports (`pub use billable::Billable` etc.) are deferred to Plan 03 to avoid unused-import churn.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 0e1c6cac | feat(234-02): define Billable trait |
| 2 | 9f0d2ee9 | feat(234-02): define BillableLoader trait |
| 3 | 0a7bfcfb | feat(234-02): add attach_session lifecycle fn |

## Test Results

- `cargo test -p ferro-payments`: 17 passed, 0 failed
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: 0 warnings

## Deviations from Plan

None — plan executed exactly as written. The unused `DatabaseTransaction` import in the loader test was removed during the GREEN phase (no warning under `-D warnings`); this is a quality fix within the task, not a plan deviation.

## Known Stubs

None. All three artifacts are complete trait/function definitions with no placeholder return values.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: tampering-mitigated | ferro-payments/src/intent/lifecycle.rs | T-234-03: `attach_session` IS NULL guard prevents session-id overwrite on retry/race — mitigation implemented as required |

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-payments/src/billable.rs | FOUND |
| ferro-payments/src/loader.rs | FOUND |
| Commit 0e1c6cac (Billable trait) | FOUND |
| Commit 9f0d2ee9 (BillableLoader trait) | FOUND |
| Commit 0a7bfcfb (attach_session) | FOUND |

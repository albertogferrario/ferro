---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: "04"
subsystem: ferro-payments
tags: [ferro-payments, ferro-queue, reaper, job, serde, generic]
dependency_graph:
  requires: [236-03]
  provides: [ReleaseExpiredPaymentIntents, ReconcileRefundsInFlight, ferro-queue dep]
  affects: [ferro-payments/src/reaper.rs, ferro-payments/src/lib.rs, ferro-payments/Cargo.toml]
tech_stack:
  added: [ferro-queue (path dep)]
  patterns: [serde-skipped Arc handle, #[serde(bound = "")], manual Debug impl, ferro_queue::Job]
key_files:
  created: [ferro-payments/src/reaper.rs]
  modified: [ferro-payments/src/lib.rs, ferro-payments/Cargo.toml]
decisions:
  - Manual Debug impl on both job structs (PaymentService<L> is not Debug; derive propagates the bound)
  - #[derive(Clone)] only (no Debug derive); Arc<T>: Clone is always satisfied regardless of T
  - #[tokio::test] for all async test cases (tokio already in dev-dependencies; avoids futures crate)
metrics:
  duration: "~12 minutes"
  completed: "2026-06-21"
  tasks: 2
  files: 3
requirements: [PAY-POLY-REAP-03]
---

# Phase 236 Plan 04: Reaper Queue Job Structs Summary

One-liner: Two `ferro_queue::Job` structs wrapping `PaymentService` reaper methods via serde-skipped `Arc` handle injection, following the `ProcessStripeWebhook` template exactly.

## What Was Built

`ferro-payments/src/reaper.rs` exposes two queue-compatible job structs:

- `ReleaseExpiredPaymentIntents<L>` — wraps `PaymentService::release_expired()`
- `ReconcileRefundsInFlight<L>` — wraps `PaymentService::reconcile_refunds_in_flight()`

Both follow the `ferro_stripe::ProcessStripeWebhook` template exactly:
- `#[derive(Clone, serde::Serialize, serde::Deserialize)]` + `#[serde(bound = "")]`
- Single `#[serde(skip)] pub service: Option<Arc<PaymentService<L>>>` field
- `::new(service: Arc<PaymentService<L>>)` constructor sets `service: Some(service)`
- `Job::handle()` errors with `ferro_queue::Error::JobFailed` when `service` is `None`
- `Job::handle()` calls the matching reaper method and maps `PaymentError` to `JobFailed`
- `Job::name()` returns a fixed `&'static str`

Both job structs are re-exported from `ferro-payments/src/lib.rs`:
```rust
pub use reaper::{ReconcileRefundsInFlight, ReleaseExpiredPaymentIntents};
```

`ferro-payments/Cargo.toml` now depends on `ferro-queue = { path = "../ferro-queue", version = "0.2" }`.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 3a2140ea | chore(236-04): add ferro-queue dependency to ferro-payments |
| 2 | ea322c1c | feat(236-04): add ReleaseExpiredPaymentIntents + ReconcileRefundsInFlight job structs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] PaymentService<L> not Debug — manual impl required**
- **Found during:** Task 2 (RED phase compile errors)
- **Issue:** `#[derive(Debug)]` on the job structs propagates the bound `PaymentService<L>: Debug`. `PaymentService<L>` contains an `Arc<dyn Fn(...)>` field which is not `Debug`.
- **Fix:** Removed `Debug` from the derive; implemented `std::fmt::Debug` manually on both structs, showing `"<injected>"` when service is present or `None`.
- **Files modified:** `ferro-payments/src/reaper.rs`

**2. [Rule 3 - Blocking] `futures` crate not in dev-dependencies**
- **Found during:** Task 2 (RED phase compile errors)
- **Issue:** Test helpers used `futures::executor::block_on` for sync wrappers around async code; `futures` not in `[dev-dependencies]`.
- **Fix:** Replaced all sync wrappers with `#[tokio::test]` — tokio is already in dev-dependencies and is the correct runtime for this codebase.
- **Files modified:** `ferro-payments/src/reaper.rs` (test section)

## Verification

All acceptance criteria met:

- `grep` confirms `impl<L: BillableLoader + 'static> ferro_queue::Job for ReleaseExpiredPaymentIntents<L>` in reaper.rs
- `grep` confirms `impl<L: BillableLoader + 'static> ferro_queue::Job for ReconcileRefundsInFlight<L>` in reaper.rs
- `grep` confirms `#[serde(bound = "")]` present twice in reaper.rs
- `grep` confirms `pub use reaper::` in lib.rs
- `grep` confirms `mod reaper;` in lib.rs
- `cargo test -p ferro-payments reaper` — 12 tests pass
- `cargo test -p ferro-payments job_no_service_injected` — 2 tests pass
- `cargo clippy -p ferro-payments --all-targets -- -D warnings` — clean

Note: The plan acceptance criteria grep patterns `release_expired().await` and `reconcile_refunds_in_flight().await` look for a single-line form. The implementation uses idiomatic chained style across two lines (`svc.release_expired()\n    .await`). The calls are semantically identical.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries introduced. The job structs add no serialized fields, satisfying T-236-06 (no forged payload can drive the money path). T-236-05 (clean error when handle missing) is covered by `job_no_service_injected` and `job_no_service_injected_release` tests.

## Self-Check: PASSED

Files exist:
- `ferro-payments/src/reaper.rs` — FOUND
- `ferro-payments/src/lib.rs` — FOUND (modified)
- `ferro-payments/Cargo.toml` — FOUND (modified)

Commits exist:
- `3a2140ea` — FOUND (chore: ferro-queue dep)
- `ea322c1c` — FOUND (feat: reaper job structs)

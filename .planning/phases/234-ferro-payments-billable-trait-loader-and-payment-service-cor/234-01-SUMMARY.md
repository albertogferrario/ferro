---
phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor
plan: "01"
subsystem: ferro-payments
tags: [payments, ferro-stripe, error-types, publish-pipeline]
dependency_graph:
  requires: [233-03]
  provides: [ferro-stripe-dep-in-ferro-payments, PaymentError-full-variant-set, publish-wave-1c]
  affects: [.github/workflows/publish.yml, ferro-payments/src/error.rs, ferro-payments/Cargo.toml]
tech_stack:
  added: [ferro-stripe as dependency of ferro-payments]
  patterns: [thiserror #[from] conversion, publish wave ordering]
key_files:
  created: []
  modified:
    - ferro-payments/Cargo.toml
    - ferro-payments/src/error.rs
    - .github/workflows/publish.yml
decisions:
  - "D-18: Extend PaymentError with Stripe(#[from]), Loader (no #[from], consumer wraps manually), AutoRefundTriggered"
  - "D-19: Pin ferro-stripe = { path = \"../ferro-stripe\", version = \"0.9\" }"
  - "D-21: ferro-payments moves to Wave 1c (new step) so it publishes after ferro-stripe is crates.io-indexed"
metrics:
  duration: "~7 minutes"
  completed: "2026-06-17T03:24:35Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 234 Plan 01: Dependency Wiring + Error Model + Publish Wave Fix Summary

**One-liner:** ferro-stripe dependency added to ferro-payments, PaymentError extended to its full 6-variant set with AutoRefundReason, and publish.yml Wave 1b intra-wave ordering violation resolved by a new Wave 1c step.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add ferro-stripe dependency to ferro-payments | 4a84fd81 | ferro-payments/Cargo.toml |
| 2 | Extend PaymentError + AutoRefundReason | 0a658710 | ferro-payments/src/error.rs |
| 3 | Move ferro-payments to Wave 1c in publish.yml | 3843ea44 | .github/workflows/publish.yml |

## What Was Built

### Task 1 — ferro-stripe dependency
Added `ferro-stripe = { path = "../ferro-stripe", version = "0.9" }` to `ferro-payments/Cargo.toml` under `[dependencies]`, after the existing `ferro-orm` line. No `test-helpers` feature added — the `MockStripeGateway` will live in `#[cfg(test)]` (D-20). `cargo tree -p ferro-payments` confirms `ferro-stripe v0.9.0` resolves.

### Task 2 — Extended PaymentError + AutoRefundReason
Extended the 3-variant `PaymentError` enum in `ferro-payments/src/error.rs` with three new variants:
- `Stripe(#[from] ferro_stripe::Error)` — automatic `From` conversion for Stripe API failures
- `Loader(Box<dyn std::error::Error + Send + Sync>)` — **no `#[from]`** (Pitfall 4: would conflict with `Stripe`'s `#[from]`); consumers construct via `PaymentError::Loader(Box::new(err))`
- `AutoRefundTriggered { reason: AutoRefundReason }` — defined now (D-18), returned only by phase-235 webhook handlers

Added `AutoRefundReason` enum after `PaymentError` with three variants: `LoaderError`, `BillableVanished`, `SideStateConflict`.

`cargo build -p ferro-payments` and `cargo clippy -p ferro-payments --all-targets -- -D warnings` both exit 0.

### Task 3 — Wave 1c publish step
Removed `ferro-payments` from `WAVE1B_CRATES` (was in the same loop as `ferro-stripe`, creating an unordered intra-wave dependency — DISCREPANCY-3 / D-21). Inserted two new steps immediately after the Wave 1b index-wait and before Wave 2:
- `Publish Wave 1c (depends on Wave 1b only)` — loop over `WAVE1C_CRATES="ferro-payments"` with same pattern as Wave 1b
- `Wait for crates.io index update (Wave 1c)` — `sleep 30`

Wave ordering verified: 1b index-wait (line 266) → Wave 1c publish (line 271) → Wave 1c index-wait (line 291) → Wave 2 (line 296).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

`AutoRefundTriggered` is defined in `error.rs` but not yet returned anywhere in the codebase. This is intentional per D-18: it is defined in phase 234 for type stability; webhook handlers in phase 235 will return it. Not a stub — a deliberate forward declaration.

## Threat Flags

None. The two threat model items from the plan are satisfied:
- T-234-01 (publish ordering): mitigated by Wave 1c index-wait.
- T-234-02 (PaymentError Display): no secrets or PII in error strings — accepted.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-payments/Cargo.toml | FOUND |
| ferro-payments/src/error.rs | FOUND |
| .github/workflows/publish.yml | FOUND |
| 234-01-SUMMARY.md | FOUND |
| Commit 4a84fd81 (Task 1) | FOUND |
| Commit 0a658710 (Task 2) | FOUND |
| Commit 3843ea44 (Task 3) | FOUND |

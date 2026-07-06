---
phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor
plan: "03"
subsystem: ferro-payments
tags: [payments, stripe, service, testing, seam, polymorphic]
dependency_graph:
  requires: ["234-01", "234-02"]
  provides: ["PaymentService", "StripeGateway", "StripeClientGateway", "CheckoutRequest", "CheckoutResponse", "ReturnUrls", "CheckoutUrl"]
  affects: ["ferro-payments/src/service.rs", "ferro-payments/src/lib.rs"]
tech_stack:
  added: []
  patterns: ["StripeGateway seam for testability (D-02)", "GuardedUpdate WHERE IS NULL for atomic dedup (T-234-06)", "MockStripeGateway with call recording (#[cfg(test)])"]
key_files:
  created:
    - ferro-payments/src/service.rs
  modified:
    - ferro-payments/src/lib.rs
decisions:
  - "StripeGateway returns CheckoutResponse{intent, application_fee_cents} so PaymentService never calls Stripe::config() — fee computed only inside StripeClientGateway (D-02/03, Open Question 1 resolution)"
  - "return_url_builder stored as Arc<dyn Fn(...)> with #[allow(clippy::type_complexity)] on the field"
  - "seed_paid_with_charge takes billable_id parameter to avoid partial unique index collisions across test cases"
metrics:
  duration_seconds: 1399
  completed_date: "2026-06-17"
  tasks_completed: 3
  files_modified: 2
requirements: [PAY-POLY-SVC-03, PAY-POLY-SVC-04]
---

# Phase 234 Plan 03: PaymentService orchestrator + StripeGateway seam Summary

**One-liner:** `PaymentService<L>` with `StripeGateway` seam enables fully mock-tested polymorphic checkout and refund orchestration without `Stripe::init`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | StripeGateway seam, types, PaymentService struct | 7b083b1e | ferro-payments/src/service.rs, ferro-payments/src/lib.rs |
| 2 | start_checkout + request_refund + unit tests | 26954b49 | ferro-payments/src/service.rs |
| 3 | lib.rs re-exports + full pre-commit gate | f2a81387, 4fe4f702 | ferro-payments/src/service.rs, ferro-payments/src/lib.rs, Cargo.lock |

## What Was Built

### StripeGateway seam (D-02/D-03)

`StripeGateway` trait in `service.rs` with two methods:
- `create_checkout_session(req: CheckoutRequest) -> Result<CheckoutResponse, ferro_stripe::Error>`
- `create_refund(charge_id, amount_cents, idempotency_key) -> Result<(), ferro_stripe::Error>`

`CheckoutResponse { intent: ferro_stripe::CheckoutIntent, application_fee_cents: Option<i64> }` wraps the Stripe-minted intent and returns the fee the production gateway computed — `PaymentService` never calls `Stripe::config()`.

### Production gateway (StripeClientGateway)

Wraps `ferro_stripe::CheckoutBuilder` + `ferro_stripe::refund::create`. Fee computation via `Stripe::config().application_fee_for(amount_cents)` is confined here — the only location in the crate that touches the Stripe global static.

### PaymentService<L: BillableLoader>

Two methods:
- `start_checkout(billable, ttl)`: `create_reserved` → build `CheckoutRequest` → `self.stripe.create_checkout_session` → `lifecycle::attach_session` → `Ok(CheckoutUrl)`
- `request_refund(intent_id, amount_cents)`: load by id → require `status=paid + charge_id present` → `GuardedUpdate WHERE refund_amount_cents IS NULL` → `self.stripe.create_refund`

`loader: L` field carries `#[allow(dead_code)] // wired by handle_* in phase 235`.

### Unit tests (6 named, PAY-POLY-SVC-03/04)

`MockStripeGateway` with `Mutex<Vec<CheckoutRequest>>` and `Mutex<Vec<(String, Option<i64>)>>` call logs. All test names match VALIDATION.md filters:

| Test | Requirement | Result |
|------|-------------|--------|
| `start_checkout` | PAY-POLY-SVC-03a | green |
| `start_checkout_no_connect` | PAY-POLY-SVC-03b | green |
| `request_refund` | PAY-POLY-SVC-03c | green |
| `request_refund_precondition` | PAY-POLY-SVC-03d | green |
| `request_refund_dedup` | PAY-POLY-SVC-03e | green |
| `mock_gateway_records_calls` | PAY-POLY-SVC-04 | green |

### lib.rs re-exports

Added: `AutoRefundReason`, `attach_session`, `Billable`, `BillableLoader`, `CheckoutRequest`, `CheckoutResponse`, `CheckoutUrl`, `PaymentService`, `ReturnUrls`, `StripeClientGateway`, `StripeGateway`.

## Verification Results

- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0
- `cargo test -p ferro-payments`: 23 passed, 0 failed
- `cargo test --all-features`: exit 0 (all suites green)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Partial unique index collision in request_refund_precondition test**
- **Found during:** Task 2 test run
- **Issue:** `seed_paid_with_charge` used a hardcoded `billable_id = 99`; the precondition test seeds two rows for `(booking, 99)` both with `status = 'paid'`, hitting the partial unique index `WHERE status IN ('reserved','paid')`
- **Fix:** Added `billable_id: i64` parameter to `seed_paid_with_charge`; each call site passes a distinct value (101, 201, 202, 301)
- **Files modified:** ferro-payments/src/service.rs
- **Commit:** 26954b49

**2. [Rule 2 - Clippy] #[allow(clippy::type_complexity)] on return_url_builder field**
- **Found during:** Task 3 clippy run
- **Issue:** `Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>` triggered `clippy::type_complexity` under `-D warnings`
- **Fix:** Added `#[allow(clippy::type_complexity)]` on the `return_url_builder` field (not on the struct)
- **Files modified:** ferro-payments/src/service.rs
- **Commit:** f2a81387

## Security Notes

Threat mitigations from the plan's threat register:

- **T-234-06 (double-refund):** `GuardedUpdate WHERE refund_amount_cents IS NULL` is the single-statement dedup; `request_refund_dedup` asserts Stripe is called exactly once across two calls. HIGH severity guard in place and tested.
- **T-234-07 (fee integrity):** Fee computed inside `StripeClientGateway` from `Stripe::config().application_fee_for(amount_cents)`; `PaymentService` snapshots only the gateway-returned value, never a caller-supplied value.
- **T-234-08 (cross-tenant):** Documented as caller responsibility per D-08; not enforced in-crate.
- **T-234-09 (reserved row on failure):** Reserved row persists on gateway failure per D-14; phase-236 reaper is the cleanup path.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes beyond what the plan's threat model covers.

## Self-Check: PASSED

Files exist:
- ferro-payments/src/service.rs: FOUND
- ferro-payments/src/lib.rs: FOUND

Commits exist:
- 7b083b1e: FOUND (StripeGateway seam + PaymentService struct)
- 26954b49: FOUND (start_checkout + request_refund + unit tests)
- f2a81387: FOUND (fmt + clippy fixes)
- 4fe4f702: FOUND (Cargo.lock)

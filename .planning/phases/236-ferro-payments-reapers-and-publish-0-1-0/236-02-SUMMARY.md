---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: "02"
subsystem: ferro-payments / ferro-stripe
tags: [payments, stripe, refund, polling, gateway, tdd]
dependency_graph:
  requires: [236-01]
  provides: [list_for_payment_intent, RefundStatus, fetch_refund_status_for_payment_intent, MockStripeGateway-poll]
  affects: [ferro-stripe/src/refund.rs, ferro-payments/src/service.rs, ferro-payments/src/webhook.rs]
tech_stack:
  added: []
  patterns: [TDD-RED-GREEN, parse-before-client, canned-mock-result]
key_files:
  created: []
  modified:
    - ferro-stripe/src/refund.rs
    - ferro-payments/src/service.rs
    - ferro-payments/src/webhook.rs
decisions:
  - "parse pi_id before Stripe::client() call so invalid ids fail without requiring Stripe::init() (T-236-03)"
  - "empty refund list maps to RefundStatus::Pending — a missing record never becomes Succeeded (T-236-03c)"
  - "webhook.rs test MockStripeGateway gets a stub returning Pending (trait completeness, not behavioral)"
  - "failure_reason field exists on stripe::Refund 0.41 — used directly in RefundStatus::Failed"
metrics:
  duration_seconds: 384
  completed_date: "2026-06-20"
  tasks_completed: 2
  files_modified: 3
---

# Phase 236 Plan 02: Stripe Poll Primitive + RefundStatus Gateway Seam Summary

Read-only Stripe refund poll (`list_for_payment_intent`) in ferro-stripe, exposed through the `StripeGateway` seam as `fetch_refund_status_for_payment_intent` returning `RefundStatus`, with a programmable `MockStripeGateway` extension so the Plan 03 reconcile reaper is offline-testable.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Add list_for_payment_intent poll primitive to ferro-stripe | 8b094e42 | ferro-stripe/src/refund.rs |
| 2 | Add RefundStatus + gateway poll method + impls | 0ef53abd | ferro-payments/src/service.rs, ferro-payments/src/webhook.rs |

## Key Artifacts

### ferro-stripe/src/refund.rs — `list_for_payment_intent`

```rust
pub async fn list_for_payment_intent(
    payment_intent_id: &str,
) -> Result<Vec<stripe::Refund>, Error>
```

- Parses `pi_id` before calling `Stripe::client()` (T-236-03: invalid id fails fast without requiring Stripe::init)
- Uses `stripe::ListRefunds::new()` + `Refund::list(client, &params)` (async-stripe 0.41, not feature-gated)
- Returns `list.data` (Vec, limit=10)
- Unit test: `list_for_payment_intent("not-a-pi")` → `Err(Error::Stripe(_))` without network

### ferro-payments/src/service.rs — `RefundStatus` + trait + impls

```rust
pub enum RefundStatus {
    Succeeded { amount_cents: i64 },
    Pending,
    Failed { reason: Option<String> },
}
```

- `StripeGateway` trait: 4th method `fetch_refund_status_for_payment_intent`
- `StripeClientGateway` impl: calls `ferro_stripe::refund::list_for_payment_intent`, maps `.status.as_deref()` to `RefundStatus`; empty list → `Pending`
- `MockStripeGateway`: `poll_calls: Mutex<Vec<String>>` + `canned_refund_status` + `set_refund_status()`
- Default mock return: `RefundStatus::Succeeded { amount_cents: 1000 }`

### ferro-payments/src/webhook.rs — stub impl

Webhook test `MockStripeGateway` gets a stub `fetch_refund_status_for_payment_intent` returning `Ok(RefundStatus::Pending)` (trait completeness only — webhook tests don't exercise polling).

## Verification

- `cargo test -p ferro-stripe list_for_payment_intent` → 1 passed
- `cargo test -p ferro-payments mock_poll` → 3 passed
- `cargo test -p ferro-payments` → 48 passed, 0 failed
- `cargo clippy -p ferro-stripe --all-targets -- -D warnings` → clean
- `cargo clippy -p ferro-payments --all-targets -- -D warnings` → clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Reordered parse before Stripe::client() call**
- **Found during:** Task 1 GREEN phase
- **Issue:** The analog `create_for_payment_intent` calls `Stripe::client()` before parsing the id. Copying this order causes the test (`list_for_payment_intent("not-a-pi")`) to panic with "Stripe::init() not called" before the parse error is reached.
- **Fix:** Moved `pi_id` parse to the top of `list_for_payment_intent`, before `Stripe::client()`. This also makes T-236-03 stricter: invalid ids never touch the network layer.
- **Files modified:** ferro-stripe/src/refund.rs
- **Commit:** 8b094e42

**2. [Rule 2 - Missing] webhook.rs test MockStripeGateway needed stub for new trait method**
- **Found during:** Task 2 GREEN phase
- **Issue:** `webhook.rs` contains its own `MockStripeGateway` that also implements `StripeGateway`. Adding a 4th method to the trait without updating this mock breaks compilation.
- **Fix:** Added stub `fetch_refund_status_for_payment_intent` returning `Ok(RefundStatus::Pending)` to the webhook.rs test mock.
- **Files modified:** ferro-payments/src/webhook.rs
- **Commit:** 0ef53abd

## Known Stubs

None. Both implementations are production-ready:
- `list_for_payment_intent`: real async-stripe 0.41 call
- `fetch_refund_status_for_payment_intent` on `StripeClientGateway`: real status mapping

## Threat Flags

No new network endpoints or auth paths introduced beyond what was planned. The `list_for_payment_intent` read-only Stripe call was explicitly in the threat model (T-236-03, T-236-03b, T-236-03c).

## Self-Check: PASSED

- `ferro-stripe/src/refund.rs` contains `pub async fn list_for_payment_intent` ✓
- `ferro-payments/src/service.rs` contains `pub enum RefundStatus` ✓
- `ferro-payments/src/service.rs` contains `fn fetch_refund_status_for_payment_intent` ✓
- commit 8b094e42 exists ✓
- commit 0ef53abd exists ✓

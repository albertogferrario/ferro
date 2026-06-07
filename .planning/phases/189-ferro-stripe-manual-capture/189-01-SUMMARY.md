---
phase: 189-ferro-stripe-manual-capture
plan: "01"
subsystem: ferro-stripe
tags: [stripe, payments, checkout, manual-capture]
dependency_graph:
  requires: []
  provides: [ManualCaptureRequiresPaymentMode error variant, CheckoutBuilder::manual_capture() setter, merged payment_intent_data construction]
  affects: [ferro-stripe/src/error.rs, ferro-stripe/src/checkout.rs]
tech_stack:
  added: []
  patterns: [runtime pre-flight guard before Stripe::client(), single-assignment merged payment_intent_data, private build helper for unit-testable merge logic]
key_files:
  created: []
  modified:
    - ferro-stripe/src/error.rs
    - ferro-stripe/src/checkout.rs
decisions:
  - "build_payment_intent_data() private helper extracted to enable unit testing of merged construction without live Stripe client"
  - "Partial-move fix: replaced ok_or() on Option<String> with is_none() check + clone().unwrap() to allow subsequent borrow of self in build_payment_intent_data()"
metrics:
  duration: "~4 minutes"
  completed: "2026-06-07T15:22:35Z"
  tasks_completed: 2
  files_modified: 2
requirements: [STRIPE-MC-01, STRIPE-MC-04]
---

# Phase 189 Plan 01: Manual Capture Builder Flag Summary

One-line summary: CheckoutBuilder gains `manual_capture()` that sets `capture_method=manual` in payment mode and composes with `destination()` via a single merged `payment_intent_data` construction.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add ManualCaptureRequiresPaymentMode error variant | be36bab0 | ferro-stripe/src/error.rs |
| 2 | Add manual_capture builder field, setter, guard, and merged payment_intent_data | 6e96afae | ferro-stripe/src/checkout.rs |

## What Was Built

### Task 1 — Error variant (be36bab0)

Added `ManualCaptureRequiresPaymentMode` unit variant to `ferro-stripe/src/error.rs` immediately after `MissingIdempotencyKey`. Same shape: doc comment explaining the invariant, `#[error]` message with exact user-facing text `"manual capture requires payment mode; use Mode::Payment with manual_capture()"`.

### Task 2 — Builder extension (6e96afae)

Four coordinated edits to `ferro-stripe/src/checkout.rs`:

1. **Import** — added `CreateCheckoutSessionPaymentIntentDataCaptureMethod` to the existing `use stripe::{...}` block.

2. **Struct + init** — `manual_capture: bool` field added to `CheckoutBuilder`; zero-initialized to `false` in `new()`.

3. **Setter** — no-args consuming `pub fn manual_capture(mut self) -> Self` following the existing `mut self -> Self` convention.

4. **Guard + merged construction** — `create()` now:
   - Checks idempotency key presence via `is_none()` (not `ok_or()`) to avoid partial-move that would block subsequent `&self` borrow.
   - Fires `ManualCaptureRequiresPaymentMode` guard for `manual_capture && mode == Subscription`, before `Stripe::client()`.
   - Delegates to `build_payment_intent_data()` — a new private `&self` helper that computes the merged `CreateCheckoutSessionPaymentIntentData`. This approach keeps `create()` clean and makes the merge logic unit-testable without a live client.
   - Single `params.payment_intent_data = self.build_payment_intent_data()` assignment (replaces the old single-branch destination block — verified by grep count = 1).

5. **Tests** — 3 new tests added:
   - `checkout_create_manual_capture_subscription_returns_err` — guard test (async).
   - `checkout_create_manual_capture_sets_capture_method` — capture-only, asserts `capture_method=Manual` and `transfer_data.is_none()` (sync, calls `build_payment_intent_data()` directly).
   - `checkout_create_manual_capture_with_destination_sets_both_fields` — D-08 composition test, asserts `capture_method=Manual` AND `transfer_data.is_some()` AND `on_behalf_of` AND `application_fee_amount`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Partial-move compiler error in create()**
- **Found during:** Task 2, first compile attempt
- **Issue:** The original `let idempotency_key = self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?` call moved `self.idempotency_key` out of `self`. After that partial move, calling `self.build_payment_intent_data()` (which takes `&self`) produced `E0382: borrow of partially moved value`.
- **Fix:** Replaced `ok_or()` with `is_none()` guard + `clone().unwrap()` pattern. Guard fires first, then the clone is infallible. Subsequent `&self` borrow in `build_payment_intent_data()` compiles cleanly.
- **Files modified:** ferro-stripe/src/checkout.rs
- **Commit:** 6e96afae (included in task commit — single fix, single change)

## Threat Mitigations Applied

Per plan threat register:
- **T-189-01** (Tampering — manual_capture + Subscription): Pre-flight guard in `create()` returns `Err(ManualCaptureRequiresPaymentMode)` before any network call. Guard ordering verified: `is_none()` check → mode guard → `Stripe::client()`.
- **T-189-02** (Tampering — double-overwrite): Single-assignment merge via `build_payment_intent_data()` ensures `capture_method` and `transfer_data` cannot silently overwrite each other. D-08 test asserts both fields coexist.
- **T-189-03** (Information Disclosure — error message): No secrets in error text. Accepted per threat register.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced.

## Known Stubs

None — all functionality is fully wired. `build_payment_intent_data()` is exercised by both the production `create()` path and the unit tests.

## Self-Check

PASSED
- `ferro-stripe/src/error.rs` contains `ManualCaptureRequiresPaymentMode` variant — confirmed.
- `ferro-stripe/src/checkout.rs` contains `manual_capture: bool`, `manual_capture: false`, `pub fn manual_capture(mut self) -> Self`, `CreateCheckoutSessionPaymentIntentDataCaptureMethod`, `self.manual_capture && self.mode == Mode::Subscription`, `needs_payment_intent_data`, and all three test functions — confirmed.
- Commit `be36bab0` exists — confirmed.
- Commit `6e96afae` exists — confirmed.
- `cargo test -p ferro-stripe checkout`: 7 passed, 0 failed — confirmed.
- `grep -c "params.payment_intent_data = " ferro-stripe/src/checkout.rs` == 1 — confirmed.

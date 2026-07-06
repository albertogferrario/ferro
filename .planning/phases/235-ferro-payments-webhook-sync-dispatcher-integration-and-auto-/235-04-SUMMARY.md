---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
plan: "04"
subsystem: ferro-payments
tags: [payments, stripe, gateway, idempotency, refund, wr-03]
dependency_graph:
  requires: [ferro_stripe::refund::create_for_payment_intent (plan 02)]
  provides:
    - StripeGateway::create_refund_for_payment_intent (trait + prod + mock)
    - PaymentService::processed_log field
    - PaymentService::new() with processed_log param
    - WR-03 amount_cents guard in start_checkout
  affects: [ferro-payments Wave 3 webhook handlers (plan 05)]
tech_stack:
  added: []
  patterns:
    - mirror-existing-gateway-method
    - mock-call-recorder
    - early-return-guard
    - cascade-constructor-update
key_files:
  created: []
  modified:
    - ferro-payments/src/service.rs
decisions:
  - "processed_log inserted after loader in new() param order (matches 235-PATTERNS.md verbatim)"
  - "Both processed_log and loader carry #[allow(dead_code)] until plan 05 wires handle_* methods"
  - "Added #[derive(Debug)] to CheckoutUrl (required by expect_err in WR-03 test)"
  - "pi_refund_calls accessor carries #[allow(dead_code)] — used by plan 05 tests"
metrics:
  duration_seconds: 227
  completed_date: "2026-06-17"
  tasks_completed: 2
  files_modified: 1
---

# Phase 235 Plan 04: StripeGateway pi-refund seam + PaymentService processed_log + WR-03 guard Summary

**One-liner:** Extended `StripeGateway` with `create_refund_for_payment_intent` (prod delegates to `ferro_stripe::refund::create_for_payment_intent`, mock records calls in `pi_refund_calls`), added `processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>` to `PaymentService` with reshaped `new()`, and enforced the WR-03 `amount_cents <= 0` guard in `start_checkout`.

## What Was Built

### Task 1: StripeGateway pi-refund method + WR-03 guard

- Added `async fn create_refund_for_payment_intent(&self, payment_intent_id, amount_cents, idempotency_key)` to the `StripeGateway` trait.
- Implemented on `StripeClientGateway`: delegates to `ferro_stripe::refund::create_for_payment_intent(...).await?; Ok(())` — identical structure to `create_refund`.
- Extended `MockStripeGateway` with:
  - `pi_refund_calls: Mutex<Vec<(String, Option<i64>)>>` field
  - `canned_pi_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>` field
  - `create_refund_for_payment_intent` impl that pushes to `pi_refund_calls` and returns `canned_pi_refund`
  - `pi_refund_calls()` accessor (for Wave 3 tests to assert call count/args)
- Added WR-03 guard at top of `start_checkout`: rejects `amount_cents() <= 0` with `PaymentError::StatusPrecondition("amount_cents must be positive to start checkout")` before any DB write or Stripe call.
- Added `#[derive(Debug)]` to `CheckoutUrl` (needed for `expect_err` in test assertions).
- Added test `start_checkout_rejects_nonpositive_amount`: confirms `StatusPrecondition` returned, zero DB rows inserted, zero Stripe calls made.

### Task 2: processed_log field + new() reshape + cascade

- Added `processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>` field to `PaymentService<L>`, after `stripe`, with `#[allow(dead_code)]` (wired by `handle_*` in plan 05).
- Reshaped `PaymentService::new()` to accept `processed_log` after `loader` (per 235-PATTERNS.md).
- Updated all 7 `PaymentService::new(...)` call sites in the `#[cfg(test)]` module to pass `Arc::new(ferro_stripe::MemoryProcessedLog::new())` in the matching position.

## Verification

- `cargo test -p ferro-payments`: 27 passed, 0 failed
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: exit 0

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend StripeGateway + MockStripeGateway + WR-03 guard | f6a98a9b | ferro-payments/src/service.rs |
| 2 | Add processed_log to PaymentService; reshape new(); cascade call sites | c703ab90 | ferro-payments/src/service.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] CheckoutUrl missing Debug derive**
- **Found during:** Task 1 (compiling the WR-03 test)
- **Issue:** `expect_err()` requires `T: Debug`; `CheckoutUrl(pub String)` had no `Debug` derive.
- **Fix:** Added `#[derive(Debug)]` to `CheckoutUrl`.
- **Files modified:** ferro-payments/src/service.rs
- **Commit:** f6a98a9b

**2. [Rule 2 - Missing critical functionality] #[allow(dead_code)] on processed_log field**
- **Found during:** Task 2 (clippy gate)
- **Issue:** `processed_log` field unused until plan 05 wires `handle_*` methods; clippy `-D warnings` flags dead code.
- **Fix:** Added `#[allow(dead_code)]` with comment `// read by handle_* in webhook.rs (plan 05)` — same approach as `loader` field, per plan guidance.
- **Files modified:** ferro-payments/src/service.rs
- **Commit:** c703ab90

## Known Stubs

None. All new code either delegates to concrete implementations (`StripeClientGateway`) or records calls for test assertion (`MockStripeGateway`). The `processed_log` and `loader` fields are stored but not yet read — their consumers are plan 05 (`handle_*` webhook handlers). The `#[allow(dead_code)]` annotations are the explicit tracking mechanism.

## Threat Flags

None. All new surface matches the plan's threat model:
- T-235-06 (Tampering — WR-03 guard): mitigated — `amount_cents <= 0` returns `StatusPrecondition` before any DB write.
- T-235-07 (Repudiation replay — processed_log): mitigated by storing the field; Wave 3 handlers will call `try_mark_processed`.
- T-235-08 (Information disclosure — pi_refund_calls): accepted — test-only behind `#[cfg(test)]`.

## Self-Check: PASSED

- ferro-payments/src/service.rs — FOUND (modified)
- Commit f6a98a9b — FOUND
- Commit c703ab90 — FOUND

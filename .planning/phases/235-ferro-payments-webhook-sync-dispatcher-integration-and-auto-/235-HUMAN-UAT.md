---
status: partial
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
source: [235-VERIFICATION.md]
started: 2026-06-17
updated: 2026-06-17
---

## Current Test

[awaiting human review — non-blocking sign-offs; phase goal verified 12/12 automated]

## Tests

### 1. is_transient error classification — business sign-off
expected: In the webhook handlers, `is_transient(e)` treats `PaymentError::Db | PaymentError::Stripe` as transient (propagate → Stripe retries the webhook) and all other variants (`StatusPrecondition`, `Loader`, `NotFound`, `AutoRefundTriggered`) as terminal (absorb → return Ok, or trigger auto-refund for the capture-not-honorable cases). This is the money-safety boundary introduced by code-review fix WR-02/WR-03: a transient DB error inside a consumer's `on_paid` must NOT cause an irreversible refund. Confirm the classification matches your intended policy for the real consumer (gestiscilo) callbacks.
result: [pending]

### 2. Live Stripe refund-by-payment_intent (`create_refund_for_payment_intent`)
expected: The auto-refund path calls Stripe's refund API with `payment_intent` (not `charge`), because `checkout.session.completed` carries no charge_id. Unit-tested offline via MockStripeGateway; live exercise is deferred to the phase-236 workspace integration bin against ferro-stripe test mode.
result: [pending — phase 236]

### 3. Stuck-refund recovery via ReconcileRefundsInFlight reaper
expected: If a refund's Stripe call fails AFTER the `refund_amount_cents` snapshot, the row stays refund-in-flight (no compensate-reset, to avoid double-refund given async-stripe 0.41 doesn't forward idempotency keys). Recovery is the phase-236 `ReconcileRefundsInFlight` reaper. Verify once 236 ships.
result: [pending — phase 236]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps

None blocking — all 3 are deferred sign-offs (2 are phase-236 scope; 1 is a business-logic confirmation of an already-implemented, tested classification). Phase 235 automated verification is 12/12.

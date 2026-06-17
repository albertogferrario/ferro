---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
plan: "02"
subsystem: ferro-stripe
tags: [payments, refund, stripe, ferro-stripe]
dependency_graph:
  requires: []
  provides: [ferro_stripe::refund::create_for_payment_intent]
  affects: [ferro-payments Wave 3 auto-refund path]
tech_stack:
  added: []
  patterns: [mirror-existing-function, id-parse-error-mapping]
key_files:
  created: []
  modified:
    - ferro-stripe/src/refund.rs
decisions:
  - "Placed create_for_payment_intent immediately after create() — same file section, identical structure, only params.payment_intent substituted for params.charge"
  - "Offline unit test validates the Err path via parse failure without a network call"
metrics:
  duration_seconds: 150
  completed_date: "2026-06-17"
  tasks_completed: 1
  files_modified: 1
---

# Phase 235 Plan 02: refund::create_for_payment_intent Summary

**One-liner:** Payment-intent-based refund primitive in ferro-stripe (`create_for_payment_intent`) — verbatim mirror of charge-based `create`, enabling auto-refund on `checkout.session.completed` events that carry no `charge_id`.

## What Was Built

Added `pub async fn create_for_payment_intent(payment_intent_id, amount_cents, idempotency_key, reason)` to `ferro-stripe/src/refund.rs`. The function:

- Parses `payment_intent_id` into `stripe::PaymentIntentId` with graceful `Err(Error::Stripe("invalid payment intent id: …"))` on bad input — no panic (T-235-02 mitigation).
- Sets `params.payment_intent = Some(pi_id)` instead of `params.charge`.
- Carries the same async-stripe 0.41 idempotency-key-not-forwarded caveat as `create()`, documented in the doc comment (T-235-03 accepted-at-caller).
- Is reachable as `ferro_stripe::refund::create_for_payment_intent` via the existing `pub mod refund` in `ferro-stripe/src/lib.rs` — no extra re-export line needed.

Offline unit test `refund::tests::invalid_payment_intent_id_does_not_parse` asserts the parse guard fires without a network call.

## Verification

- `cargo check -p ferro-stripe` — exit 0
- `cargo test -p ferro-stripe invalid_payment_intent` — 1 passed, 0 failed
- `cargo clippy -p ferro-stripe --all-targets -- -D warnings` — exit 0

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add refund::create_for_payment_intent | c9f01ff8 | ferro-stripe/src/refund.rs |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The function delegates to `stripe::Refund::create` — no stub data paths.

## Threat Flags

None. The new surface (outbound Stripe API call) is already covered by the plan's threat model (T-235-02 / T-235-03).

## Self-Check: PASSED

- ferro-stripe/src/refund.rs — FOUND (modified)
- Commit c9f01ff8 — FOUND

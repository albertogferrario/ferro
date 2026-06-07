---
phase: 189-ferro-stripe-manual-capture
plan: "02"
subsystem: ferro-stripe
tags: [stripe, payments, payment-intent, capture, cancel]
dependency_graph:
  requires: [189-01]
  provides: [payment_intent::capture, payment_intent::cancel, payment_intent::retrieve, pub mod payment_intent in lib.rs]
  affects: [ferro-stripe/src/payment_intent.rs, ferro-stripe/src/lib.rs]
tech_stack:
  added: []
  patterns: [capability-module free functions mirroring refund.rs, id-parse-first ordering for offline invalid-input tests, u64::try_from guard for negative amount rejection]
key_files:
  created:
    - ferro-stripe/src/payment_intent.rs
  modified:
    - ferro-stripe/src/lib.rs
decisions:
  - "retrieve() included for API parity with refund.rs (cheap, prevents follow-up patch)"
  - "u64::try_from(n) used instead of n as u64 to avoid cast_sign_loss clippy lint and provide a structured error for negative inputs"
  - "id parse placed BEFORE Stripe::client() in all three functions so invalid-id tests work without an initialized client"
  - "pub mod payment_intent unconditional (no cfg gate), no top-level re-export per D-04"
metrics:
  duration: "~3 minutes"
  completed: "2026-06-07T15:27:45Z"
  tasks_completed: 2
  files_modified: 2
requirements: [STRIPE-MC-02]
---

# Phase 189 Plan 02: payment_intent Capability Module Summary

One-line summary: New `ferro-stripe/src/payment_intent.rs` capability module exposes `capture(id, Option<i64>)`, `cancel(id)`, and `retrieve(id)` free functions mirroring `refund.rs`, with `pub mod payment_intent` registered unconditionally in `lib.rs`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create payment_intent.rs with capture, cancel, retrieve | a17e5657 | ferro-stripe/src/payment_intent.rs |
| 2 | Register payment_intent module in lib.rs | 42951a5c | ferro-stripe/src/lib.rs |

## What Was Built

### Task 1 — payment_intent.rs capability module (a17e5657)

New `ferro-stripe/src/payment_intent.rs` with three async free functions:

1. **`capture(payment_intent_id, amount_cents)`** — validates id via `PaymentIntentId::parse()` before calling `Stripe::client()`, then converts `Option<i64>` to `Option<u64>` via `u64::try_from` (returns `Err(Error::Stripe("amount_to_capture must be positive"))` for negative values), builds `CapturePaymentIntent { amount_to_capture, ..Default::default() }`, and calls `stripe::PaymentIntent::capture`.

2. **`cancel(payment_intent_id)`** — validates id parse before client, calls `stripe::PaymentIntent::cancel` with `CancelPaymentIntent::default()`.

3. **`retrieve(payment_intent_id)`** — validates id parse before client, calls `stripe::PaymentIntent::retrieve` with empty expand slice.

Module doc covers the semantic parallel with `ferro-reservation` hold/commit/release and the idempotency caveat (async-stripe 0.41 does not forward per-request keys to `PaymentIntent::capture`).

4 inline unit tests (all work without a live Stripe client — id parse failures happen before `Stripe::client()`):
- `capture_rejects_invalid_id_before_network` — string without `pi_` prefix
- `capture_rejects_negative_amount` — valid `pi_` id + `Some(-5)` triggers `u64::try_from` error
- `cancel_rejects_invalid_id_before_network` — string with spaces (fails id parse)
- `retrieve_rejects_invalid_id_before_network` — empty string (fails id parse)

### Task 2 — lib.rs module registration (42951a5c)

Single-line addition: `pub mod payment_intent;` placed alphabetically (between `idempotency` and `refund`), unconditional (no `#[cfg(` gate), no `pub use payment_intent::*` re-export per D-04. Consumers call `ferro_stripe::payment_intent::capture(...)` directly.

## Deviations from Plan

None — plan executed exactly as written. The file content matches the plan's `<action>` block verbatim.

## Threat Mitigations Applied

Per plan threat register:
- **T-189-04** (Tampering — negative amount): `u64::try_from(n)` rejects negative `i64` values with `Error::Stripe("amount_to_capture must be positive")` BEFORE any network call. Tested by `capture_rejects_negative_amount`.
- **T-189-05** (Tampering — malformed payment_intent_id): `PaymentIntentId::parse()` runs before `Stripe::client()` in all three functions. Malformed ids return structured `Err` without a network call. Tested by three invalid-id unit tests.
- **T-189-06** (Repudiation — double-capture on retry): Accepted and documented in module doc. async-stripe 0.41 does not forward per-request idempotency keys to `PaymentIntent::capture`; application-layer dedup is required.
- **T-189-07** (Elevation of Privilege — connected-account impersonation): capture/cancel are platform-scoped only per D-07. No `Stripe-Account` parameter exposed.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced. The module wraps existing Stripe API calls.

## Known Stubs

None — all three functions are fully implemented. Unit tests exercise the offline error paths; live capture/cancel/retrieve paths are owned by the gestiscilo consumer field test.

## Self-Check

PASSED
- `ferro-stripe/src/payment_intent.rs` exists — confirmed.
- `grep -q "pub async fn capture(" ferro-stripe/src/payment_intent.rs` — PASS.
- `grep -q "amount_cents: Option<i64>" ferro-stripe/src/payment_intent.rs` — PASS.
- `grep -q "pub async fn cancel(" ferro-stripe/src/payment_intent.rs` — PASS.
- `grep -q "pub async fn retrieve(" ferro-stripe/src/payment_intent.rs` — PASS.
- `grep -q "u64::try_from" ferro-stripe/src/payment_intent.rs` — PASS.
- `grep -vq "n as u64" ferro-stripe/src/payment_intent.rs` — PASS (no bare cast).
- `grep -c "invalid payment intent id:" ferro-stripe/src/payment_intent.rs` == 3 — PASS.
- `grep -q "pub mod payment_intent;" ferro-stripe/src/lib.rs` — PASS.
- No `pub use payment_intent` in lib.rs — PASS.
- `cargo test -p ferro-stripe payment_intent` — 4 passed, 0 failed.
- `cargo build -p ferro-stripe` — Finished successfully.
- Commit `a17e5657` exists — confirmed.
- Commit `42951a5c` exists — confirmed.

---
phase: 189-ferro-stripe-manual-capture
plan: "03"
subsystem: ferro-stripe
tags: [stripe, payments, webhook, events, manual-capture]
dependency_graph:
  requires: [189-02]
  provides: [StripePaymentIntentAmountCapturableUpdated, StripePaymentIntentCanceled, golden fixtures, parser-contract tests]
  affects: [ferro-stripe/src/webhook/events.rs, ferro-stripe/src/lib.rs, ferro-stripe/tests/parser_contract.rs, ferro-stripe/tests/fixtures/stripe_events/]
tech_stack:
  added: []
  patterns: [StripeEvent::from_raw type-guard pattern, EventObject::PaymentIntent arm, golden-JSON fixture + parser-contract test registration]
key_files:
  created:
    - ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json
    - ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json
  modified:
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/src/lib.rs
    - ferro-stripe/tests/parser_contract.rs
decisions:
  - "currency extracted via pi.currency.to_string() — Currency implements Display in async-stripe 0.41; as_str() fallback not needed"
  - "cancellation_reason extracted via pi.cancellation_reason.map(|r| r.as_str().to_string()) — Copy enum, no borrow needed"
  - "metadata cloned directly — verified HashMap<String,String> (not a wrapper type) in async-stripe 0.41"
metrics:
  duration: "~2.5 minutes"
  completed: "2026-06-07T15:32:25Z"
  tasks_completed: 2
  files_modified: 5
requirements: [STRIPE-MC-03]
---

# Phase 189 Plan 03: Typed Webhook Events for Manual Capture Lifecycle Summary

One-line summary: Two new `StripeEvent` implementations (`StripePaymentIntentAmountCapturableUpdated`, `StripePaymentIntentCanceled`) with verified async-stripe 0.41 field extraction, golden-JSON fixtures, and 4 parser-contract tests covering parse-all-fields and cross-type rejection.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add two event structs, StripeEvent impls, marker-test entries, and lib.rs re-export | aa3bc443 | ferro-stripe/src/webhook/events.rs, ferro-stripe/src/lib.rs |
| 2 | Add golden-JSON fixtures and parser-contract tests | 43ed7587 | ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json, ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json, ferro-stripe/tests/parser_contract.rs |

## What Was Built

### Task 1 — Event structs + StripeEvent impls (aa3bc443)

**`StripePaymentIntentAmountCapturableUpdated`** — fires when a manual-capture hold is live and capturable:
- Fields: `event_id`, `payment_intent_id`, `amount_capturable_cents: i64`, `currency: String`, `metadata: HashMap<String,String>`
- `from_raw` guards on `EventType::PaymentIntentAmountCapturableUpdated`; extracts from `EventObject::PaymentIntent(pi)` with `pi.amount_capturable` (no conversion — already `i64`), `pi.currency.to_string()`, `pi.metadata.clone()`

**`StripePaymentIntentCanceled`** — fires on manual cancel or Stripe's ~7-day auto-expiry:
- Fields: `event_id`, `payment_intent_id`, `cancellation_reason: Option<String>`, `metadata: HashMap<String,String>`
- `from_raw` guards on `EventType::PaymentIntentCanceled`; extracts `pi.cancellation_reason.map(|r| r.as_str().to_string())` (Copy enum, safe to call `.map` without reference borrow)

Both added to `events_are_clone_send_sync` and `all_event_types_implement_stripe_event` marker tests.

Both re-exported from `ferro_stripe` crate root in the existing `pub use webhook::events::{...}` block.

### Task 2 — Golden fixtures + parser-contract tests (43ed7587)

**`payment_intent_amount_capturable_updated.json`**: type=`"payment_intent.amount_capturable_updated"`, `amount_capturable=5000`, `status="requires_capture"`, `capture_method="manual"`, `metadata.booking_id="bk_42"`

**`payment_intent_canceled.json`**: type=`"payment_intent.canceled"`, `status="canceled"`, `cancellation_reason="requested_by_customer"`, `capture_method="manual"`, `metadata.booking_id="bk_43"`

**4 new parser-contract tests** registered in `tests/parser_contract.rs`:
1. `payment_intent_amount_capturable_updated_parses_all_fields` — asserts all 5 fields
2. `payment_intent_canceled_parses_all_fields` — asserts all 4 fields including `cancellation_reason`
3. `payment_intent_amount_capturable_updated_rejects_canceled_event` — cross-type rejection
4. `payment_intent_canceled_rejects_amount_capturable_event` — cross-type rejection

Total parser-contract test count: 19 (was 15).

## Deviations from Plan

None — plan executed exactly as written.

## Threat Mitigations Applied

Per plan threat register:
- **T-189-08** (Spoofing — inbound webhook): Both new events flow through the existing `verify_webhook` signature-verification path. `from_raw` operates on an already-deserialized `stripe::Event`; no new ingress bypasses verification.
- **T-189-09** (Tampering — event type confusion): Each `from_raw` guards on its exact `EventType` variant and returns `None` otherwise. Cross-reject tests 3 and 4 prove `amount_capturable_updated` and `canceled` cannot be parsed as each other.
- **T-189-10** (Tampering — fixture type-string drift): Fixtures use the verified serde-rename strings. Both parse-all-fields tests would fail loudly if a fixture deserialized as `EventType::Other`.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes. Additive event structs only.

## Known Stubs

None — both event structs are fully implemented with all D-05 fields present.

## Self-Check

PASSED
- `grep -q "pub struct StripePaymentIntentAmountCapturableUpdated" ferro-stripe/src/webhook/events.rs` — PASS
- `grep -q "pub struct StripePaymentIntentCanceled" ferro-stripe/src/webhook/events.rs` — PASS
- `grep -q "EventType::PaymentIntentAmountCapturableUpdated" ferro-stripe/src/webhook/events.rs` — PASS
- `grep -q "EventType::PaymentIntentCanceled" ferro-stripe/src/webhook/events.rs` — PASS
- `grep -c "StripePaymentIntentAmountCapturableUpdated" ferro-stripe/src/webhook/events.rs` == 4 — PASS
- `grep -q "StripePaymentIntentAmountCapturableUpdated" ferro-stripe/src/lib.rs` — PASS
- `grep -q "StripePaymentIntentCanceled" ferro-stripe/src/lib.rs` — PASS
- Fixture `payment_intent_amount_capturable_updated.json` — valid JSON with `"payment_intent.amount_capturable_updated"` type — PASS
- Fixture `payment_intent_canceled.json` — valid JSON with `"payment_intent.canceled"` type — PASS
- `cargo test -p ferro-stripe --lib events` — 2 passed — PASS
- `cargo test -p ferro-stripe --test parser_contract` — 19 passed — PASS
- Commit `aa3bc443` exists — PASS
- Commit `43ed7587` exists — PASS

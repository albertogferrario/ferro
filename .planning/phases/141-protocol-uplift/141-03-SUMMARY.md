---
phase: 141-protocol-uplift
plan: "03"
subsystem: ferro-stripe
tags: [stripe, webhook, testing, fixtures, parser-contract]
dependency_graph:
  requires:
    - StripeEvent trait (ferro-stripe::StripeEvent) — Plan 01
    - 10 typed event structs with from_raw — Plan 01
  provides:
    - 10 golden-JSON fixtures (ferro-stripe/tests/fixtures/stripe_events/)
    - parser_contract integration test suite (15 tests: 10 positive + 5 negative)
  affects:
    - ferro-stripe/tests/ (new test directory and fixtures)
key_files:
  created:
    - ferro-stripe/tests/fixtures/stripe_events/checkout_session_completed.json
    - ferro-stripe/tests/fixtures/stripe_events/checkout_session_expired.json
    - ferro-stripe/tests/fixtures/stripe_events/payment_intent_payment_failed.json
    - ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json
    - ferro-stripe/tests/fixtures/stripe_events/charge_dispute_created.json
    - ferro-stripe/tests/fixtures/stripe_events/account_updated.json
    - ferro-stripe/tests/fixtures/stripe_events/customer_subscription_updated.json
    - ferro-stripe/tests/fixtures/stripe_events/customer_subscription_deleted.json
    - ferro-stripe/tests/fixtures/stripe_events/invoice_paid.json
    - ferro-stripe/tests/fixtures/stripe_events/payment_intent_succeeded_connect.json
    - ferro-stripe/tests/parser_contract.rs
  modified: []
decisions:
  - "Fixtures required iterative field additions beyond RESEARCH Pattern 4 — async-stripe 0.41 enforces non-optional fields that are not documented in the minimal envelope spec"
  - "account_updated and invoice_paid fixtures deserialized cleanly on first attempt; checkout.session, subscription, payment_intent, charge, and dispute each required 1-3 rounds of missing-field additions"
metrics:
  duration: "~20 min"
  completed: "2026-04-20"
  tasks: 2
  files: 11
---

# Phase 141 Plan 03: Golden-JSON Fixtures + Parser-Contract Tests Summary

10 golden-JSON fixture files and 15 parser-contract integration tests covering all `StripeEvent::from_raw` implementations — 10 positive field-by-field assertions and 5 cross-type negative guards.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | 10 golden-JSON fixtures under tests/fixtures/stripe_events/ | a9423e09 | 10 fixture JSON files |
| 2 | parser_contract.rs with 15 integration tests | c42aee9f | tests/parser_contract.rs + 8 fixture updates |

## Fixtures Created

| File | Event Type | Object Type | First-attempt deserialize |
|------|-----------|-------------|--------------------------|
| `checkout_session_completed.json` | `checkout.session.completed` | `checkout.session` | Required `payment_method_types` |
| `checkout_session_expired.json` | `checkout.session.expired` | `checkout.session` | Required `payment_method_types` |
| `payment_intent_payment_failed.json` | `payment_intent.payment_failed` | `payment_intent` | Required `capture_method`, `confirmation_method`, `created`, `payment_method_types` |
| `charge_refunded.json` | `charge.refunded` | `charge` | Required `billing_details`, `captured`, `created`, `disputed` |
| `charge_dispute_created.json` | `charge.dispute.created` | `dispute` | Required `evidence`, `metadata` |
| `account_updated.json` | `account.updated` | `account` | Clean on first attempt |
| `customer_subscription_updated.json` | `customer.subscription.updated` | `subscription` | Required `automatic_tax`, `billing_cycle_anchor`, `currency`, `start_date` |
| `customer_subscription_deleted.json` | `customer.subscription.deleted` | `subscription` | Required `automatic_tax`, `billing_cycle_anchor`, `currency`, `start_date` |
| `invoice_paid.json` | `invoice.paid` | `invoice` | Clean on first attempt |
| `payment_intent_succeeded_connect.json` | `payment_intent.succeeded` | `payment_intent` | Required `capture_method`, `confirmation_method`, `created`, `payment_method_types` |

## Tests Produced

### Positive tests (10) — field-by-field assertions

| Test | Event struct | Key assertions |
|------|-------------|----------------|
| `checkout_session_completed_parses_all_fields` | `StripeCheckoutCompleted` | event_id, session_id, payment_intent_id, amount_total_cents, currency, customer_email, metadata |
| `checkout_session_expired_parses_all_fields` | `StripeCheckoutExpired` | event_id, session_id, metadata |
| `payment_intent_failed_parses_all_fields` | `StripePaymentIntentFailed` | event_id, payment_intent_id, session_id, failure_code, failure_message, metadata keys |
| `charge_refunded_parses_all_fields` | `StripeChargeRefunded` | event_id, charge_id, payment_intent_id, amount_refunded_cents, metadata |
| `charge_dispute_created_parses_all_fields` | `StripeChargeDisputeCreated` | event_id, charge_id, payment_intent_id, dispute_reason, amount_cents |
| `account_updated_parses_all_fields` | `StripeConnectAccountUpdated` | event_id, account_id, charges_enabled, payouts_enabled, details_submitted |
| `subscription_updated_parses_all_fields` | `StripeSubscriptionUpdated` | event_id, subscription_id, customer_id |
| `subscription_deleted_parses_all_fields` | `StripeSubscriptionDeleted` | event_id, subscription_id, customer_id |
| `invoice_paid_parses_all_fields` | `StripeInvoicePaid` | event_id, invoice_id, customer_id |
| `connect_payment_succeeded_parses_all_fields` | `StripeConnectPaymentSucceeded` | event_id, payment_intent_id, connect_account_id |

### Negative tests (5) — cross-type guards

| Test | What it guards |
|------|---------------|
| `checkout_completed_rejects_expired_event` | `StripeCheckoutCompleted::from_raw` returns `None` for `checkout.session.expired` |
| `checkout_expired_rejects_completed_event` | `StripeCheckoutExpired::from_raw` returns `None` for `checkout.session.completed` |
| `subscription_updated_rejects_deleted_event` | `StripeSubscriptionUpdated::from_raw` returns `None` for `customer.subscription.deleted` |
| `subscription_deleted_rejects_updated_event` | `StripeSubscriptionDeleted::from_raw` returns `None` for `customer.subscription.updated` |
| `invoice_paid_rejects_checkout_event` | `StripeInvoicePaid::from_raw` returns `None` for `checkout.session.completed` (different object type) |

## Verification Results

- `cargo test -p ferro-stripe --all-features --test parser_contract`: 15 passed, 0 failed
- `cargo test -p ferro-stripe --all-features`: 43 passed (23 lib + 5 dispatcher + 15 parser_contract), 0 failed
- `cargo clippy -p ferro-stripe --all-targets --all-features -- -D warnings`: clean
- `ls ferro-stripe/tests/fixtures/stripe_events/*.json | wc -l`: 10

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] async-stripe 0.41 requires more fields than RESEARCH Pattern 4 documented**
- **Found during:** Task 2 first test run
- **Issue:** async-stripe 0.41 enforces non-optional struct fields not present in the minimal envelope spec from RESEARCH Pattern 4. Deserialization failed for 8 of 10 fixtures on first run.
- **Fix:** Iteratively added required fields across 4 test runs until all 15 tests passed. Fields added per object type:
  - `checkout.session`: `payment_method_types`
  - `payment_intent`: `capture_method`, `confirmation_method`, `created`, `payment_method_types`
  - `charge`: `billing_details`, `captured`, `created`, `disputed`
  - `dispute`: `evidence`, `metadata`
  - `subscription`: `automatic_tax`, `billing_cycle_anchor`, `currency`, `start_date`
- **Impact on Plan 04:** Plan 04 must use these augmented fixture shapes when constructing `ProcessStripeWebhook`'s `raw_body` in tests. The plan's note "reuse the same JSON shape" is valid — use the final fixture files as-is.
- **Files modified:** 8 of 10 fixtures (account_updated.json and invoice_paid.json were clean on first attempt)
- **Commit:** c42aee9f

## Notes for Plan 04

All 10 fixture files are valid input for `serde_json::from_str::<stripe::Event>()` without signature verification. Plan 04's queue path tests can load any fixture via `include_str!` or `std::fs::read_to_string` and pass the raw string as `ProcessStripeWebhook::raw_body`. No additional fields should be needed.

## Known Stubs

None. `ProcessStripeWebhook::handle()` stub from Plan 01 is intentionally deferred to Plan 04.

## Threat Flags

None. All fixtures are static test-only input. No new network endpoints or auth paths introduced.

## Self-Check: PASSED

- `ferro-stripe/tests/fixtures/stripe_events/` contains exactly 10 `.json` files
- `ferro-stripe/tests/parser_contract.rs` exists with `include_str!` for all 10 fixture paths
- All 15 test functions named in acceptance criteria are present
- `cargo test -p ferro-stripe --all-features --test parser_contract`: 15 passed
- `cargo clippy -p ferro-stripe --all-targets --all-features -- -D warnings`: clean
- Commits a9423e09 and c42aee9f exist in git log

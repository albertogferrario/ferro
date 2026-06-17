---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
plan: "03"
subsystem: ferro-payments
tags: [lifecycle, guarded-update, webhook, sea-orm]
dependency_graph:
  requires: [ferro-payments/src/intent/entity.rs, ferro-orm GuardedUpdate]
  provides: [find_by_payment_intent, find_by_charge_id, attach_payment_intent]
  affects: [235-05 handle_charge_refunded, 235-04 handle_session_completed]
tech_stack:
  added: []
  patterns: [GuardedUpdate IS NULL guard, ConnectionTrait-generic async fn, in-memory SQLite test harness]
key_files:
  modified: [ferro-payments/src/intent/lifecycle.rs]
decisions:
  - attach_payment_intent uses IS NULL guard (idempotent Stripe retries — T-235-04 mitigation)
  - find_by_payment_intent is primary lookup; find_by_charge_id is fallback — mirrors find_by_stripe_session pattern exactly
  - attach_payment_intent does not require status=paid precondition — the caller (handle_session_completed) owns the sequencing
metrics:
  duration_minutes: 4
  completed_date: "2026-06-17"
  tasks_completed: 1
  files_modified: 1
requirements_closed: [PAY-POLY-WH-02, PAY-POLY-WH-04]
---

# Phase 235 Plan 03: Lifecycle Query/Update Helpers Summary

Three `ConnectionTrait`-generic lifecycle helpers added to `ferro-payments/src/intent/lifecycle.rs`, enabling the Wave 3 webhook handlers to locate and annotate intent rows by `payment_intent_id` / `charge_id`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | find_by_payment_intent + find_by_charge_id + attach_payment_intent | 8e5ae574 | ferro-payments/src/intent/lifecycle.rs |

## What Was Built

**`find_by_payment_intent(payment_intent_id, conn)`** — mirrors `find_by_stripe_session` exactly, filtering `Column::PaymentIntentId.eq(...)`. Primary lookup for `handle_charge_refunded` (the refund event carries no `session_id`).

**`find_by_charge_id(charge_id, conn)`** — same pattern, filtering `Column::ChargeId.eq(...)`. Fallback lookup when the refund event's `payment_intent_id` field is absent or matches no row.

**`attach_payment_intent(id, payment_intent_id, conn)`** — `GuardedUpdate` setting `Column::PaymentIntentId` guarded by `Column::Id.eq(id)` AND `Column::PaymentIntentId.is_null()`. Returns `Ok(true)` on first write, `Ok(false)` on replay (IS NULL guard excluded the row — idempotent per T-235-04). Called by `handle_session_completed` after `mark_paid` to persist the `payment_intent_id` learned from the event.

## Tests Added

- `find_by_payment_intent_matches` — seeds a row, calls `attach_payment_intent`, asserts `find_by_payment_intent` returns the row; confirms a non-matching ID returns `None`.
- `find_by_charge_id_matches` — inserts a row with `charge_id` set directly via `ActiveModel`, asserts lookup succeeds and a non-matching ID returns `None`.
- `attach_payment_intent_idempotent_second_call_noops` — first call returns `Ok(true)`; second call with a different value returns `Ok(false)`; first value remains unchanged.

All 26 ferro-payments tests pass. `cargo clippy -p ferro-payments --all-targets -- -D warnings` and `cargo fmt -p ferro-payments -- --check` both clean.

## Deviations from Plan

None — plan executed exactly as written. The three helpers and their tests match the PATTERNS.md verbatim target bodies.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundaries introduced. The two lookup functions are read-only parameterized queries (`.eq()` binds prevent SQL injection — T-235-05 disposition: accept). `attach_payment_intent` IS NULL guard closes T-235-04 (double-attach tamper prevention).

## Self-Check: PASSED

- `ferro-payments/src/intent/lifecycle.rs` contains `pub async fn find_by_payment_intent` — FOUND
- `ferro-payments/src/intent/lifecycle.rs` contains `pub async fn find_by_charge_id` — FOUND
- `ferro-payments/src/intent/lifecycle.rs` contains `pub async fn attach_payment_intent` — FOUND
- `ferro-payments/src/intent/lifecycle.rs` contains `Column::PaymentIntentId.is_null()` — FOUND
- Commit 8e5ae574 — FOUND
- `cargo test -p ferro-payments find_by_payment_intent_matches` exits 0 — VERIFIED
- `cargo test -p ferro-payments attach_payment_intent_idempotent_second_call_noops` exits 0 — VERIFIED
- `cargo clippy -p ferro-payments --all-targets -- -D warnings` exits 0 — VERIFIED

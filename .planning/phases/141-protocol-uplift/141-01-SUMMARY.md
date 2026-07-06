---
phase: 141-protocol-uplift
plan: "01"
subsystem: ferro-stripe
tags: [stripe, webhook, events, refactor]
dependency_graph:
  requires: []
  provides:
    - StripeEvent trait (ferro-stripe::StripeEvent)
    - 10 typed event structs with from_raw (StripeSubscriptionUpdated, StripeSubscriptionDeleted, StripeCheckoutCompleted, StripeInvoicePaid, StripeConnectPaymentSucceeded, StripeCheckoutExpired, StripePaymentIntentFailed, StripeChargeRefunded, StripeChargeDisputeCreated, StripeConnectAccountUpdated)
    - ProcessStripeWebhook stub with raw_body field
  affects:
    - framework/Cargo.toml (version reference bump)
    - ferro-stripe public API surface
tech_stack:
  added:
    - tracing = "0.1" in ferro-stripe
  patterns:
    - EventObject pattern-match for from_raw (no JSON re-parsing)
    - StripeEvent marker trait with from_raw -> Option<Self>
    - Consuming builder pattern retained for ProcessStripeWebhook compat
key_files:
  created: []
  modified:
    - ferro-stripe/Cargo.toml
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/webhook/verify.rs
    - ferro-stripe/src/testing.rs
    - ferro-stripe/src/lib.rs
    - framework/Cargo.toml
decisions:
  - "Invoice.id accessed directly as String (not Option<String>) — as_ref()? pattern was wrong"
  - "Dispute.reason is String in async-stripe 0.41, not DisputeReason enum — used .clone()"
  - "framework/Cargo.toml ferro-stripe version bumped 0.4 -> 0.5 (deviation: not in plan but required for workspace to build)"
metrics:
  duration: "317s (~5 min)"
  completed: "2026-04-20"
  tasks: 3
  files: 7
---

# Phase 141 Plan 01: StripeEvent Trait + Event Struct Rewrite Summary

Foundation rewrite of ferro-stripe's webhook event layer: `StripeEvent` marker trait with `from_raw` on 10 typed structs, `event_json` removed, `ferro_events::Event` impls removed, `signed_webhook_payload` relocated to `testing.rs`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | Cargo deps + version bump + events.rs rewrite | 32eb4d1d | ferro-stripe/Cargo.toml, framework/Cargo.toml, testing.rs, verify.rs, events.rs, Cargo.lock |
| 3 | Update webhook/mod.rs and lib.rs re-exports | af4e7b16 | webhook/mod.rs, lib.rs |

## Verification Results

- `cargo build -p ferro-stripe --all-features`: 0 errors
- `cargo test -p ferro-stripe --all-features`: 20 passed, 0 failed
- `cargo clippy -p ferro-stripe --all-targets --all-features -- -D warnings`: clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Invoice.id is String, not Option<String>**
- **Found during:** Task 2 compile
- **Issue:** `inv.id.as_ref()?.to_string()` — `as_ref()` on `String` yields `&String`, not `Option`; `?` does not apply
- **Fix:** Changed to `inv.id.to_string()` directly
- **Files modified:** ferro-stripe/src/webhook/events.rs
- **Commit:** 32eb4d1d

**2. [Rule 3 - Blocking] framework/Cargo.toml version reference mismatch**
- **Found during:** Task 1 build
- **Issue:** `framework/Cargo.toml` required `ferro-stripe = "^0.4"` — workspace refused to resolve after version bump to 0.5.0
- **Fix:** Updated framework/Cargo.toml `ferro-stripe` version to `"0.5"`
- **Files modified:** framework/Cargo.toml
- **Commit:** 32eb4d1d

### Observation: Dispute.reason type

The plan noted `dispute.reason` might be a `DisputeReason` enum requiring `.to_string()`. In async-stripe 0.41 it is a `String` — `.clone()` was used as instructed by the plan's fallback note.

## Deferred Items

- `SyncDispatcher` re-export in `webhook/mod.rs` and `lib.rs` — Plan 02 adds `sync.rs` fill, then re-exports
- `ProcessStripeWebhook` relocation to `webhook/queue.rs` — Plan 04
- `ProcessStripeWebhook::handle()` wiring to `Arc<SyncDispatcher>` — Plan 04
- Framework `lib.rs` re-exports for `SyncDispatcher` + 5 new event types — Plan 02/04
- Golden-JSON fixtures and parser-contract tests — Plan 03
- `SyncDispatcher` unit tests — Plan 02

## Known Stubs

- `ProcessStripeWebhook::handle()` returns `Ok(())` unconditionally. This is intentional: Plan 04 wires the dispatcher. The stub keeps Plans 02 and 03 unblocked.

## Threat Flags

None. All `from_raw` implementations guard on `event.type_` before matching `event.data.object` (T-141-01 mitigation). `event_id: String` is present on all 10 structs (T-141-03 mitigation). No JSON re-serialization in `from_raw` (T-141-04 mitigation).

## Self-Check: PASSED

- `ferro-stripe/src/webhook/events.rs` exists and contains `pub trait StripeEvent`
- `ferro-stripe/src/testing.rs` exists and contains `pub fn signed_webhook_payload`
- `ferro-stripe/src/webhook/verify.rs` contains `use crate::testing::signed_webhook_payload`
- Commits 32eb4d1d and af4e7b16 exist in git log
- 20 tests pass, clippy clean, build clean

---
phase: 141-protocol-uplift
plan: "04"
subsystem: ferro-stripe
tags: [stripe, webhook, queue, dispatcher, reexport, framework]
dependency_graph:
  requires:
    - ProcessStripeWebhook stub in events.rs — Plan 01
    - SyncDispatcher with on/dispatch — Plan 02
    - 10 golden-JSON fixtures in tests/fixtures/stripe_events/ — Plan 03
  provides:
    - ProcessStripeWebhook in webhook/queue.rs with Arc<SyncDispatcher> field and handle() wired
    - ferro-stripe 0.5.0 complete public API surface
    - framework crate re-exports: SyncDispatcher, StripeEvent, StripeChargeDisputeCreated,
      StripeChargeRefunded, StripeCheckoutExpired, StripeConnectAccountUpdated,
      StripePaymentIntentFailed added to stripe feature block
  affects:
    - ferro-stripe/src/webhook/queue.rs (rewritten from stub)
    - ferro-stripe/src/webhook/events.rs (ProcessStripeWebhook removed)
    - ferro-stripe/src/webhook/sync.rs (manual Debug impl added)
    - ferro-stripe/src/webhook/mod.rs (re-export routing updated)
    - ferro-stripe/src/lib.rs (re-export routing updated)
    - framework/src/lib.rs (stripe feature block extended)
tech_stack:
  added: []
  patterns:
    - "#[serde(skip)] on Arc<SyncDispatcher> field — runtime-only injection, not persisted"
    - "ferro_queue::Error::JobFailed { job, message } struct variant (not tuple)"
    - "manual std::fmt::Debug impl on SyncDispatcher (BoxedHandler closures are not Debug)"
    - "ProcessStripeWebhook::new() as the only valid construction path at enqueue time"
key_files:
  created: []
  modified:
    - ferro-stripe/src/webhook/queue.rs
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/src/webhook/sync.rs
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/lib.rs
    - framework/src/lib.rs
    - ferro-json-ui/src/render.rs
decisions:
  - "ferro_queue::Error::JobFailed is a struct variant {job, message} — not tuple; handle() uses struct syntax throughout"
  - "SyncDispatcher cannot derive Debug (BoxedHandler closures); manual impl reports handlers_count only"
  - "ProcessStripeWebhook re-parse uses serde_json::from_str::<stripe::Event> directly — no signature re-verification needed for already-verified bodies"
metrics:
  duration: "~25 min"
  completed: "2026-04-20"
  tasks: 2
  files: 7
---

# Phase 141 Plan 04: ProcessStripeWebhook Relocation + Framework Re-exports Summary

`ProcessStripeWebhook` relocated from `webhook/events.rs` to `webhook/queue.rs` with `Arc<SyncDispatcher>` injection, `handle()` wired through `SyncDispatcher::dispatch`, and full workspace gate green — closes Phase 141.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Relocate and reshape ProcessStripeWebhook into webhook/queue.rs | 91d420f3 | queue.rs, events.rs, sync.rs, mod.rs, lib.rs |
| 2 | Update framework re-exports; run full workspace gate | fb93d328 | framework/src/lib.rs, ferro-json-ui/src/render.rs, fmt normalization |

## Final ProcessStripeWebhook Shape

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    pub event_type: String,
    pub raw_body: String,
    pub connect_account_id: Option<String>,
    #[serde(skip)]                              // T-141-10: not persisted to queue storage
    pub dispatcher: Option<Arc<SyncDispatcher>>,
}
```

**Constructor:** `ProcessStripeWebhook::new(event_type, raw_body, connect_account_id, dispatcher)` — sets `dispatcher: Some(Arc<...>)`.

**handle():** `.expect()` on dispatcher (T-141-12 — missing dispatcher is a caller bug, panics with clear message); `serde_json::from_str::<stripe::Event>` for raw_body re-parse (T-141-11 — maps parse errors to `Error::JobFailed { job, message }`); delegates to `dispatcher.dispatch(event).await`.

## Unit Tests (webhook::queue, 4 tests)

| Test | What it verifies |
|------|-----------------|
| `process_stripe_webhook_job_name` | `name()` returns `"ProcessStripeWebhook"` |
| `new_sets_dispatcher_to_some` | Constructor sets `dispatcher: Some(...)` |
| `handle_dispatches_parsed_event_through_dispatcher` | AtomicBool flipped by registered handler; `Ok(())` returned |
| `handle_maps_parse_errors_to_job_failed` | Invalid JSON body maps to `Error::JobFailed { message: "parse stripe event: ..." }` |

## framework/src/lib.rs — Stripe Feature Block (final)

```rust
#[cfg(feature = "stripe")]
pub use ferro_stripe::{
    account, checkout, refund, verify_webhook, CheckoutBuilder, CheckoutIntent,
    Error as StripeError, LineItem, MemoryProcessedLog, Mode, ProcessStripeWebhook,
    ProcessedEventLog, Stripe, StripeChargeDisputeCreated, StripeChargeRefunded,
    StripeCheckoutCompleted, StripeCheckoutExpired, StripeConfig, StripeConnectAccountUpdated,
    StripeConnectPaymentSucceeded, StripeEvent, StripeInvoicePaid, StripePaymentIntentFailed,
    StripeSubscriptionDeleted, StripeSubscriptionUpdated, SyncDispatcher,
};
```

Additions vs Phase 140: `SyncDispatcher`, `StripeEvent`, `StripeChargeDisputeCreated`, `StripeChargeRefunded`, `StripeCheckoutExpired`, `StripeConnectAccountUpdated`, `StripePaymentIntentFailed`.

## Workspace Gate Results

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | clean |
| `cargo build --all --all-features` | clean |
| `cargo test --all-features` | all pass |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | clean |

## Phase 141 Success Criteria Status

| SC | Description | Status |
|----|-------------|--------|
| SC-5 | `ProcessStripeWebhook` in `webhook/queue.rs`, accepts `Arc<SyncDispatcher>`, `handle()` delegates to `dispatcher.dispatch(event)` | DELIVERED |
| SC-6 | Doc comments distinguish payment-correctness (sync) vs eventual-consistency (queue) paths | DELIVERED (sync.rs Plan 02 + queue.rs Plan 04) |
| SC-14 | `ferro-stripe 0.5.0` version + workspace CI green | DELIVERED |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro_queue::Error::JobFailed is a struct variant, not tuple**
- **Found during:** Task 1 first build
- **Issue:** Plan spec used `JobFailed(message)` tuple syntax. Actual ferro-queue definition is `JobFailed { job: String, message: String }`.
- **Fix:** Updated both `map_err` calls in `handle()` and the test match arm to use struct variant syntax with `job: "ProcessStripeWebhook".to_string()`.
- **Files modified:** `ferro-stripe/src/webhook/queue.rs`
- **Commit:** 91d420f3

**2. [Rule 2 - Missing functionality] SyncDispatcher lacks Debug impl required by #[derive(Debug)] on ProcessStripeWebhook**
- **Found during:** Task 1 first build
- **Issue:** `ProcessStripeWebhook` derives `Debug`, but its `dispatcher: Option<Arc<SyncDispatcher>>` field requires `SyncDispatcher: Debug`. `BoxedHandler` closures cannot derive Debug.
- **Fix:** Added manual `impl std::fmt::Debug for SyncDispatcher` reporting `handlers_count` only.
- **Files modified:** `ferro-stripe/src/webhook/sync.rs`
- **Commit:** 91d420f3

**3. [Rule 3 - Blocking] Pre-existing ferro-json-ui/src/component.rs DataTableProps field addition broke 5 test initializers**
- **Found during:** Task 2 workspace test gate
- **Issue:** `component.rs` had uncommitted `row_href: Option<String>` field added before this plan started. Five `DataTableProps` struct initializers in render.rs tests were missing the field, causing compile errors that blocked the workspace gate.
- **Fix:** Added `row_href: None` to all 5 initializers.
- **Files modified:** `ferro-json-ui/src/render.rs`
- **Commit:** fb93d328

## Verification Checks

- `grep -R 'ferro_events' ferro-stripe/`: no matches
- `grep -n 'version = "0.5.0"' ferro-stripe/Cargo.toml`: matches line 3
- `grep -n 'SyncDispatcher' framework/src/lib.rs`: matches inside stripe feature block
- `grep -n 'StripeEvent' framework/src/lib.rs`: matches
- `grep -n 'ProcessStripeWebhook' ferro-stripe/src/webhook/events.rs`: no matches (0)
- `grep -n 'ferro_queue::Job' ferro-stripe/src/webhook/events.rs`: no matches (0)

## Known Stubs

None. `ProcessStripeWebhook::handle()` is fully wired. All phase objectives delivered.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `ferro-stripe/src/webhook/queue.rs` exists and contains `pub struct ProcessStripeWebhook`
- `ferro-stripe/src/webhook/queue.rs` contains `#[serde(skip)]`
- `ferro-stripe/src/webhook/queue.rs` contains `pub dispatcher: Option<Arc<SyncDispatcher>>`
- `ferro-stripe/src/webhook/queue.rs` contains `pub fn new(`
- `ferro-stripe/src/webhook/queue.rs` contains `serde_json::from_str`
- `ferro-stripe/src/webhook/queue.rs` contains `dispatcher.dispatch(event)`
- `ferro-stripe/src/webhook/queue.rs` contains `ProcessStripeWebhook requires dispatcher`
- `ferro-stripe/src/webhook/events.rs` does NOT contain `ProcessStripeWebhook`
- `framework/src/lib.rs` contains `SyncDispatcher` in stripe block
- Commits 91d420f3 and fb93d328 exist in git log
- `cargo fmt --all -- --check`: clean
- `cargo build --all --all-features`: clean
- `cargo test --all-features`: all pass
- `cargo clippy --all --all-targets --all-features -- -D warnings`: clean

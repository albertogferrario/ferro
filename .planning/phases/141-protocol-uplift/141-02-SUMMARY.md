---
phase: 141-protocol-uplift
plan: "02"
subsystem: ferro-stripe
tags: [stripe, webhook, dispatcher, sync, testing]
dependency_graph:
  requires:
    - StripeEvent trait (ferro-stripe::StripeEvent) — Plan 01
    - 10 typed event structs with from_raw — Plan 01
  provides:
    - SyncDispatcher (ferro-stripe::SyncDispatcher)
    - Arc<SyncDispatcher>: Send + Sync handler registry
    - dispatcher integration test suite (5 tests)
  affects:
    - ferro-stripe/src/webhook/mod.rs (SyncDispatcher re-export)
    - ferro-stripe/src/lib.rs (SyncDispatcher re-export)
    - ferro-stripe/src/testing.rs (mock_checkout_completed_event fix)
tech_stack:
  added: []
  patterns:
    - BoxedHandler type alias with (bool, Result) return — matched-flag pattern for unknown event detection
    - Consuming builder on<E, H, Fut>(mut self, handler) -> Self
    - Arc<Handler> inside BoxedHandler closure for Send + Sync without unsafe
key_files:
  created:
    - ferro-stripe/tests/dispatcher.rs
  modified:
    - ferro-stripe/src/webhook/sync.rs
    - ferro-stripe/src/webhook/mod.rs
    - ferro-stripe/src/lib.rs
    - ferro-stripe/src/testing.rs
decisions:
  - "BoxedHandler returns (bool, Result) tuple so dispatch can detect unmatched events without a separate sentinel — RESEARCH Pitfall 1 fix"
  - "Arc<H> wraps each registered handler closure so the Box<dyn Fn> is clone-free and Send + Sync without unsafe impl"
  - "mock_checkout_completed_event required all non-optional CheckoutSession fields; added created, expires_at, mode, custom_fields, custom_text, shipping_options, automatic_tax"
metrics:
  duration: "~15 min"
  completed: "2026-04-20"
  tasks: 2
  files: 5
---

# Phase 141 Plan 02: SyncDispatcher Handler Registry Summary

`SyncDispatcher` handler registry with typed `on`/`dispatch` API, `Vec<BoxedHandler>` type-erased storage, matched-flag unknown-event detection, and 5 integration tests covering Err bubbling, Ok path, unknown event no-op, handler isolation, and `Arc` thread safety.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement SyncDispatcher in webhook/sync.rs | e286d481 | ferro-stripe/src/webhook/sync.rs |
| 2 | Integration tests + re-export SyncDispatcher | 1487af8d | ferro-stripe/tests/dispatcher.rs, webhook/mod.rs, lib.rs, testing.rs |

## Public API Produced

```rust
pub struct SyncDispatcher { /* Vec<BoxedHandler> */ }

impl SyncDispatcher {
    pub fn new() -> Self;
    pub fn on<E, H, Fut>(mut self, handler: H) -> Self
    where
        E: StripeEvent,
        H: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static;
    pub async fn dispatch(&self, event: stripe::Event) -> Result<(), Error>;
}

impl Default for SyncDispatcher { ... }
```

`Arc<SyncDispatcher>` is `Send + Sync`. Multiple tokio tasks can call `dispatch` concurrently — verified by `dispatcher_is_thread_safe_across_arc`.

## Internal Storage Decision

`Vec<BoxedHandler>` where `BoxedHandler` is:

```rust
Box<dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = (bool, Result<(), Error>)> + Send>> + Send + Sync>
```

The `(bool, Result)` tuple is the key design: `bool` indicates whether `from_raw` matched. Without this, an empty handler list and a non-matching handler are indistinguishable — both produce `Ok(())` from every closure — and `dispatch` cannot reliably log unregistered events (RESEARCH Pitfall 1).

## Contract for Plan 04

Plan 04 (`ProcessStripeWebhook` queue job) consumes:
- `SyncDispatcher` is constructed once via consuming builder
- Shared across HTTP webhook path and queue path via `Arc<SyncDispatcher>`
- `dispatch(stripe::Event) -> Result<(), Error>` is the sole dispatch entry point
- Thread-safe: `Arc<SyncDispatcher>` can be cloned into tokio tasks without additional synchronization

## Verification Results

- `cargo build -p ferro-stripe --all-features`: 0 errors
- `cargo test -p ferro-stripe --all-features`: 25 passed (20 lib + 5 integration), 0 failed
- `cargo clippy -p ferro-stripe --all-targets --all-features -- -D warnings`: clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] mock_checkout_completed_event missing required CheckoutSession fields**
- **Found during:** Task 2 integration test run
- **Issue:** `stripe::CheckoutSession` deserialization required non-optional fields: `created`, `expires_at`, `mode`, `custom_fields`, `custom_text`, `shipping_options`, `automatic_tax`. The mock JSON was missing all of these — tests panicked at `parse_event()`.
- **Fix:** Added all required fields to the mock JSON in `ferro-stripe/src/testing.rs`. Values use realistic defaults (timestamps 1700000000, mode "payment", empty arrays, disabled automatic_tax).
- **Files modified:** ferro-stripe/src/testing.rs
- **Commit:** 1487af8d

## Known Stubs

None. `SyncDispatcher` is fully functional. `ProcessStripeWebhook::handle()` stub from Plan 01 is intentionally deferred to Plan 04 and documented in Plan 01's summary.

## Threat Flags

None. No new network endpoints or auth paths introduced. `SyncDispatcher` operates entirely within the app process boundary — callers supply already-verified `stripe::Event` values.

## Self-Check: PASSED

- `ferro-stripe/src/webhook/sync.rs` contains `pub struct SyncDispatcher` ✓
- `ferro-stripe/tests/dispatcher.rs` exists with 5 `#[tokio::test]` functions ✓
- `ferro-stripe/src/webhook/mod.rs` contains `pub use sync::SyncDispatcher;` ✓
- `ferro-stripe/src/lib.rs` contains `pub use webhook::sync::SyncDispatcher;` ✓
- Commits e286d481 and 1487af8d exist in git log ✓
- 25 tests pass, clippy clean, build clean ✓

# Phase 141: Protocol Uplift - Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 8 new/modified files + 10 test fixtures + 2 integration test files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-stripe/src/webhook/events.rs` | model/event | event-driven | `ferro-stripe/src/webhook/events.rs` (current) | self — rewrite |
| `ferro-stripe/src/webhook/sync.rs` | service | event-driven | `ferro-stripe/src/idempotency.rs` + `ferro-stripe/src/checkout.rs` | role-match (builder + async trait) |
| `ferro-stripe/src/webhook/queue.rs` | job | event-driven | `ferro-stripe/src/webhook/events.rs` lines 103-156 (ProcessStripeWebhook) | exact — relocation + reshape |
| `ferro-stripe/src/webhook/mod.rs` | config/re-export | — | `ferro-stripe/src/webhook/mod.rs` (current) | self — extend |
| `ferro-stripe/src/lib.rs` | config/re-export | — | `ferro-stripe/src/lib.rs` (current) | self — extend |
| `ferro-stripe/Cargo.toml` | config | — | `ferro-stripe/Cargo.toml` (current) | self — edit |
| `framework/src/lib.rs` | config/re-export | — | `framework/src/lib.rs` lines 93-100 | self — extend |
| `ferro-stripe/src/testing.rs` | utility | — | `ferro-stripe/src/testing.rs` (current) + `ferro-stripe/src/webhook/events.rs` lines 235-250 | self — extend (receive `signed_webhook_payload`) |
| `ferro-stripe/tests/fixtures/stripe_events/*.json` | test fixture | — | `ferro-stripe/src/testing.rs` mock JSON patterns | data-match |
| `ferro-stripe/tests/parser_contract.rs` | test | event-driven | `ferro-stripe/src/idempotency.rs` tests (lines 83-122) | role-match |
| `ferro-stripe/tests/dispatcher.rs` | test | event-driven | `ferro-stripe/src/idempotency.rs` tests (lines 83-122) | role-match |

---

## Pattern Assignments

### `ferro-stripe/src/webhook/events.rs` (model, event-driven) — rewrite

**Analog:** current `ferro-stripe/src/webhook/events.rs`

**Current imports to replace** (lines 1):
```rust
use ferro_events::Event;
```
Replace with:
```rust
use std::collections::HashMap;
```

**StripeEvent trait — new, no analog; define at top of file:**
```rust
/// Marker trait for typed Stripe webhook event structs.
///
/// Every event struct implements this trait. `from_raw` converts a
/// verified [`stripe::Event`] to the typed struct, or returns `None`
/// when the event does not match.
pub trait StripeEvent: Send + Sync + 'static {
    fn from_raw(event: &stripe::Event) -> Option<Self>
    where
        Self: Sized;
}
```

**Event type guard pattern — copy for EVERY `from_raw` impl** (from RESEARCH.md Pitfall 2):
```rust
// Always check event.type_ FIRST before matching event.data.object.
// Both checkout.session.completed and checkout.session.expired produce
// EventObject::CheckoutSession — type_ is the discriminant, not the object variant.
if event.type_ != stripe::EventType::CheckoutSessionCompleted {
    return None;
}
match &event.data.object {
    stripe::EventObject::CheckoutSession(session) => { /* ... */ }
    _ => None,
}
```

**Expandable<T> extraction pattern** (from RESEARCH.md Pitfall 4):
```rust
// Expandable<T> is either Id(XxxId) or Object(Box<Xxx>).
// Webhook payloads deliver IDs, not expanded objects. .id() works for both.
let payment_intent_id = charge.payment_intent.as_ref().map(|e| e.id().to_string());
```

**Currency enum to String** (from RESEARCH.md Pitfall 6):
```rust
currency: session.currency.map(|c| c.to_string()).unwrap_or_default(),
```

**Existing struct derive pattern** (events.rs lines 6-7, 26-27, 44-45):
```rust
#[derive(Debug, Clone)]
pub struct StripeXxx { ... }
```
All 10 event structs use `#[derive(Debug, Clone)]` — no serde needed on event structs.

**signed_webhook_payload removal:** Lines 235-250 of current `events.rs` move verbatim to `ferro-stripe/src/testing.rs`. After the move, remove the function from `events.rs` and update the re-export in `testing.rs` line 132 (was `pub use crate::webhook::events::signed_webhook_payload`) — the function now lives directly in `testing.rs`.

**Unit tests to remove:** The five `*_event_name()` tests (lines 253-305) test `ferro_events::Event::name()` — those impls are removed. Replace with compile-time trait bound assertions pattern from lines 307-316:
```rust
fn _assert_stripe_event<T: crate::webhook::events::StripeEvent>() {}

#[test]
fn all_event_types_implement_stripe_event() {
    _assert_stripe_event::<StripeSubscriptionUpdated>();
    // ... all 10 types
}
```

---

### `ferro-stripe/src/webhook/sync.rs` (service, event-driven) — fill stub

**Analog:** `ferro-stripe/src/checkout.rs` (consuming builder pattern) + `ferro-stripe/src/idempotency.rs` (async trait + Send + Sync)

**Module doc comment** (design doc §3.6 specifies this verbatim — use the text from sync.rs stub lines 1-6, extend it):
```rust
//! Synchronous Stripe webhook dispatch.
//!
//! [`SyncDispatcher`] is a handler registry for typed Stripe webhook events.
//! Register handlers with [`SyncDispatcher::on`] (consuming builder) and call
//! [`SyncDispatcher::dispatch`] from your webhook HTTP endpoint.
```

**Builder pattern** (from `ferro-stripe/src/checkout.rs` lines 62-130):
```rust
// Consuming builder: mut self, returns Self.
// Pattern from CheckoutBuilder::line_item, success_url, etc.
pub fn on<E, H, Fut>(mut self, handler: H) -> Self
where
    E: crate::webhook::events::StripeEvent,
    H: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
{
    // wrap into BoxedHandler, push to self.handlers
    self
}
```

**Type-erased handler storage** (RESEARCH.md Pattern 2 + Corrected Example):
```rust
// Return (bool, Result) from BoxedHandler so dispatch can detect unmatched events.
// bool = true means from_raw returned Some (handler was actually invoked).
type BoxedHandler = Box<
    dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = (bool, Result<(), Error>)> + Send>>
        + Send + Sync,
>;
```

**dispatch — unknown event detection** (RESEARCH.md SyncDispatcher Corrected example):
```rust
pub async fn dispatch(&self, event: stripe::Event) -> Result<(), Error> {
    let mut any_matched = false;
    for handler in &self.handlers {
        let (matched, result) = handler(event.clone()).await;
        if matched {
            any_matched = true;
            result?;
        }
    }
    if !any_matched {
        tracing::debug!(event_type = ?event.type_, "unregistered stripe event type — skipping");
    }
    Ok(())
}
```

**Error type:** `crate::Error` (from `ferro-stripe/src/error.rs`). Add `HandlerFailed` variant if dispatch needs to wrap handler errors — check if `crate::Error` can be returned directly from `Fn` closures first.

**tracing dep** — not yet in `ferro-stripe/Cargo.toml` (confirmed by Cargo.toml read). Add `tracing = "0.1"` to `[dependencies]`.

---

### `ferro-stripe/src/webhook/queue.rs` (job, event-driven) — fill stub

**Analog:** `ferro-stripe/src/webhook/events.rs` lines 103-156 (current `ProcessStripeWebhook`)

**Module doc comment** (queue.rs stub lines 1-6):
```rust
//! Queue-based Stripe webhook dispatch (eventual-consistency path).
//!
//! [`ProcessStripeWebhook`] is a [`ferro_queue::Job`] that receives a raw
//! Stripe event body and dispatches it through [`SyncDispatcher`] in the
//! background worker process.
```

**Struct definition pattern** (from events.rs lines 103-111, reshape per D-17/D-18):
```rust
// Keep serde derives — ferro_queue::Job requires Serialize + Deserialize for persistence.
// Arc<SyncDispatcher> is NOT serialized — use #[serde(skip)] + Option<Arc<...>>.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    pub event_type: String,
    pub raw_body: String,
    pub connect_account_id: Option<String>,
    #[serde(skip)]
    pub dispatcher: Option<std::sync::Arc<crate::webhook::sync::SyncDispatcher>>,
}
```

**Constructor pattern** (RESEARCH.md Pattern 3):
```rust
impl ProcessStripeWebhook {
    pub fn new(
        event_type: String,
        raw_body: String,
        connect_account_id: Option<String>,
        dispatcher: std::sync::Arc<crate::webhook::sync::SyncDispatcher>,
    ) -> Self { ... }
}
```

**Job impl pattern** (from events.rs lines 113-156 — keep `#[ferro_queue::async_trait]` and `Job` impl shape, replace handle body):
```rust
#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let dispatcher = self.dispatcher.as_ref()
            .expect("ProcessStripeWebhook requires dispatcher — use ProcessStripeWebhook::new()");
        let event = crate::webhook::verify::verify_webhook_raw(&self.raw_body)
            // OR: stripe::Webhook::construct_event without sig check for already-verified bodies
            .map_err(|e| ferro_queue::Error::JobFailed(e.to_string()))?;
        dispatcher.dispatch(event).await
            .map_err(|e| ferro_queue::Error::JobFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "ProcessStripeWebhook"
    }
}
```

**OPEN QUESTION (from RESEARCH.md):** `verify_webhook` requires a signature. For queue re-parse, the event is already verified. Check `verify.rs` for a parse-only function, or use `serde_json::from_str::<stripe::Event>(&self.raw_body)` directly (async-stripe `Event` implements `Deserialize`).

---

### `ferro-stripe/src/webhook/mod.rs` (re-export module) — extend

**Analog:** current `ferro-stripe/src/webhook/mod.rs` (lines 1-9)

**Current content** (lines 1-9):
```rust
//! Stripe webhook handling — signature verification, typed event structs,
//! and (in Phase 141) sync/queue dispatch.

pub mod events;
pub mod queue;
pub mod sync;
pub mod verify;

pub use verify::verify_webhook;
```

**Add after Phase 141:**
```rust
pub use events::StripeEvent;
pub use events::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripePaymentIntentFailed, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
};
pub use queue::ProcessStripeWebhook;
pub use sync::SyncDispatcher;
```

---

### `ferro-stripe/src/lib.rs` (crate root re-exports) — update

**Analog:** current `ferro-stripe/src/lib.rs` (lines 54-64)

**Current webhook re-export block** (lines 61-63):
```rust
pub use webhook::events::{
    ProcessStripeWebhook, StripeCheckoutCompleted, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripeSubscriptionDeleted, StripeSubscriptionUpdated,
};
```

**Replace with** (after Phase 141):
```rust
pub use webhook::{
    ProcessStripeWebhook, StripeChargeDisputeCreated, StripeChargeRefunded,
    StripeCheckoutCompleted, StripeCheckoutExpired, StripeConnectAccountUpdated,
    StripeConnectPaymentSucceeded, StripeEvent, StripeInvoicePaid, StripePaymentIntentFailed,
    StripeSubscriptionDeleted, StripeSubscriptionUpdated, SyncDispatcher,
};
```

**testing.rs re-export** (lines 50-51) — stays as-is since testing.rs already exists:
```rust
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
```
The `pub use crate::webhook::events::signed_webhook_payload` line in `testing.rs` line 132 must be removed (function moves to `testing.rs` directly).

---

### `ferro-stripe/Cargo.toml` — edit

**Analog:** current `ferro-stripe/Cargo.toml` (lines 1-37)

**Remove** (line 26):
```toml
ferro-events = { path = "../ferro-events", version = "0.2" }
```

**Add** (in `[dependencies]` section, after `async-trait`):
```toml
tracing = "0.1"
```

**Keep unchanged:** `ferro-queue`, `async-stripe` with `webhook-events` feature, `serde`, `serde_json`, `thiserror`, `chrono`, `dashmap`, `async-trait`, `hmac`, `sha2`, `hex`.

**Version bump:** Change line 3: `version = "0.4.0"` → `version = "0.5.0"` (D-14 in ROADMAP / SC-14 in RESEARCH).

---

### `framework/src/lib.rs` (framework re-exports) — extend

**Analog:** `framework/src/lib.rs` lines 93-100

**Current stripe re-export block** (lines 93-100):
```rust
#[cfg(feature = "stripe")]
pub use ferro_stripe::{
    account, checkout, refund, verify_webhook, CheckoutBuilder, CheckoutIntent,
    Error as StripeError, LineItem, MemoryProcessedLog, Mode, ProcessStripeWebhook,
    ProcessedEventLog, Stripe, StripeCheckoutCompleted, StripeConfig,
    StripeConnectPaymentSucceeded, StripeInvoicePaid, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
};
```

**Replace with:**
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

---

### `ferro-stripe/src/testing.rs` (utility) — extend

**Analog:** current `ferro-stripe/src/testing.rs` (lines 1-186)

**Receive `signed_webhook_payload`:** The function body (events.rs lines 235-250) moves here. Replace the `pub use crate::webhook::events::signed_webhook_payload` re-export on line 132 with the actual function definition.

**Pattern for existing mock functions** (lines 27-47):
```rust
// All mock functions use serde_json::json! macro, Utc::now().timestamp() for `created`,
// and return .to_string(). New mock functions for Phase 141 event types follow identical shape.
pub fn mock_checkout_expired_event(session_id: &str) -> String {
    serde_json::json!({
        "id": "evt_mock_checkout_expired",
        "object": "event",
        "api_version": "2023-10-16",
        "created": Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "type": "checkout.session.expired",
        "data": {
            "object": {
                "id": session_id,
                "object": "checkout.session",
                "status": "expired"
            }
        }
    })
    .to_string()
}
```

**`verify.rs` import fix** (RESEARCH.md Pitfall 5): After moving `signed_webhook_payload` to `testing.rs`, update `verify.rs` line 28:
```rust
// Old:
use crate::webhook::events::signed_webhook_payload;
// New:
use crate::testing::signed_webhook_payload;
```
This import is inside `#[cfg(test)]` so it compiles only in test builds.

---

### `ferro-stripe/tests/fixtures/stripe_events/*.json` (test fixtures) — create

**Analog:** `ferro-stripe/src/testing.rs` mock JSON structures (lines 27-125) and RESEARCH.md Pattern 4.

**Required structure for all fixtures** (RESEARCH.md Pattern 4 — minimal valid `stripe::Event` JSON):
```json
{
  "id": "evt_test_<event_slug>_001",
  "object": "event",
  "api_version": "2023-10-16",
  "created": 1700000000,
  "livemode": false,
  "pending_webhooks": 1,
  "request": null,
  "type": "<stripe.event.type>",
  "data": {
    "object": { ... }
  }
}
```

**10 fixture files required:**
1. `checkout_session_completed.json` — type: `checkout.session.completed`, object: `checkout.session`
2. `checkout_session_expired.json` — type: `checkout.session.expired`, object: `checkout.session`
3. `payment_intent_payment_failed.json` — type: `payment_intent.payment_failed`, object: `payment_intent`
4. `charge_refunded.json` — type: `charge.refunded`, object: `charge`
5. `charge_dispute_created.json` — type: `charge.dispute.created`, object: `dispute`
6. `account_updated.json` — type: `account.updated`, object: `account`
7. `customer_subscription_updated.json` — type: `customer.subscription.updated`, object: `subscription`
8. `customer_subscription_deleted.json` — type: `customer.subscription.deleted`, object: `subscription`
9. `invoice_paid.json` — type: `invoice.paid`, object: `invoice`
10. `payment_intent_succeeded_connect.json` — type: `payment_intent.succeeded`, object: `payment_intent`

**Note on `verify.rs` test helper:** The `minimal_event_json` function in `verify.rs` tests (lines 33-62) uses `invoiceitem` as the inner object. This pattern is fine for signature tests but fixtures for `parser_contract.rs` must have the correct `object` type matching the event (e.g., `checkout.session` for `checkout_session_completed.json`) so `EventObject` deserializes to the right variant.

---

### `ferro-stripe/tests/parser_contract.rs` (integration test) — create

**Analog:** `ferro-stripe/src/idempotency.rs` tests (lines 83-122) — tokio async tests, `use super::*` / specific imports, table-style assertions.

**Test structure pattern** (from idempotency.rs lines 84-122):
```rust
// Integration test file — no `mod tests {}` wrapper, tests are at top level.
// Import path: use ferro_stripe::{StripeXxx, StripeEvent};
// Load fixture: include_str!("fixtures/stripe_events/checkout_session_completed.json")
// Parse:        serde_json::from_str::<stripe::Event>(fixture_json)
// Convert:      StripeCheckoutCompleted::from_raw(&event)
// Assert:       per-field checks

#[test]
fn checkout_session_completed_parses_all_fields() {
    let raw = include_str!("fixtures/stripe_events/checkout_session_completed.json");
    let event: stripe::Event = serde_json::from_str(raw)
        .expect("fixture should deserialize as stripe::Event");
    let typed = ferro_stripe::StripeCheckoutCompleted::from_raw(&event)
        .expect("from_raw should return Some for matching event type");
    assert_eq!(typed.event_id, "evt_test_checkout_completed_001");
    assert_eq!(typed.session_id, "cs_test_001");
    // ... assert all fields
}
```

**Negative test pattern** (RESEARCH.md Pitfall 2 — cross-type guard):
```rust
#[test]
fn checkout_session_completed_does_not_parse_expired_event() {
    let raw = include_str!("fixtures/stripe_events/checkout_session_expired.json");
    let event: stripe::Event = serde_json::from_str(raw).unwrap();
    assert!(
        ferro_stripe::StripeCheckoutCompleted::from_raw(&event).is_none(),
        "StripeCheckoutCompleted::from_raw must return None for checkout.session.expired"
    );
}
```

---

### `ferro-stripe/tests/dispatcher.rs` (integration test) — create

**Analog:** `ferro-stripe/src/idempotency.rs` tests (lines 100-122) — tokio async tests with `Arc`, concurrent task patterns.

**Concurrent Arc test pattern** (idempotency.rs lines 100-122):
```rust
// Thread-safety test: two tokio tasks sharing same Arc<SyncDispatcher>
#[tokio::test]
async fn dispatcher_is_thread_safe_across_arc() {
    use std::sync::Arc;
    let dispatcher = Arc::new(
        ferro_stripe::SyncDispatcher::new()
            .on(|_: ferro_stripe::StripeInvoicePaid| async { Ok(()) })
    );
    let d1 = Arc::clone(&dispatcher);
    let d2 = Arc::clone(&dispatcher);
    // build minimal event...
    let t1 = tokio::spawn(async move { d1.dispatch(event1).await });
    let t2 = tokio::spawn(async move { d2.dispatch(event2).await });
    let (r1, r2) = tokio::join!(t1, t2);
    assert!(r1.unwrap().is_ok());
    assert!(r2.unwrap().is_ok());
}
```

**Error bubbling test pattern:**
```rust
#[tokio::test]
async fn dispatch_bubbles_handler_error() {
    let dispatcher = ferro_stripe::SyncDispatcher::new()
        .on(|_: ferro_stripe::StripeInvoicePaid| async {
            Err(ferro_stripe::Error::Stripe("test error".into()))
        });
    // dispatch a matching event, assert Err is returned
}
```

**Unknown event no-op pattern:**
```rust
#[tokio::test]
async fn dispatch_unknown_event_returns_ok() {
    let dispatcher = ferro_stripe::SyncDispatcher::new()
        .on(|_: ferro_stripe::StripeInvoicePaid| async { Ok(()) });
    // dispatch a checkout.session.completed event (no handler registered for it)
    // assert Ok(())
}
```

---

## Shared Patterns

### Builder Pattern (consuming self → Self)
**Source:** `ferro-stripe/src/checkout.rs` lines 62-130
**Apply to:** `SyncDispatcher::on()`
```rust
// Method signature convention:
pub fn method_name(mut self, param: Type) -> Self {
    self.field = value;
    self
}
```

### Error Type
**Source:** `ferro-stripe/src/error.rs` (full file)
**Apply to:** `SyncDispatcher::dispatch()` return type, `ProcessStripeWebhook::handle()` error mapping
```rust
// thiserror-derived enum, one per crate.
// Add HandlerFailed variant if needed:
#[error("stripe webhook handler failed: {0}")]
HandlerFailed(String),
```
Map to `ferro_queue::Error::JobFailed` in `ProcessStripeWebhook::handle()`:
```rust
.map_err(|e| ferro_queue::Error::JobFailed(e.to_string()))
```

### async_trait Usage
**Source:** `ferro-stripe/src/idempotency.rs` lines 29-45
**Apply to:** `ProcessStripeWebhook` Job impl
```rust
#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook { ... }
```

### Doc Comment Style
**Source:** `ferro-stripe/src/checkout.rs` lines 1-6 and `ferro-stripe/src/idempotency.rs` lines 1-27
**Apply to:** module doc comments in `sync.rs` and `queue.rs`
- Three-slash `///` for item docs, `//!` for module docs
- Scientific, no marketing language
- Include `# Errors` section when function can fail

### #[cfg(test)] Import Guard
**Source:** `ferro-stripe/src/webhook/verify.rs` lines 25-29
**Apply to:** `testing.rs` import in `verify.rs` tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::signed_webhook_payload;  // after move from events.rs
    ...
}
```

### Tokio Test with Arc
**Source:** `ferro-stripe/src/idempotency.rs` lines 100-122
**Apply to:** `tests/dispatcher.rs` thread-safety test
```rust
#[tokio::test]
async fn test_name() {
    let shared = Arc::new(/* ... */);
    let s1 = Arc::clone(&shared);
    let s2 = Arc::clone(&shared);
    let t1 = tokio::spawn(async move { /* use s1 */ });
    let t2 = tokio::spawn(async move { /* use s2 */ });
    let (r1, r2) = tokio::join!(t1, t2);
    // assert
}
```

---

## No Analog Found

All files have analogs or are self-modifying. No files require falling back to RESEARCH.md patterns only.

| File | Note |
|------|------|
| `tests/fixtures/stripe_events/*.json` | JSON content derived from RESEARCH.md Pattern 4 + `testing.rs` mock structures. No existing fixture files in codebase. |
| `StripeEvent` trait definition | New abstraction — no existing trait in codebase matches. RESEARCH.md §Pattern 1 is authoritative. |

---

## Critical Implementation Notes for Planner

1. **`signed_webhook_payload` move:** Must update `verify.rs` line 28 import AND remove the re-export line from `testing.rs` line 132 as part of the same task to avoid compile errors.

2. **`ProcessStripeWebhook` re-parse path:** `verify_webhook` requires a signature header. For re-parsing a stored raw body in queue execution, use `serde_json::from_str::<stripe::Event>(&self.raw_body)` directly — `stripe::Event` is `Deserialize`. No signature needed since the event was already verified at enqueue time.

3. **`SyncDispatcher: Send + Sync`:** The `BoxedHandler` type's `Send + Sync` bounds on the closure and the `Vec<BoxedHandler>` field together satisfy `Arc<SyncDispatcher>: Send + Sync`. No `unsafe impl` needed.

4. **tracing dep is new:** Not currently in `ferro-stripe/Cargo.toml` (verified from file read). Must be added before any `tracing::debug!` call in `sync.rs`.

5. **All event fixtures must match async-stripe's `Event` serde format:** The top-level JSON shape must include `id`, `object: "event"`, `api_version`, `created`, `livemode`, `pending_webhooks`, `request`, `type`, and `data.object`. Inner `data.object` must include the `object` discriminant field (e.g., `"object": "checkout.session"`).

---

## Metadata

**Analog search scope:** `ferro-stripe/src/`, `framework/src/lib.rs`
**Files scanned:** 10
**Pattern extraction date:** 2026-04-20

# Phase 141: Protocol Uplift - Research

**Researched:** 2026-04-20
**Domain:** ferro-stripe webhook dispatch — typed events, SyncDispatcher, ProcessStripeWebhook relocation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Remove `event_json: String` from all five existing event structs.
- **D-02:** Remove `impl ferro_events::Event` from all five structs.
- **D-03:** Add `event_id: String` to all existing structs (maps to `stripe::Event.id`).
- **D-04:** `StripeCheckoutCompleted` gains full field set: `event_id`, `session_id`, `payment_intent_id: Option<String>`, `amount_total_cents: i64`, `currency: String`, `metadata: HashMap<String, String>`, `customer_email: Option<String>`. Old `customer_id: Option<String>` removed.
- **D-05:** `StripeCheckoutExpired` — `event_id`, `session_id`, `metadata: HashMap<String, String>`.
- **D-06:** `StripePaymentIntentFailed` — `event_id`, `payment_intent_id`, `session_id: Option<String>`, `failure_code: Option<String>`, `failure_message: Option<String>`, `metadata: HashMap<String, String>`.
- **D-07:** `StripeChargeRefunded` — `event_id`, `charge_id`, `payment_intent_id: Option<String>`, `amount_refunded_cents: i64`, `metadata: HashMap<String, String>`.
- **D-08:** `StripeChargeDisputeCreated` — `event_id`, `charge_id`, `payment_intent_id: Option<String>`, `dispute_reason: String`, `amount_cents: i64`. No `metadata`.
- **D-09:** `StripeConnectAccountUpdated` — `event_id`, `account_id`, `charges_enabled: bool`, `payouts_enabled: bool`, `details_submitted: bool`.
- **D-10:** `pub trait StripeEvent: Send + Sync + 'static` with `fn from_raw(event: &stripe::Event) -> Option<Self> where Self: Sized`.
- **D-11:** All ten event structs implement `StripeEvent`.
- **D-12:** `metadata` fields use `HashMap<String, String>` (non-optional). Empty map when no metadata.
- **D-13:** `SyncDispatcher` in `webhook/sync.rs`. API exactly as design doc §3.4.
- **D-14:** `on()` is consuming builder (returns `Self`).
- **D-15:** Internal handler storage is type-erased. Exact representation at planner's discretion; must satisfy `Arc<SyncDispatcher>: Send + Sync`.
- **D-16:** `dispatch` contract: handler `Err` bubbles immediately; unknown events logged, return `Ok(())`.
- **D-17:** `ProcessStripeWebhook` moves to `webhook/queue.rs`, accepts `Arc<SyncDispatcher>`, `handle()` calls `self.dispatcher.dispatch(event)`.
- **D-18:** `ProcessStripeWebhook` struct fields: remove `event_json`, add `dispatcher: Arc<SyncDispatcher>`. Job stores `event_type: String` + `raw_body: String` for serde; `Arc<SyncDispatcher>` is not serialized.
- **D-19:** Remove `ferro-events` dep from `ferro-stripe/Cargo.toml`.
- **D-20:** Retain `ferro-queue` dep.
- **D-21:** Golden-JSON fixtures in `ferro-stripe/tests/fixtures/stripe_events/`.
- **D-22:** Parser-contract tests in `ferro-stripe/tests/` (integration test files).
- **D-23:** `SyncDispatcher` unit tests: Err bubbles; Ok completes; unknown event no-op; thread-safe across `Arc`.
- **D-24:** `ferro-stripe/src/lib.rs` re-exports updated: add `SyncDispatcher`, `StripeEvent`, 5 new event structs; remove dead re-exports; update `ProcessStripeWebhook` re-export path.
- **D-25:** `framework/src/lib.rs` re-exports updated: add `SyncDispatcher`, 5 new event types; update existing event struct re-exports.
- **D-26:** `signed_webhook_payload` moves from `webhook/events.rs` to `testing.rs`.

### Claude's Discretion

- Internal `SyncDispatcher` handler storage type (Vec vs HashMap keyed by event type string).
- Whether `dispatch` logs at `tracing::debug` or `tracing::warn` for unknown events.
- Exact `ProcessStripeWebhook` serialization shape — `event_type: String` + `raw_body: String` is the natural choice.

### Deferred Ideas (OUT OF SCOPE)

- `tracing` dep: if not already available transitively, add explicitly (minor operational concern left to planner).
- Phase 142 ferro-mcp parity — explicitly out of scope for this phase.
</user_constraints>

---

## Summary

Phase 141 reshapes the ferro-stripe webhook layer along three axes: typed events (remove `event_json`, add `event_id`, add full field coverage via `from_raw`), a new `SyncDispatcher` handler registry for synchronous dispatch, and relocation of `ProcessStripeWebhook` to `webhook/queue.rs` wired to `Arc<SyncDispatcher>`.

The primary technical challenge is `from_raw(event: &stripe::Event) -> Option<Self>`. The `stripe::Event` struct (async-stripe 0.41) exposes `event.data.object` as `EventObject` — a typed enum that pattern-matches to the correct async-stripe resource struct. This is the idiomatic access path, not re-serializing to JSON. Fields are accessed directly from the typed resource structs. The existing `parse_*` functions in `events.rs` use JSON string re-parsing as a workaround (because they took `&str`), which `from_raw` obsoletes.

`SyncDispatcher` requires type erasure: each `on::<E, H, Fut>` call wraps the handler into a `Box<dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>` stored in a `Vec`. The Vec approach is simpler than a HashMap; with 10 event types it is not a performance concern.

`ProcessStripeWebhook` has a serde boundary: `Arc<SyncDispatcher>` is not `Serialize/Deserialize`. The job must store `event_type: String` + `raw_body: String` as its persisted state, re-parse on `handle()`, then call `dispatcher.dispatch(parsed_event)`. The `dispatcher` field is provided at construction time (not from the serde payload) — this requires the `ferro_queue::Job` impl to accept the dispatcher via a constructor, not via deserialization.

**Primary recommendation:** Implement `from_raw` via `EventObject` pattern-matching on `event.data.object`, not via JSON re-serialization. Implement `SyncDispatcher` with a `Vec<Box<dyn Fn(...) -> Pin<Box<dyn Future<...>>> + Send + Sync>>`. For `ProcessStripeWebhook`, split the struct into persisted fields (`event_type`, `raw_body`) and runtime fields (`dispatcher: Arc<SyncDispatcher>`) using a constructor that takes the dispatcher separately.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Typed event parsing (`from_raw`) | ferro-stripe crate | — | Event shape knowledge lives in ferro-stripe; no HTTP layer involved |
| Synchronous dispatch (`SyncDispatcher`) | ferro-stripe crate | App webhook handler | Dispatcher registry lives in ferro; HTTP endpoint that calls it lives in app |
| Queue dispatch (`ProcessStripeWebhook`) | ferro-queue (job runtime) | ferro-stripe (job definition) | Job defined in ferro-stripe, executed by ferro-queue worker |
| Public API re-exports | ferro-stripe lib.rs + framework lib.rs | — | Both surfaces need updating |
| Test fixtures | ferro-stripe/tests/ | — | Integration tests, not app-level |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-stripe | 0.41.0 | Stripe API types and webhook verification | Already a dep; `EventObject` enum provides typed event data access |
| tokio | 1 | Async runtime for `SyncDispatcher::dispatch` | Already in dev-deps; required for `#[tokio::test]` in dispatch tests |
| tracing | 0.1 | Logging unknown events in `dispatch` | Standard across ferro crates; ferro-queue already uses it |
| std::collections::HashMap | stdlib | `metadata` fields | Matches `stripe::Metadata` type alias definition |
| std::pin::Pin + std::future::Future | stdlib | Type-erased async handler storage | Required for the `Box<dyn Fn(...) -> Pin<Box<dyn Future<...>>>>` pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| async-trait | 0.1 | Async trait bounds | Not needed for `StripeEvent` (not async), but already in deps |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `EventObject` pattern-match | Re-serialize event to JSON | JSON re-parse is simpler to write but slower and loses type safety; `EventObject` is the authoritative source |
| Vec handler storage in `SyncDispatcher` | HashMap keyed by `EventType` | HashMap is O(1) lookup but adds complexity; Vec with 10 max entries has negligible cost |
| Constructor-injected `Arc<SyncDispatcher>` in job | Serialize dispatcher | Dispatchers are not serializable; constructor injection is the only correct approach |

**Installation — add tracing dep:**
```bash
# In ferro-stripe/Cargo.toml [dependencies]
tracing = "0.1"
```

**Remove ferro-events dep:**
```toml
# DELETE this line from ferro-stripe/Cargo.toml:
ferro-events = { path = "../ferro-events", version = "0.2" }
```

---

## Architecture Patterns

### System Architecture Diagram

```
HTTP webhook endpoint (app)
        |
        v
verify_webhook(raw_body, sig, secret) -> stripe::Event
        |
        +--- [sync path] SyncDispatcher::dispatch(event) --->  handlers -> Ok/Err
        |                                                              |
        |                                              Err bubbles to HTTP 500
        |
        +--- [queue path] enqueue ProcessStripeWebhook { event_type, raw_body }
                                        |
                                        v (ferro-queue worker)
                              parse raw_body -> stripe::Event
                                        |
                                        v
                              SyncDispatcher::dispatch(event) -> Ok/Err (logged)
```

### Recommended Project Structure (ferro-stripe/src/)

```
ferro-stripe/src/
  webhook/
    mod.rs          # pub use verify_webhook; pub use sync::SyncDispatcher; pub use events::{...}
    events.rs       # 10 typed event structs + StripeEvent trait (no ferro_events::Event impls)
    sync.rs         # SyncDispatcher (Phase 141 fills this stub)
    queue.rs        # ProcessStripeWebhook (Phase 141 fills this stub)
    verify.rs       # unchanged
  testing.rs        # signed_webhook_payload moves here from events.rs

ferro-stripe/tests/
  fixtures/
    stripe_events/
      checkout_session_completed.json
      checkout_session_expired.json
      payment_intent_payment_failed.json
      charge_refunded.json
      charge_dispute_created.json
      account_updated.json
      customer_subscription_updated.json
      customer_subscription_deleted.json
      invoice_paid.json
      payment_intent_succeeded_connect.json
  parser_contract.rs   # from_raw field-by-field tests
  dispatcher.rs        # SyncDispatcher unit tests
```

### Pattern 1: StripeEvent Trait and from_raw via EventObject

**What:** `from_raw` matches on `event.data.object` (the typed `EventObject` enum from async-stripe) to extract fields directly without JSON re-parsing.

**When to use:** Every `StripeEvent` implementation.

```rust
// Source: async-stripe 0.41 webhook_events.rs — EventObject enum
// event.data.object is EventObject; match the variant to extract typed fields.

use std::collections::HashMap;
use crate::webhook::events::StripeEvent;

pub struct StripeCheckoutCompleted {
    pub event_id: String,
    pub session_id: String,
    pub payment_intent_id: Option<String>,
    pub amount_total_cents: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
    pub customer_email: Option<String>,
}

impl StripeEvent for StripeCheckoutCompleted {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CheckoutSessionCompleted {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::CheckoutSession(session) => {
                let payment_intent_id = session
                    .payment_intent
                    .as_ref()
                    .map(|e| e.id().to_string());
                Some(Self {
                    event_id: event.id.to_string(),
                    session_id: session.id.to_string(),
                    payment_intent_id,
                    amount_total_cents: session.amount_total.unwrap_or(0),
                    currency: session.currency
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                    metadata: session.metadata.clone().unwrap_or_default(),
                    customer_email: session.customer_email.clone(),
                })
            }
            _ => None,
        }
    }
}
```

### Pattern 2: SyncDispatcher Type-Erased Handler Storage

**What:** Store handlers as `Box<dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>`. Each `on::<E, H, Fut>` call wraps the user handler to call `E::from_raw(&event)` internally.

**When to use:** `SyncDispatcher::on()` implementation.

```rust
// Source: CONTEXT.md D-13/D-15 + design doc §3.4
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::Error;

type BoxedHandler = Box<
    dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>
        + Send
        + Sync,
>;

pub struct SyncDispatcher {
    handlers: Vec<BoxedHandler>,
}

impl SyncDispatcher {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn on<E, H, Fut>(mut self, handler: H) -> Self
    where
        E: crate::webhook::events::StripeEvent,
        H: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.handlers.push(Box::new(move |event: stripe::Event| {
            let handler = Arc::clone(&handler);
            let typed = E::from_raw(&event);
            Box::pin(async move {
                if let Some(e) = typed {
                    handler(e).await
                } else {
                    Ok(())
                }
            })
        }));
        self
    }

    pub async fn dispatch(&self, event: stripe::Event) -> Result<(), Error> {
        let mut matched = false;
        for handler in &self.handlers {
            // Each handler calls from_raw internally and skips if None.
            // We run all handlers that match this event type.
            // NOTE: to detect "unknown event", track whether any from_raw returned Some.
            // Simpler: log at the call site based on event.type_ vs registered types.
            handler(event.clone()).await?;
            matched = true;  // simplified — see pitfall note below
        }
        if !matched {
            tracing::debug!(event_type = ?event.type_, "unregistered stripe event — skipping");
        }
        Ok(())
    }
}
```

**CRITICAL PITFALL:** The "unknown event" detection above is simplified. Since each handler calls `from_raw` internally and silently returns `Ok(())` for non-matching events, all handlers run for every event. An "unknown event" is one where ALL handlers returned `Ok(())` without actually processing (all `from_raw` returned `None`). The correct implementation must track whether any handler's `from_raw` returned `Some`. See the Common Pitfalls section.

### Pattern 3: ProcessStripeWebhook Split — Serializable State vs Runtime Dependency

**What:** `ProcessStripeWebhook` must implement `serde::Serialize + Deserialize` for job persistence, but `Arc<SyncDispatcher>` is not serializable. Solution: store only serializable fields (`event_type`, `raw_body`) in the struct; receive `dispatcher` via a constructor that is called at enqueue time (not restored from serde).

**When to use:** `ProcessStripeWebhook` implementation.

```rust
// Source: CONTEXT.md D-17/D-18, ferro_queue::Job pattern
use std::sync::Arc;
use crate::webhook::sync::SyncDispatcher;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    /// The Stripe event type string (e.g. "checkout.session.completed").
    pub event_type: String,
    /// Raw Stripe webhook body for re-parsing on execution.
    pub raw_body: String,
    /// Connected account ID for Connect webhooks (None for platform webhooks).
    pub connect_account_id: Option<String>,
    /// Runtime-only: not serialized. Set via new() before enqueueing.
    #[serde(skip)]
    pub dispatcher: Option<Arc<SyncDispatcher>>,
}

impl ProcessStripeWebhook {
    pub fn new(
        event_type: String,
        raw_body: String,
        connect_account_id: Option<String>,
        dispatcher: Arc<SyncDispatcher>,
    ) -> Self {
        Self {
            event_type,
            raw_body,
            connect_account_id,
            dispatcher: Some(dispatcher),
        }
    }
}

#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let dispatcher = self.dispatcher.as_ref()
            .expect("ProcessStripeWebhook requires dispatcher — use ProcessStripeWebhook::new()");
        let event = crate::verify::parse_event(&self.raw_body)
            .map_err(|e| ferro_queue::Error::JobFailed(e.to_string()))?;
        dispatcher.dispatch(event).await
            .map_err(|e| ferro_queue::Error::JobFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "ProcessStripeWebhook"
    }
}
```

**Note on `#[serde(skip)]`:** Fields with `#[serde(skip)]` are excluded from serialization. On deserialization they are initialized to their `Default` value — `Option<Arc<SyncDispatcher>>` defaults to `None`. This is correct for queue persistence (the dispatcher is not persisted), but the worker must re-inject the dispatcher before calling `handle()` — or the job must receive the dispatcher when the worker calls it. The exact injection mechanism depends on the ferro-queue worker API. See Open Questions.

### Pattern 4: Golden-JSON Fixture Structure

**What:** Minimal valid JSON that satisfies `stripe::Webhook::construct_event` parsing.

**When to use:** Each fixture in `tests/fixtures/stripe_events/`.

```json
{
  "id": "evt_test_checkout_completed_001",
  "object": "event",
  "api_version": "2023-10-16",
  "created": 1700000000,
  "livemode": false,
  "pending_webhooks": 1,
  "request": null,
  "type": "checkout.session.completed",
  "data": {
    "object": {
      "id": "cs_test_001",
      "object": "checkout.session",
      "amount_total": 1000,
      "currency": "usd",
      "customer_email": "test@example.com",
      "metadata": { "order_id": "order_42" },
      "payment_intent": "pi_test_001",
      "mode": "payment",
      "payment_status": "paid",
      "status": "complete",
      "success_url": "https://example.com/ok",
      "cancel_url": "https://example.com/cancel"
    }
  }
}
```

### Anti-Patterns to Avoid

- **Re-serializing `stripe::Event` to JSON in `from_raw`:** The `EventObject` enum provides direct typed access. Re-serialization loses type safety and requires `serde_json` round-trips.
- **Implementing `ferro_events::Event` on event structs:** Explicitly removed by D-02. Do not add it back.
- **Making `SyncDispatcher` methods `&mut self`:** The builder pattern requires consuming `self` → `Self` for `on()`. `dispatch` takes `&self` (reads the handler vec).
- **Putting `Arc<SyncDispatcher>` in the serialized state of `ProcessStripeWebhook`:** Not serializable; use `#[serde(skip)]` + `Option<Arc<...>>`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Webhook signature verification | Custom HMAC | `stripe::Webhook::construct_event` (already in `verify.rs`) | Already implemented; don't duplicate |
| Event type discrimination | String matching in dispatch | `stripe::EventType` enum + `EventObject` pattern-match | async-stripe provides typed discrimination |
| Async trait bounds on handlers | Custom trait | Standard Rust `Fn(E) -> Fut + Send + Sync + 'static` | No `async_trait` needed for handler registration |

**Key insight:** The `EventObject` enum in async-stripe 0.41 eliminates the need for any JSON manipulation in `from_raw`. The typed variant match is both safer and faster than the current `serde_json::from_str` approach in `events.rs`.

---

## Common Pitfalls

### Pitfall 1: Unknown Event Detection in SyncDispatcher

**What goes wrong:** With the Vec handler approach, every handler runs for every event. Handlers whose `from_raw` returns `None` silently return `Ok(())`. The dispatcher cannot distinguish "event matched at least one handler" from "event matched no handlers" without tracking this.

**Why it happens:** The type erasure wrapping calls `E::from_raw(&event)` internally, hiding the match result from the outer dispatch loop.

**How to avoid:** Two valid approaches:
1. Track match inside the erased handler: change the `BoxedHandler` return type to `Result<bool, Error>` where `bool = was_invoked`. The outer loop collects whether any `true` was returned.
2. Log at the `EventType` level before dispatch: check `event.type_` against a registered type set.

Approach 1 is cleaner. Change `BoxedHandler` return to include a "did this handler match?" flag.

**Warning signs:** Dispatcher silently ignores all events regardless of type. Regression: write a test that registers a handler for `checkout.session.completed`, dispatch a `checkout.session.expired` event, and assert the handler was NOT called.

### Pitfall 2: from_raw Type Guard Omission

**What goes wrong:** `from_raw` on `StripeCheckoutCompleted` pattern-matches on `EventObject::CheckoutSession`, which could succeed for both `checkout.session.completed` AND `checkout.session.expired` since both have `EventObject::CheckoutSession` as their data object.

**Why it happens:** The `EventObject` discriminant reflects the resource type, not the event type. `checkout.session.completed` and `checkout.session.expired` both carry `CheckoutSession` data.

**How to avoid:** Always check `event.type_` first before pattern-matching on `event.data.object`:

```rust
if event.type_ != stripe::EventType::CheckoutSessionCompleted {
    return None;
}
match &event.data.object {
    stripe::EventObject::CheckoutSession(session) => { ... }
    _ => None,
}
```

**Warning signs:** Parser-contract test for `checkout.session.expired` incorrectly parses as `StripeCheckoutCompleted`. Add cross-type negative tests.

### Pitfall 3: ProcessStripeWebhook Dispatcher Injection Gap

**What goes wrong:** `ProcessStripeWebhook` is deserialized from the queue store with `dispatcher: None` (because `#[serde(skip)]` defaults it). If the ferro-queue worker calls `handle()` directly on the deserialized struct, `dispatcher.as_ref().expect(...)` panics.

**Why it happens:** The job's serde boundary and its runtime dependency are in tension. Queue persistence cannot store `Arc<SyncDispatcher>`.

**How to avoid:** Two options:
1. The app webhook handler passes `dispatcher` to the job constructor (`ProcessStripeWebhook::new(...)`) at enqueue time. The ferro-queue worker only ever executes jobs that were enqueued with a live dispatcher.
2. Register the dispatcher in a static/global registry keyed by job name (not recommended — global state).

Option 1 is the correct approach. The dispatcher is always present when the job is constructed; the `#[serde(skip)]` field means the persisted representation doesn't carry it. This is fine as long as the worker process has the dispatcher available when it calls `handle()` — which it does because the dispatcher is constructed at app startup, and the worker runs in the same process.

**Warning signs:** Panic with "requires dispatcher — use ProcessStripeWebhook::new()" in worker logs. Verify the test in D-23 constructs jobs with `ProcessStripeWebhook::new(...)` not struct literal syntax.

### Pitfall 4: Charge.payment_intent Expandable Extraction

**What goes wrong:** `Charge.payment_intent` is `Option<Expandable<PaymentIntent>>`. `Expandable` is either `Id(PaymentIntentId)` or `Object(Box<PaymentIntent>)`. Webhook payloads typically contain IDs only (not expanded objects). Accessing `.id()` works for both variants.

**Why it happens:** async-stripe's `Expandable` type is untagged union; forgetting to call `.id()` gives a type mismatch.

**How to avoid:**
```rust
let payment_intent_id = charge.payment_intent.as_ref().map(|e| e.id().to_string());
```

### Pitfall 5: verify.rs Still Imports signed_webhook_payload from events.rs

**What goes wrong:** After `signed_webhook_payload` moves to `testing.rs`, `verify.rs` test module still imports `use crate::webhook::events::signed_webhook_payload`. This causes a compile error.

**Why it happens:** Move operation without updating import.

**How to avoid:** Update the import in `verify.rs` test module to `use crate::testing::signed_webhook_payload` (behind `#[cfg(test)]`).

### Pitfall 6: currency Field — stripe::Currency Display

**What goes wrong:** `CheckoutSession.currency` is `Option<stripe::Currency>` (an enum), not `Option<String>`. Converting to the "usd" string requires `.to_string()` (which calls the `Display` impl).

**Why it happens:** Confusion between the async-stripe `Currency` enum and a raw string.

**How to avoid:**
```rust
currency: session.currency.map(|c| c.to_string()).unwrap_or_default(),
```

---

## Code Examples

### StripeSubscriptionUpdated (reshaped existing struct)

```rust
// Source: CONTEXT.md D-01/D-03 + async-stripe 0.41 Subscription struct
pub struct StripeSubscriptionUpdated {
    pub event_id: String,
    pub subscription_id: String,
    pub customer_id: String,
}

impl StripeEvent for StripeSubscriptionUpdated {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::CustomerSubscriptionUpdated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Subscription(sub) => Some(Self {
                event_id: event.id.to_string(),
                subscription_id: sub.id.to_string(),
                customer_id: sub.customer.id().to_string(),
            }),
            _ => None,
        }
    }
}
```

### StripePaymentIntentFailed — session_id extraction note

`PaymentIntent` in async-stripe 0.41 has no `session_id` field. The `session_id: Option<String>` for `StripePaymentIntentFailed` must be extracted from `payment_intent.metadata` if the app stores it there (e.g., `metadata.get("session_id")`), or left as `None` when not present:

```rust
impl StripeEvent for StripePaymentIntentFailed {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentPaymentFailed {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => {
                let failure_code = pi.last_payment_error.as_ref()
                    .and_then(|e| e.code.as_ref())
                    .map(|c| c.to_string());
                let failure_message = pi.last_payment_error.as_ref()
                    .and_then(|e| e.message.clone());
                let session_id = pi.metadata.get("checkout_session_id").cloned();
                Some(Self {
                    event_id: event.id.to_string(),
                    payment_intent_id: pi.id.to_string(),
                    session_id,
                    failure_code,
                    failure_message,
                    metadata: pi.metadata.clone(),
                })
            }
            _ => None,
        }
    }
}
```

### StripeChargeDisputeCreated — Dispute.charge extraction

```rust
// Source: async-stripe 0.41 Dispute struct
// Dispute.charge is Expandable<Charge> (non-optional).
// Dispute.payment_intent is Option<Expandable<PaymentIntent>>.
impl StripeEvent for StripeChargeDisputeCreated {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::ChargeDisputeCreated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::Dispute(dispute) => Some(Self {
                event_id: event.id.to_string(),
                charge_id: dispute.charge.id().to_string(),
                payment_intent_id: dispute.payment_intent.as_ref()
                    .map(|e| e.id().to_string()),
                dispute_reason: dispute.reason.clone(),
                amount_cents: dispute.amount,
            }),
            _ => None,
        }
    }
}
```

### SyncDispatcher — Corrected Unknown Event Detection

```rust
// BoxedHandler returns (was_invoked: bool, result: Result<(), Error>)
// so the outer loop can detect no-match.
type BoxedHandler = Box<
    dyn Fn(stripe::Event) -> Pin<Box<dyn Future<Output = (bool, Result<(), Error>)> + Send>>
        + Send + Sync,
>;

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

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `event_json: String` in event structs | Fully-parsed typed fields + `from_raw` | Phase 141 | Consumers get typed fields; no re-parsing in handlers |
| `impl ferro_events::Event` on stripe structs | `StripeEvent` marker trait | Phase 141 | Decouples ferro-stripe from ferro-events for webhook dispatch |
| Inline match in `ProcessStripeWebhook::handle` | Delegate to `SyncDispatcher::dispatch` | Phase 141 | Single handler registry shared by sync and queue paths |
| `ferro_events::Event::dispatch_sync()` call | `SyncDispatcher::dispatch(event)` | Phase 141 | Synchronous dispatch without the ferro-events bus |

**Deprecated/outdated after this phase:**
- `parse_subscription_updated`, `parse_checkout_completed`, etc. (private fns in `events.rs`): removed, replaced by `StripeEvent::from_raw`
- `impl ferro_events::Event for StripeXxx`: removed from all five existing structs
- `event_json: String` field: removed from all five existing structs

---

## Open Questions

1. **ProcessStripeWebhook dispatcher injection in ferro-queue worker**
   - What we know: `#[serde(skip)]` fields deserialize to `Default`, which for `Option<Arc<SyncDispatcher>>` is `None`.
   - What's unclear: Does ferro-queue's worker call `Job::handle()` directly on the deserialized struct? If so, how does the worker get the `Arc<SyncDispatcher>` to inject before calling `handle()`?
   - Recommendation: Check `ferro-queue/src/lib.rs` for the worker execution path. If the worker calls `handle()` directly on the deserialized job, the planner must design a registry (e.g., `Arc<SyncDispatcher>` stored in `AppState`) that the job retrieves at handle-time rather than accepting via constructor. Alternatively, accept that queue-path dispatch requires app-level glue that isn't fully encapsulated by the job struct.

2. **`payment_intent.payment_failed` EventType variant name**
   - What we know: async-stripe 0.41 `EventType` enum has `PaymentIntentPaymentFailed` (serde rename `payment_intent.payment_failed`) — confirmed in `webhook_events.rs` scan.
   - What's unclear: Verify the exact `serde` name against Stripe's docs to ensure fixture JSON uses the correct type string.
   - Recommendation: Use `"payment_intent.payment_failed"` in fixtures (the documented Stripe event type string).

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies beyond the existing cargo workspace. All needed crates are already present.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio-test |
| Config file | none (cargo test) |
| Quick run command | `cargo test -p ferro-stripe` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | Existing event structs carry full parsed fields, no `event_json` | unit (compile check) | `cargo test -p ferro-stripe` | Wave 0 — needs events.rs rewrite |
| SC-2 | `StripeEvent` marker trait + `from_raw` on all 10 structs | unit | `cargo test -p ferro-stripe` | Wave 0 |
| SC-3 | `SyncDispatcher` API: `new`, `on`, `dispatch` | unit | `cargo test -p ferro-stripe -- dispatcher` | Wave 0 — `tests/dispatcher.rs` |
| SC-4 | `dispatch` returns Err on handler error; unknown events Ok | unit | `cargo test -p ferro-stripe -- dispatcher` | Wave 0 |
| SC-5 | `ProcessStripeWebhook` in `webhook/queue.rs`, accepts `Arc<SyncDispatcher>` | compile | `cargo test -p ferro-stripe` | Wave 0 — `webhook/queue.rs` rewrite |
| SC-6 | Doc comments guide sync vs queue path | manual | — | Wave 0 |
| SC-7..11 | Five new event types with correct fields | integration | `cargo test -p ferro-stripe -- parser_contract` | Wave 0 — `tests/parser_contract.rs` |
| SC-12 | Golden-JSON fixtures in `tests/fixtures/stripe_events/` | manual/fixture | `cargo test -p ferro-stripe -- parser_contract` | Wave 0 — fixture files |
| SC-13 | Unit tests: Err bubbles, Ok passes, unknown no-op, thread-safe | unit | `cargo test -p ferro-stripe -- dispatcher` | Wave 0 — `tests/dispatcher.rs` |
| SC-14 | `ferro-stripe 0.5.0`, CI green | release | `cargo test --all-features` | post-implementation |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-stripe`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-stripe/tests/parser_contract.rs` — covers SC-7..11, SC-12
- [ ] `ferro-stripe/tests/dispatcher.rs` — covers SC-3, SC-4, SC-13
- [ ] `ferro-stripe/tests/fixtures/stripe_events/*.json` — 10 fixture files (one per event type)
- [ ] `ferro-stripe/src/webhook/events.rs` rewrite — SC-1, SC-2
- [ ] `ferro-stripe/src/webhook/sync.rs` fill — SC-3, SC-4
- [ ] `ferro-stripe/src/webhook/queue.rs` fill — SC-5

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `stripe::Webhook::construct_event` (signature + timestamp validation) already in `verify.rs` — unchanged |
| V6 Cryptography | yes | HMAC-SHA256 via `stripe::Webhook` — already implemented in `verify.rs` — do not hand-roll |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Replayed webhook events | Spoofing/Tampering | `verify_webhook` checks 5-minute timestamp window + HMAC (unchanged) |
| Duplicate event processing | Elevation of Privilege | `ProcessedEventLog::try_mark_processed` (Phase 140, already shipped) |
| Handler panic exposes 500 to Stripe (causing retry storm) | Denial of Service | `dispatch` returns `Err` on handler error; Stripe retries — idempotency log prevents duplicate processing |

---

## Sources

### Primary (HIGH confidence)
- async-stripe 0.41.0 source (`~/.cargo/registry/src/.../async-stripe-0.41.0/`) — `Event` struct, `EventObject` enum, `NotificationEventData`, `Expandable`, `stripe::Metadata`, `ApiErrors`, `Charge`, `Dispute`, `Account`, `PaymentIntent`, `CheckoutSession` field sets — all verified by direct source read
- `ferro-stripe/src/webhook/events.rs` — existing parse patterns and `ProcessStripeWebhook` implementation
- `ferro-stripe/src/lib.rs` — current re-exports
- `ferro-stripe/Cargo.toml` — current deps (ferro-events present, tracing absent)
- `framework/src/lib.rs` — current stripe re-exports under `#[cfg(feature = "stripe")]`
- CONTEXT.md (D-01..D-26) — all locked decisions

### Secondary (MEDIUM confidence)
- Design doc `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` §3.3/§3.4/§3.6 — field sets, SyncDispatcher API, doc comment text

### Tertiary (LOW confidence)
- None.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `payment_intent.payment_failed` is the correct Stripe event type string for `StripePaymentIntentFailed` | Code Examples | Wrong fixture JSON type string; `from_raw` never matches; tests fail |
| A2 | `session_id` in `StripePaymentIntentFailed` is extracted from `pi.metadata` (key `"checkout_session_id"`) since `PaymentIntent` has no native session_id field | Code Examples | If the app uses a different metadata key, `session_id` is always `None`; fixture must reflect real app convention |

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against async-stripe 0.41 source in cargo registry
- Architecture: HIGH — all decisions locked in CONTEXT.md; `EventObject` access pattern verified from source
- Pitfalls: HIGH — derived from direct source inspection of async-stripe types and existing code structure

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (async-stripe types are stable within 0.41.x)

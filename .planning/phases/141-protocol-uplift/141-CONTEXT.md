# Phase 141: Protocol Uplift - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Drop `event_json: String` from all five existing typed event structs and remove their `ferro_events::Event` implementations. Ship `SyncDispatcher` as the default webhook handler registry in `webhook/sync.rs`. Relocate `ProcessStripeWebhook` to `webhook/queue.rs` wired to `Arc<SyncDispatcher>`. Add five new event types with fully-parsed fields. Provide golden-JSON fixtures with parser-contract tests. Release `ferro-stripe 0.5.0`.

This phase does NOT modify ferro-mcp (that is Phase 142).

</domain>

<decisions>
## Implementation Decisions

### Event Struct Reshaping (existing 5 structs)

- **D-01:** Remove `event_json: String` field from all five existing event structs (`StripeCheckoutCompleted`, `StripeSubscriptionUpdated`, `StripeSubscriptionDeleted`, `StripeInvoicePaid`, `StripeConnectPaymentSucceeded`).
- **D-02:** Remove `impl ferro_events::Event` from all five structs — they do not implement `ferro_events::Event` after this phase.
- **D-03:** Add `event_id: String` to all existing structs (maps to `stripe::Event.id`). Keep all other existing typed fields.
- **D-04:** `StripeCheckoutCompleted` gains the full design doc §3.3 field set: `event_id`, `session_id`, `payment_intent_id: Option<String>`, `amount_total_cents: i64`, `currency: String`, `metadata: HashMap<String, String>`, `customer_email: Option<String>`. The old `customer_id: Option<String>` field is replaced by this expanded set.

### New Event Types (5 new structs)

- **D-05:** `StripeCheckoutExpired` — fields: `event_id: String`, `session_id: String`, `metadata: HashMap<String, String>`.
- **D-06:** `StripePaymentIntentFailed` — fields: `event_id: String`, `payment_intent_id: String`, `session_id: Option<String>`, `failure_code: Option<String>`, `failure_message: Option<String>`, `metadata: HashMap<String, String>`.
- **D-07:** `StripeChargeRefunded` — fields: `event_id: String`, `charge_id: String`, `payment_intent_id: Option<String>` (Option — older charges may lack payment intent; design doc §3.3 wins over roadmap SC9 which shows non-optional), `amount_refunded_cents: i64`, `metadata: HashMap<String, String>`.
- **D-08:** `StripeChargeDisputeCreated` — fields: `event_id: String`, `charge_id: String`, `payment_intent_id: Option<String>`, `dispute_reason: String`, `amount_cents: i64`. No `metadata` (design doc §3.3 omits it here).
- **D-09:** `StripeConnectAccountUpdated` — fields: `event_id: String`, `account_id: String`, `charges_enabled: bool`, `payouts_enabled: bool`, `details_submitted: bool`.

### StripeEvent Marker Trait

- **D-10:** Define `pub trait StripeEvent: Send + Sync + 'static` with one method: `fn from_raw(event: &stripe::Event) -> Option<Self> where Self: Sized;` — exactly as design doc §3.3. `stripe::Event` is from `async-stripe` (already a dep, already used in `verify_webhook`).
- **D-11:** All ten event structs (5 existing reshaped + 5 new) implement `StripeEvent`.

### Metadata Field Type

- **D-12:** `metadata` fields use `HashMap<String, String>` (non-optional). If the Stripe event has no metadata, the field is an empty `HashMap`. This avoids Option unwrapping in handlers and matches the design doc §3.3 literal.

### SyncDispatcher

- **D-13:** `SyncDispatcher` lives in `webhook/sync.rs`. Public API exactly as design doc §3.4:
  ```rust
  pub fn new() -> Self
  pub fn on<E, H, Fut>(self, handler: H) -> Self
  pub async fn dispatch(&self, event: stripe::Event) -> Result<(), Error>
  ```
- **D-14:** `on()` is consuming builder (returns `Self`) — consistent with `CheckoutBuilder` pattern from Phase 140.
- **D-15:** Internal storage: type-erased handler vec. Each `on::<E, H, Fut>` call wraps the handler into a `Box<dyn Fn(stripe::Event) -> Pin<Box<dyn Future<...>>> + Send + Sync>` that calls `E::from_raw(&event)` internally. Planner decides the exact representation; must satisfy `Arc<SyncDispatcher>: Send + Sync`.
- **D-16:** `dispatch` contract: handler returning `Err` bubbles up immediately; unknown event types (no `from_raw` returns `Some`) are logged and return `Ok(())`. No implicit retry inside dispatch.

### Queue Path

- **D-17:** `ProcessStripeWebhook` moves from `webhook/events.rs` to `webhook/queue.rs`. Accepts `Arc<SyncDispatcher>` (replaces the inline match block). `handle()` calls `self.dispatcher.dispatch(event)`.
- **D-18:** `ProcessStripeWebhook` struct fields change: remove `event_json: String`, add `dispatcher: Arc<SyncDispatcher>`. The job deserializes the event from a stored raw JSON string (or receives the event type + raw body). Planner resolves exact serialization shape — `ferro_queue::Job` requires `serde::Serialize/Deserialize` for job persistence; `Arc<SyncDispatcher>` is not serializable, so the job likely stores `event_type + raw_body` and re-parses on execution.

### Dependency Cleanup

- **D-19:** Remove `ferro-events = { path = "../ferro-events" }` from `ferro-stripe/Cargo.toml` — no remaining usage after this phase removes the `Event` impls.
- **D-20:** `ferro-queue` dep is retained (still needed for `ProcessStripeWebhook`).

### Test Fixtures and Parser Contract Tests

- **D-21:** Golden-JSON fixtures in `ferro-stripe/tests/fixtures/stripe_events/`, one file per event type (e.g., `checkout_session_completed.json`, `charge_refunded.json`). JSON must be deserializable as `stripe::Event` (async-stripe's format).
- **D-22:** Parser-contract test for each event type: deserialize fixture → call `from_raw` → assert each field matches expected value. Tests live in `ferro-stripe/tests/` (integration test files, not unit tests in `events.rs`).
- **D-23:** `SyncDispatcher` unit tests: Err handler bubbles up; Ok path completes; unknown event no-op; thread-safe across `Arc` (2 tokio tasks sharing same dispatcher instance).

### Public API Updates

- **D-24:** `lib.rs` re-exports: add `SyncDispatcher`, `StripeEvent` trait, all 5 new event structs. Remove dead re-exports. Update `ProcessStripeWebhook` re-export path (now from `webhook::queue`).
- **D-25:** `framework/src/lib.rs` re-exports: add `SyncDispatcher` and the 5 new event types; update existing event struct re-exports to reflect new fields.

### testing.rs Relocation

- **D-26:** Move `signed_webhook_payload` from `webhook/events.rs` to `testing.rs` (under `#[cfg(any(test, feature = "test-helpers"))]`). It was test infrastructure living in the wrong file.

### Claude's Discretion

- Exact internal `SyncDispatcher` handler storage type (Vec vs HashMap keyed by event type string) — either works; HashMap is an optimization but Vec is simpler and correct.
- Whether `dispatch` logs at `tracing::debug` or `tracing::warn` for unknown events — follow existing crate conventions.
- Exact `ProcessStripeWebhook` serialization shape for storing the raw event — `event_type: String` + `raw_body: String` is the natural choice since `stripe::Event` deserialization is already available.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Doc (authoritative)
- `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` — Full capability-axis refactor design. §3.3 defines all event struct field sets. §3.4 defines `SyncDispatcher` API. §3.6 defines queue path relocation. §4.6 is the breaking-change ledger for CHANGELOG content.

### Roadmap
- `.planning/ROADMAP.md` §"Phase 141: Protocol uplift" — 14 success criteria. Note: SC9 (`StripeChargeRefunded.payment_intent_id`) shows `String` but design doc §3.3 shows `Option<String>` — **design doc wins**.

### Existing Source (read before touching)
- `ferro-stripe/src/webhook/events.rs` — current event structs with `event_json` and `ferro_events::Event` impls to be removed
- `ferro-stripe/src/webhook/sync.rs` — empty stub, Phase 141 fills this
- `ferro-stripe/src/webhook/queue.rs` — empty stub, Phase 141 fills this
- `ferro-stripe/src/webhook/verify.rs` — `verify_webhook` returns `stripe::Event`; shows how `stripe::Event` is used
- `ferro-stripe/src/lib.rs` — current re-exports to update
- `ferro-stripe/Cargo.toml` — `ferro-events` dep to remove; `async-stripe` already present with `webhook-events` feature

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `stripe::Event` from `async-stripe` is already used in `verify_webhook` — the `from_raw(event: &stripe::Event)` signature is already the natural fit.
- `signed_webhook_payload` in `events.rs` — moves to `testing.rs` this phase.
- `MemoryProcessedLog` (DashMap-backed) — reusable for test setup.
- `CheckoutBuilder` builder pattern (consuming `with_*` → `Self`) — `SyncDispatcher::on()` follows this same consuming builder convention.

### Established Patterns
- Builder pattern: consuming `self → Self` methods (see `CheckoutBuilder`).
- Error type: `thiserror`-derived `ferro_stripe::Error` enum — add new variants if needed (e.g., `Error::HandlerFailed`).
- `#[async_trait]` used in `ProcessedEventLog` — use same for any async trait bounds.
- `tracing` crate is likely available through `ferro-queue` or `framework`; check before adding dep.

### Integration Points
- `framework/src/lib.rs` re-exports `ferro-stripe` types — needs update for new public API.
- `ProcessStripeWebhook` is a `ferro_queue::Job` — keep that impl; just update its fields and `handle` body.
- `webhook/mod.rs` — update `pub use` statements to expose `SyncDispatcher` from `sync.rs`.

</code_context>

<specifics>
## Specific Ideas

- Design doc §3.6 doc comment text is specified verbatim — use it for `sync.rs` and `queue.rs` module doc comments.
- Golden-JSON fixture content should be minimal valid Stripe webhook JSON (enough fields for `stripe::Event` deserialization to succeed and `from_raw` to extract the target fields).

</specifics>

<deferred>
## Deferred Ideas

- `tracing` dep: if not already available transitively, add explicitly — this is a minor operational concern left to the planner.
- Phase 142 ferro-mcp parity (updating `stripe_webhook_events` and `stripe_config_status`) — explicitly out of scope for this phase.

</deferred>

---

*Phase: 141-protocol-uplift*
*Context gathered: 2026-04-20*

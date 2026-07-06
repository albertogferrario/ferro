# Phase 141: Protocol Uplift - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 141-protocol-uplift
**Mode:** --auto (all areas auto-resolved with recommended defaults)
**Areas discussed:** Event struct reshaping, SyncDispatcher design, Field types, Dependency cleanup, Test fixtures

---

## Event Struct Reshaping

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal (add event_id only) | Keep existing typed fields, add event_id, drop event_json | ✓ |
| Full redesign | Redesign all 5 existing struct field sets from scratch | |

**Auto-selected:** Minimal migration — add `event_id`, keep existing typed fields, remove `event_json` and `ferro_events::Event` impls. Exception: `StripeCheckoutCompleted` receives the full design doc §3.3 field set since the old `customer_id: Option<String>` is too sparse.
**Notes:** Design doc §3.3 comment "same parse-into-fields treatment" for existing structs interpreted as: add `event_id`, keep existing typed fields (subscription_id, customer_id, etc.).

---

## SyncDispatcher Design

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<BoxedHandler> | Linear scan; simpler internal representation | ✓ |
| HashMap keyed by event type | O(1) dispatch; requires StripeEvent to expose type string | |

**Auto-selected:** Vec<BoxedHandler> — simpler, correct, sufficient for the event volume of any webhook handler. Planner may optimize to HashMap if `StripeEvent` trait is extended with an event type string method.
**Notes:** `Arc<SyncDispatcher>` requirement means storage must be `Send + Sync`.

---

## Metadata Field Type

| Option | Description | Selected |
|--------|-------------|----------|
| HashMap<String, String> non-optional | Empty HashMap when no metadata; no unwrap needed | ✓ |
| Option<HashMap<String, String>> | Explicit absence signal | |

**Auto-selected:** `HashMap<String, String>` non-optional — matches design doc §3.3 literal.

---

## ferro-events Dependency

| Option | Description | Selected |
|--------|-------------|----------|
| Drop dep this phase | No remaining users after Event impl removal | ✓ |
| Keep as transitive | Retain for potential future use | |

**Auto-selected:** Drop — no remaining `impl ferro_events::Event` after this phase.

---

## Test Fixtures

| Option | Description | Selected |
|--------|-------------|----------|
| stripe::Event-deserializable JSON in tests/fixtures/ | Matches async-stripe deserialization path | ✓ |
| Minimal hand-rolled JSON with serde_json::Value | Simpler but decoupled from actual Stripe format | |

**Auto-selected:** `stripe::Event`-deserializable fixtures — ensures `from_raw` is tested against real Stripe event shapes.

---

## payment_intent_id on StripeChargeRefunded

| Option | Description | Selected |
|--------|-------------|----------|
| Option<String> (design doc §3.3) | Older charges may lack payment intent | ✓ |
| String (roadmap SC9) | Non-optional as stated in success criteria | |

**Auto-selected:** `Option<String>` — design doc §3.3 wins over roadmap SC9. Older Stripe charges predating PaymentIntents don't have this field.

---

## Claude's Discretion

- Exact internal `SyncDispatcher` handler storage representation
- `dispatch` log level for unknown event types
- `ProcessStripeWebhook` serialization shape for job persistence

## Deferred Ideas

- Phase 142 ferro-mcp parity — explicitly deferred to Phase 142

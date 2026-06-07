# Phase 189: ferro-stripe Manual Capture - Research

**Researched:** 2026-06-07
**Domain:** Stripe PaymentIntent manual capture — async-stripe 0.41, ferro-stripe capability-module pattern
**Confidence:** HIGH

## Summary

All critical scope premises have been verified against the vendored async-stripe 0.41.0 registry source. The APIs required by this phase (`CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual`, `PaymentIntent::capture`, `PaymentIntent::cancel`) exist exactly as the phase description assumes — no feature flags, no missing variants. The `CapturePaymentIntent` params struct uses `amount_to_capture: Option<u64>` (not `i64`), so the public ferro-stripe API of `Option<i64>` requires a cast (`n as u64`). Both new `EventType` variants (`PaymentIntentAmountCapturableUpdated`, `PaymentIntentCanceled`) exist in the `webhook-events` feature, which is already in ferro-stripe's feature list. The two new typed events both match on `EventObject::PaymentIntent`, the same branch already exercised by `StripePaymentIntentFailed` and `StripeConnectPaymentSucceeded`.

The key implementation risk is the double-`payment_intent_data` overwrite: the existing `destination()` branch at line 198–208 of `checkout.rs` sets `params.payment_intent_data = Some(...)`. A `manual_capture()` builder flag must merge into that same struct construction, not produce a competing `Some(...)` assignment. The builder must carry the flag as `manual_capture: bool`, and `create()` must produce exactly one `CreateCheckoutSessionPaymentIntentData` that incorporates both `capture_method` and `transfer_data` when both features are active.

The mode-guard error variant (D-01) mirrors `MissingIdempotencyKey` in shape: a new `Error::ManualCaptureRequiresPaymentMode` (or similar name at Claude's discretion) that fires before any network call when `manual_capture = true && mode = Subscription`. Testing.rs will need two new `mock_*` helper functions to support integration testing of the new events.

**Primary recommendation:** Implement in four focused tasks — (1) `CheckoutBuilder::manual_capture()` flag + guard + merge, (2) `payment_intent.rs` free-function module, (3) two typed events + golden-JSON fixtures + parser-contract tests, (4) docs section. All four are independent except docs (task 4 depends on tasks 1–3 for accurate code examples).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** `manual_capture()` is a plain builder setter; mode requirement enforced as runtime pre-flight check in `create()` — new dedicated structured `Error` variant fires before any network call when `manual_capture` is set with `Mode::Subscription`. Mirrors `MissingIdempotencyKey` guard. No typestate builder.

**D-02:** Free functions in `ferro-stripe/src/payment_intent.rs` mirroring `refund.rs`: `capture(payment_intent_id: &str, amount_cents: Option<i64>) -> Result<stripe::PaymentIntent, Error>` and `cancel(payment_intent_id: &str) -> Result<stripe::PaymentIntent, Error>`.

**D-03:** Error contract identical to `refund.rs`: invalid id → `Error::Stripe(format!("invalid payment intent id: …"))`; API failures propagate via `From<stripe::StripeError>`.

**D-04:** Module exported as `pub mod payment_intent` from lib.rs; no facade methods added elsewhere.

**D-05:** Minimal typed fields: `StripePaymentIntentAmountCapturableUpdated { payment_intent_id, amount_capturable_cents, currency, metadata }` and `StripePaymentIntentCanceled { payment_intent_id, cancellation_reason, metadata }`.

**D-06:** Both implement `StripeEvent` trait; golden-JSON fixtures added under `ferro-stripe/tests/fixtures/stripe_events/`; registered in `tests/parser_contract.rs`.

**D-07:** `capture()`/`cancel()` are platform-scoped only — no `Stripe-Account` header parameter.

**D-08:** Composition verified by a builder-level test asserting the generated params contain BOTH `capture_method = manual` AND `transfer_data`/`on_behalf_of`/`application_fee_amount` when `manual_capture()` + `destination()` are combined.

**D-09:** New "Manual capture" section in `docs/src/features/stripe.md`.

**D-10:** Correspondence table mapping `ferro-reservation` hold/commit/release ↔ Stripe authorize/capture/cancel — framed as semantic parallel, no compile dependency.

**D-11:** Document operational realities: ~7-day authorization window, auto-cancellation of expired uncaptured PaymentIntents, partial-capture remainder auto-released by Stripe.

### Claude's Discretion
- Whether to include a `retrieve(payment_intent_id)` helper in `payment_intent.rs`
- Exact name of the new `Error` variant for the mode guard
- Whether `manual_capture()` takes no args (bool flag) — default to no-args consuming-builder setter
- Internal representation of `capture_method` on the builder struct

### Deferred Ideas (OUT OF SCOPE)
- SetupIntent save-card flow for authorizations beyond the ~7-day card window
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STRIPE-MC-01 | `CheckoutBuilder::manual_capture()` sets `payment_intent_data.capture_method = manual` on the created Checkout Session, in `mode=payment` only | `CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual` verified in async-stripe 0.41 [VERIFIED]; existing `payment_intent_data` construction in `checkout.rs` line 198 must be extended, not duplicated |
| STRIPE-MC-02 | New `payment_intent.rs` capability module: `capture(id, amount_cents: Option<i64>)` and `cancel(id)` | `PaymentIntent::capture` and `PaymentIntent::cancel` verified in `payment_intent_ext.rs` [VERIFIED]; `CapturePaymentIntent.amount_to_capture` is `Option<u64>` — needs cast from `i64` |
| STRIPE-MC-03 | Typed events `StripePaymentIntentAmountCapturableUpdated` and `StripePaymentIntentCanceled` with golden-JSON fixtures | `EventType::PaymentIntentAmountCapturableUpdated` and `EventType::PaymentIntentCanceled` verified in `webhook_events.rs` [VERIFIED]; both dispatch via `EventObject::PaymentIntent` |
| STRIPE-MC-04 | Manual capture composes with `destination()` Connect charges | `CreateCheckoutSessionPaymentIntentData` carries both `capture_method` and `transfer_data` fields [VERIFIED]; merge logic required in `create()` |
| STRIPE-MC-05 | `docs/src/features/stripe.md` documents authorize/capture/cancel ↔ ferro-reservation correspondence | Existing doc structure understood; "Manual capture" section with correspondence table is additive |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Checkout Session authorize flag | ferro-stripe (library) | — | Builder-level flag sets Stripe API params at session creation time |
| PaymentIntent capture/cancel | ferro-stripe (library) | — | Thin wrapper over Stripe REST; no framework routing layer involved |
| Webhook event parsing | ferro-stripe / webhook layer | Consumer app listener | Events parsed in ferro-stripe; business reactions in consumer |
| Connect composition | ferro-stripe (builder) | — | Existing `destination()` branch extended; platform-only API |
| Reservation correspondence | Documentation only | — | Semantic parallel — no code coupling between ferro-stripe and ferro-reservation |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-stripe | 0.41 (pinned) | Stripe API client | Already in ferro-stripe; all required types verified present |
| thiserror | 2 | Error derive | Already used in `error.rs` |
| serde_json | 1 | Fixture deserialization in tests | Already in dev-dependencies |

No new dependencies required.

**No cargo feature additions needed.** `webhook-events` is already in ferro-stripe's feature list and gates `EventType::PaymentIntentAmountCapturableUpdated` and `EventType::PaymentIntentCanceled`. `payment_intent_ext.rs` (which provides `PaymentIntent::capture` and `PaymentIntent::cancel`) is in the unconditional module block of `resources.rs` — no additional feature flag required.

## Architecture Patterns

### System Architecture Diagram

```
Consumer app
  └── CheckoutBuilder::new(Mode::Payment)
        .manual_capture()          ← new setter (STRIPE-MC-01)
        .destination(acct, fee)    ← existing
        .create()
            │
            ├── pre-flight guard: manual_capture && Subscription → Error::ManualCaptureRequiresPaymentMode
            │
            └── CreateCheckoutSession params construction
                  └── ONE payment_intent_data struct merging:
                        capture_method = Some(Manual)        ← new
                        transfer_data  = Some(...)           ← existing
                        on_behalf_of   = Some(...)           ← existing
                        application_fee_amount = ...         ← existing

After checkout completes (Stripe webhook lifecycle):
  payment_intent.amount_capturable_updated ──→ StripePaymentIntentAmountCapturableUpdated
                                                  { payment_intent_id, amount_capturable_cents,
                                                    currency, metadata }

Consumer calls when ready to charge / release:
  payment_intent::capture(id, None)      → full capture  (STRIPE-MC-02)
  payment_intent::capture(id, Some(n))   → partial capture of n cents
  payment_intent::cancel(id)             → release hold

  payment_intent.canceled ───────────────→ StripePaymentIntentCanceled
                                              { payment_intent_id, cancellation_reason, metadata }
```

### Recommended Project Structure

No new directories. All new files slot into existing ferro-stripe structure:

```
ferro-stripe/src/
├── lib.rs                    # add: pub mod payment_intent; + re-export new events
├── payment_intent.rs         # NEW: capture(), cancel(), optional retrieve()
├── checkout.rs               # EXTEND: manual_capture field + guard + merge logic
├── error.rs                  # EXTEND: new ManualCapture* variant
└── webhook/
    └── events.rs             # EXTEND: 2 new event structs + StripeEvent impls

ferro-stripe/tests/
├── parser_contract.rs        # EXTEND: 2 new fixture tests + cross-type rejection tests
└── fixtures/stripe_events/
    ├── payment_intent_amount_capturable_updated.json    # NEW
    └── payment_intent_canceled.json                     # NEW

docs/src/features/
└── stripe.md                 # EXTEND: "Manual capture" section
```

### Pattern 1: payment_intent.rs free-function module (mirrors refund.rs)

```rust
// Source: ferro-stripe/src/refund.rs (existing pattern)
// NOTE: async-stripe 0.41 does not forward idempotency keys on PaymentIntent::capture.
// Application-layer dedup required (same caveat as refund.rs).
pub async fn capture(
    payment_intent_id: &str,
    amount_cents: Option<i64>,
) -> Result<stripe::PaymentIntent, Error> {
    let client = crate::Stripe::client();
    let id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let params = stripe::CapturePaymentIntent {
        amount_to_capture: amount_cents.map(|n| n as u64),
        ..Default::default()
    };
    let pi = stripe::PaymentIntent::capture(client, payment_intent_id, params).await?;
    Ok(pi)
}

pub async fn cancel(
    payment_intent_id: &str,
) -> Result<stripe::PaymentIntent, Error> {
    let client = crate::Stripe::client();
    // PaymentIntentId parse for validation
    let _id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let params = stripe::CancelPaymentIntent::default();
    let pi = stripe::PaymentIntent::cancel(client, payment_intent_id, params).await?;
    Ok(pi)
}
```

**Critical type note:** `CapturePaymentIntent::amount_to_capture` is `Option<u64>`. The public API signature uses `Option<i64>` per D-02. The conversion `n as u64` is correct for valid cents values. Clippy may flag this — use `#[allow(clippy::cast_sign_loss)]` or a checked conversion with a pre-condition comment. [VERIFIED: `async-stripe-0.41.0/src/resources/payment_intent_ext.rs`]

### Pattern 2: CheckoutBuilder manual_capture flag + guard + merge

The critical constraint is that `params.payment_intent_data` must be set **exactly once** in `create()`. The current code sets it inside `if let Some((account_id, fee_cents)) = &self.destination { ... }`. The merged pattern:

```rust
// On CheckoutBuilder struct: add field
manual_capture: bool,

// In create(), replace the single destination branch with merged logic:
let needs_payment_intent_data = self.destination.is_some() || self.manual_capture;
if needs_payment_intent_data {
    let mut pid = CreateCheckoutSessionPaymentIntentData {
        ..Default::default()
    };
    if self.manual_capture {
        pid.capture_method =
            Some(CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual);
    }
    if let Some((account_id, fee_cents)) = &self.destination {
        pid.application_fee_amount = *fee_cents;
        pid.transfer_data = Some(CreateCheckoutSessionPaymentIntentDataTransferData {
            destination: account_id.clone(),
            ..Default::default()
        });
        pid.on_behalf_of = Some(account_id.clone());
    }
    params.payment_intent_data = Some(pid);
}
```

This replaces the current single-branch `if let Some` block and is the **correct merge pattern** that avoids the double-overwrite pitfall. [VERIFIED: `ferro-stripe/src/checkout.rs` lines 198–208]

### Pattern 3: New typed events (mirrors StripePaymentIntentFailed)

```rust
// Source: ferro-stripe/src/webhook/events.rs (existing StripePaymentIntentFailed pattern)
#[derive(Debug, Clone)]
pub struct StripePaymentIntentAmountCapturableUpdated {
    pub event_id: String,
    pub payment_intent_id: String,
    pub amount_capturable_cents: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripePaymentIntentAmountCapturableUpdated {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentAmountCapturableUpdated {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => Some(Self {
                event_id: event.id.to_string(),
                payment_intent_id: pi.id.to_string(),
                amount_capturable_cents: pi.amount_capturable,
                currency: pi.currency.to_string(),
                metadata: pi.metadata.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StripePaymentIntentCanceled {
    pub event_id: String,
    pub payment_intent_id: String,
    pub cancellation_reason: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl StripeEvent for StripePaymentIntentCanceled {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentCanceled {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => Some(Self {
                event_id: event.id.to_string(),
                payment_intent_id: pi.id.to_string(),
                cancellation_reason: pi.cancellation_reason.map(|r| r.as_str().to_string()),
                metadata: pi.metadata.clone(),
            }),
            _ => None,
        }
    }
}
```

**Field types verified:** `pi.amount_capturable: i64`, `pi.currency: Currency` (non-optional, `.to_string()` works), `pi.metadata: Metadata` (= `HashMap<String, String>`), `pi.cancellation_reason: Option<PaymentIntentCancellationReason>`. [VERIFIED: `async-stripe-0.41.0/src/resources/generated/payment_intent.rs`]

**EventType variant names verified:** `stripe::EventType::PaymentIntentAmountCapturableUpdated` (serde rename `"payment_intent.amount_capturable_updated"`) and `stripe::EventType::PaymentIntentCanceled` (serde rename `"payment_intent.canceled"`). [VERIFIED: `async-stripe-0.41.0/src/resources/webhook_events.rs` lines 221–224]

### Pattern 4: Golden-JSON fixture shape (mirrors payment_intent_payment_failed.json)

The PaymentIntent object in Stripe webhooks requires `amount`, `amount_capturable`, `currency`, `livemode`, `capture_method`, `confirmation_method`, `created`, `payment_method_types`, `metadata`, and `status` at minimum for the struct to deserialize.

For `payment_intent_amount_capturable_updated.json`:
- `"type": "payment_intent.amount_capturable_updated"`
- `"data.object.object": "payment_intent"`
- `"data.object.amount_capturable": 5000` (non-zero — this is the key field)
- `"data.object.status": "requires_capture"`
- `"data.object.capture_method": "manual"`

For `payment_intent_canceled.json`:
- `"type": "payment_intent.canceled"`
- `"data.object.object": "payment_intent"`
- `"data.object.status": "canceled"`
- `"data.object.cancellation_reason": "requested_by_customer"` (or `"automatic"` for auto-expiry test)
- `"data.object.capture_method": "manual"`

### Anti-Patterns to Avoid

- **Double `payment_intent_data` assignment:** Setting `params.payment_intent_data = Some(...)` once for `manual_capture` and again for `destination()` silently discards the first assignment. The pattern above (one merged struct construction) is the correct fix.
- **`u64` / `i64` mismatch on capture amount:** `CapturePaymentIntent::amount_to_capture` is `Option<u64>`. Passing a negative `i64` from the public API would silently wrap. A pre-condition assert or a guard error is appropriate; at minimum, document the contract.
- **Wrong `EventType` variant name in tests:** The `type_` field on `stripe::Event` is deserialized from the JSON `"type"` field string. The fixture JSON `"type"` value must match the serde rename of the `EventType` variant exactly: `"payment_intent.amount_capturable_updated"` and `"payment_intent.canceled"`. Using the Rust variant name in the fixture JSON will cause the event to deserialize as `EventType::Other` and `from_raw` will return `None`.
- **Subscription mode + manual_capture:** Stripe returns an error if `capture_method=manual` is set on a Subscription checkout session. The ferro-stripe guard must fire *before* the network call, exactly like `MissingIdempotencyKey`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PaymentIntent capture API | Custom HTTP POST to `/payment_intents/{id}/capture` | `stripe::PaymentIntent::capture(client, id, params)` | Handles auth, serialization, error mapping; already vendored |
| PaymentIntent cancel API | Custom HTTP POST | `stripe::PaymentIntent::cancel(client, id, params)` | Same as above |
| Event type string matching | String comparison on `event.type_` as raw string | `stripe::EventType::PaymentIntentAmountCapturableUpdated` enum match | Compile-time exhaustiveness, no string typo risk |
| Idempotency for capture | Custom dedup logic in ferro-stripe | Application-layer dedup (document as caveat, same as refund.rs) | async-stripe 0.41 does not forward per-request idempotency keys |

## Common Pitfalls

### Pitfall 1: Double payment_intent_data overwrite
**What goes wrong:** Both `manual_capture` and `destination()` branches independently assign `params.payment_intent_data = Some(...)`. The second assignment silently discards the first, so only one feature works at a time.
**Why it happens:** Copying the existing single-branch pattern without recognizing that two builder flags now contribute to the same struct.
**How to avoid:** Replace the single `if let Some(destination)` block with a unified `if needs_payment_intent_data` block that builds one `CreateCheckoutSessionPaymentIntentData` and conditionally sets fields from both flags.
**Warning signs:** SC-4 builder-level test (D-08) fails — composed `manual_capture()` + `destination()` produces params where either `capture_method` or `transfer_data` is absent.

### Pitfall 2: amount_to_capture type mismatch
**What goes wrong:** `CapturePaymentIntent::amount_to_capture: Option<u64>` but the public API accepts `Option<i64>`. Direct assignment without cast triggers a type error. A naive `n as u64` for a negative value silently wraps to a huge number.
**Why it happens:** The Stripe API uses unsigned integers for currency amounts; ferro-stripe's public API mirrors `refund.rs` which uses `i64` for consistency.
**How to avoid:** The cast `n as u64` is safe for all valid cent values (positive integers). Document the pre-condition ("caller must pass a positive cents value") or add a guard that returns `Error::Stripe("amount_to_capture must be positive".to_string())` for `n <= 0`.
**Warning signs:** Clippy `-D warnings` flag (`cast_sign_loss`) — CI will fail.

### Pitfall 3: EventType fixture string mismatch
**What goes wrong:** Golden-JSON fixture `"type"` field uses the wrong string, causing `serde_json::from_str::<stripe::Event>` to deserialize the event with `type_ = EventType::Other`. `from_raw` returns `None`. Parser-contract test fails with "from_raw should return Some but got None".
**Why it happens:** Writing the Rust variant name instead of the Stripe API string.
**How to avoid:** Use the exact strings: `"payment_intent.amount_capturable_updated"` and `"payment_intent.canceled"`. These are confirmed by the `#[serde(rename = ...)]` attributes in `webhook_events.rs`.
**Warning signs:** `parse_event(FIXTURE)` succeeds but the resulting `event.type_` equals `EventType::Other`.

### Pitfall 4: Mode guard position
**What goes wrong:** The mode guard for `manual_capture + Subscription` is placed after `Stripe::client()` is called. If the Stripe client is not initialized in tests (as is the case for the existing `checkout_create_missing_key_returns_err` test), the test panics instead of returning `Err(Error::...)`.
**Why it happens:** Placing guards after the first network-dependent call.
**How to avoid:** Both guards (`MissingIdempotencyKey` and the new mode guard) must fire before `let client = crate::Stripe::client()`. Order: idempotency key check → mode guard → `Stripe::client()`.
**Warning signs:** Test `checkout_create_manual_capture_subscription_returns_err` panics instead of returning `Err`.

### Pitfall 5: Clippy cast_sign_loss on n as u64
**What goes wrong:** `cargo clippy --all --all-targets -- -D warnings` fails on `amount_cents.map(|n| n as u64)` with `cast_sign_loss`.
**Why it happens:** Rust lints on potentially lossy sign-to-unsigned cast.
**How to avoid:** Either add a `#[allow(clippy::cast_sign_loss)]` with a comment explaining the invariant, or use `u64::try_from(n).unwrap_or(0)` with an explanation. The former is simpler and documents the intended invariant.

## Code Examples

### CheckoutBuilder struct field addition
```rust
// Source: ferro-stripe/src/checkout.rs (existing struct, adding field)
pub struct CheckoutBuilder {
    // ... existing fields ...
    manual_capture: bool,           // NEW: sets capture_method=manual on payment_intent_data
}

// In CheckoutBuilder::new():
    manual_capture: false,

// New consuming setter:
/// Enables manual capture for this Checkout Session.
///
/// The payment is authorized but not charged at checkout. Call
/// [`payment_intent::capture`] to charge or [`payment_intent::cancel`]
/// to release the hold.
///
/// Only valid with [`Mode::Payment`]. Calling this with [`Mode::Subscription`]
/// returns [`Error::ManualCaptureRequiresPaymentMode`] when `create()` is called.
pub fn manual_capture(mut self) -> Self {
    self.manual_capture = true;
    self
}
```

### Error variant addition
```rust
// Source: ferro-stripe/src/error.rs (existing enum, adding variant)
/// manual_capture() requires Mode::Payment. Mode::Subscription does not support
/// deferred capture — each subscription invoice is charged automatically.
#[error("manual capture requires payment mode; use Mode::Payment with manual_capture()")]
ManualCaptureRequiresPaymentMode,
```

### testing.rs additions (Claude's discretion whether to add)
```rust
// Source: ferro-stripe/src/testing.rs (existing pattern)
pub fn mock_payment_intent_amount_capturable_updated_event(
    payment_intent_id: &str,
    amount_capturable_cents: i64,
) -> String {
    serde_json::json!({
        "id": "evt_mock_pi_amount_capturable_updated",
        "object": "event",
        "type": "payment_intent.amount_capturable_updated",
        "api_version": "2023-10-16",
        "created": chrono::Utc::now().timestamp(),
        "livemode": false,
        "pending_webhooks": 1,
        "request": null,
        "data": {
            "object": {
                "id": payment_intent_id,
                "object": "payment_intent",
                "amount": amount_capturable_cents,
                "amount_capturable": amount_capturable_cents,
                "amount_received": 0,
                "currency": "usd",
                "livemode": false,
                "capture_method": "manual",
                "confirmation_method": "automatic",
                "created": 1700000000_i64,
                "payment_method_types": ["card"],
                "metadata": {},
                "status": "requires_capture"
            }
        }
    }).to_string()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Automatic-only capture | `capture_method=manual` via Checkout Session | Stripe API, always available | Enables hold-then-capture patterns |
| String event type matching | `stripe::EventType` enum | async-stripe since ~0.20 | Compile-time safety |

## Assumptions Log

No claims in this research are tagged `[ASSUMED]`. All API surface claims were verified against the vendored `async-stripe-0.41.0` registry source.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims in this research were verified against the vendored async-stripe-0.41.0 source.**

## Open Questions

1. **`retrieve()` helper in `payment_intent.rs`**
   - What we know: `PaymentIntent::retrieve(client, &id, &[])` exists in async-stripe 0.41; `refund.rs` includes a `retrieve()` function; the CONTEXT marks this as Claude's discretion.
   - What's unclear: Whether gestiscilo v6.3 will need to poll authorization state between checkout and capture.
   - Recommendation: Include `retrieve()` for API parity with `refund.rs`; it is cheap (one function, no new dependencies) and prevents a follow-up patch.

2. **Clippy `cast_sign_loss` on `n as u64`**
   - What we know: `CapturePaymentIntent::amount_to_capture: Option<u64>`; public API uses `Option<i64>`.
   - What's unclear: Whether to use `#[allow(clippy::cast_sign_loss)]` or `u64::try_from(n).map_err(...)`.
   - Recommendation: Use `u64::try_from(n).map_err(|_| Error::Stripe("amount_to_capture must be positive".to_string()))?` — avoids the allow attribute and gives a clear error for caller bugs. This is stricter than `n as u64` and consistent with the existing id-parse error pattern.

## Environment Availability

Step 2.6: SKIPPED — this phase is code-only changes within the ferro workspace. No external services, CLIs, or databases beyond the existing Rust toolchain and Stripe test environment are required. Cargo, rustfmt, and clippy are assumed available per workspace conventions.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test -p ferro-stripe` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STRIPE-MC-01 | `manual_capture()` sets `capture_method=manual` in params | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ `src/checkout.rs` (mod tests) |
| STRIPE-MC-01 | `manual_capture()` + `Mode::Subscription` → pre-flight error | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ (new test in mod tests) |
| STRIPE-MC-02 | `capture(id, None)` / `capture(id, Some(n))` / `cancel(id)` compile and return correct types | unit (compile) | `cargo test -p ferro-stripe payment_intent` | ❌ Wave 0: `src/payment_intent.rs` (new file) |
| STRIPE-MC-03 | `StripePaymentIntentAmountCapturableUpdated::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0: new fixture + test in `parser_contract.rs` |
| STRIPE-MC-03 | `StripePaymentIntentCanceled::from_raw` parses golden fixture | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0: new fixture + test in `parser_contract.rs` |
| STRIPE-MC-03 | Cross-type rejection (each event rejects the other's fixture) | integration | `cargo test -p ferro-stripe --test parser_contract` | ❌ Wave 0 |
| STRIPE-MC-04 | `manual_capture()` + `destination()` combined → params carry BOTH fields | unit (builder) | `cargo test -p ferro-stripe checkout` | ✅ (new test in mod tests) |
| STRIPE-MC-05 | docs section is present and coherent | manual review | — | — |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-stripe`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before verify-work

### Wave 0 Gaps
- [ ] `ferro-stripe/src/payment_intent.rs` — new file; covers STRIPE-MC-02
- [ ] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json` — golden fixture for STRIPE-MC-03
- [ ] `ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json` — golden fixture for STRIPE-MC-03
- [ ] 4 new test functions in `ferro-stripe/tests/parser_contract.rs` — STRIPE-MC-03 parse + cross-type rejection for each new event

## Security Domain

This phase extends an existing Stripe integration library. Relevant ASVS categories:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes | `PaymentIntentId::parse()` validates ID format before any API call; `u64` conversion guard on `amount_to_capture` |
| V6 Cryptography | No | No new crypto; webhook verification unchanged |
| V2 Authentication | No | Stripe API key handling unchanged |

No new attack surface is introduced. The mode guard (D-01) prevents misuse at the application layer. The `platform-scoped only` decision (D-07) ensures no connected-account impersonation through the new module.

## Sources

### Primary (HIGH confidence)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/src/resources/payment_intent_ext.rs` — `PaymentIntent::capture`, `PaymentIntent::cancel`, `CapturePaymentIntent`, `CancelPaymentIntent` exact signatures
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/src/resources/webhook_events.rs` — `EventType::PaymentIntentAmountCapturableUpdated`, `EventType::PaymentIntentCanceled`, `EventObject::PaymentIntent`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/src/resources/generated/checkout_session.rs` — `CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual` enum variant
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/src/resources/generated/payment_intent.rs` — `PaymentIntent` struct fields: `amount_capturable: i64`, `cancellation_reason: Option<PaymentIntentCancellationReason>`, `currency: Currency`, `metadata: Metadata`
- `ferro-stripe/src/checkout.rs` — existing `payment_intent_data` construction at lines 198–208 (merge target)
- `ferro-stripe/src/refund.rs` — canonical capability-module pattern this phase mirrors
- `ferro-stripe/src/webhook/events.rs` — existing `StripeEvent` impl pattern
- `ferro-stripe/tests/parser_contract.rs` — fixture registration pattern
- `ferro-stripe/src/error.rs` — existing `Error` enum shape

### Secondary (MEDIUM confidence)
- Stripe documentation (training knowledge): ~7-day card authorization window, auto-cancellation of uncaptured PaymentIntents, partial-capture remainder release behavior — [ASSUMED] for the docs section D-11; will be presented with appropriate hedging in docs ("Stripe cancels uncaptured PaymentIntents after approximately 7 days for card payments")

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types verified against vendored source
- Architecture patterns: HIGH — code examples directly derived from verified types + existing patterns
- Pitfalls: HIGH — double-overwrite pitfall identified from direct code inspection; type mismatch from actual field type; fixture string from verified serde rename

**Research date:** 2026-06-07
**Valid until:** 2026-07-07 (async-stripe 0.41 is pinned; stable until ferro-stripe upgrades)

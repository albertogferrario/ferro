# Phase 189: ferro-stripe Manual Capture - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 8 (5 modified, 2 new source files, 2 new fixtures, 1 new test scope)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-stripe/src/payment_intent.rs` | service (capability module) | request-response | `ferro-stripe/src/refund.rs` | exact |
| `ferro-stripe/src/checkout.rs` | builder + service | request-response | `ferro-stripe/src/checkout.rs` (existing) | self-extension |
| `ferro-stripe/src/webhook/events.rs` | event struct + trait impl | event-driven | `StripePaymentIntentFailed` in same file (lines 191–229) | exact |
| `ferro-stripe/src/error.rs` | error enum | — | existing `MissingIdempotencyKey` variant (line 26) | exact |
| `ferro-stripe/src/lib.rs` | module registry + re-exports | — | existing `pub mod refund` + `pub use webhook::events::{...}` block | exact |
| `ferro-stripe/tests/parser_contract.rs` | integration test | event-driven | existing `payment_intent_failed_parses_all_fields` block (lines 79–102) | exact |
| `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json` | test fixture | — | `payment_intent_payment_failed.json` | exact |
| `ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json` | test fixture | — | `payment_intent_payment_failed.json` | exact |
| `docs/src/features/stripe.md` | documentation | — | existing "Stripe Connect / Destination Charges" section (lines 176–229) | role-match |

---

## Pattern Assignments

### `ferro-stripe/src/payment_intent.rs` (capability module, request-response)

**Analog:** `ferro-stripe/src/refund.rs`

**Imports pattern** (`refund.rs` lines 1–5):
```rust
//! Payment intent capture and cancel operations.
//!
//! Thin capability-axis wrappers over the `stripe::PaymentIntent` API.

use crate::Error;
```

**Core function pattern** (`refund.rs` lines 18–50 — id-parse → params → API call → typed return):
```rust
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    let _ = idempotency_key;
    let client = crate::Stripe::client();

    let mut params = stripe::CreateRefund::new();
    let charge: stripe::ChargeId = charge_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid charge id: {charge_id}")))?;
    params.charge = Some(charge);
    params.amount = amount_cents;
    params.reason = reason;

    let refund = stripe::Refund::create(client, params).await?;
    Ok(refund)
}
```

**Retrieve helper pattern** (`refund.rs` lines 43–50 — parity function):
```rust
pub async fn retrieve(refund_id: &str) -> Result<stripe::Refund, Error> {
    let client = crate::Stripe::client();
    let id: stripe::RefundId = refund_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid refund id: {refund_id}")))?;
    let refund = stripe::Refund::retrieve(client, &id, &[]).await?;
    Ok(refund)
}
```

**Error handling pattern** (id parse error, `refund.rs` lines 31–33):
```rust
let charge: stripe::ChargeId = charge_id
    .parse()
    .map_err(|_| Error::Stripe(format!("invalid charge id: {charge_id}")))?;
```
For `payment_intent.rs`: replace `ChargeId` with `PaymentIntentId`, update the format string to `"invalid payment intent id: {payment_intent_id}"`.

**Idempotency caveat note** (`refund.rs` lines 24–27 — copy verbatim, adapt for capture):
```rust
// Note: async-stripe 0.41 does not expose a per-request idempotency-key strategy
// on PaymentIntent::capture. Application-layer dedup required (same caveat as refund.rs).
```

**Critical type note for capture:** `CapturePaymentIntent::amount_to_capture` is `Option<u64>`, but the public API takes `Option<i64>` for consistency with `refund.rs`. Use `u64::try_from(n).map_err(|_| Error::Stripe("amount_to_capture must be positive".to_string()))?` to avoid the `cast_sign_loss` clippy lint. Do NOT use bare `n as u64`.

---

### `ferro-stripe/src/checkout.rs` (builder, request-response) — MODIFIED

**Analog:** `ferro-stripe/src/checkout.rs` (self-extension of existing file)

**Struct field addition pattern** (`checkout.rs` lines 51–60 — existing builder struct):
```rust
pub struct CheckoutBuilder {
    mode: Mode,
    line_items: Vec<LineItem>,
    success_url: Option<String>,
    cancel_url: Option<String>,
    metadata: Vec<(String, String)>,
    customer_email: Option<String>,
    destination: Option<(String, Option<i64>)>,
    idempotency_key: Option<String>,
    // ADD: manual_capture: bool,
}
```

**Constructor zero-init pattern** (`checkout.rs` lines 64–75 — `CheckoutBuilder::new`):
```rust
pub fn new(mode: Mode) -> Self {
    Self {
        mode,
        line_items: Vec::new(),
        success_url: None,
        cancel_url: None,
        metadata: Vec::new(),
        customer_email: None,
        destination: None,
        idempotency_key: None,
        // ADD: manual_capture: false,
    }
}
```

**Consuming setter pattern** (`checkout.rs` lines 117–119 — `destination()` as model):
```rust
pub fn destination(mut self, account_id: &str, fee_cents: Option<i64>) -> Self {
    self.destination = Some((account_id.to_string(), fee_cents));
    self
}
```
New `manual_capture()` follows same `mut self -> Self` shape with no parameters.

**Pre-flight guard pattern** (`checkout.rs` lines 138–141 — `MissingIdempotencyKey` guard):
```rust
pub async fn create(self) -> Result<CheckoutIntent, Error> {
    // Runtime guard — fail before any network call.
    let idempotency_key = self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?;

    let client = crate::Stripe::client();
```
New mode guard fires BETWEEN the idempotency check and `Stripe::client()`:
```rust
let idempotency_key = self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?;
// New guard:
if self.manual_capture && self.mode == Mode::Subscription {
    return Err(Error::ManualCaptureRequiresPaymentMode);
}
let client = crate::Stripe::client();
```

**payment_intent_data merge pattern** (`checkout.rs` lines 198–208 — existing destination branch to replace):
```rust
// EXISTING (single-feature, to be replaced):
if let Some((account_id, fee_cents)) = &self.destination {
    params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
        application_fee_amount: *fee_cents,
        transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
            destination: account_id.clone(),
            ..Default::default()
        }),
        on_behalf_of: Some(account_id.clone()),
        ..Default::default()
    });
}

// REPLACEMENT (merged, avoids double-overwrite):
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

**Import addition needed** (`checkout.rs` line 9–13 — existing import block):
```rust
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
    CreateCheckoutSessionPaymentIntentData, CreateCheckoutSessionPaymentIntentDataTransferData,
    // ADD: CreateCheckoutSessionPaymentIntentDataCaptureMethod,
};
```

**Test pattern for new guard** (`checkout.rs` lines 268–290 — `checkout_create_missing_key_returns_err`):
```rust
#[tokio::test]
async fn checkout_create_missing_key_returns_err() {
    let result = CheckoutBuilder::new(Mode::Payment)
        .success_url("https://example.com/ok")
        .cancel_url("https://example.com/cancel")
        .line_item(LineItem { ... })
        .create()
        .await;
    assert!(
        matches!(result, Err(Error::MissingIdempotencyKey)),
        "expected Err(MissingIdempotencyKey), got {result:?}"
    );
}
```
Follow this exact shape for:
- `checkout_create_manual_capture_subscription_returns_err` — asserts `Err(Error::ManualCaptureRequiresPaymentMode)`
- `checkout_create_manual_capture_sets_capture_method` — asserts params contain `capture_method = manual` (via inspecting builder state before `create()`, since no live Stripe in tests)
- `checkout_create_manual_capture_with_destination_sets_both_fields` — asserts D-08 composition

---

### `ferro-stripe/src/webhook/events.rs` (event structs, event-driven) — MODIFIED

**Analog:** `StripePaymentIntentFailed` in `ferro-stripe/src/webhook/events.rs` (lines 191–229)

**Struct declaration pattern** (`events.rs` lines 191–198):
```rust
#[derive(Debug, Clone)]
pub struct StripePaymentIntentFailed {
    pub event_id: String,
    pub payment_intent_id: String,
    pub session_id: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

**StripeEvent impl pattern for EventObject::PaymentIntent** (`events.rs` lines 200–229 — the PaymentIntent arm, also in `StripeConnectPaymentSucceeded` lines 142–159):
```rust
impl StripeEvent for StripePaymentIntentFailed {
    fn from_raw(event: &stripe::Event) -> Option<Self> {
        if event.type_ != stripe::EventType::PaymentIntentPaymentFailed {
            return None;
        }
        match &event.data.object {
            stripe::EventObject::PaymentIntent(pi) => {
                // field extraction here
                Some(Self { event_id: event.id.to_string(), ... })
            }
            _ => None,
        }
    }
}
```

**New event struct field types (verified against async-stripe 0.41):**
- `pi.id.to_string()` → `payment_intent_id: String`
- `pi.amount_capturable` → `amount_capturable_cents: i64` (field is `i64`, no conversion needed)
- `pi.currency.to_string()` → `currency: String` (non-optional `Currency` type)
- `pi.metadata.clone()` → `metadata: HashMap<String, String>` (already `HashMap<String, String>`)
- `pi.cancellation_reason.map(|r| r.as_str().to_string())` → `cancellation_reason: Option<String>`

**`events_are_clone_send_sync` test update** (`events.rs` lines 329–340 — add two new types):
```rust
_assert_clone_send_sync::<StripePaymentIntentAmountCapturableUpdated>();
_assert_clone_send_sync::<StripePaymentIntentCanceled>();
```
Same for `all_event_types_implement_stripe_event` (lines 343–354).

---

### `ferro-stripe/src/error.rs` (error enum) — MODIFIED

**Analog:** `MissingIdempotencyKey` variant in `ferro-stripe/src/error.rs` (lines 25–26)

**Existing variant pattern** (`error.rs` lines 25–26):
```rust
/// Idempotency key not set on CheckoutBuilder before calling create().
#[error("idempotency key required: call .idempotency_key() before .create()")]
MissingIdempotencyKey,
```

**New variant follows identical shape** — doc comment explains the invariant, `#[error]` message is user-facing:
```rust
/// manual_capture() requires Mode::Payment. Mode::Subscription does not support
/// deferred capture — each subscription invoice is charged automatically.
#[error("manual capture requires payment mode; use Mode::Payment with manual_capture()")]
ManualCaptureRequiresPaymentMode,
```

---

### `ferro-stripe/src/lib.rs` (module registry + re-exports) — MODIFIED

**Analog:** `ferro-stripe/src/lib.rs` lines 43–69 (existing module declarations + re-exports)

**Module declaration pattern** (`lib.rs` lines 43–51):
```rust
pub mod account;
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
// ADD: pub mod payment_intent;
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
pub mod webhook;
```
`payment_intent` is unconditional (same as `refund`), not gated behind `test-helpers`.

**Re-export pattern** (`lib.rs` lines 54–55 — `refund` has no top-level re-exports, only module-level):
```rust
pub use account::{billing_portal_url, create_account, create_link, retrieve_account};
pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
// No pub use refund::* — consumers call ferro_stripe::refund::create(...)
// Same pattern for payment_intent: no top-level re-exports; access via ferro_stripe::payment_intent::capture(...)
```

**Webhook events re-export extension** (`lib.rs` lines 61–66):
```rust
pub use webhook::events::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded,
    StripeInvoicePaid, StripePaymentIntentFailed, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
    // ADD: StripePaymentIntentAmountCapturableUpdated, StripePaymentIntentCanceled,
};
```

---

### `ferro-stripe/tests/parser_contract.rs` (integration tests) — MODIFIED

**Analog:** Existing `payment_intent_failed_parses_all_fields` block (lines 76–102) and `checkout_completed_rejects_expired_event` cross-rejection pattern (lines 43–49).

**Fixture `const` + `parse_event` pattern** (`parser_contract.rs` lines 16–18, 76):
```rust
fn parse_event(raw: &str) -> stripe::Event {
    serde_json::from_str::<stripe::Event>(raw).expect("fixture should deserialize as stripe::Event")
}

const PI_FAILED: &str = include_str!("fixtures/stripe_events/payment_intent_payment_failed.json");
```
New constants:
```rust
const PI_AMOUNT_CAPTURABLE: &str =
    include_str!("fixtures/stripe_events/payment_intent_amount_capturable_updated.json");
const PI_CANCELED: &str =
    include_str!("fixtures/stripe_events/payment_intent_canceled.json");
```

**Parse-all-fields test pattern** (`parser_contract.rs` lines 79–102):
```rust
#[test]
fn payment_intent_failed_parses_all_fields() {
    let event = parse_event(PI_FAILED);
    let typed = StripePaymentIntentFailed::from_raw(&event)
        .expect("from_raw should return Some for payment_intent.payment_failed");
    assert_eq!(typed.event_id, "evt_test_pi_failed_001");
    assert_eq!(typed.payment_intent_id, "pi_test_failed_001");
    // ...field assertions matching fixture values...
}
```

**Cross-rejection test pattern** (`parser_contract.rs` lines 43–49 — `checkout_completed_rejects_expired_event`):
```rust
#[test]
fn checkout_completed_rejects_expired_event() {
    let event = parse_event(CHECKOUT_EXPIRED);
    assert!(
        StripeCheckoutCompleted::from_raw(&event).is_none(),
        "StripeCheckoutCompleted must reject checkout.session.expired (type_ guard)"
    );
}
```
Four tests needed (two parse-all-fields, two cross-rejection):
1. `payment_intent_amount_capturable_updated_parses_all_fields`
2. `payment_intent_canceled_parses_all_fields`
3. `payment_intent_amount_capturable_updated_rejects_canceled_event`
4. `payment_intent_canceled_rejects_amount_capturable_event`

**Import block update** (`parser_contract.rs` lines 9–14):
```rust
use ferro_stripe::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    StripeCheckoutExpired, StripeConnectAccountUpdated, StripeConnectPaymentSucceeded, StripeEvent,
    StripeInvoicePaid, StripePaymentIntentFailed, StripeSubscriptionDeleted,
    StripeSubscriptionUpdated,
    // ADD: StripePaymentIntentAmountCapturableUpdated, StripePaymentIntentCanceled,
};
```

---

### `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json` (NEW)

**Analog:** `ferro-stripe/tests/fixtures/stripe_events/payment_intent_payment_failed.json`

**Fixture structure pattern** (the full `payment_intent_payment_failed.json`):
```json
{
  "id": "evt_test_pi_failed_001",
  "object": "event",
  "api_version": "2023-10-16",
  "created": 1700000002,
  "livemode": false,
  "pending_webhooks": 1,
  "request": null,
  "type": "payment_intent.payment_failed",
  "data": {
    "object": {
      "id": "pi_test_failed_001",
      "object": "payment_intent",
      "amount": 1500,
      "amount_capturable": 0,
      "amount_received": 0,
      "currency": "usd",
      "livemode": false,
      "metadata": { ... },
      "capture_method": "automatic",
      "confirmation_method": "automatic",
      "created": 1700000002,
      "payment_method_types": ["card"],
      "status": "requires_payment_method",
      ...
    }
  }
}
```

**For `payment_intent_amount_capturable_updated.json`, change:**
- `"type"`: `"payment_intent.amount_capturable_updated"` (exact serde rename — must not use Rust variant name)
- `"data.object.amount_capturable"`: non-zero, e.g. `5000`
- `"data.object.status"`: `"requires_capture"`
- `"data.object.capture_method"`: `"manual"`
- `"data.object.amount_received"`: `0` (not yet captured)
- Keep `"amount"`: same as `amount_capturable` (e.g. `5000`)
- Use deterministic test IDs: `"id": "evt_test_pi_capturable_001"`, `"data.object.id": "pi_test_capturable_001"`

---

### `ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json` (NEW)

**Analog:** same as above.

**For `payment_intent_canceled.json`, change:**
- `"type"`: `"payment_intent.canceled"` (exact serde rename)
- `"data.object.status"`: `"canceled"`
- `"data.object.cancellation_reason"`: `"requested_by_customer"` (or `"automatic"` for auto-expiry)
- `"data.object.capture_method"`: `"manual"`
- `"data.object.amount_capturable"`: `0` (cancelled, nothing capturable)
- Use deterministic test IDs: `"id": "evt_test_pi_canceled_001"`, `"data.object.id": "pi_test_canceled_001"`

---

### `docs/src/features/stripe.md` (documentation) — MODIFIED

**Analog:** "Stripe Connect / Destination Charges" section (`stripe.md` lines 176–229) — additive section after the Connect section.

**Section structure pattern** (existing Connect section, lines 176–229):
- H2 heading (`##`)
- 1–2 sentence intro
- Code block showing the builder API
- Prose explanation of operational realities
- Table for structured data (webhook events, operational parameters)

**"Manual capture" section placement:** after the Connect section (`## Stripe Connect`), before `## Webhook Configuration`.

**Correspondence table pattern** (existing webhook event dispatch table, `stripe.md` lines 270–277):
```markdown
| Stripe event | ferro-events Event |
|---|---|
| `customer.subscription.updated` | `StripeSubscriptionUpdated` |
```
Adapt for the ferro-reservation correspondence table (D-10):
```markdown
| ferro-reservation | Stripe PaymentIntent |
|---|---|
| `hold()` | Authorize at checkout (`capture_method=manual`) |
| `commit()` | `payment_intent::capture(id, amount)` |
| `release()` | `payment_intent::cancel(id)` |
```

---

## Shared Patterns

### Error propagation via `From<stripe::StripeError>`
**Source:** `ferro-stripe/src/error.rs` lines 29–33
**Apply to:** `payment_intent.rs` functions (already implemented via `?` on async-stripe calls)
```rust
impl From<stripe::StripeError> for Error {
    fn from(e: stripe::StripeError) -> Self {
        Error::Stripe(e.to_string())
    }
}
```

### Consuming builder `mut self -> Self`
**Source:** `ferro-stripe/src/checkout.rs` lines 78–129 (all setter methods)
**Apply to:** `CheckoutBuilder::manual_capture()` — no args, flips the bool field
```rust
pub fn manual_capture(mut self) -> Self {
    self.manual_capture = true;
    self
}
```

### Pre-flight guard before `Stripe::client()`
**Source:** `ferro-stripe/src/checkout.rs` lines 138–142
**Apply to:** new `ManualCaptureRequiresPaymentMode` guard in `checkout.rs::create()` — must fire after idempotency check, before `Stripe::client()`

### ID parse error format string
**Source:** `ferro-stripe/src/refund.rs` lines 31–33
**Apply to:** all three `payment_intent.rs` functions (`capture`, `cancel`, `retrieve`)
```rust
.map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?
```

### `StripeEvent::from_raw` type guard + PaymentIntent arm
**Source:** `ferro-stripe/src/webhook/events.rs` lines 200–229 (`StripePaymentIntentFailed`) and lines 142–159 (`StripeConnectPaymentSucceeded`)
**Apply to:** both new event structs — identical structure, only the `EventType` variant and field extraction differ

### Module declaration + conditional test-helpers gate
**Source:** `ferro-stripe/src/lib.rs` lines 43–51
**Apply to:** `pub mod payment_intent;` addition — unconditional, same as `refund`

---

## No Analog Found

All files in scope have close or exact analogs in the existing codebase. No entries.

---

## Metadata

**Analog search scope:** `ferro-stripe/src/`, `ferro-stripe/tests/`, `docs/src/features/`
**Files scanned:** 7 source files + 1 test file + 10 fixtures + 1 docs file
**Pattern extraction date:** 2026-06-07

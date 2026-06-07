# Phase 189: ferro-stripe Manual Capture - Context

**Gathered:** 2026-06-07
**Status:** Ready for planning
**Mode:** --auto (all gray areas auto-resolved with recommended defaults)

<domain>
## Phase Boundary

Extend `ferro-stripe` with Stripe manual capture so consumer apps can authorize card funds without charging (booking deposits): `CheckoutBuilder::manual_capture()` sets `payment_intent_data.capture_method = manual`; a new `payment_intent.rs` capability module exposes `capture(payment_intent_id, amount_cents: Option<i64>)` (partial capture supported) and `cancel(payment_intent_id)`; two new typed webhook events (`StripePaymentIntentAmountCapturableUpdated`, `StripePaymentIntentCanceled`) join the parser contract with golden-JSON fixtures; manual capture composes with `destination()` Connect charges. The authorize/capture/cancel triple mirrors `ferro-reservation` hold/commit/release — documented correspondence, no compile coupling.

**Out of scope:** SetupIntent save-card flow for authorizations beyond the ~7-day card window (consumer-side decision at gestiscilo v6.3 plan time).

**Consumer:** gestiscilo-it v6.3 Online Checkout & Payments, via published crates.io bump (Phase 176 ↔ Phase 181 pattern — publish once at phase close).

</domain>

<decisions>
## Implementation Decisions

### Builder Guard Semantics
- **D-01:** `manual_capture()` is a plain builder setter; the Payment-mode requirement is enforced as a **runtime pre-flight check in `create()`** — a new dedicated structured `Error` variant fires before any network call when `manual_capture` is set with `Mode::Subscription`. Mirrors the existing `MissingIdempotencyKey` guard. No typestate builder (disproportionate complexity for one mode constraint).

### payment_intent Module API Shape
- **D-02:** Free functions in `ferro-stripe/src/payment_intent.rs` mirroring `refund.rs`: `capture(payment_intent_id: &str, amount_cents: Option<i64>) -> Result<stripe::PaymentIntent, Error>` (`None` = full capture, `Some(n)` = partial capture of `n` cents) and `cancel(payment_intent_id: &str) -> Result<stripe::PaymentIntent, Error>`. Signatures are roadmap-locked.
- **D-03:** Error contract identical to `refund.rs`: invalid id → `Error::Stripe(format!("invalid payment intent id: …"))`; API failures propagate via the existing `From<stripe::StripeError>` path.
- **D-04:** Module exported as `pub mod payment_intent` from lib.rs (capability-module pattern, like `refund`); no facade methods added elsewhere.

### Typed Event Payload Shape
- **D-05:** Minimal typed fields following the existing `events.rs` pattern:
  - `StripePaymentIntentAmountCapturableUpdated { payment_intent_id, amount_capturable_cents, currency, metadata }`
  - `StripePaymentIntentCanceled { payment_intent_id, cancellation_reason, metadata }`
- **D-06:** Both implement the `StripeEvent` trait (`from_raw` matching on `stripe::EventObject::PaymentIntent`); golden-JSON fixtures `payment_intent_amount_capturable_updated.json` and `payment_intent_canceled.json` added under `ferro-stripe/tests/fixtures/stripe_events/` and registered in `tests/parser_contract.rs`; non-matching event types continue to return `None` (pass-through preserved).

### Connect Composition Scope
- **D-07:** `capture()`/`cancel()` are **platform-scoped only**. Destination charges authorize on the platform account (`transfer_data.destination` + `on_behalf_of` already set by `destination()`); capture executes on the platform and Stripe performs the transfer. No `Stripe-Account` header parameter added to the new module.
- **D-08:** Composition verified by a builder-level test asserting the generated `CreateCheckoutSession` params contain BOTH `payment_intent_data.capture_method = manual` AND `transfer_data`/`on_behalf_of`/`application_fee_amount` when `manual_capture()` + `destination()` are combined. Live-mode verification is owned by the gestiscilo consumer field test.

### Documentation Structure
- **D-09:** New "Manual capture" section in `docs/src/features/stripe.md`: authorize-at-checkout flow, full/partial capture, cancel, webhook lifecycle, Connect composition.
- **D-10:** Correspondence table mapping `ferro-reservation` hold/commit/release ↔ Stripe authorize/capture/cancel — framed as a semantic parallel (conventions), explicitly no compile dependency between the crates.
- **D-11:** Document operational realities: ~7-day authorization window, Stripe auto-cancels expired uncaptured PaymentIntents (surfacing as the Canceled event), partial-capture remainder is auto-released by Stripe.

### Claude's Discretion
- Whether to include a `retrieve(payment_intent_id)` helper in `payment_intent.rs` for parity with `refund.rs` (cheap, useful for consumers polling authorization state)
- Exact name of the new `Error` variant for the mode guard
- Whether `manual_capture()` takes no args (bool flag) or is future-proofed — default to the no-args consuming-builder setter per existing `with_*`/builder conventions
- Internal representation of `capture_method` on the builder struct

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & phase definition
- `.planning/ROADMAP.md` §"v11.6.1 ferro-stripe Manual Capture (Phase 189)" — requirements STRIPE-MC-01..05, success criteria, out-of-scope boundary

### Existing code patterns (templates for this phase)
- `ferro-stripe/src/refund.rs` — capability-module pattern the new `payment_intent.rs` must mirror (free functions, error contract, async-stripe 0.41 idempotency caveat note)
- `ferro-stripe/src/checkout.rs` — `CheckoutBuilder` (builder conventions, `create()` pre-flight guard pattern, existing `payment_intent_data` construction in the `destination()` branch at ~line 198)
- `ferro-stripe/src/webhook/events.rs` — `StripeEvent` trait + 10 existing typed event structs (the template for the 2 new events)
- `ferro-stripe/tests/parser_contract.rs` + `ferro-stripe/tests/fixtures/stripe_events/` — golden-JSON fixture registration pattern (e.g. `payment_intent_payment_failed.json`)
- `ferro-stripe/src/error.rs` — `Error` enum (new mode-guard variant lands here)

### Prior phase context
- `.planning/phases/96-stripe-integration/96-CONTEXT.md` — crate-level decisions (Checkout Sessions, webhook architecture, two-tier billing model) that this phase extends

### Docs
- `docs/src/features/stripe.md` — existing Stripe docs; manual-capture section + correspondence table land here

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CheckoutBuilder` already constructs `CreateCheckoutSessionPaymentIntentData` for the `destination()` branch — `capture_method` slots into the same struct; the two features must merge into ONE `payment_intent_data` construction (not two competing `Some(...)` assignments)
- `refund.rs` free-function pattern transfers directly to `payment_intent.rs` (id parse → params → API call → typed return)
- `StripeEvent::from_raw` + `stripe::EventObject::PaymentIntent` matching already exercised by `StripePaymentIntentFailed` and `StripeConnectPaymentSucceeded`
- Parser contract test + fixtures directory established — adding 2 events is additive

### Established Patterns
- async-stripe pinned at `0.41` (default-features = false) — verify `CreateCheckoutSessionPaymentIntentData` exposes `capture_method` and `PaymentIntent::capture`/`cancel` exist in this version BEFORE planning task breakdown (validate-scope-premises)
- Per-request idempotency keys not forwarded by async-stripe 0.41 — same caveat note as `refund.rs` applies to capture (application-layer dedup)
- `thiserror` single Error enum per crate; consuming builder `mut self -> Self`
- Workspace publish via GH Actions on master push; ferro-stripe is an existing crate (publish-update token sufficient — no manual bootstrap needed)

### Integration Points
- `ferro-stripe/src/lib.rs` — `pub mod payment_intent;` + re-export the 2 new events in the existing `pub use webhook::events::{...}` block
- `docs/src/features/stripe.md` — docs requirement is a success criterion, not an afterthought
- No framework/ferro-mcp changes expected (no new routes/models/commands) — confirm at plan time

</code_context>

<specifics>
## Specific Ideas

- The authorize/capture/cancel ↔ hold/commit/release correspondence is deliberate design language: consumer apps (gestiscilo bookings) pair a `ferro-reservation` hold with a Stripe authorization and resolve both together. The docs table is the contract; no code coupling.
- Partial capture (`Some(n)`) is first-class because booking deposits commonly capture less than the authorized amount (e.g. no-show fee < full hold).

</specifics>

<deferred>
## Deferred Ideas

- **SetupIntent save-card flow** for authorizations beyond the ~7-day card window — explicitly out of scope per roadmap; promote to a ferro phase only if gestiscilo v6.3 picks that path.

</deferred>

---

*Phase: 189-ferro-stripe-manual-capture*
*Context gathered: 2026-06-07*

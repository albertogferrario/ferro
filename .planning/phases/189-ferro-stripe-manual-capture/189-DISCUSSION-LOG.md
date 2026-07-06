# Phase 189: ferro-stripe Manual Capture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 189-ferro-stripe-manual-capture
**Mode:** --auto (recommended defaults selected without interactive questions)
**Areas discussed:** Builder guard semantics, payment_intent module API shape, Typed event payload shape, Connect composition scope, Documentation structure

---

## Builder Guard Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime pre-flight error in `create()` | Dedicated structured `Error` variant fired before any network call when `manual_capture` set with `Mode::Subscription` — mirrors `MissingIdempotencyKey` guard | ✓ |
| Typestate builder | Compile-time rejection via mode-parameterized builder type | |
| Silent ignore | Drop `capture_method` in subscription mode | |

**Auto-selected:** Runtime structured error — consistent with the crate's existing guard pattern; typestate is disproportionate for one constraint; silent ignore hides consumer bugs.

---

## payment_intent Module API Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Free functions mirroring `refund.rs` | `capture(id, amount_cents: Option<i64>)` + `cancel(id)` in `payment_intent.rs`, same error contract | ✓ |
| `PaymentIntent` wrapper struct | Methods on a typed handle | |
| Methods on `CheckoutIntent` | Couple capture to the checkout return value | |

**Auto-selected:** Free functions — roadmap-locked signatures; matches the established capability-module pattern.

---

## Typed Event Payload Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal typed fields (existing pattern) | AmountCapturableUpdated: pi id, amount_capturable_cents, currency, metadata; Canceled: pi id, cancellation_reason, metadata | ✓ |
| Full PaymentIntent passthrough | Expose the whole `stripe::PaymentIntent` object | |

**Auto-selected:** Minimal typed fields — consistent with all 10 existing event structs; consumers needing more can `retrieve()`.

---

## Connect Composition Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Platform-scoped capture only | Destination charges capture on platform; Stripe handles the transfer; no `Stripe-Account` header surface | ✓ |
| Connected-account parameter | Optional account scoping on capture/cancel | |

**Auto-selected:** Platform-scoped — correct for the destination-charge pattern already used by `destination()`; adding account scoping would be a second control surface with no current consumer.

---

## Documentation Structure

| Option | Description | Selected |
|--------|-------------|----------|
| "Manual capture" section + correspondence table in stripe.md | Flow docs + hold/commit/release ↔ authorize/capture/cancel table + 7-day window / auto-cancel / remainder-release notes | ✓ |
| Separate doc page | New docs page for manual capture | |

**Auto-selected:** Section in existing `docs/src/features/stripe.md` — roadmap names this file explicitly.

---

## Claude's Discretion

- `retrieve(payment_intent_id)` helper inclusion (parity with refund.rs)
- Name of the new mode-guard `Error` variant
- `manual_capture()` setter internal representation

## Deferred Ideas

- SetupIntent save-card flow (beyond ~7-day auth window) — out of scope per roadmap; consumer-side decision pending.

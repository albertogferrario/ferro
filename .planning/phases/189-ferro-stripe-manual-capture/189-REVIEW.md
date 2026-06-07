---
phase: 189-ferro-stripe-manual-capture
reviewed: 2026-06-07T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - docs/src/features/stripe.md
  - ferro-stripe/src/checkout.rs
  - ferro-stripe/src/error.rs
  - ferro-stripe/src/lib.rs
  - ferro-stripe/src/payment_intent.rs
  - ferro-stripe/src/webhook/events.rs
  - ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json
  - ferro-stripe/tests/fixtures/stripe_events/payment_intent_canceled.json
  - ferro-stripe/tests/parser_contract.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 189: Code Review Report

**Reviewed:** 2026-06-07
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 189 adds Stripe manual capture to ferro-stripe: `CheckoutBuilder::manual_capture()`,
a `payment_intent` module (capture/cancel/retrieve), two new typed webhook events with
golden fixtures, and a docs section.

The implementation is sound. I verified the focus areas directly against async-stripe
0.41 source:

- **i64→u64 conversion** (`payment_intent.rs:29-35`) is correct: `amount_to_capture`
  is `Option<u64>` in async-stripe, and `u64::try_from` rejects negatives before the
  network call.
- **Pre-flight guards** (`checkout.rs:184-189`) correctly check idempotency key first,
  then `manual_capture + Subscription`, both before any `Stripe::client()` call. Tests
  prove this.
- **Unified `payment_intent_data` construction** (`checkout.rs:150-170`) correctly merges
  `manual_capture` and `destination` into one struct, eliminating the double-overwrite
  hazard. Field types verified: `application_fee_amount: Option<i64>`,
  `capture_method`, `on_behalf_of: Option<String>` all match.
- **Event parsing** (`events.rs:244-290`) follows the established `type_` guard +
  `EventObject` match pattern; `amount_capturable: i64` confirmed in async-stripe.
- **Docs examples** match shipped API signatures (`capture(&id, Option<i64>)`,
  `cancel(&id)`, `manual_capture()`).

No critical or security issues. Two warnings concern an off-by-one in the
"must be positive" contract and a docs/code semantic mismatch on the capturable amount.
Info items are minor consistency notes.

## Warnings

### WR-01: `capture(amount = Some(0))` is accepted despite the "must be positive" contract

**File:** `ferro-stripe/src/payment_intent.rs:29-35`
**Issue:** The doc comment (line 16) states `n` "must be positive", and the error
message (line 33) says `"amount_to_capture must be positive"`. But the validation uses
`u64::try_from(n)`, which succeeds for `n == 0` — zero is not negative. A zero-amount
partial capture is forwarded to Stripe, which then rejects it with a less actionable
API error (or, depending on Stripe behavior, captures nothing). The unit test
`capture_rejects_negative_amount` only covers `Some(-5)`, so the `0` boundary is
untested. This is a classic off-by-one in the guard: the message promises `> 0` but the
code enforces `>= 0`.
**Fix:**
```rust
let amount_to_capture = match amount_cents {
    None => None,
    Some(n) if n <= 0 => {
        return Err(Error::Stripe(
            "amount_to_capture must be positive".to_string(),
        ));
    }
    Some(n) => Some(n as u64), // n > 0 here, cast is safe
};
```
Add a `capture_rejects_zero_amount` test asserting `Some(0)` returns the
"must be positive" error before any network call.

### WR-02: Docs imply the capturable amount is captured; field is informational only

**File:** `docs/src/features/stripe.md:289-303` (webhook lifecycle / operational realities)
**Issue:** `StripePaymentIntentAmountCapturableUpdated` exposes `amount_capturable_cents`,
and the docs describe the event as "Funds authorized and capturable (hold is live)".
A reader integrating this may reasonably pass `amount_capturable_cents` straight into
`payment_intent::capture(id, Some(amount_capturable_cents))`, expecting a full capture.
For a full capture that works, but the idiomatic and safer call is `capture(id, None)`
(let Stripe capture the full authorized amount). Threading the webhook-reported integer
back into a later capture call invites a stale-value bug if the authorization changed.
The docs never state the relationship between the event field and the capture argument,
leaving the safe pattern implicit.
**Fix:** Add one sentence to the webhook lifecycle section: "To capture the full
authorized amount, prefer `capture(&id, None)` rather than echoing
`amount_capturable_cents` from the event — the `None` form always captures the current
full authorization and avoids stale-amount races." This is a docs-only change.

## Info

### IN-01: `idempotency_key.clone().unwrap()` is redundant given the earlier ref check

**File:** `ferro-stripe/src/checkout.rs:184-191`
**Issue:** Line 184 checks `self.idempotency_key.is_none()` and returns early; line 191
then does `self.idempotency_key.clone().unwrap()`. The `unwrap()` is safe (commented
SAFETY), but the clone-then-unwrap can be expressed without the panic-path. Since `self`
is owned by `create()`, the value can be moved out.
**Fix:** Replace the guard + unwrap with a single binding that consumes the Option, e.g.
`let Some(idempotency_key) = self.idempotency_key.clone() else { return Err(Error::MissingIdempotencyKey); };`
placed before the `manual_capture` guard. Removes the `unwrap()` and the SAFETY comment.
Cosmetic — current code is correct.

### IN-02: Fixtures only cover the happy path; no missing-currency or absent-metadata case

**File:** `ferro-stripe/tests/fixtures/stripe_events/payment_intent_amount_capturable_updated.json`
**Issue:** Both new fixtures populate `currency`, `metadata`, and (for canceled)
`cancellation_reason`. `from_raw` for `StripePaymentIntentCanceled` maps
`cancellation_reason` through `Option::map`, and the docs (stripe.md:302) specifically
call out the auto-expiry path where `cancellation_reason` differs. No fixture exercises
`cancellation_reason: null` or empty metadata, so the `Option`/`unwrap_or_default`
branches in `events.rs:281-288` are untested.
**Fix:** Add a second `payment_intent_canceled` fixture (e.g. auto-expiry with
`cancellation_reason: "abandoned"` or `null` and no `metadata`) and a parser test
asserting `cancellation_reason` and empty-metadata handling. Strengthens the
pass-through contract the phase is built on.

### IN-03: Stale `TODO(140)` markers carried into the manual-capture docs file

**File:** `docs/src/features/stripe.md:210`, `docs/src/features/stripe.md:482`
**Issue:** Two `<!-- TODO(140): ... -->` comments remain in the doc file this phase
edited. They predate Phase 189 but live in a file the phase touched; leaving stale
phase-140 TODOs in shipped docs is debt that accumulates.
**Fix:** Either resolve the rewording the TODOs request or drop the comments. Not a
Phase 189 regression — flagged because the file is in scope.

---

_Reviewed: 2026-06-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

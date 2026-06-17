---
phase: 234-ferro-payments-billable-trait-loader-and-payment-service-cor
reviewed: 2026-06-17T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - ferro-payments/src/service.rs
  - ferro-payments/src/billable.rs
  - ferro-payments/src/loader.rs
  - ferro-payments/src/error.rs
  - ferro-payments/src/intent/lifecycle.rs
  - ferro-payments/src/lib.rs
  - ferro-payments/Cargo.toml
  - .github/workflows/publish.yml
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 234: Code Review Report

**Reviewed:** 2026-06-17T00:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

The `ferro-payments` crate is well-structured. The `Billable` / `BillableLoader` traits are correctly async-trait-annotated and object-safe. `PaymentService<L>` composes cleanly over the `StripeGateway` seam. The `#[allow(dead_code)]` is correctly scoped to the field. No hardcoded app identity was found — `return_url_builder` is consumer-supplied throughout.

The four warnings below are correctness-affecting: one is a silent data-loss risk in `request_refund` when Stripe returns an error after the dedup write succeeds, two are integer-math edge cases (`application_fee_for` / zero amount), and one is a `BillableKind` API limitation. The info items are minor polish.

## Warnings

### WR-01: Stripe failure after dedup write leaves `refund_amount_cents` set but no refund submitted

**File:** `ferro-payments/src/service.rs:279-296`

**Issue:** `request_refund` atomically writes `refund_amount_cents` (the IS-NULL guard, line 279) and then calls `stripe.create_refund` (line 293). If the Stripe call fails (network error, rate limit, invalid charge), the function returns `Err(PaymentError::Stripe(...))`. On any subsequent retry the guard evaluates the column as NOT NULL, `snapshot_ok` is `false`, and the function returns `Ok(())` — a silent no-op. The intent row records a refund amount that was never submitted and the status is never flipped to `refunded`. The caller sees success on retry and never realises the refund did not reach Stripe.

The dedup invariant ("exactly one call to Stripe") is correct for the concurrency case, but the write-then-fail sequence creates an irrecoverable stuck state with no error signal on subsequent attempts.

**Fix:** Two acceptable approaches:

Option A — Write a `refund_requested_at` timestamp column as the dedup sentinel instead of `refund_amount_cents`. Write the sentinel before Stripe; write `refund_amount_cents` only after Stripe confirms. Then a retry that finds the sentinel but no `refund_amount_cents` knows the previous attempt failed and can retry Stripe (idempotency key `"refund-{intent_id}"` is stable — safe to re-submit).

Option B — On Stripe failure, clear the dedup column in a compensating write so the next attempt can proceed. Simpler but adds a compensating path that itself can fail.

Option A is recommended because it distinguishes "in-flight" from "confirmed" without a compensating delete.

---

### WR-02: `application_fee_for` is called with the total `amount_cents`, not the fee amount; silent wrong-fee path

**File:** `ferro-payments/src/service.rs:101-104`

**Issue:**
```rust
let application_fee_cents = req
    .connect_account_id
    .as_ref()
    .and_then(|_| ferro_stripe::Stripe::config().application_fee_for(req.amount_cents));
```

`application_fee_for` is a method on the `Stripe` config object. Whether it computes a percentage of `req.amount_cents` or looks up a flat fee, the argument being passed is the **full charge amount**. If the intent is "compute the platform fee from the total", this is correct. However, the `and_then` discards the `connect_account_id` value — the inner closure receives the account id but ignores it, then calls a config-level method that has no visibility of which account is being charged. If different connected accounts have different fee structures, this silently applies a single global rate regardless of destination.

This is architectural (the fee API lives in `ferro-stripe`, not here), but the service code reinforces the assumption that fee rate is global. Add a doc comment explaining the fee model or, if per-account fees are required, pass `account_id` into `application_fee_for`.

**Fix:**
```rust
// If ferro_stripe::StripeConfig::application_fee_for is always a global rate,
// document that assumption here explicitly.
let application_fee_cents = req
    .connect_account_id
    .as_ref()
    .and_then(|account_id| {
        // Global platform fee rate — all connected accounts share one rate.
        let _ = account_id; // account_id intentionally unused (global rate)
        ferro_stripe::Stripe::config().application_fee_for(req.amount_cents)
    });
```

If per-account fees are intended, the signature of `application_fee_for` needs an account id parameter.

---

### WR-03: `start_checkout` does not guard against `amount_cents <= 0`

**File:** `ferro-payments/src/service.rs:192-236`

**Issue:** `billable.amount_cents()` is passed directly to `CheckoutRequest` without validation. Stripe rejects zero and negative amounts with a 400, leaving an orphaned `reserved` row for the reaper. This is a predictable misconfiguration; catching it before the row is inserted produces a better error (`StatusPrecondition`) and avoids the orphan.

**Fix:**
```rust
pub async fn start_checkout(
    &self,
    billable: &dyn Billable,
    ttl: chrono::Duration,
) -> Result<CheckoutUrl, PaymentError> {
    if billable.amount_cents() <= 0 {
        return Err(PaymentError::StatusPrecondition(
            "amount_cents must be positive".to_string(),
        ));
    }
    // ...
}
```

---

### WR-04: `BillableKind::new` accepts only `&'static str` — prevents runtime-constructed kinds

**File:** `ferro-payments/src/lib.rs:29-38`

**Issue:**
```rust
pub struct BillableKind(&'static str);

impl BillableKind {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}
```

The `&'static str` constraint means all kind strings must be compile-time literals. A consumer that stores `billable_kind` as a `String` from a database row cannot construct a `BillableKind` for dispatch without leaking memory via `Box::leak` or requiring every kind to be a static constant. This is an intentional design decision to keep kinds constant (open-set enum), but it blocks the loader pattern: `BillableLoader::load` receives a `BillableKind` — fine — but where does a webhook handler construct one from a database-read `String`? If this path exists in phase 235 it will require `Box::leak` or a from-String constructor.

**Fix:** Add a `from_string` constructor that stores an owned `Arc<str>` or `Cow<'static, str>`, and change the internal representation accordingly. If the static-only constraint is intentional (all valid kinds are named constants), document it explicitly so phase 235 authors know they need a static constant per kind, not a from-db constructor.

---

## Info

### IN-01: `#[allow(dead_code)]` comment cites phase 235 — confirm scope and remove when wired

**File:** `ferro-payments/src/service.rs:151-152`

**Issue:**
```rust
#[allow(dead_code)] // wired by handle_* in phase 235
loader: L,
```

The allow is correctly scoped to the field (not the struct), which is the right pattern. The comment documents the intended phase. No action required until phase 235, but the allow should be removed once `loader` is used — otherwise clippy's dead-code lint is permanently suppressed for this field.

---

### IN-02: `error.rs` — `AutoRefundReason::SideStateConflict` is unused in this phase

**File:** `ferro-payments/src/error.rs:44-45`

**Issue:** `SideStateConflict` is defined in `AutoRefundReason` with the comment "Defined here (D-18); only RETURNED by the webhook handlers in phase 235." This is fine as forward-declaration, but `#[allow(dead_code)]` is absent — if clippy's dead-code lint fires on this variant in CI, the build will fail under `-D warnings`. The enum itself is `pub`, so exported dead variants typically don't trigger the lint, but worth verifying.

**Fix:** If CI reports a dead-code warning for this variant, add `#[allow(dead_code)]` to the `SideStateConflict` variant or construct a dummy usage path in a doc test.

---

### IN-03: `publish.yml` — Wave 1c sleep delay is identical to Wave 1b but Wave 1b has more crates

**File:** `.github/workflows/publish.yml:291-292`

**Issue:** Wave 1b publishes 9 crates with a `sleep 5` between each (45 s of publication time + `sleep 30` index wait). Wave 1c publishes only `ferro-payments` but still waits 30 s before Wave 2. The wave structure and wait times are consistent but the 30 s wait after Wave 1c is unnecessary since Wave 2 does not depend on Wave 1c (`ferro-payments` is not listed in Wave 2's `WAVE2_CRATES`). This is a minor CI time concern, not a correctness issue.

---

_Reviewed: 2026-06-17T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-
reviewed: 2026-06-17T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-payments/src/webhook.rs
  - ferro-payments/src/service.rs
  - ferro-payments/src/intent/lifecycle.rs
  - ferro-payments/src/lib.rs
  - ferro-payments/Cargo.toml
  - ferro-stripe/src/refund.rs
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 235: Code Review Report

**Reviewed:** 2026-06-17
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 235 delivers the webhook integration layer: `wire_dispatcher`, three idempotency-guarded handlers (`handle_session_completed`, `handle_session_expired`, `handle_charge_refunded`), the `trigger_auto_refund` fallback, and the `refund::create_for_payment_intent` primitive in `ferro-stripe`.

The core safety properties hold. Idempotency via `ProcessedEventLog` is correct. All GuardedUpdate state transitions use `WHERE` guards that make concurrent callers safe. The D-11 no-compensate-reset invariant is respected — `refund_amount_cents` is never cleared on Stripe failure, and the reaper path is documented. No direct `stripe::` imports appear in `ferro-payments`. No hardcoded app identity strings. No `unwrap` on happy or webhook paths. `wire_dispatcher` closures are `Fn` (double `Arc::clone` pattern is correct). `BillableKind` uses `Cow<'static, str>` correctly. The test matrix is comprehensive.

Four warnings were found: two correctness concerns in transaction/error handling, one silent data loss risk in `attach_session`'s return value being ignored, and one concern with the error bridge classification. Three info items note the non-forwarded idempotency key, a missing no-pi-id test case, and a dead variant in the public error type.

---

## Warnings

### WR-01: `attach_session` return value silently discarded — a second `start_checkout` call overwrites nothing but the caller cannot distinguish retry from first-write

**File:** `ferro-payments/src/service.rs:258-264`
**Issue:** `lifecycle::attach_session` returns `Ok(bool)` — `false` means the guard excluded the row (session already attached). The call site discards this value entirely with no `?` and no branch, so if a second `start_checkout` for the same row arrives concurrently (retry scenario), the guard fires and the caller returns `Ok(CheckoutUrl(…))` with the URL from `resp.intent.url` — which came from the *new* Stripe response, not the one that got durably written. Under normal operation this is harmless because the call returns a valid URL. Under a specific retry race (first call's `attach_session` succeeds, second call reaches `attach_session` with a different session_id), the second caller silently returns a different Checkout URL that will never be found in the DB by session. This is not currently exploitable because `create_reserved` enforces the partial-unique-index for active rows, so no two concurrent `start_checkout` calls for the same billable can both proceed past the INSERT. However the unhandled `bool` is still semantically incorrect and will mask bugs if that invariant ever softens.

**Fix:** Log a warning or return early when `attach_session` returns `Ok(false)`:
```rust
let attached = lifecycle::attach_session(
    row.id,
    &resp.intent.session_id,
    resp.application_fee_cents,
    &self.db,
)
.await?;
if !attached {
    tracing::warn!(row_id = row.id, "attach_session no-op: session already attached");
}
```

---

### WR-02: `handle_session_completed` — `on_paid` error swallowed and re-classified as `SideStateConflict`

**File:** `ferro-payments/src/webhook.rs:172-180`
**Issue:** When `billable.on_paid(&txn)` returns `Err(e)`, the error `e` is silently dropped and `trigger_auto_refund` is called with `AutoRefundReason::SideStateConflict`. This means:
1. A transient DB error inside `on_paid` (e.g. a deadlock) triggers an auto-refund. That is destructive — the money is returned to the customer even though the failure was transient and the payment was legitimate.
2. The original error is lost; operators see only "auto-refund triggered: SideStateConflict" in the logs with no root cause.

`SideStateConflict` is the correct reason when the billable's business state is incompatible (e.g. the slot was already released), but it is not the correct response to an infrastructure error inside `on_paid`. At minimum, a transient `Db` error variant should propagate (causing Stripe to retry) rather than triggering a permanent refund.

**Fix:** Inspect the error before deciding whether to auto-refund:
```rust
Err(e) => {
    txn.rollback().await.ok();
    match e {
        // Transient infrastructure error — propagate so Stripe retries.
        PaymentError::Db(_) | PaymentError::Stripe(_) => return Err(e),
        // Permanent business-state conflict — auto-refund.
        _ => {
            self.trigger_auto_refund(
                &event.payment_intent_id,
                event.amount_total_cents,
                intent.id,
                AutoRefundReason::SideStateConflict,
            )
            .await
        }
    }
}
```

---

### WR-03: `payment_to_stripe_error` bridge passes `NotFound` and `StatusPrecondition` to Stripe — causing retry storms for terminal conditions

**File:** `ferro-payments/src/webhook.rs:86-91`
**Issue:** The doc-comment on `payment_to_stripe_error` states: *"Terminal outcomes (NotFound, StatusPrecondition, AutoRefundTriggered) must never reach this bridge — handlers absorb them and return `Ok(())`."* However, the three handlers do NOT universally absorb all three variants before the bridge is reached.

In `handle_session_completed`:
- `find_by_stripe_session` returns `?` on `Err` — a `PaymentError::Db` reaches the bridge correctly.
- `mark_paid` returns `?` — same.
- `attach_payment_intent` returns `?` — same.

In `handle_session_expired` and `handle_charge_refunded`:
- `on_released` and `on_refunded` errors are explicitly returned with `return Err(e)` (lines 220, 276), then converted by `payment_to_stripe_error`. If a consumer's `on_released`/`on_refunded` returns `PaymentError::StatusPrecondition(...)`, that variant hits `payment_to_stripe_error`'s wildcard arm:
  ```rust
  other => ferro_stripe::Error::Stripe(format!("payment: {other}")),
  ```
  This wraps it as a generic Stripe error, causing Stripe to retry — but the condition is terminal (the billable's state machine already rejected it). This will produce a retry storm on every such event.

**Fix:** The bridge's wildcard arm should be treated as a programming error rather than a retriable condition. Either:
1. Have handlers absorb `StatusPrecondition` before propagating (mirror the `mark_released`/`mark_paid` no-op pattern), or
2. Add a guard in the bridge:
```rust
fn payment_to_stripe_error(e: PaymentError) -> ferro_stripe::Error {
    match e {
        PaymentError::Stripe(s) => s,
        // These are terminal — log and swallow rather than causing Stripe retries.
        PaymentError::NotFound
        | PaymentError::StatusPrecondition(_)
        | PaymentError::AutoRefundTriggered { .. } => {
            tracing::warn!(err = %e, "terminal PaymentError absorbed at webhook bridge — not retrying");
            // Return an error type that the dispatcher absorbs without re-queuing,
            // or restructure handlers to return Ok(()) on these variants.
            ferro_stripe::Error::Stripe(format!("terminal (not retrying): {e}"))
        }
        other => ferro_stripe::Error::Stripe(format!("payment: {other}")),
    }
}
```
The cleanest fix is for `handle_session_expired` and `handle_charge_refunded` to absorb `StatusPrecondition` from `on_released`/`on_refunded` (returning `Ok(())`) rather than propagating it.

---

### WR-04: `request_refund` Stripe failure leaves `refund_amount_cents` set but Stripe was never called — the row is silently "refund-in-flight" with no log

**File:** `ferro-payments/src/service.rs:323-328`
**Issue:** The D-15 flow snapshots `refund_amount_cents` then calls `create_refund`. If `create_refund` returns an error, `request_refund` propagates it to the caller via `map_err(PaymentError::Stripe)`. The refund dedup guard (the IS NULL check) has already fired, so any subsequent call to `request_refund` will return `Ok(())` as a no-op, and Stripe will never be called again. Unlike `trigger_auto_refund` (which logs at `tracing::error` when Stripe fails), `request_refund` leaves no trace of the stuck state.

The phase-236 reaper is documented as the recovery path for `trigger_auto_refund` failures; but `request_refund`'s failure mode produces the same "refund-in-flight" sentinel (`refund_amount_cents IS NOT NULL`, `refunded_at IS NULL`) without any documentation or log entry pointing to the reaper. A future operator will not know why the row is stuck.

**Fix:** Log the stuck state explicitly on Stripe failure, mirroring `trigger_auto_refund`:
```rust
self.stripe
    .create_refund(&charge_id, Some(amount_cents), &idempotency_key)
    .await
    .map_err(|e| {
        tracing::error!(
            intent_id,
            %charge_id,
            err = %e,
            "request_refund Stripe call failed; row is refund-in-flight \
             (refund_amount_cents set, refunded_at NULL) — phase-236 reaper recovers"
        );
        PaymentError::Stripe(e)
    })
```

---

## Info

### IN-01: `idempotency_key` parameter silently unused in `ferro-stripe/refund.rs` — no deduplication guarantee at the Stripe layer

**File:** `ferro-stripe/src/refund.rs:27` and `67`
**Issue:** Both `create` and `create_for_payment_intent` accept an `idempotency_key` parameter but immediately discard it (`let _ = idempotency_key`). The doc-comment notes this honestly (async-stripe 0.41 caveat), and the auto-refund path uses the `WHERE refund_amount_cents IS NULL` guard as application-layer dedup. However, the idempotency_key on `auto-refund-{intent_id}` is deterministic precisely to survive an async-stripe upgrade without caller changes. When async-stripe exposes the mechanism, a future developer must not miss either call site.

**Suggestion:** Add a `// TODO(async-stripe-upgrade): pass idempotency_key here` comment at both suppressed `let _ = idempotency_key` lines so a search for `async-stripe` upgrade notes finds them.

---

### IN-02: Missing test case — `trigger_auto_refund` when `payment_intent_id` is `None`

**File:** `ferro-payments/src/webhook.rs`
**Issue:** `trigger_auto_refund` has an early return (`Ok(())`) when `payment_intent_id` is `None` (line 305-308), documented as "free/setup session". No test exercises this branch. The existing tests all provide a `Some("pi_…")`. If the early-return branch were accidentally removed or the condition inverted, no test would catch it.

**Suggestion:** Add a test where `make_completed_event` is called with `pi_id = None` and a loader that returns `Ok(None)`, asserting that the result is `Ok(())` and no refund call is made:
```rust
#[tokio::test]
async fn auto_refund_skipped_when_no_payment_intent_id() {
    // ... seed reserved, loader returning None, event with pi_id=None
    // assert: handle_session_completed returns Ok(())
    // assert: mock_stripe.pi_refund_calls().is_empty()
}
```

---

### IN-03: `PaymentError::AutoRefundTriggered` is defined but never returned — dead public variant

**File:** `ferro-payments/src/error.rs:30-33`
**Issue:** The doc-comment on `AutoRefundTriggered` says "Only RETURNED by the webhook handlers in phase 235." But the phase-235 handlers do not return this variant: all auto-refund branches call `trigger_auto_refund` and return its `Ok(())`. `AutoRefundTriggered` is also listed in `payment_to_stripe_error`'s doc-comment as a "must never reach this bridge" variant, but since it is never actually constructed, this is dead code. Consumers who pattern-match `PaymentError` will never observe this arm.

**Suggestion:** Either remove the `AutoRefundTriggered` variant if it is confirmed unused, or implement the code path that returns it (and update `payment_to_stripe_error` to absorb it). Leaving a documented-but-unused variant is misleading.

---

_Reviewed: 2026-06-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

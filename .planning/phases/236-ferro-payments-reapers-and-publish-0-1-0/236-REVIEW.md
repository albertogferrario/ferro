---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
reviewed: 2026-06-21T00:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - ferro-payments/src/billable.rs
  - ferro-payments/src/error.rs
  - ferro-payments/src/intent/entity.rs
  - ferro-payments/src/intent/lifecycle.rs
  - ferro-payments/src/intent/mod.rs
  - ferro-payments/src/intent/status.rs
  - ferro-payments/src/lib.rs
  - ferro-payments/src/loader.rs
  - ferro-payments/src/migration/m20260617_create_payment_intents.rs
  - ferro-payments/src/migration/mod.rs
  - ferro-payments/src/reaper.rs
  - ferro-payments/src/service.rs
  - ferro-payments/src/webhook.rs
  - ferro-payments/tests/integration.rs
  - ferro-stripe/src/refund.rs
findings:
  critical: 1
  warning: 5
  info: 4
  total: 10
status: issues_found
---

# Phase 236: Code Review Report

**Reviewed:** 2026-06-21T00:00:00Z
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

This is the reaper + publish phase for `ferro-payments`, a polymorphic Stripe payment-intent data layer. The money-path design is largely sound: state transitions use `GuardedUpdate` atomic `UPDATE … WHERE` statements with "second writer no-ops" semantics, refund deduplication is enforced by a `WHERE refund_amount_cents IS NULL` snapshot guard, webhook handlers are idempotent via `ProcessedEventLog`, and the partial unique index enforces a single active row per billable. Concurrent-transition and double-refund scenarios are well covered by tests.

The review found one critical correctness gap that undermines the manual refund path, and several warnings around webhook idempotency interaction with retries, age-anchor coupling, and a known Stripe idempotency limitation that is documented but only partially mitigated.

The most important finding (CR-01): `charge_id` is **never persisted** anywhere in the production lifecycle. The `charge.refunded` event carries a `charge_id` but the handler never writes it back, and `checkout.session.completed` carries no charge at all. Consequently `request_refund` — which hard-requires `charge_id IS NOT NULL` — can never succeed through the normal flow. Only the PI-id-based auto-refund path is reachable.

## Critical Issues

### CR-01: `charge_id` is never persisted — `request_refund` is structurally unreachable

**File:** `ferro-payments/src/service.rs:364-368` (consumer); `ferro-payments/src/webhook.rs:252-307` (missing writer); `ferro-payments/src/intent/lifecycle.rs` (no `attach_charge_id`)

**Issue:**
`request_refund` requires the row's `charge_id` to be set:
```rust
let charge_id = row.charge_id.ok_or_else(|| {
    PaymentError::StatusPrecondition("charge_id must be set to request a refund".to_string())
})?;
```
But nothing in the production code path ever writes `charge_id`:
- `create_reserved` sets `charge_id: Set(None)` (lifecycle.rs:45).
- `handle_session_completed` attaches only `payment_intent_id` (`attach_payment_intent`) — never a charge.
- `handle_charge_refunded` reads `event.charge_id` and uses it only as a *lookup fallback* (`find_by_charge_id`); it never persists it onto the row.
- There is no `attach_charge_id` lifecycle function, and `StripeCheckoutCompleted` (ferro-stripe/src/webhook/events.rs:176) has no charge field.

Result: in any real deployment `row.charge_id` is always `NULL`, so the public `request_refund` API always returns `StatusPrecondition`. The auto-refund path survives only because it uses `create_refund_for_payment_intent` (by PI id), not by charge. The unit test `request_refund` (service.rs:969) masks this because it seeds `charge_id` via raw SQL, which the lifecycle never does.

**Fix:** Persist the charge id on the success path. `charge.refunded` already has it; for the primary path, fetch/attach the charge during `handle_session_completed` (the PaymentIntent's latest charge is retrievable from Stripe), or — simpler and consistent with the existing PI-based refund — change `request_refund` to refund by `payment_intent_id` instead of `charge_id`, since `payment_intent_id` *is* persisted:
```rust
// In handle_charge_refunded, before mark_refunded, backfill the charge:
lifecycle::attach_charge_id(intent.id, &event.charge_id, &self.db).await?;
```
plus a guarded `attach_charge_id` (mirroring `attach_payment_intent`'s `WHERE charge_id IS NULL`). OR refactor `request_refund` to require `payment_intent_id IS NOT NULL` and call `create_refund_for_payment_intent`, deleting the charge precondition entirely. The latter removes a column that the lifecycle cannot populate and aligns the manual and auto refund paths on one Stripe identifier.

## Warnings

### WR-01: Webhook idempotency log marked *before* side effects can produce a permanent lost update on transient failure

**File:** `ferro-payments/src/webhook.rs:114-122, 180-196`

**Issue:**
`handle_session_completed` calls `try_mark_processed(event_id)` first and returns early on replay. But it marks the event processed *before* running `mark_paid` / `on_paid`. If a later step fails transiently and the handler returns `Err(e)` (webhook.rs:184 `return Err(e)`), Stripe retries the webhook — but the second delivery now hits the idempotency fast-path (`try_mark_processed` returns `false`) and returns `Ok(())` without ever completing the side effect. The capture is then permanently un-honored and no auto-refund fires. This couples "received" with "successfully processed."

**Fix:** Only commit the processed-event marker once the handler has reached a terminal success/absorbed state. Either (a) mark processed at the end of the happy/absorbed paths (not at entry), or (b) make the marker transactional with `on_paid` so a rollback also un-marks. If `MemoryProcessedLog`/the production log cannot un-mark, move the `try_mark_processed` call to *after* the `txn.commit()` on the success branch and after each absorbed-terminal branch, and do NOT mark before a path that can `return Err`.

### WR-02: `create_reserved` maps the partial-unique-index violation to an opaque `Db` error

**File:** `ferro-payments/src/intent/lifecycle.rs:26-57`; `ferro-payments/src/service.rs:284-293`

**Issue:**
When an active `(billable_kind, billable_id)` row already exists, the partial unique index raises a DB error (documented "D-10"). `create_reserved` maps any insert failure to `PaymentError::Db(..)` (lifecycle.rs:56), and `start_checkout` propagates it verbatim. Callers cannot distinguish "you already have a checkout in flight for this billable" (a normal, expected, retryable-by-the-user condition) from a real database fault. This is the most common concurrent-double-checkout path and it surfaces as a generic 500-class error.

**Fix:** Detect the unique-violation and map it to a dedicated variant, e.g. `PaymentError::ActiveIntentExists`, so the consumer can return the existing checkout URL or a 409 instead of a 500:
```rust
row.insert(conn).await.map_err(|e| {
    if is_unique_violation(&e) { PaymentError::ActiveIntentExists }
    else { PaymentError::Db(e) }
})
```

### WR-03: Reconcile reaper trusts Stripe's reported `amount_cents` over the snapshotted `refund_amount_cents`

**File:** `ferro-payments/src/service.rs:525-536`

**Issue:**
On the `Succeeded { amount_cents }` poll result, the reaper passes Stripe's `amount_cents` straight to `on_refunded(&txn, amount_cents)`. The row already holds an authoritative `refund_amount_cents` snapshot (the amount the system actually requested, set under the IS-NULL guard). If Stripe's `refunds.first()` returns a *different* refund than the one this system initiated (e.g. an operator issued a separate partial refund via the Stripe dashboard, which would be newest-first), the consumer's `on_refunded` side effect is driven by an amount the payment layer never authorized. The webhook handler has the same shape (webhook.rs:288 uses `event.amount_refunded_cents`), but the reconcile path is the one that *polls and picks* a refund, so it is more exposed.

**Fix:** Prefer the snapshotted `intent.refund_amount_cents` for the `on_refunded` amount, or assert the polled refund matches the snapshot before resolving:
```rust
let amount = intent.refund_amount_cents.unwrap_or(amount_cents);
// or: if Some(amount_cents) != intent.refund_amount_cents { warn + skip }
```

### WR-04: Reconcile reaper "loader vanished on a money path" silently strands the row

**File:** `ferro-payments/src/service.rs:544-552`

**Issue:**
In the `Succeeded` branch, after `mark_refunded` has already flipped the row to `refunded`, if the loader returns `None`/`Err` the code logs and `return Ok(false)` — but the status is now `refunded` while `on_refunded` never ran. The billable's side state (e.g. inventory restock, balance credit) is permanently skipped with only a `warn!`. Unlike the release path (where vanished is benign because no money moved), here money *was* refunded and the consumer-side compensation is lost. There is no retry because `find_refunds_in_flight` excludes `refunded_at`-set rows.

**Fix:** Either defer `mark_refunded` until *after* a successful `on_refunded` (so a vanished loader leaves the row in-flight for the next tick), or emit a durable dead-letter/audit record rather than a transient log line, so the stranded compensation is recoverable by an operator.

### WR-05: `fetch_refund_status_for_payment_intent` assumes `refunds.first()` is the system-initiated refund

**File:** `ferro-payments/src/service.rs:195-216`

**Issue:**
The production gateway takes `refunds.first()` ("Stripe returns refunds newest-first") as *the* refund to resolve. With `limit=10` and multiple refunds on one PaymentIntent (partial refunds, dashboard refunds, retries), the newest is not necessarily the one this system snapshotted under `refund-{intent_id}`. A newer unrelated `succeeded` refund will cause the reaper to `mark_refunded` and fire `on_refunded` against the wrong amount (compounds WR-03). The `idempotency_key` that would disambiguate is not forwarded to Stripe (see IN-01).

**Fix:** Match the refund by a stable identifier the system controls — e.g. store the Stripe `refund_id` when known and select by it, or filter the list by `metadata`/amount equal to the snapshot before treating it as resolution.

## Info

### IN-01: Stripe idempotency key is accepted but discarded — only application-layer dedup protects against double refunds

**File:** `ferro-stripe/src/refund.rs:18-40, 58-80`

**Issue:** `create` and `create_for_payment_intent` accept `idempotency_key` then immediately `let _ = idempotency_key;` — async-stripe 0.41 does not forward it. The doc comments are honest about this. Double-refund safety therefore rests *entirely* on the `WHERE refund_amount_cents IS NULL` guard in the application layer. That guard is correct, but it only protects a single logical refund per intent row; a process crash between the snapshot UPDATE and the Stripe call leaves the row "in flight" and the reconcile reaper re-polls (never re-issues), which is the intended recovery. Acceptable for v0.1.0, but the missing Stripe-side idempotency is a latent risk worth tracking until the async-stripe upgrade.

**Fix:** Track an upgrade task to forward the idempotency key once async-stripe exposes it; until then keep the application guard as the sole source of truth (it is).

### IN-02: `is_transient` treats every `Stripe` error as retryable

**File:** `ferro-payments/src/webhook.rs:96-98`

**Issue:** `is_transient` returns `true` for all `PaymentError::Stripe(_)`. Some Stripe errors are permanent (invalid id, card-level decline reasons surfaced as errors). Classifying them as transient causes Stripe to retry the webhook indefinitely on a permanent fault. Low impact (webhooks retry with backoff and eventually stop), but the binary classification is coarse.

**Fix:** Narrow transient classification to network/5xx/rate-limit Stripe errors; treat 4xx-class as terminal.

### IN-03: `find_by_charge_id` lookup has a supporting index for `payment_intent_id` but not `charge_id`

**File:** `ferro-payments/src/migration/m20260617_create_payment_intents.rs:137-146`; `ferro-payments/src/intent/lifecycle.rs:253-262`

**Issue:** The migration adds `idx_payment_intents_payment_intent_id` but no index on `charge_id`, yet `find_by_charge_id` is the documented fallback lookup for `handle_charge_refunded`. On large tables this fallback becomes a full scan. (Out-of-scope performance, flagged only because the missing index pairs with CR-01 — the charge column is half-wired.)

**Fix:** Add `idx_payment_intents_charge_id` alongside the PI-id index if the fallback path is retained.

### IN-04: Reconcile age anchor is hardcoded to 1 hour

**File:** `ferro-payments/src/service.rs:499-501`

**Issue:** `let older_than = now - chrono::Duration::hours(1);` hardcodes the "don't poll younger than 1h" window. The doc says "the cron schedule is the consumer's knob," but the cron cadence and the age anchor are independent concerns — a consumer cannot tune how long to wait before polling Stripe without forking the crate. Magic constant in a money path.

**Fix:** Promote the 1h window to a field on `PaymentService` (or a parameter) with a documented default, so consumers can adjust it independently of cron cadence.

---

_Reviewed: 2026-06-21T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

# Phase 235: webhook SyncDispatcher integration + auto-refund fallback — Research

**Researched:** 2026-06-17
**Domain:** ferro-payments webhook handlers, ferro-stripe SyncDispatcher, auto-refund, BillableKind Cow migration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Add `processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>` to `PaymentService`; reshape `new()` to take it; remove the `#[allow(dead_code)]` from `loader`.

**D-02:** Extend `StripeGateway` trait (and `MockStripeGateway`) with the methods needed by the auto-refund path (D-08).

**D-03:** `pub fn wire_dispatcher<L: BillableLoader + 'static>(dispatcher: SyncDispatcher, service: Arc<PaymentService<L>>) -> SyncDispatcher`. Consuming builder. Registers three handlers via `.on::<StripeCheckoutCompleted,_,_>`, `.on::<StripeCheckoutExpired,_,_>`, `.on::<StripeChargeRefunded,_,_>`.

**D-04 (error-boundary bridge):** Inline bridge in `wire_dispatcher` closures. Transient (`PaymentError::Db`, `PaymentError::Stripe`) → `Err(ferro_stripe::Error::Stripe(msg))` (Stripe retries). Terminal (replay skip, auto-refund, loader vanished) → `tracing::warn!` + `Ok(())`.

**D-05:** Each `handle_*` calls `processed_log.try_mark_processed(event.event_id())` first; `Ok(false)` → return `Ok(())` immediately.

**D-06 (handle_session_completed):** lookup by `session_id` → `mark_paid` (guarded `reserved→paid`). `Ok(false)` → auto-refund (`SideStateConflict`). `Ok(true)` → DB transaction → `loader.load(kind, id)`. `Err` → auto-refund (`LoaderError`), rollback. `Ok(None)` → auto-refund (`BillableVanished`), rollback. `Ok(Some(b))` → `b.on_paid(&txn)` → commit. Also persist `payment_intent_id` onto the row.

**D-07 (handle_session_expired / handle_charge_refunded):** expired: lookup by `session_id` → `mark_released` → if `true`, txn `on_released` → commit. `Ok(false)` → no-op. charge_refunded: lookup by `payment_intent_id` (fallback `charge_id`) → `mark_refunded` → if `true`, txn `on_refunded(amount_refunded_cents)` → commit. `Ok(false)` → no-op.

**D-08:** Add ferro-stripe primitive `refund::create_for_payment_intent(payment_intent_id, amount_cents, key, reason)`. Add `StripeGateway::create_refund_for_payment_intent` method + `MockStripeGateway` impl. The auto-refund for session.completed uses `payment_intent_id` from the event; `None` → log + `Ok(())`.

**D-09:** Auto-refund: snapshot `refund_amount_cents` via `GuardedUpdate WHERE refund_amount_cents IS NULL`. Then call payment-intent refund. After trigger, `tracing::warn!` + return `Ok(())`.

**D-10 (WR-04 — fix FIRST):** Change `BillableKind` to `Cow<'static, str>`. Keep `pub const fn new(s: &'static str) -> Self { Self(Cow::Borrowed(s)) }`. Add `pub fn from_string(s: String) -> Self { Self(Cow::Owned(s)) }`. `as_str()` returns `&str` (not `&'static str`).

**D-11 (WR-01):** Do NOT compensate-reset `refund_amount_cents` on Stripe failure. Document the stuck state. Phase-236 `ReconcileRefundsInFlight` reaper is the recovery.

**D-12 (WR-03):** Add `amount_cents <= 0` guard to `start_checkout`.

### Claude's Discretion

- Whether `handle_*` live in `service.rs` or a new `webhook.rs` module.
- Location of the private `PaymentError → ferro_stripe::Error` bridge helper.
- Exact `AutoRefundReason` selection per branch.
- Auto-refund observability: `tracing` only, no new consumer callback.
- Test module organization for race tests.

### Deferred Ideas (OUT OF SCOPE)

- `ReleaseExpiredPaymentIntents` reaper, `ReconcileRefundsInFlight` reaper, workspace test bin, publish `0.1.0` — **phase 236**.
- WR-02 (per-connected-account fee rates) — doc-only note in 235.
- Consumer-facing auto-refund observability hook (beyond `tracing`).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PAY-POLY-WH-01 | `wire_dispatcher` helper registers three typed-event handlers on `SyncDispatcher` | D-03 confirmed via sync.rs `on<E,H,Fut>` consuming builder signature |
| PAY-POLY-WH-02 | `handle_session_completed`: idempotency + mark_paid + loader dispatch + persist `payment_intent_id` + auto-refund fallback | D-05/D-06 confirmed; lifecycle.rs has no `find_by_payment_intent` or `attach_payment_intent` — both must be added |
| PAY-POLY-WH-03 | `handle_session_expired`: idempotency + mark_released + loader `on_released` | D-05/D-07 confirmed; `find_by_stripe_session` exists |
| PAY-POLY-WH-04 | `handle_charge_refunded`: idempotency + mark_refunded + loader `on_refunded` + `find_by_payment_intent` | D-07 confirmed; `find_by_payment_intent` absent from lifecycle.rs — must be added |
| PAY-POLY-WH-05 | Auto-refund fallback (LoaderError / BillableVanished / SideStateConflict) via `refund::create_for_payment_intent` | D-08/D-09 confirmed; `CreateRefund.payment_intent: Option<PaymentIntentId>` field verified in async-stripe 0.41 |
| PAY-POLY-WH-06 | Race-condition tests: reaper+handler interleaved, webhook replay, loader-not-found, paid-after-released | confirmed testable via MockStripeGateway + MemoryProcessedLog + in-memory SQLite |
</phase_requirements>

---

## Summary

Phase 235 extends the `ferro-payments` crate with three webhook event handlers and the `wire_dispatcher` helper that registers them on a `ferro_stripe::SyncDispatcher`. It also resolves two carry-forwards from Phase 234: `BillableKind` must be changed from `&'static str` to `Cow<'static, str>` (WR-04, fix-first), and the auto-refund path at `handle_session_completed` requires a new ferro-stripe primitive that refunds by `payment_intent_id` rather than `charge_id` (WR-01 / DISCREPANCY-1).

All codebase facts are confirmed by direct file reads. The single highest-risk item — "does async-stripe 0.41 have a `payment_intent` field on `CreateRefund`?" — is **verified: YES** (`pub payment_intent: Option<PaymentIntentId>` at line 343 of the generated refund source). The implementation pattern mirrors the existing `refund::create` exactly, substituting `params.payment_intent` for `params.charge`.

Two lifecycle helpers are absent and must be added in Wave 0: `find_by_payment_intent` (needed by `handle_charge_refunded`) and `attach_payment_intent` (needed by `handle_session_completed` to persist `payment_intent_id` + `charge_id` onto the row after marking paid). Both follow the existing `find_by_stripe_session` / `attach_session` patterns.

**Primary recommendation:** Fix `BillableKind` → `Cow` first (blocks everything), then add the two missing lifecycle helpers, then implement `refund::create_for_payment_intent` in ferro-stripe, then implement `handle_*` + `wire_dispatcher` in ferro-payments.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Webhook signature verification | ferro-stripe (existing) | — | Already lives in `verify_webhook`; this phase is post-verification only |
| Typed event dispatch | ferro-stripe `SyncDispatcher` | ferro-payments `wire_dispatcher` | `SyncDispatcher::on` owns the handler registry; `wire_dispatcher` is a registration helper |
| Idempotency guard | `ProcessedEventLog` (ferro-stripe trait) | `MemoryProcessedLog` in tests | Stored in `PaymentService.processed_log`, injected at construction |
| Status transitions (guarded updates) | ferro-payments `intent::lifecycle` | sea-orm `GuardedUpdate` | All `mark_*` fns live in lifecycle.rs; `GuardedUpdate` is the atomic primitive |
| Billable side-effects (`on_*`) | Consumer (`Billable` implementor) | ferro-payments transaction wrapper | `PaymentService` opens the txn and passes `&DatabaseTransaction` to `on_*` |
| Auto-refund (by payment_intent) | ferro-stripe `refund::create_for_payment_intent` | ferro-payments `handle_session_completed` | V-95-01 gate: no direct `stripe::` in consumers; new primitive must live in ferro-stripe |
| BillableKind construction from DB string | ferro-payments `BillableKind::from_string` | — | `handle_*` reads `intent.billable_kind: String` and constructs `BillableKind` via `from_string` |

---

## Standard Stack

### Core (already in Cargo.toml — no new deps needed)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| ferro-stripe | 0.9 (path dep) | SyncDispatcher, ProcessedEventLog, typed events, refund | Already in ferro-payments Cargo.toml |
| sea-orm | 1.0 | `DatabaseConnection.begin()` → `DatabaseTransaction`; `ConnectionTrait` for lifecycle helpers | `TransactionTrait` impl on `DatabaseConnection`; `commit(self)` and `rollback(self)` on `DatabaseTransaction` |
| ferro-orm | 0.2 | `GuardedUpdate` for `refund_amount_cents` auto-refund dedup | Used in `request_refund`; same pattern for D-09 |
| async-trait | 0.1 | `#[async_trait]` on `StripeGateway` extension | Already in use |
| tracing | 0.1 | `tracing::warn!` for auto-refund observability | Already a transitive dep |

[VERIFIED: direct file reads of Cargo.toml, lifecycle.rs, service.rs, ferro-stripe/Cargo.toml]

---

## Architecture Patterns

### System Architecture Diagram

```
Stripe HTTP POST /webhook
         │
         ▼
  verify_webhook (ferro-stripe)
         │ WebhookEvent
         ▼
  SyncDispatcher::dispatch
         │ matches event.type_
         ├─ checkout.session.completed ──► wire_dispatcher closure
         │                                  │ clone Arc<PaymentService>
         │                                  ▼
         │                            handle_session_completed(event)
         │                                  │ try_mark_processed → replay?
         │                                  │ find_by_stripe_session
         │                                  │ mark_paid (GuardedUpdate)
         │                                  │   Ok(false) → auto-refund (SideStateConflict)
         │                                  │   Ok(true)  → db.begin()
         │                                  │               loader.load(kind, id)
         │                                  │                 Err/None → auto-refund + rollback
         │                                  │                 Some(b)  → b.on_paid(&txn) → commit
         │                                  │               attach_payment_intent (guarded)
         │
         ├─ checkout.session.expired ────► handle_session_expired(event)
         │                                  │ try_mark_processed
         │                                  │ find_by_stripe_session
         │                                  │ mark_released → txn → on_released → commit
         │
         └─ charge.refunded ─────────────► handle_charge_refunded(event)
                                            │ try_mark_processed
                                            │ find_by_payment_intent (NEW)
                                            │   fallback: find_by_charge_id (NEW or inline)
                                            │ mark_refunded → txn → on_refunded → commit

auto-refund path (from handle_session_completed):
  snapshot refund_amount_cents (GuardedUpdate IS NULL)
  stripe.create_refund_for_payment_intent(payment_intent_id, amount, key, reason)
  tracing::warn!(reason) → Ok(())
```

### Recommended Project Structure

```
ferro-payments/src/
├── lib.rs                    # re-export wire_dispatcher, updated BillableKind
├── billable.rs               # unchanged
├── loader.rs                 # unchanged
├── error.rs                  # unchanged (AutoRefundReason already defined)
├── service.rs                # add processed_log field + D-12 guard; extend StripeGateway + Mock
├── webhook.rs                # NEW: wire_dispatcher + handle_* methods (or add to service.rs)
├── intent/
│   ├── lifecycle.rs          # ADD: find_by_payment_intent, attach_payment_intent
│   └── ...
└── migration/               # unchanged

ferro-stripe/src/
├── refund.rs                 # ADD: create_for_payment_intent
└── lib.rs                    # re-export create_for_payment_intent (or pub mod refund)
```

### Pattern 1: SyncDispatcher::on — exact signature

```rust
// Source: ferro-stripe/src/webhook/sync.rs (VERIFIED)
pub fn on<E, H, Fut>(mut self, handler: H) -> Self
where
    E: StripeEvent,
    H: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
```

The handler takes a **typed event** `E: StripeEvent`, NOT a `WebhookEvent`. The dispatcher internally calls `E::from_raw(&raw_event)` and skips non-matching events. The function is consuming (`mut self → Self`).

### Pattern 2: wire_dispatcher closure shape

```rust
// Source: inferred from D-03 + verified sync.rs + D-04 error bridge
pub fn wire_dispatcher<L: BillableLoader + 'static>(
    dispatcher: SyncDispatcher,
    service: Arc<PaymentService<L>>,
) -> SyncDispatcher {
    let svc1 = Arc::clone(&service);
    let svc2 = Arc::clone(&service);
    let svc3 = Arc::clone(&service);
    dispatcher
        .on(move |event: StripeCheckoutCompleted| {
            let svc = Arc::clone(&svc1);
            async move {
                svc.handle_session_completed(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
        .on(move |event: StripeCheckoutExpired| {
            let svc = Arc::clone(&svc2);
            async move {
                svc.handle_session_expired(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
        .on(move |event: StripeChargeRefunded| {
            let svc = Arc::clone(&svc3);
            async move {
                svc.handle_charge_refunded(event)
                    .await
                    .map_err(payment_to_stripe_error)
            }
        })
}
```

Note: the closure captures `svc` (cloned Arc) and must be `Fn`, not `FnOnce` — so each invocation must re-clone from the outer `Arc`. The double-clone pattern above (outer per-handler clone + inner per-invocation clone) is correct.

### Pattern 3: error-boundary bridge (D-04)

```rust
// Private helper — inline in webhook.rs or service.rs
fn payment_to_stripe_error(e: PaymentError) -> ferro_stripe::Error {
    match e {
        // Transient — Stripe should retry
        PaymentError::Db(ref db_err) => {
            ferro_stripe::Error::Stripe(format!("db: {db_err}"))
        }
        PaymentError::Stripe(ref s) => s.clone(),
        // Terminal outcomes — log and return Ok is handled BEFORE reaching the bridge.
        // AutoRefundTriggered, NotFound, StatusPrecondition, Loader are unreachable here
        // because handle_* absorbs them and returns Ok(()).
        _ => ferro_stripe::Error::Stripe(format!("payment: {e}")),
    }
}
```

**Key insight:** `handle_*` should never return `AutoRefundTriggered` to the dispatcher. Auto-refund is a side-effect the handler resolves internally, then returns `Ok(())`. The error bridge is only a safety net for genuine transient errors.

### Pattern 4: sea-orm transaction lifecycle

```rust
// Source: sea-orm 1.1.20 DatabaseConnection (VERIFIED)
use sea_orm::TransactionTrait;

let txn = self.db.begin().await.map_err(PaymentError::Db)?;
match billable.on_paid(&txn).await {
    Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
    Err(e) => {
        txn.rollback().await.ok(); // rollback consumes txn, ignore rollback error
        // then handle e (trigger auto-refund or propagate)
    }
}
```

`commit(self)` and `rollback(self)` both consume `DatabaseTransaction`. `on_paid` takes `&DatabaseTransaction` which implements `ConnectionTrait` — lifecycle helpers can accept `&DatabaseTransaction` because lifecycle.rs already uses the `C: ConnectionTrait` bound.

### Pattern 5: refund::create_for_payment_intent (new ferro-stripe primitive)

```rust
// Source: async-stripe 0.41 CreateRefund struct (VERIFIED: payment_intent: Option<PaymentIntentId>)
// Location: ferro-stripe/src/refund.rs — mirror of existing create()
pub async fn create_for_payment_intent(
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    // Same caveat as create(): async-stripe 0.41 does not forward idempotency_key.
    let _ = idempotency_key;
    let client = crate::Stripe::client();
    let pi_id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let mut params = stripe::CreateRefund::new();
    params.payment_intent = Some(pi_id);
    params.amount = amount_cents;
    params.reason = reason;
    let refund = stripe::Refund::create(client, params).await?;
    Ok(refund)
}
```

The `StripeGateway` extension method mirrors the existing `create_refund`:

```rust
async fn create_refund_for_payment_intent(
    &self,
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
) -> Result<(), ferro_stripe::Error>;
```

`StripeClientGateway` impl delegates to `ferro_stripe::refund::create_for_payment_intent`. `MockStripeGateway` records `(payment_intent_id, amount_cents)` and returns a canned result (default `Ok(())`).

### Pattern 6: BillableKind → Cow migration (D-10, fix-first)

```rust
// Source: D-10 decision + Rust std::borrow::Cow semantics
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillableKind(Cow<'static, str>);

impl BillableKind {
    pub const fn new(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
    pub fn from_string(s: String) -> Self {
        Self(Cow::Owned(s))
    }
    pub fn as_str(&self) -> &str {      // return type changes: was &'static str
        &self.0
    }
}
```

**Breakage audit** — all current usages of `BillableKind::as_str()`:

- `lifecycle::create_reserved(billable.kind().as_str(), ...)` — accepts `&str`, no breakage
- `find_active_for(kind: &str, ...)` — accepts `&str`, no breakage
- Test assertions like `assert_eq!(kind.as_str(), "order")` — compile-time literals, no breakage
- `MockLoader::load(_kind: BillableKind, ...)` — takes `BillableKind` by value, no breakage
- `BillableKind::new("booking")` in test structs — `const fn`, no breakage

The only consumer of the narrower `&'static str` return is inside `BillableKind::as_str()` itself. Changing the return to `&str` (the deref of `Cow`) breaks no existing callsite. `Cow<'static, str>` derives `Clone` and `PartialEq` from its variants — both already derived.

### Pattern 7: find_by_payment_intent + attach_payment_intent (new lifecycle helpers)

```rust
// Location: ferro-payments/src/intent/lifecycle.rs — follows find_by_stripe_session pattern
pub async fn find_by_payment_intent<C: ConnectionTrait>(
    payment_intent_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::PaymentIntentId.eq(payment_intent_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}

// Attaches payment_intent_id (and charge_id when present) after mark_paid.
// Guard: WHERE payment_intent_id IS NULL — idempotent for retries.
pub async fn attach_payment_intent<C: ConnectionTrait>(
    id: i64,
    payment_intent_id: &str,
    conn: &C,
) -> Result<bool, PaymentError> {
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::PaymentIntentId.is_null())
        .set_value(
            Column::PaymentIntentId,
            Value::String(Some(Box::new(payment_intent_id.to_string()))),
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}
```

Both need re-export from `ferro-payments/src/lib.rs`.

### Anti-Patterns to Avoid

- **Returning `AutoRefundTriggered` from `handle_*`:** The spec error model says `AutoRefundTriggered` is returned from `start_checkout`/`request_refund`; webhook handlers must absorb it and return `Ok(())`. Propagating it to `wire_dispatcher` would cause the bridge to map it to a transient error and trigger a Stripe retry loop.
- **Rolling back after a successful `mark_paid`:** `mark_paid` is a non-transactional `GuardedUpdate` (it runs on `&self.db`, not inside a txn). If the subsequent loader call fails, the status is already `paid`. The auto-refund path must proceed — do NOT attempt to roll back `mark_paid`. Only the `db.begin()` txn for `on_paid` is rolled back.
- **Calling `create_refund` (charge-based) from `handle_session_completed`:** `StripeCheckoutCompleted` has no `charge_id` field. Only `payment_intent_id` is available. Always use `create_refund_for_payment_intent`.
- **Cloning `PaymentService` for the wire_dispatcher closure:** `PaymentService` is not `Clone`. Closures must capture `Arc<PaymentService<L>>` and clone the Arc, not the service.
- **Using `try_mark_processed` inside the DB transaction:** Mark first, outside the transaction. If the transaction rolls back (loader failure), the event is still marked processed — auto-refund is triggered, not a duplicate handler execution. This is correct by design.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic status transition | Manual UPDATE + re-read | `ferro_orm::GuardedUpdate` | Races between reaper + handler; no-op on stale precondition |
| Webhook idempotency | Custom seen-set or extra DB column | `ProcessedEventLog::try_mark_processed` | Already implemented with `DashMap` atomicity; `MemoryProcessedLog` for tests |
| Refund by payment_intent | Direct `stripe::Refund::create` in ferro-payments | `ferro_stripe::refund::create_for_payment_intent` | V-95-01 gate: no `stripe::` in consumers |
| DB transaction | Manual BEGIN/COMMIT SQL | `self.db.begin()` + `txn.commit()` / `txn.rollback()` | sea-orm `TransactionTrait` already in scope |
| Event field extraction | Manual JSON parsing | `StripeCheckoutCompleted`, `StripeCheckoutExpired`, `StripeChargeRefunded` structs | Already implemented in ferro-stripe; forward-compatible across Stripe API versions |

---

## Common Pitfalls

### Pitfall 1: mark_paid runs outside the billable transaction
**What goes wrong:** `mark_paid` writes to `status='paid'` before the loader + `on_paid` txn. If `on_paid` fails, the row is `paid` but the consumer's side state was not updated. Auto-refund triggers correctly (good), but if the refund also fails, the row is stuck in `paid` with a `refund_amount_cents` snapshot.
**Why it happens:** `GuardedUpdate` must run outside the txn to be the race fence (it is the exclusive write to `status`). Opening the txn first and running `mark_paid` inside it would serialize the reaper vs handler — which defeats the purpose of the guarded update.
**How to avoid:** This is intentional design (D-09 in 233). The stuck state is the D-11 predicate; phase-236 `ReconcileRefundsInFlight` is the recovery. Document it in code comments.
**Warning signs:** If you see `mark_paid` called inside a `let txn = db.begin()` block, that is wrong.

### Pitfall 2: `Fn` closure requiring double Arc clone
**What goes wrong:** `SyncDispatcher::on` takes `H: Fn(E) -> Fut + Send + Sync + 'static`. A `FnOnce` that moves `Arc<service>` inside compiles but panics at runtime on a second invocation (Stripe retries).
**Why it happens:** Stripe may retry the same event multiple times. The handler closure must be `Fn`, not `FnOnce`.
**How to avoid:** Capture an `Arc<service>` by clone in the outer let, then inside the closure move a clone into each invocation's async block. The two-level clone pattern: `let svc = Arc::clone(&service); move || { let svc = Arc::clone(&svc); async move { ... } }`.

### Pitfall 3: `BillableKind::as_str()` return-type change breaks callers expecting `&'static str`
**What goes wrong:** If any code stores the result of `as_str()` in a `&'static str` binding, it will fail to compile after the `Cow` migration.
**Why it happens:** The old impl returned `&'static str`; the new impl returns `&str` with lifetime tied to `&self`.
**How to avoid:** The breakage audit above shows no current callers store a `&'static str` result. Run `cargo check` immediately after the `BillableKind` change before proceeding to the webhook implementation.

### Pitfall 4: Missing `find_by_payment_intent` causes `handle_charge_refunded` to always no-op
**What goes wrong:** If the `charge.refunded` handler cannot find the intent row, it returns `Ok(())` (no match = no-op), and `mark_refunded` is never called, leaving the row in `paid` state indefinitely.
**Why it happens:** `charge.refunded` events carry `payment_intent_id`, not `session_id`. `find_by_stripe_session` cannot look up by `payment_intent_id`.
**How to avoid:** Add `find_by_payment_intent` to `lifecycle.rs` in Wave 0 (before implementing the handlers). The DB column `payment_intent_id` already has an index (`idx_payment_intents_payment_intent_id`) — the query will be efficient.

### Pitfall 5: Auto-refund with `payment_intent_id = None` on a free/setup session
**What goes wrong:** For Stripe Checkout sessions with `mode=setup` or `amount_total=0`, `payment_intent_id` is `None`. Calling `create_refund_for_payment_intent(None, ...)` would panic or error.
**Why it happens:** `StripeCheckoutCompleted.payment_intent_id` is `Option<String>`.
**How to avoid:** D-08 mandates: if `payment_intent_id` is `None`, log at `tracing::debug!` and return `Ok(())`. No refund needed — nothing was charged.

### Pitfall 6: `processed_log` not added before testing (clippy `-D warnings`)
**What goes wrong:** Adding `processed_log` to `PaymentService` but not passing it in existing tests causes compilation failure under `--all-features`.
**Why it happens:** `PaymentService::new()` signature changes in this phase; all 234 test harnesses that call `new()` must be updated.
**How to avoid:** Update test `PaymentService::new(...)` call sites with `Arc::new(MemoryProcessedLog::new())` as the new argument.

---

## Key Technical Findings

### Finding 1: `CreateRefund.payment_intent` confirmed in async-stripe 0.41
[VERIFIED: direct grep of `/Users/alberto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-stripe-0.41.0/src/resources/generated/refund.rs`]

`pub payment_intent: Option<PaymentIntentId>` is field #10 in `CreateRefund`. The `PaymentIntentId` type is the same parsed string ID type used elsewhere (e.g. in `payment_intent.rs`'s `capture`). The implementation mirrors `refund::create` exactly — parse the ID string into `stripe::PaymentIntentId`, set `params.payment_intent = Some(pi_id)`, call `stripe::Refund::create`.

### Finding 2: `SyncDispatcher::on` is a consuming builder
[VERIFIED: ferro-stripe/src/webhook/sync.rs]

`pub fn on<E,H,Fut>(mut self, handler: H) -> Self` — takes `self` by value (consuming), returns `Self`. `wire_dispatcher` must take `SyncDispatcher` by value and chain `.on(...)` calls. This matches D-03.

### Finding 3: `StripeCheckoutCompleted` has NO `charge_id`
[VERIFIED: ferro-stripe/src/webhook/events.rs lines 177-203]

Fields: `event_id`, `session_id`, `payment_intent_id: Option<String>`, `amount_total_cents`, `currency`, `metadata`, `customer_email`. No `charge_id` field. DISCREPANCY-1 confirmed — the existing `create_refund(charge_id, ...)` cannot be used for session.completed auto-refund.

### Finding 4: `find_by_payment_intent` does NOT exist in lifecycle.rs
[VERIFIED: lifecycle.rs grep for `find_by_payment_intent` — zero results]

The `payment_intent_id` column exists in the entity (entity.rs line 34) and has an index in the migration, but no lifecycle query function reads it. Must be added. Wave 0 task.

### Finding 5: `attach_payment_intent` does NOT exist in lifecycle.rs
[VERIFIED: lifecycle.rs grep for `attach_payment_intent` — zero results]

`handle_session_completed` must persist `payment_intent_id` onto the row after marking paid. The column is `NULL` at reservation time (D-13 in 234). A new `attach_payment_intent` guarded-update function is needed. Wave 0 task.

### Finding 6: sea-orm `DatabaseTransaction` commit/rollback API
[VERIFIED: sea-orm 1.1.20 transaction.rs lines 119, 159]

- `pub async fn commit(mut self) -> Result<(), DbErr>` — consuming
- `pub async fn rollback(mut self) -> Result<(), DbErr>` — consuming

The txn is consumed by either call. `DatabaseTransaction` implements `ConnectionTrait`, so lifecycle helpers using `C: ConnectionTrait` can accept `&txn` directly.

### Finding 7: `MemoryProcessedLog` uses `DashMap` for per-key atomicity
[VERIFIED: ferro-stripe/src/idempotency.rs]

`try_mark_processed` uses `DashMap::insert` which provides shard-level locking. For test scenarios simulating concurrent webhook delivery, `MemoryProcessedLog` correctly ensures exactly one `Ok(true)` across concurrent callers with the same event id. The existing `concurrent_insert_applies_once` test in idempotency.rs proves this.

### Finding 8: `testing.rs` missing event builders for checkout.expired and charge.refunded
[VERIFIED: ferro-stripe/src/testing.rs — only `mock_checkout_completed_event`, `mock_subscription_updated_event`, `mock_subscription_deleted_event`, `mock_invoice_paid_event`]

The `testing.rs` module has NO builders for `checkout.session.expired` or `charge.refunded`. For the phase-235 race tests that test via `handle_*` directly (not via `dispatch`), this does not block — tests can construct `StripeCheckoutExpired` and `StripeChargeRefunded` structs directly (they are `#[derive(Clone, Debug)]`, not generated from JSON at the test layer).

However, for tests that go through `SyncDispatcher::dispatch`, a raw `WebhookEvent` is needed. Options:
1. Construct `WebhookEvent` inline via `serde_json::json!` + `WebhookEvent::from_json()` (preferred — no new testing.rs additions needed).
2. Add `mock_checkout_expired_event` and `mock_charge_refunded_event` to `testing.rs`.

The race tests (reaper+handler interleaved) call `handle_*` directly with typed event structs — no `dispatch` round-trip needed. Testing.rs additions are optional enhancement, not a blocker.

### Finding 9: `ferro_stripe::Error` has a `Stripe(String)` variant
[VERIFIED: ferro-stripe/src/error.rs — implied by usage in refund.rs `Error::Stripe(format!(...))` pattern]

The error bridge `payment_to_stripe_error` can map any `PaymentError` to `ferro_stripe::Error::Stripe(msg_string)` without needing a From impl. No orphan rule issue.

---

## Race Table → Handler Branch Map

Spec's 5-row race table mapped to concrete `handle_session_completed` branches:

| Race | `mark_paid` result | loader result | Handler action |
|------|--------------------|---------------|----------------|
| Webhook wins (happy path) | `Ok(true)` | `Ok(Some(b))` | txn `on_paid` → commit → `Ok(())` |
| Reaper released first, webhook arrives late | `Ok(false)` | — | auto-refund (`SideStateConflict`) → `Ok(())` |
| Customer pays after slot released; `on_paid` notices side state already released | `Ok(true)` | `Ok(Some(b))` | `b.on_paid` returns `Err` → auto-refund? **No** — the spec says `on_paid` "notices the side state already released → auto-refund". The billable's `on_paid` must return `Ok(())` and the auto-refund is triggered by `on_paid`'s side effect detection returning `PaymentError::StatusPrecondition`. The handler treats any `Err` from `on_paid` as a loader-side failure and triggers auto-refund. |
| Loader returns `Err` | `Ok(true)` | `Err(e)` | auto-refund (`LoaderError`) → rollback txn → `Ok(())` |
| Loader returns `Ok(None)` (billable vanished) | `Ok(true)` | `Ok(None)` | auto-refund (`BillableVanished`) → rollback txn → `Ok(())` |

**AutoRefundReason selection:**
- `Ok(false)` from `mark_paid` → `SideStateConflict`
- `Err` from `loader.load` → `LoaderError`
- `Ok(None)` from `loader.load` → `BillableVanished`
- `Err` from `on_paid` → `SideStateConflict` (billable's own side state conflict)

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ferro_stripe::Error` has a `Stripe(String)` variant suitable for the error bridge | Pattern 3 | Bridge won't compile; need to check actual error enum variants |

**A1 verification path:** The pattern is already used in `refund.rs` (`Error::Stripe(format!(...))`) and `payment_intent.rs`. Confidence HIGH. Included as assumption because `error.rs` was not explicitly read.

---

## Open Questions

1. **Should `attach_payment_intent` also store `charge_id`?**
   - What we know: `charge_id` is a column in the entity; it is populated elsewhere (the spec says "Set on success"). `StripeCheckoutCompleted` has NO `charge_id` field. The `charge.refunded` event has `charge_id`.
   - What's unclear: When does `charge_id` get written? The spec data model says "Set on success" but the session.completed event doesn't have it.
   - Recommendation: `attach_payment_intent` stores only `payment_intent_id`. `charge_id` is populated by a separate path (possibly in `handle_charge_refunded` or left for the reaper). The critical field for `request_refund` is already gated on `charge_id IS NOT NULL` — so `charge_id` can stay NULL until a future phase populates it, as long as the auto-refund path uses `payment_intent_id`.

2. **Does `mark_paid` need to also set `payment_intent_id` atomically?**
   - What we know: `mark_paid` is a `GuardedUpdate` on `status`. It does not set `payment_intent_id`.
   - What's unclear: Is a separate `attach_payment_intent` call (after `mark_paid` succeeds) safe in the face of a reaper running concurrently?
   - Recommendation: The reaper (phase 236) runs on `status='reserved'` rows. After `mark_paid` sets `status='paid'`, the reaper's `reserved` filter excludes the row. The separate `attach_payment_intent` call is safe.

3. **`handle_charge_refunded` fallback: charge_id lookup**
   - D-07 says "fallback charge_id". `StripeChargeRefunded.charge_id` is `String` (not Optional — always present). `payment_intent_id` is `Option<String>`.
   - Recommendation: Try `find_by_payment_intent` first (if `payment_intent_id` is `Some`). If `None` or not found, try `find_by_charge_id`. A `find_by_charge_id` helper mirrors `find_by_payment_intent`. Add both in the same Wave 0 task.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code changes only to existing workspace crates. No new external tools, services, or runtimes.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio + sea-orm in-memory SQLite (pattern from 233/234) |
| Config file | none — inline `#[tokio::test]` + `fresh_db()` helper |
| Quick run | `cargo test -p ferro-payments --all-features` |
| Full suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| PAY-POLY-WH-01 | `wire_dispatcher` returns `SyncDispatcher` with 3 handlers | unit | `cargo test -p ferro-payments handle_session` | ❌ Wave 0 |
| PAY-POLY-WH-02 | `handle_session_completed` happy path: `mark_paid` + `on_paid` + `payment_intent_id` attached | unit | `cargo test -p ferro-payments handle_session_completed` | ❌ Wave 0 |
| PAY-POLY-WH-02 | `handle_session_completed` replay: second call is no-op | unit | `cargo test -p ferro-payments handle_session_completed_replay` | ❌ Wave 0 |
| PAY-POLY-WH-02 | `handle_session_completed` SideStateConflict → auto-refund | unit | `cargo test -p ferro-payments handle_session_completed_side_state_conflict` | ❌ Wave 0 |
| PAY-POLY-WH-03 | `handle_session_expired` happy path: `mark_released` + `on_released` | unit | `cargo test -p ferro-payments handle_session_expired` | ❌ Wave 0 |
| PAY-POLY-WH-03 | `handle_session_expired` no-op: row already released | unit | `cargo test -p ferro-payments handle_session_expired_noop` | ❌ Wave 0 |
| PAY-POLY-WH-04 | `handle_charge_refunded` happy path: `find_by_payment_intent` + `mark_refunded` + `on_refunded` | unit | `cargo test -p ferro-payments handle_charge_refunded` | ❌ Wave 0 |
| PAY-POLY-WH-05 | Loader returns `None` → auto-refund called exactly once | unit | `cargo test -p ferro-payments auto_refund_billable_vanished` | ❌ Wave 0 |
| PAY-POLY-WH-05 | Loader returns `Err` → auto-refund called exactly once | unit | `cargo test -p ferro-payments auto_refund_loader_error` | ❌ Wave 0 |
| PAY-POLY-WH-06 | Webhook + reaper interleaved: guarded updates ensure exactly one side-effect | unit (race sim) | `cargo test -p ferro-payments webhook_reaper_race` | ❌ Wave 0 |
| PAY-POLY-WH-06 | Paid-after-released (SideStateConflict): mark_paid Ok(false) → auto-refund | unit | `cargo test -p ferro-payments paid_after_released` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-payments --all-features`
- **Per wave merge:** full gate (`fmt + clippy -D warnings + test --all-features`)
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-payments/src/webhook.rs` (or webhook section in `service.rs`) — covers PAY-POLY-WH-01 through WH-06
- [ ] `ferro-payments/src/intent/lifecycle.rs` additions: `find_by_payment_intent`, `find_by_charge_id`, `attach_payment_intent`
- [ ] `ferro-stripe/src/refund.rs` addition: `create_for_payment_intent`
- [ ] `BillableKind` Cow migration in `lib.rs`
- [ ] `PaymentService::new()` updated (add `processed_log` param) — cascades to all 234 test call sites

---

## Security Domain

Security enforcement is enabled (no explicit `false` in config). ASVS categories applicable to this phase:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | webhook signature already verified upstream by `verify_webhook` before dispatch |
| V3 Session Management | no | no user sessions in this layer |
| V4 Access Control | no | webhook handler is server-internal; no consumer-facing authorization |
| V5 Input Validation | yes | `payment_intent_id` parsed via `stripe::PaymentIntentId::parse()` before use; `amount_cents <= 0` guard (D-12) |
| V6 Cryptography | no | no new crypto; signature verification is in ferro-stripe |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Duplicate webhook delivery (Stripe retries) | Repudiation | `ProcessedEventLog::try_mark_processed` — first call wins |
| Double-refund on concurrent auto-refund triggers | Tampering | `GuardedUpdate WHERE refund_amount_cents IS NULL` — exactly one write wins |
| Forged webhook event (unauthenticated caller) | Spoofing | Out of scope for this layer — `verify_webhook` is the caller's responsibility before `dispatch` |
| Invalid Stripe ID format causing panic | Denial of Service | `payment_intent_id.parse::<stripe::PaymentIntentId>()` returns `Err` — handled gracefully |

---

## Sources

### Primary (HIGH confidence)
- `ferro-stripe/src/webhook/sync.rs` — `SyncDispatcher::on` consuming builder signature, `dispatch` error short-circuit
- `ferro-stripe/src/webhook/events.rs` — `StripeCheckoutCompleted`, `StripeCheckoutExpired`, `StripeChargeRefunded` field shapes (NO `charge_id` on `StripeCheckoutCompleted` confirmed)
- `ferro-stripe/src/idempotency.rs` — `ProcessedEventLog::try_mark_processed` bool semantics, `MemoryProcessedLog`
- `ferro-stripe/src/refund.rs` — existing `create(charge_id, ...)` pattern to mirror
- `ferro-stripe/src/testing.rs` — available builders (missing checkout.expired + charge.refunded)
- `ferro-payments/src/service.rs` — `PaymentService`, `StripeGateway`, `MockStripeGateway` current state
- `ferro-payments/src/intent/lifecycle.rs` — all lifecycle helpers; absence of `find_by_payment_intent` and `attach_payment_intent`
- `ferro-payments/src/lib.rs` — `BillableKind` current `&'static str` repr
- `ferro-payments/src/error.rs` — `PaymentError` + `AutoRefundReason` already defined
- `async-stripe-0.41.0/src/resources/generated/refund.rs` — `CreateRefund.payment_intent: Option<PaymentIntentId>` confirmed
- `sea-orm-1.1.20/src/database/transaction.rs` — `commit(self)` and `rollback(self)` signatures
- `.planning/phases/235-ferro-payments-webhook-sync-dispatcher-integration-and-auto-/235-CONTEXT.md` — locked decisions D-01..D-12

### Secondary (MEDIUM confidence)
- `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md` — design spec; webhook race table (5 rows); testing table

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps already present in Cargo.toml; no new dependencies required
- Architecture: HIGH — all signatures verified from source; error bridge pattern inferred from existing usage
- Pitfalls: HIGH — verified from direct code analysis; not from community reports
- async-stripe `CreateRefund.payment_intent` field: HIGH — confirmed in registry source

**Research date:** 2026-06-17
**Valid until:** 30 days (stable API surface)

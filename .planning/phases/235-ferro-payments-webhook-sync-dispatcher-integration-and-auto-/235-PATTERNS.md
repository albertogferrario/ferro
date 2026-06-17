# Phase 235: webhook SyncDispatcher integration + auto-refund fallback — Pattern Map

**Mapped:** 2026-06-17
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-payments/src/lib.rs` | config/re-export | — | `ferro-payments/src/lib.rs` (self) | exact |
| `ferro-stripe/src/refund.rs` | service | request-response | `ferro-stripe/src/refund.rs` (existing `create`) | exact |
| `ferro-payments/src/intent/lifecycle.rs` | service | CRUD | `ferro-payments/src/intent/lifecycle.rs` (existing `find_by_stripe_session`, `attach_session`) | exact |
| `ferro-payments/src/service.rs` | service | CRUD | `ferro-payments/src/service.rs` (existing `StripeGateway` + `MockStripeGateway` + `request_refund`) | exact |
| `ferro-payments/src/webhook.rs` | service | event-driven | `ferro-stripe/src/webhook/sync.rs` (SyncDispatcher), `ferro-payments/src/service.rs` (PaymentService patterns) | role-match |
| `ferro-payments/src/intent/entity.rs` | model | — | `ferro-payments/src/intent/entity.rs` (self — read-only, no schema change) | exact |

---

## Pattern Assignments

### `ferro-payments/src/lib.rs` (config/re-export) — D-10 BillableKind migration + wire_dispatcher re-export

**Analog:** `ferro-payments/src/lib.rs` (lines 1-39)

**Current BillableKind** (lines 28-39):
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
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

**Target BillableKind after D-10** — change to `Cow<'static, str>`:
```rust
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
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Re-export additions needed** (after line 24):
```rust
pub use service::wire_dispatcher;
pub use intent::lifecycle::{find_by_payment_intent, find_by_charge_id, attach_payment_intent};
```

**Breakage audit:** All callers of `as_str()` (e.g. `lifecycle::create_reserved(billable.kind().as_str(), ...)`) accept `&str`, not `&'static str` — no caller breakage. `const fn new` still works. `from_string` is net-new.

---

### `ferro-stripe/src/refund.rs` (service, request-response) — add `create_for_payment_intent`

**Analog:** `ferro-stripe/src/refund.rs` existing `create` (lines 18-40)

**Existing `create` pattern to mirror exactly** (lines 18-40):
```rust
pub async fn create(
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    let _ = idempotency_key;  // async-stripe 0.41 does not forward this key
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

**New function — identical structure, swap `params.charge` → `params.payment_intent`, `ChargeId` → `PaymentIntentId`:**
```rust
/// Creates a refund by `payment_intent_id`.
///
/// Used by the auto-refund path in `ferro-payments` when a
/// `checkout.session.completed` event carries no `charge_id`.
/// Mirrors `create()` exactly — swap `params.charge` for `params.payment_intent`.
///
/// NOTE: same async-stripe 0.41 idempotency caveat as `create()`.
pub async fn create_for_payment_intent(
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
    reason: Option<stripe::RefundReasonFilter>,
) -> Result<stripe::Refund, Error> {
    let _ = idempotency_key;
    let client = crate::Stripe::client();

    let mut params = stripe::CreateRefund::new();
    let pi_id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    params.payment_intent = Some(pi_id);
    params.amount = amount_cents;
    params.reason = reason;

    let refund = stripe::Refund::create(client, params).await?;
    Ok(refund)
}
```

`ferro-stripe/src/lib.rs` already re-exports `pub mod refund` — no additional re-export line needed; `ferro_stripe::refund::create_for_payment_intent` is reachable once the function exists.

---

### `ferro-payments/src/intent/lifecycle.rs` (service, CRUD) — three new helpers

**Analog:** existing `find_by_stripe_session` (lines 182-191) and `attach_session` (lines 135-158)

**`find_by_stripe_session` pattern to copy for `find_by_payment_intent` and `find_by_charge_id`** (lines 182-191):
```rust
pub async fn find_by_stripe_session<C: ConnectionTrait>(
    session_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::StripeSessionId.eq(session_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}
```

**`attach_session` pattern to copy for `attach_payment_intent`** (lines 135-158):
```rust
pub async fn attach_session<C: ConnectionTrait>(
    id: i64,
    stripe_session_id: &str,
    application_fee_cents: Option<i64>,
    conn: &C,
) -> Result<bool, PaymentError> {
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::StripeSessionId.is_null())
        .set_value(
            Column::StripeSessionId,
            Value::String(Some(Box::new(stripe_session_id.to_string()))),
        )
        .set_value(
            Column::ApplicationFeeCents,
            match application_fee_cents {
                Some(f) => Value::BigInt(Some(f)),
                None => Value::BigInt(None),
            },
        )
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}
```

**New helpers to add at end of the queries section:**

`find_by_payment_intent` — copy `find_by_stripe_session`, substitute `Column::PaymentIntentId`:
```rust
/// Return the `payment_intents` row whose `payment_intent_id` matches,
/// or `None` if absent.
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
```

`find_by_charge_id` — copy the same pattern, substitute `Column::ChargeId`:
```rust
/// Return the `payment_intents` row whose `charge_id` matches,
/// or `None` if absent. Fallback for `handle_charge_refunded` when
/// `payment_intent_id` is absent from the event.
pub async fn find_by_charge_id<C: ConnectionTrait>(
    charge_id: &str,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::ChargeId.eq(charge_id))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}
```

`attach_payment_intent` — copy `attach_session` shape, single column, `IS NULL` guard:
```rust
/// Persist `payment_intent_id` onto the row after marking paid.
/// Guard: `WHERE payment_intent_id IS NULL` — idempotent for retries.
/// Returns `Ok(true)` when written, `Ok(false)` when already set.
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

---

### `ferro-payments/src/service.rs` (service, CRUD) — extend StripeGateway + MockStripeGateway + D-12 guard

**Analog:** existing `StripeGateway::create_refund` method (lines 75-81) and `MockStripeGateway::create_refund` (lines 373-384)

**Existing `StripeGateway` trait method to mirror** (lines 75-81):
```rust
async fn create_refund(
    &self,
    charge_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
) -> Result<(), ferro_stripe::Error>;
```

**New trait method — same shape, swap `charge_id` for `payment_intent_id`:**
```rust
async fn create_refund_for_payment_intent(
    &self,
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    idempotency_key: &str,
) -> Result<(), ferro_stripe::Error>;
```

**Existing `MockStripeGateway` struct fields and `create_refund` impl to extend** (lines 342-384):
```rust
#[derive(Default)]
struct MockStripeGateway {
    checkout_calls: Mutex<Vec<CheckoutRequest>>,
    canned_checkout: Mutex<Option<Result<CheckoutResponse, ferro_stripe::Error>>>,
    refund_calls: Mutex<Vec<(String, Option<i64>)>>,
    canned_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>,
}

// existing create_refund records (charge_id.to_string(), amount_cents)
async fn create_refund(
    &self,
    charge_id: &str,
    amount_cents: Option<i64>,
    _key: &str,
) -> Result<(), ferro_stripe::Error> {
    self.refund_calls
        .lock()
        .unwrap()
        .push((charge_id.to_string(), amount_cents));
    self.canned_refund.lock().unwrap().take().unwrap_or(Ok(()))
}
```

**New fields and impl to add to `MockStripeGateway`** — separate call recorder for the payment_intent refund path:
```rust
// Add to struct:
pi_refund_calls: Mutex<Vec<(String, Option<i64>)>>,
canned_pi_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>,

// Add to impl:
async fn create_refund_for_payment_intent(
    &self,
    payment_intent_id: &str,
    amount_cents: Option<i64>,
    _key: &str,
) -> Result<(), ferro_stripe::Error> {
    self.pi_refund_calls
        .lock()
        .unwrap()
        .push((payment_intent_id.to_string(), amount_cents));
    self.canned_pi_refund.lock().unwrap().take().unwrap_or(Ok(()))
}
```

**D-01: `processed_log` field addition to `PaymentService`** — copy the existing `loader` field pattern (lines 150-154):
```rust
// Current PaymentService fields:
pub struct PaymentService<L: BillableLoader> {
    db: DatabaseConnection,
    stripe: Arc<dyn StripeGateway>,
    #[allow(dead_code)]   // remove this allow — loader is now read
    loader: L,
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

// Add processed_log after stripe:
processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>,
```

**D-01: `PaymentService::new` signature change** — existing `new` (lines 157-170) adds one param:
```rust
pub fn new(
    db: DatabaseConnection,
    stripe: Arc<dyn StripeGateway>,
    loader: L,
    processed_log: Arc<dyn ferro_stripe::ProcessedEventLog>,  // new
    return_url_builder: impl Fn(&dyn Billable) -> ReturnUrls + Send + Sync + 'static,
) -> Self {
    Self {
        db,
        stripe,
        loader,
        processed_log,
        return_url_builder: Arc::new(return_url_builder),
    }
}
```

**All existing test call sites** pass `PaymentService::new(db, mock, MockLoader, |_b| ReturnUrls {...})` — add `Arc::new(MemoryProcessedLog::new())` as the fourth argument:
```rust
// Before (line 556 pattern):
PaymentService::new(db.clone(), mock.clone(), MockLoader, |_b| ReturnUrls { ... })
// After:
PaymentService::new(
    db.clone(),
    mock.clone(),
    MockLoader,
    Arc::new(ferro_stripe::MemoryProcessedLog::new()),
    |_b| ReturnUrls { ... },
)
```

**D-12: `amount_cents <= 0` guard in `start_checkout`** — add immediately after `fn start_checkout` before the `create_reserved` call (after line 196):
```rust
if billable.amount_cents() <= 0 {
    return Err(PaymentError::StatusPrecondition(
        "amount_cents must be positive to start checkout".to_string(),
    ));
}
```

---

### `ferro-payments/src/webhook.rs` (NEW — service, event-driven)

**Analogs:**
- `ferro-stripe/src/webhook/sync.rs` lines 66-86 — `SyncDispatcher::on` consuming builder + handler closure shape
- `ferro-payments/src/service.rs` lines 279-297 — `GuardedUpdate` `IS NULL` dedup pattern for auto-refund snapshot
- `ferro-payments/src/service.rs` lines 259-265 — `Entity::find_by_id + map_err + ok_or` error chain

**Imports block to use** (mirrors service.rs imports + adds TransactionTrait):
```rust
use std::sync::Arc;

use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, EntityTrait, TransactionTrait};
use tracing;

use ferro_stripe::{
    MemoryProcessedLog, ProcessedEventLog, SyncDispatcher,
    StripeChargeRefunded, StripeCheckoutCompleted, StripeCheckoutExpired,
};

use crate::error::{AutoRefundReason, PaymentError};
use crate::intent::entity::{Column, Entity};
use crate::intent::lifecycle;
use crate::loader::BillableLoader;
use crate::service::PaymentService;
use crate::BillableKind;
```

**`wire_dispatcher` function** — consuming builder pattern mirroring `SyncDispatcher::on` (sync.rs lines 66-86):
```rust
/// Register three typed Stripe webhook handlers on `dispatcher` for the
/// payment intent lifecycle. Returns the updated dispatcher.
///
/// Call once at app startup:
/// ```rust,ignore
/// let dispatcher = ferro_payments::wire_dispatcher(SyncDispatcher::new(), service.clone());
/// ```
pub fn wire_dispatcher<L: BillableLoader + 'static>(
    dispatcher: SyncDispatcher,
    service: Arc<PaymentService<L>>,
) -> SyncDispatcher {
    // Pre-clone one Arc per handler slot; each invocation re-clones from these.
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

**Private error bridge** (D-04 — terminal outcomes must NOT reach this bridge):
```rust
fn payment_to_stripe_error(e: PaymentError) -> ferro_stripe::Error {
    match e {
        PaymentError::Db(ref db_err) => ferro_stripe::Error::Stripe(format!("db: {db_err}")),
        PaymentError::Stripe(ref s) => s.clone(),
        _ => ferro_stripe::Error::Stripe(format!("payment: {e}")),
    }
}
```

**`handle_session_completed`** — copy the `request_refund` `GuardedUpdate` dedup (service.rs lines 279-297) for the auto-refund snapshot, and `TransactionTrait::begin` for the billable call:
```rust
impl<L: BillableLoader> PaymentService<L> {
    pub(crate) async fn handle_session_completed(
        &self,
        event: StripeCheckoutCompleted,
    ) -> Result<(), PaymentError> {
        // D-05: idempotency fast-path
        if !self.processed_log
            .try_mark_processed(&event.event_id)
            .await
            .map_err(PaymentError::Stripe)?
        {
            return Ok(());
        }

        // D-06: lookup by session_id
        let Some(intent) = lifecycle::find_by_stripe_session(&event.session_id, &self.db).await?
        else {
            // No row for this session — nothing to do.
            return Ok(());
        };

        // D-06: mark reserved→paid (GuardedUpdate)
        let marked = lifecycle::mark_paid(intent.id, &self.db).await?;
        if !marked {
            // Side-state conflict: row not in reserved state (reaper released first).
            return self.trigger_auto_refund(
                &event.payment_intent_id,
                event.amount_total_cents,
                intent.id,
                AutoRefundReason::SideStateConflict,
            ).await;
        }

        // Attach payment_intent_id (guarded — idempotent).
        if let Some(ref pi_id) = event.payment_intent_id {
            lifecycle::attach_payment_intent(intent.id, pi_id, &self.db).await?;
        }

        // Open billable transaction
        let txn = self.db.begin().await.map_err(PaymentError::Db)?;
        let kind = BillableKind::from_string(intent.billable_kind.clone());
        match self.loader.load(kind, intent.billable_id).await {
            Err(_) => {
                txn.rollback().await.ok();
                return self.trigger_auto_refund(
                    &event.payment_intent_id,
                    event.amount_total_cents,
                    intent.id,
                    AutoRefundReason::LoaderError,
                ).await;
            }
            Ok(None) => {
                txn.rollback().await.ok();
                return self.trigger_auto_refund(
                    &event.payment_intent_id,
                    event.amount_total_cents,
                    intent.id,
                    AutoRefundReason::BillableVanished,
                ).await;
            }
            Ok(Some(billable)) => match billable.on_paid(&txn).await {
                Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                Err(_) => {
                    txn.rollback().await.ok();
                    return self.trigger_auto_refund(
                        &event.payment_intent_id,
                        event.amount_total_cents,
                        intent.id,
                        AutoRefundReason::SideStateConflict,
                    ).await;
                }
            },
        }
        Ok(())
    }
}
```

**`trigger_auto_refund` helper** (private — mirrors `request_refund` `GuardedUpdate` dedup, service.rs lines 279-285):
```rust
async fn trigger_auto_refund(
    &self,
    payment_intent_id: &Option<String>,
    amount_cents: i64,
    intent_id: i64,
    reason: AutoRefundReason,
) -> Result<(), PaymentError> {
    let Some(pi_id) = payment_intent_id else {
        tracing::debug!("auto_refund skipped: payment_intent_id absent (free/setup session)");
        return Ok(());
    };

    // D-09: atomic snapshot (WHERE IS NULL dedup — mirrors request_refund)
    let snapshot_ok = GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(intent_id))
        .filter(Column::RefundAmountCents.is_null())
        .set_value(Column::RefundAmountCents, Value::BigInt(Some(amount_cents)))
        .exec_at_most_one(&self.db)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;

    if !snapshot_ok {
        // Another caller already snapshotted the refund — no-op.
        return Ok(());
    }

    let idempotency_key = format!("auto-refund-{intent_id}");
    match self.stripe.create_refund_for_payment_intent(
        pi_id, Some(amount_cents), &idempotency_key,
    ).await {
        Ok(()) => {
            tracing::warn!(
                intent_id,
                pi_id = %pi_id,
                reason = ?reason,
                "auto-refund triggered"
            );
        }
        Err(e) => {
            // D-11: do NOT compensate-reset refund_amount_cents on failure.
            // The row is now "refund-in-flight": status=paid, refund_amount_cents IS NOT NULL,
            // refunded_at IS NULL. Phase-236 ReconcileRefundsInFlight reaper is the recovery.
            tracing::error!(
                intent_id,
                pi_id = %pi_id,
                reason = ?reason,
                err = %e,
                "auto-refund Stripe call failed; row is refund-in-flight (phase-236 reaper recovers)"
            );
        }
    }
    Ok(())
}
```

**`handle_session_expired`** (mirrors `handle_session_completed` idempotency + mark_released):
```rust
pub(crate) async fn handle_session_expired(
    &self,
    event: StripeCheckoutExpired,
) -> Result<(), PaymentError> {
    if !self.processed_log
        .try_mark_processed(&event.event_id)
        .await
        .map_err(PaymentError::Stripe)?
    {
        return Ok(());
    }

    let Some(intent) = lifecycle::find_by_stripe_session(&event.session_id, &self.db).await?
    else {
        return Ok(());
    };

    let marked = lifecycle::mark_released(intent.id, &self.db).await?;
    if !marked {
        return Ok(());  // no-op: already released
    }

    let txn = self.db.begin().await.map_err(PaymentError::Db)?;
    let kind = BillableKind::from_string(intent.billable_kind.clone());
    match self.loader.load(kind, intent.billable_id).await {
        Ok(Some(billable)) => match billable.on_released(&txn).await {
            Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
            Err(e) => { txn.rollback().await.ok(); return Err(e); }
        },
        _ => { txn.rollback().await.ok(); }
    }
    Ok(())
}
```

**`handle_charge_refunded`** (uses `find_by_payment_intent` with `find_by_charge_id` fallback):
```rust
pub(crate) async fn handle_charge_refunded(
    &self,
    event: StripeChargeRefunded,
) -> Result<(), PaymentError> {
    if !self.processed_log
        .try_mark_processed(&event.event_id)
        .await
        .map_err(PaymentError::Stripe)?
    {
        return Ok(());
    }

    // D-07: try payment_intent_id first, fallback to charge_id
    let intent_opt = match &event.payment_intent_id {
        Some(pi_id) => lifecycle::find_by_payment_intent(pi_id, &self.db).await?,
        None => None,
    };
    let intent = match intent_opt {
        Some(i) => i,
        None => {
            match lifecycle::find_by_charge_id(&event.charge_id, &self.db).await? {
                Some(i) => i,
                None => return Ok(()),  // no row for this refund
            }
        }
    };

    let marked = lifecycle::mark_refunded(intent.id, &self.db).await?;
    if !marked {
        return Ok(());  // already refunded or wrong source state
    }

    let txn = self.db.begin().await.map_err(PaymentError::Db)?;
    let kind = BillableKind::from_string(intent.billable_kind.clone());
    match self.loader.load(kind, intent.billable_id).await {
        Ok(Some(billable)) => {
            match billable.on_refunded(&txn, event.amount_refunded_cents).await {
                Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
                Err(e) => { txn.rollback().await.ok(); return Err(e); }
            }
        }
        _ => { txn.rollback().await.ok(); }
    }
    Ok(())
}
```

**Test infrastructure for `webhook.rs`** — copy the `fresh_db` + `TestMigrator` + `MockLoader` + `MockStripeGateway` pattern from `service.rs` tests (lines 321-415). Use `Arc::new(MemoryProcessedLog::new())` as the `processed_log` argument. Construct typed event structs directly (no JSON round-trip needed for `handle_*` unit tests):
```rust
fn make_completed_event(session_id: &str, pi_id: Option<&str>, amount: i64) -> StripeCheckoutCompleted {
    StripeCheckoutCompleted {
        event_id: "evt_test_1".to_string(),
        session_id: session_id.to_string(),
        payment_intent_id: pi_id.map(str::to_string),
        amount_total_cents: amount,
        currency: "eur".to_string(),
        metadata: Default::default(),
        customer_email: None,
    }
}

fn make_expired_event(session_id: &str) -> StripeCheckoutExpired {
    StripeCheckoutExpired {
        event_id: "evt_test_2".to_string(),
        session_id: session_id.to_string(),
        metadata: Default::default(),
    }
}

fn make_charge_refunded_event(charge_id: &str, pi_id: Option<&str>, amount: i64) -> StripeChargeRefunded {
    StripeChargeRefunded {
        event_id: "evt_test_3".to_string(),
        charge_id: charge_id.to_string(),
        payment_intent_id: pi_id.map(str::to_string),
        refund_id: None,
        amount_refunded_cents: amount,
        metadata: Default::default(),
    }
}
```

---

## Shared Patterns

### GuardedUpdate atomic conditional update
**Source:** `ferro-payments/src/intent/lifecycle.rs` lines 67-83, `ferro-payments/src/service.rs` lines 279-285
**Apply to:** `attach_payment_intent`, `trigger_auto_refund` dedup, `mark_*` helpers
```rust
GuardedUpdate::new(Entity)
    .filter(Column::Id.eq(id))
    .filter(Column::SomeColumn.is_null())   // precondition
    .set_value(Column::SomeColumn, Value::String(Some(Box::new(value.to_string()))))
    .exec_at_most_one(conn)
    .await
    .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
// Returns Ok(true) = row updated, Ok(false) = precondition not met (no-op)
```

### sea-orm transaction (begin/commit/rollback)
**Source:** sea-orm 1.1.20 `TransactionTrait` (verified: `commit(self)` + `rollback(self)` both consuming)
**Apply to:** all `handle_*` billable dispatch blocks
```rust
use sea_orm::TransactionTrait;

let txn = self.db.begin().await.map_err(PaymentError::Db)?;
match billable.on_paid(&txn).await {
    Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
    Err(e) => {
        txn.rollback().await.ok();  // consuming; ignore rollback error
        // then handle e
    }
}
// Note: txn is consumed by either branch. DatabaseTransaction implements ConnectionTrait,
// so lifecycle helpers accepting &C accept &txn directly.
```

### `ProcessedEventLog` idempotency fast-path
**Source:** `ferro-stripe/src/idempotency.rs` lines 44-45, 73-80
**Apply to:** first line of every `handle_*` method
```rust
if !self.processed_log
    .try_mark_processed(&event.event_id)
    .await
    .map_err(PaymentError::Stripe)?
{
    return Ok(());  // replay — skip all side effects
}
```

### `BillableKind::from_string` construction from DB row
**Source:** D-10 decision; `entity.rs` line 31 (`billable_kind: String`)
**Apply to:** all `handle_*` methods before `loader.load`
```rust
let kind = BillableKind::from_string(intent.billable_kind.clone());
match self.loader.load(kind, intent.billable_id).await { ... }
```

### `#[cfg(test)]` in-memory SQLite harness
**Source:** `ferro-payments/src/service.rs` lines 321-336 (`TestMigrator` + `fresh_db`)
**Apply to:** `webhook.rs` test module — copy verbatim, add `MemoryProcessedLog::new()` to `PaymentService::new` call
```rust
async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");
    TestMigrator::up(&conn, None).await.expect("migrate up");
    conn
}
```

---

## No Analog Found

All files in scope have close analogs. No files require pure invention from RESEARCH.md patterns.

| File | Status |
|------|--------|
| `ferro-payments/src/webhook.rs` | New file — but all internal patterns copy from `service.rs` + `lifecycle.rs` directly. `wire_dispatcher` closure shape is explicitly derived from `sync.rs::on` signature. |

---

## Metadata

**Analog search scope:** `ferro-payments/src/`, `ferro-stripe/src/`
**Files scanned:** 8 source files read directly
**Pattern extraction date:** 2026-06-17

# Phase 236: ferro-payments Reapers + Publish 0.1.0 — Pattern Map

**Mapped:** 2026-06-17
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-payments/src/reaper.rs` | service (job wrapper) | batch + request-response | `ferro-stripe/src/webhook/queue.rs` `ProcessStripeWebhook` | exact |
| `ferro-payments/src/service.rs` | service | batch, CRUD | `ferro-payments/src/webhook.rs` `handle_session_expired` + `handle_charge_refunded` | exact |
| `ferro-payments/src/intent/lifecycle.rs` | model / query | CRUD | existing finders in same file (`find_active_for`, `find_by_payment_intent`) | exact |
| `ferro-stripe/src/refund.rs` | utility | request-response | existing `create_for_payment_intent` in same file | exact |
| `ferro-payments/src/lib.rs` | config (re-exports) | — | existing `pub use webhook::wire_dispatcher` re-export pattern | exact |
| `ferro-payments/Cargo.toml` | config | — | existing `ferro-orm`/`ferro-stripe` path+version dep pattern | exact |
| `ferro-payments/tests/integration.rs` | test | request-response | `framework/tests/constraint_map_pg_gate.rs` `#[ignore]` + env-var guard | exact |
| `docs/src/features/payments.md` | docs | — | `docs/src/features/stripe.md` (structure and voice) | role-match |
| `docs/src/SUMMARY.md` | config | — | existing `- [Stripe](features/stripe.md)` entry | exact |

---

## Pattern Assignments

### `ferro-payments/src/reaper.rs` (service, batch — NEW)

**Analog:** `ferro-stripe/src/webhook/queue.rs`

**Full struct + impl template** (lines 32–94 of queue.rs — the load-bearing pattern for D-02):

```rust
// ferro-stripe/src/webhook/queue.rs lines 32-94
// Copy: derive block, #[serde(skip)] Arc field, ::new() constructor, Job impl

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    pub event_type: String,          // serialized identity fields
    pub raw_body: String,
    pub connect_account_id: Option<String>,
    #[serde(skip)]
    pub dispatcher: Option<Arc<SyncDispatcher>>,  // runtime-only, not persisted
}

impl ProcessStripeWebhook {
    pub fn new(
        event_type: String,
        raw_body: String,
        connect_account_id: Option<String>,
        dispatcher: Arc<SyncDispatcher>,
    ) -> Self {
        Self { event_type, raw_body, connect_account_id, dispatcher: Some(dispatcher) }
    }
}

#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let dispatcher = self
            .dispatcher
            .as_ref()
            .ok_or_else(|| ferro_queue::Error::JobFailed {
                job: "ProcessStripeWebhook".to_string(),
                message: "dispatcher not injected — use ProcessStripeWebhook::new()".to_string(),
            })?;
        // ... call the runtime handle ...
        dispatcher.dispatch(event).await.map_err(|e| ferro_queue::Error::JobFailed {
            job: "ProcessStripeWebhook".to_string(),
            message: e.to_string(),
        })
    }

    fn name(&self) -> &'static str { "ProcessStripeWebhook" }
}
```

**Adaptation for `ReleaseExpiredPaymentIntents<L>` and `ReconcileRefundsInFlight<L>`:**

1. Replace `dispatcher: Option<Arc<SyncDispatcher>>` with `service: Option<Arc<PaymentService<L>>>`.
2. The reaper jobs have **no serialized identity fields** — they select rows at execution time. The struct body is just the serde-skipped service handle.
3. Add `#[serde(bound = "")]` immediately after the derive block to suppress the implicit `L: Serialize + DeserializeOwned` bound that the derive macro would otherwise inject (research pitfall 2):

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]   // suppress L: Serialize + DeserializeOwned
pub struct ReleaseExpiredPaymentIntents<L: BillableLoader + 'static> {
    #[serde(skip)]
    pub service: Option<Arc<PaymentService<L>>>,
}

impl<L: BillableLoader + 'static> ReleaseExpiredPaymentIntents<L> {
    pub fn new(service: Arc<PaymentService<L>>) -> Self {
        Self { service: Some(service) }
    }
}

#[ferro_queue::async_trait]
impl<L: BillableLoader + 'static> ferro_queue::Job for ReleaseExpiredPaymentIntents<L> {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        let svc = self.service.as_ref()
            .ok_or_else(|| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: "service not injected — use ReleaseExpiredPaymentIntents::new()".to_string(),
            })?;
        svc.release_expired().await
            .map(|_| ())
            .map_err(|e| ferro_queue::Error::JobFailed {
                job: "ReleaseExpiredPaymentIntents".to_string(),
                message: e.to_string(),
            })
    }

    fn name(&self) -> &'static str { "ReleaseExpiredPaymentIntents" }
}
```

Mirror the same structure for `ReconcileRefundsInFlight<L>` calling `svc.reconcile_refunds_in_flight()`.

**Test pattern from queue.rs** (lines 103–113) — copy the `job_name` and `new_sets_service_to_some` tests:

```rust
// ferro-stripe/src/webhook/queue.rs lines 103-113
#[test]
fn process_stripe_webhook_job_name() {
    let dispatcher = Arc::new(SyncDispatcher::new());
    let job = ProcessStripeWebhook::new("invoice.paid".to_string(), "{}".to_string(), None, dispatcher);
    use ferro_queue::Job;
    assert_eq!(job.name(), "ProcessStripeWebhook");
}

#[test]
fn new_sets_dispatcher_to_some() { ... assert!(job.dispatcher.is_some()); }
```

---

### `ferro-payments/src/service.rs` — `release_expired_at` + `release_expired` methods (MODIFY)

**Primary analog:** `ferro-payments/src/webhook.rs` lines 200–245 (`handle_session_expired`) — the per-intent transaction with `mark_released` bool handling.

**Per-intent transaction pattern** (webhook.rs lines 221–244):

```rust
// ferro-payments/src/webhook.rs lines 221-244
// Copy: begin/commit/rollback structure, is_transient propagation, loader Ok(None) skip

let marked = lifecycle::mark_released(intent.id, &self.db).await?;
if !marked {
    return Ok(()); // no-op: already released
}

let txn = self.db.begin().await.map_err(PaymentError::Db)?;
let kind = BillableKind::from_string(intent.billable_kind.clone());
match self.loader.load(kind, intent.billable_id).await {
    Ok(Some(billable)) => match billable.on_released(&txn).await {
        Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
        Err(e) => {
            txn.rollback().await.ok();
            if is_transient(&e) {
                return Err(e);
            }
            // Terminal business-state error — absorb
        }
    },
    _ => {
        txn.rollback().await.ok();
    }
}
Ok(())
```

**Adaptation for `release_expired_at`:** wrap the per-intent block in a `for intent in expired` loop; catch per-intent errors and `continue` (D-05 isolation). D-06 benign-skip differs from the webhook: for the webhook, `_` means "loader error on a paid row — absorb"; for the release reaper, "loader vanished is benign (no money captured) — log and skip". Use `tracing::warn!` on the benign skip.

**`handle_charge_refunded` as analog for `reconcile_refunds_in_flight` resolve path** (webhook.rs lines 252–307):

```rust
// ferro-payments/src/webhook.rs lines 277-307
// Copy: mark_refunded bool guard, txn → on_refunded(amount_cents) pattern

let marked = lifecycle::mark_refunded(intent.id, &self.db).await?;
if !marked {
    return Ok(()); // already refunded or wrong source state
}

let txn = self.db.begin().await.map_err(PaymentError::Db)?;
let kind = BillableKind::from_string(intent.billable_kind.clone());
match self.loader.load(kind, intent.billable_id).await {
    Ok(Some(billable)) => {
        match billable.on_refunded(&txn, event.amount_refunded_cents).await {
            Ok(()) => txn.commit().await.map_err(PaymentError::Db)?,
            Err(e) => {
                txn.rollback().await.ok();
                if is_transient(&e) { return Err(e); }
                // Terminal — absorb
            }
        }
    }
    _ => { txn.rollback().await.ok(); }
}
Ok(())
```

**`StripeGateway` trait extension pattern** (service.rs lines 69–89) — add `fetch_refund_status_for_payment_intent` as a fourth method on the trait following the same `async fn` + `ferro_stripe::Error` return shape:

```rust
// ferro-payments/src/service.rs lines 69-89
// Copy: #[async_trait::async_trait] on trait, same return-error type

#[async_trait::async_trait]
pub trait StripeGateway: Send + Sync {
    async fn create_checkout_session(&self, req: CheckoutRequest) -> Result<CheckoutResponse, ferro_stripe::Error>;
    async fn create_refund(&self, charge_id: &str, amount_cents: Option<i64>, idempotency_key: &str) -> Result<(), ferro_stripe::Error>;
    async fn create_refund_for_payment_intent(&self, payment_intent_id: &str, amount_cents: Option<i64>, idempotency_key: &str) -> Result<(), ferro_stripe::Error>;
    // NEW:
    async fn fetch_refund_status_for_payment_intent(&self, payment_intent_id: &str) -> Result<RefundStatus, ferro_stripe::Error>;
}
```

**`MockStripeGateway` extension pattern** (service.rs lines 388–454) — add `poll_calls: Mutex<Vec<String>>` and `canned_refund_status: Mutex<Option<Result<RefundStatus, ferro_stripe::Error>>>` fields following the same `Mutex<Vec<...>>` + `take().unwrap_or(...)` canned pattern:

```rust
// ferro-payments/src/service.rs lines 388-454
// Copy: Mutex<Vec<...>> recording field, Mutex<Option<Result<...>>> canned result,
// .take().unwrap_or(Ok(...)) pattern on canned field

async fn create_refund_for_payment_intent(
    &self, payment_intent_id: &str, amount_cents: Option<i64>, _key: &str,
) -> Result<(), ferro_stripe::Error> {
    self.pi_refund_calls.lock().unwrap().push((payment_intent_id.to_string(), amount_cents));
    self.canned_pi_refund.lock().unwrap().take().unwrap_or(Ok(()))
}
// Mirror this shape for fetch_refund_status_for_payment_intent
```

**`StripeClientGateway` prod impl pattern** (service.rs lines 103–162) — add the fourth method calling `ferro_stripe::refund::list_for_payment_intent`, mapping the result to `RefundStatus`:

```rust
// ferro-payments/src/service.rs lines 137-161
// Copy: call ferro_stripe::refund::create_for_payment_intent style, same ? propagation

async fn create_refund_for_payment_intent(
    &self, payment_intent_id: &str, amount_cents: Option<i64>, idempotency_key: &str,
) -> Result<(), ferro_stripe::Error> {
    ferro_stripe::refund::create_for_payment_intent(
        payment_intent_id, amount_cents, idempotency_key, None,
    ).await?;
    Ok(())
}
```

**`sea_orm::TransactionTrait` import** — already present in webhook.rs line 19 as `use sea_orm::TransactionTrait;`. Copy verbatim to service.rs when adding the reaper methods.

---

### `ferro-payments/src/intent/lifecycle.rs` — `find_expired` + `find_refunds_in_flight` (MODIFY)

**Analog:** same file, `find_active_for` (lines 166–178) and `find_by_payment_intent` (lines 198–207).

**`find_active_for` — multi-filter finder pattern** (lines 166–178):

```rust
// ferro-payments/src/intent/lifecycle.rs lines 166-178
// Copy: EntityTrait::find() + ColumnTrait filter chain + .one()/.all() + map_err(PaymentError::Db)

pub async fn find_active_for<C: ConnectionTrait>(
    kind: &str,
    billable_id: i64,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::BillableKind.eq(kind))
        .filter(Column::BillableId.eq(billable_id))
        .filter(Column::Status.is_in([PaymentIntentStatus::Reserved, PaymentIntentStatus::Paid]))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}
```

**Adaptation for `find_expired`:** replace filters with `Column::Status.eq(PaymentIntentStatus::Reserved)` + `Column::ExpiresAt.lt(now)`, return `Vec<entity::Model>` with `.all(conn)`.

**Adaptation for `find_refunds_in_flight`:** filters are `Column::Status.eq(PaymentIntentStatus::Paid)` + `Column::RefundAmountCents.is_not_null()` + `Column::RefundedAt.is_null()` + `Column::PaidAt.lt(older_than)`. Return `Vec<entity::Model>` with `.all(conn)`. The `is_not_null()` and `is_null()` methods are from `sea_orm::ColumnTrait` already imported.

**Import line** (lifecycle.rs line 10):

```rust
// ferro-payments/src/intent/lifecycle.rs line 10 — already present:
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
```

No new imports are needed for the two finders — `ColumnTrait`, `ConnectionTrait`, `EntityTrait`, `QueryFilter` are all already imported.

---

### `ferro-stripe/src/refund.rs` — `list_for_payment_intent` (MODIFY)

**Analog:** `create_for_payment_intent` in the same file (lines 58–80).

**Full function pattern** (lines 58–80):

```rust
// ferro-stripe/src/refund.rs lines 58-80
// Copy: pi_id parse → stripe::PaymentIntentId, params construction, Stripe::client(), await?

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

**Adaptation for `list_for_payment_intent`:**

- Parse `payment_intent_id` using the same `let pi_id: stripe::PaymentIntentId = payment_intent_id.parse().map_err(...)` idiom.
- Instead of `stripe::CreateRefund::new()`, use `stripe::ListRefunds::new()` and assign `params.payment_intent = Some(pi_id)`.
- Instead of `stripe::Refund::create(client, params).await?`, use `stripe::Refund::list(client, &params).await?` which returns `stripe::List<stripe::Refund>`.
- Return `Ok(list.data)` (`Vec<stripe::Refund>`).

```rust
// New function — follows create_for_payment_intent exactly, different params type and API call:
pub async fn list_for_payment_intent(
    payment_intent_id: &str,
) -> Result<Vec<stripe::Refund>, Error> {
    let client = crate::Stripe::client();
    let pi_id: stripe::PaymentIntentId = payment_intent_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid payment intent id: {payment_intent_id}")))?;
    let mut params = stripe::ListRefunds::new();
    params.payment_intent = Some(pi_id);
    params.limit = Some(10);
    let list = stripe::Refund::list(client, &params).await?;
    Ok(list.data)
}
```

Key types (verified from async-stripe 0.41 registry source): `stripe::ListRefunds::new()`, `params.payment_intent: Option<PaymentIntentId>`, `stripe::Refund::list(client, &params) -> Response<List<Refund>>`, `list.data: Vec<Refund>`, `Refund.status: Option<String>` values `"pending"/"succeeded"/"failed"/"canceled"/"requires_action"`.

---

### `ferro-payments/src/lib.rs` — re-exports (MODIFY)

**Analog:** existing re-export pattern (lib.rs lines 13–29), specifically `pub use webhook::wire_dispatcher` (line 22).

```rust
// ferro-payments/src/lib.rs lines 11-22
// Copy: mod declaration + pub use pattern

pub mod billable;
mod error;
pub mod intent;
pub mod loader;
pub mod migration;
pub mod service;
mod webhook;
// NEW:
mod reaper;

// existing re-exports...
pub use webhook::wire_dispatcher;
// NEW re-exports:
pub use reaper::{ReconcileRefundsInFlight, ReleaseExpiredPaymentIntents};
```

---

### `ferro-payments/Cargo.toml` — add `ferro-queue` dependency (MODIFY)

**Analog:** existing `ferro-orm` and `ferro-stripe` path+version deps (Cargo.toml lines 22–23):

```toml
# ferro-payments/Cargo.toml lines 22-23
# Copy: path = "../crate-name", version = "x.y" pattern

ferro-orm = { path = "../ferro-orm", version = "0.2" }
ferro-stripe = { path = "../ferro-stripe", version = "0.9" }
```

**Adaptation:** add one line after the existing ferro-* deps:

```toml
ferro-queue = { path = "../ferro-queue", version = "0.2" }
```

No publish ordering change required: `ferro-queue` is Wave 1a (leaf), `ferro-payments` is Wave 1c (already after ferro-queue). Adding a direct dep does not change wave placement.

---

### `ferro-payments/tests/integration.rs` — `#[ignore]`-gated integration test (NEW)

**Analog:** `framework/tests/constraint_map_pg_gate.rs` for the `#[ignore]` + env-var guard pattern (lines 1–85), and the `MockBillable` + in-memory SQLite harness pattern from `ferro-payments/src/webhook.rs` tests (lines 499–615) for the example `Billable` definition.

**`#[ignore]` + env-var guard pattern** (pg_gate.rs lines 43–50):

```rust
// framework/tests/constraint_map_pg_gate.rs lines 43-50
// Copy: #[ignore = "..."] attribute form, env::var early-return skip, no panic!

#[tokio::test]
#[ignore = "requires a live Postgres (set DATABASE_URL); run with -- --ignored"]
async fn pg_constraint_name_identity_match() {
    let url = pg_url(); // reads std::env::var("DATABASE_URL")
    let db = Database::connect(&url).await
        .unwrap_or_else(|e| panic!("connect to Postgres at {url}: {e}"));
    // ...
}
```

**Adaptation for `integration.rs`:**

```rust
// ferro-payments/tests/integration.rs — new file
#[tokio::test]
#[ignore = "requires STRIPE_TEST_SECRET_KEY (Stripe test mode); run with -- --ignored"]
async fn e2e_checkout_and_release() {
    let key = match std::env::var("STRIPE_TEST_SECRET_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("STRIPE_TEST_SECRET_KEY not set — skipping");
            return;  // early return, not panic!
        }
    };
    // ... init ferro_stripe::Stripe with key, run start_checkout + release_expired ...
}
```

**Example `Billable` — copy `ConnectBillable`/`DirectBillable` from service.rs tests** (service.rs lines 489–571) as the "tiny example Billable". Adapt for the integration test: use a stable `billable_kind`, meaningful `checkout_line_description`, and a small amount so Stripe test-mode does not require real card auth.

**`fresh_db()` and `TestMigrator` pattern** (lifecycle.rs lines 260–273 or service.rs lines 367–381) — copy into the integration test file:

```rust
// ferro-payments/src/intent/lifecycle.rs lines 260-273
async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect to in-memory sqlite");
    TestMigrator::up(&conn, None).await.expect("migrate up");
    conn
}
```

---

### `docs/src/features/payments.md` — consumer-facing page (NEW)

**Analog:** `docs/src/features/stripe.md` — structure, heading depth, voice, code block style.

**Structure to mirror from stripe.md:**

1. One-paragraph intro (what it does, what it covers).
2. `## Quick Start` — three-step registration code block (register migrations, `wire_dispatcher`, schedule reapers).
3. Feature sections with code blocks: Checkout, Expiry Release (reaper), Refund Reconciliation (reaper), Recovery model.
4. `## Environment Variables` — table (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET; cross-ref from stripe.md).
5. Cross-link back to `[Stripe Integration](stripe.md)`.

**Heading and code block voice from stripe.md lines 1–10:**

```markdown
# Payments

ferro-payments adds polymorphic payment intent tracking to Ferro applications...

## Quick Start

Register migrations and wire the dispatcher at application startup:
...
```

**SUMMARY.md link pattern** (SUMMARY.md line 49):

```markdown
- [Stripe](features/stripe.md)
```

Add `- [Payments](features/payments.md)` immediately after the Stripe line (line 49).

---

## Shared Patterns

### Transaction Begin/Commit/Rollback
**Source:** `ferro-payments/src/webhook.rs` lines 151–157 and 226–244
**Apply to:** `release_expired_at` and `reconcile_refunds_in_flight` in service.rs

```rust
// webhook.rs line 151
let txn = self.db.begin().await.map_err(PaymentError::Db)?;
// ... use &txn ...
txn.commit().await.map_err(PaymentError::Db)?;
// on error:
txn.rollback().await.ok();
```

The `sea_orm::TransactionTrait` import is required (webhook.rs line 19).

### GuardedUpdate Bool No-Op Semantics
**Source:** `ferro-payments/src/intent/lifecycle.rs` lines 67–83 (`mark_paid`) and `ferro-payments/src/webhook.rs` lines 131–143
**Apply to:** All reaper resolution paths that call `mark_released` or `mark_refunded`

```rust
// lifecycle.rs lines 88-105 (mark_released)
// Returns Ok(true) on success, Ok(false) when precondition not met (no-op)
let marked = lifecycle::mark_released(intent.id, &self.db).await?;
if !marked {
    return Ok(()); // racing webhook already took it — skip
}
```

### Per-Intent Error Isolation (reaper-specific)
**Source:** D-05 requirement, no exact analog — synthesize from webhook.rs batch-safe patterns.

```rust
// Pattern: per-intent async block returning Result, match on Err → log + continue
let result: Result<(), PaymentError> = async {
    // ... single intent work ...
}.await;
match result {
    Ok(()) => released += 1,
    Err(e) => {
        tracing::error!(intent_id = intent.id, err = %e, "reaper: per-intent error — continuing");
        // do not return Err here; let the loop continue
    }
}
```

### `#[serde(bound = "")]` on Generic Job Structs
**Source:** Research pitfall 2; `ProcessStripeWebhook` is not generic (no bound issue there), but the reaper structs are.
**Apply to:** Both `ReleaseExpiredPaymentIntents<L>` and `ReconcileRefundsInFlight<L>`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]  // prevents derive from injecting L: Serialize + DeserializeOwned
pub struct ReleaseExpiredPaymentIntents<L: BillableLoader + 'static> { ... }
```

### `tracing::warn!` / `tracing::error!` Observability
**Source:** `ferro-payments/src/webhook.rs` lines 355–370 (`trigger_auto_refund`)
**Apply to:** All reaper warn/error log sites

```rust
// webhook.rs lines 355-370
tracing::error!(
    intent_id,
    pi_id = %pi_id,
    reason = ?reason,
    err = %e,
    "auto-refund Stripe call failed; row is refund-in-flight (phase-236 reaper recovers)"
);
```

---

## No Analog Found

All target files have close analogs. No files in this phase lack a match.

---

## Metadata

**Analog search scope:** `ferro-payments/src/`, `ferro-stripe/src/`, `ferro-queue/src/`, `framework/tests/`, `docs/src/features/`
**Files scanned:** 10 source files read directly
**Pattern extraction date:** 2026-06-17

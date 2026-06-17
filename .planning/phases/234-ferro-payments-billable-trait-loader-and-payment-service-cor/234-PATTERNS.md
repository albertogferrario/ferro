# Phase 234: Billable trait + Loader + PaymentService core — Pattern Map

**Mapped:** 2026-06-17
**Files analyzed:** 8
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-payments/src/billable.rs` | trait-def | request-response | `ferro-stripe/src/idempotency.rs` | role-match (`#[async_trait]` object-safe trait with `Box<dyn>` return) |
| `ferro-payments/src/loader.rs` | trait-def | request-response | `ferro-stripe/src/idempotency.rs` | role-match (same `#[async_trait]` pattern) |
| `ferro-payments/src/service.rs` | service | CRUD + request-response | `ferro-payments/src/intent/lifecycle.rs` | exact (same DB layer, same error type, same `GuardedUpdate` pattern) |
| `ferro-payments/src/intent/lifecycle.rs` | lifecycle | CRUD | self (extend existing file) | exact (add `attach_session` alongside existing `mark_*` fns) |
| `ferro-payments/src/error.rs` | error | — | self (extend existing file) | exact (extend existing `thiserror` enum) |
| `ferro-payments/Cargo.toml` | config | — | `ferro-stripe/Cargo.toml` | exact (path + version dep pattern; `test-helpers` feature gate) |
| `ferro-payments/src/lib.rs` | config | — | self (extend existing file) | exact (add `pub use` lines to existing module) |
| `.github/workflows/publish.yml` | config | — | self (extend existing file) | exact (replicate Wave 1b loop structure for new Wave 1c) |

---

## Pattern Assignments

### `ferro-payments/src/billable.rs` (trait-def, request-response)

**Analog:** `ferro-stripe/src/idempotency.rs` (lines 37–46 for the `#[async_trait]` trait structure)

**Imports pattern** (derive from idempotency.rs lines 1–2 + lifecycle.rs lines 8–14):
```rust
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use crate::error::PaymentError;
use crate::BillableKind;
```

**Core trait pattern** — object-safe `#[async_trait]` with sync accessors + async side effects (modelled on `ProcessedEventLog` lines 37–46, extended with sync methods and default):
```rust
// ferro-stripe/src/idempotency.rs lines 37-46 — the #[async_trait] object-safe shape:
#[async_trait]
pub trait ProcessedEventLog: Send + Sync {
    async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>;
}
```

Apply that shape to `Billable`:
```rust
#[async_trait]
pub trait Billable: Send + Sync {
    // Sync accessors (no #[async_trait] expansion needed — plain fn)
    fn kind(&self) -> BillableKind;
    fn id(&self) -> i64;
    fn tenant_id(&self) -> i64;
    fn amount_cents(&self) -> i64;
    fn currency(&self) -> &str;
    fn checkout_line_description(&self) -> String;

    // Default: non-Connect billables return None; Connect billables override (D-05).
    fn connect_account_id(&self) -> Option<String> {
        None
    }

    // Async side effects — each takes a &DatabaseTransaction per D-04.
    async fn on_paid(&self, txn: &DatabaseTransaction) -> Result<(), PaymentError>;
    async fn on_released(&self, txn: &DatabaseTransaction) -> Result<(), PaymentError>;
    async fn on_refunded(
        &self,
        txn: &DatabaseTransaction,
        amount_cents: i64,
    ) -> Result<(), PaymentError>;
}
```

**Object-safety constraint:** `Billable: Send + Sync` is required so `Box<dyn Billable>` can be returned by `BillableLoader::load` (async-trait boxes futures as `Box<dyn Future + Send>`). Do NOT add `Clone` (D-06).

---

### `ferro-payments/src/loader.rs` (trait-def, request-response)

**Analog:** `ferro-stripe/src/idempotency.rs` (same `#[async_trait]` trait pattern)

**Core trait pattern** (single async method returning `Result<Option<Box<dyn Billable>>, PaymentError>`):
```rust
use async_trait::async_trait;
use crate::{billable::Billable, error::PaymentError, BillableKind};

#[async_trait]
pub trait BillableLoader: Send + Sync {
    /// Load a billable entity by its kind discriminator and primary key.
    ///
    /// `Ok(None)` = entity no longer exists (triggers auto-refund in phase 235).
    /// `Err(PaymentError::Loader(..))` = consumer-side failure.
    /// Tenant scoping is the loader's responsibility (D-08).
    async fn load(
        &self,
        kind: BillableKind,
        id: i64,
    ) -> Result<Option<Box<dyn Billable>>, PaymentError>;
}
```

**Test mock shape** (copy from `MemoryProcessedLog` pattern in idempotency.rs lines 53–80, but use `Mutex<Vec>` for call recording instead of `DashMap`):
```rust
// ferro-stripe/src/idempotency.rs lines 53-80 — Default + simple state struct:
pub struct MemoryProcessedLog {
    seen: dashmap::DashMap<String, ()>,
}
impl Default for MemoryProcessedLog { ... }
#[async_trait]
impl ProcessedEventLog for MemoryProcessedLog { ... }
```

For the test `MockLoader`, follow the same pattern but with `Mutex<Vec>` for call recording.

---

### `ferro-payments/src/service.rs` (service, CRUD + request-response)

**Analog:** `ferro-payments/src/intent/lifecycle.rs` (entire file — same DB layer, same error type, same `GuardedUpdate` usage)

**Imports pattern** (extend lifecycle.rs imports lines 8–14 with Arc and the new local types):
```rust
use std::sync::Arc;
use chrono::Utc;
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};

use crate::billable::Billable;
use crate::error::PaymentError;
use crate::intent::entity::{Column, Entity};
use crate::intent::status::PaymentIntentStatus;
use crate::intent::lifecycle;
use crate::loader::BillableLoader;
```

**`PaymentService` struct + constructor pattern** (D-09/D-10/D-11):
```rust
pub struct PaymentService<L: BillableLoader> {
    db: DatabaseConnection,
    stripe: Arc<dyn StripeGateway>,
    #[allow(dead_code)] // wired by handle_* in phase 235
    loader: L,
    return_url_builder: Arc<dyn Fn(&dyn Billable) -> ReturnUrls + Send + Sync>,
}

impl<L: BillableLoader> PaymentService<L> {
    pub fn new(
        db: DatabaseConnection,
        stripe: Arc<dyn StripeGateway>,
        loader: L,
        return_url_builder: impl Fn(&dyn Billable) -> ReturnUrls + Send + Sync + 'static,
    ) -> Self {
        Self {
            db,
            stripe,
            loader,
            return_url_builder: Arc::new(return_url_builder),
        }
    }
}
```

**`StripeGateway` trait pattern** (local seam per D-02/D-03 — copy `#[async_trait]` shape from idempotency.rs):
```rust
#[async_trait::async_trait]
pub trait StripeGateway: Send + Sync {
    async fn create_checkout_session(
        &self,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse, ferro_stripe::Error>;

    async fn create_refund(
        &self,
        charge_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error>;
}
```

Note: `CheckoutResponse { intent: ferro_stripe::CheckoutIntent, application_fee_cents: Option<i64> }` wraps `CheckoutIntent` so the production gateway (which calls `Stripe::config()`) can return the computed fee back to `PaymentService` for snapshotting. This keeps `Stripe::config()` calls inside production-only code (D-20).

**Production `StripeClientGateway` impl pattern** (wrap `ferro-stripe/src/checkout.rs` CheckoutBuilder, lines 64–273, and `ferro-stripe/src/refund.rs` lines 18–40):
```rust
pub struct StripeClientGateway;

#[async_trait::async_trait]
impl StripeGateway for StripeClientGateway {
    async fn create_checkout_session(
        &self,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse, ferro_stripe::Error> {
        // Fee computation lives here — safe to call Stripe::config() in production.
        let application_fee_cents = req.connect_account_id.as_ref().map(|_| {
            ferro_stripe::Stripe::config().application_fee_for(req.amount_cents)
        }).flatten();

        let mut builder = ferro_stripe::CheckoutBuilder::new(ferro_stripe::Mode::Payment)
            .line_item(ferro_stripe::LineItem {
                name: req.line_description.clone(),
                description: None,
                unit_amount_cents: req.amount_cents,
                quantity: 1,
                currency: req.currency.clone(),
            })
            .success_url(&req.success_url)
            .cancel_url(&req.cancel_url)
            .idempotency_key(&req.idempotency_key);
        if let Some(account_id) = &req.connect_account_id {
            builder = builder.destination(account_id, application_fee_cents);
        }
        let intent = builder.create().await?;
        Ok(CheckoutResponse { intent, application_fee_cents })
    }

    async fn create_refund(
        &self,
        charge_id: &str,
        amount_cents: Option<i64>,
        idempotency_key: &str,
    ) -> Result<(), ferro_stripe::Error> {
        // ferro-stripe/src/refund.rs line 18-40 — discard Refund return value.
        ferro_stripe::refund::create(charge_id, amount_cents, idempotency_key, None).await?;
        Ok(())
    }
}
```

**`start_checkout` flow pattern** (compose lifecycle functions — lifecycle.rs lines 26–57 for `create_reserved`; GuardedUpdate for `attach_session`):
```rust
pub async fn start_checkout(
    &self,
    billable: &dyn Billable,
    ttl: chrono::Duration,
) -> Result<CheckoutUrl, PaymentError> {
    // Step 1: INSERT reserved row (lifecycle.rs create_reserved pattern, lines 26-57).
    let expires_at = Utc::now() + ttl;
    let row = lifecycle::create_reserved(
        billable.tenant_id(),
        billable.kind().as_str(),
        billable.id(),
        billable.amount_cents(),
        billable.currency(),
        expires_at,
        &self.db,
    ).await?;

    // Step 2: Build CheckoutRequest and call gateway (never call CheckoutBuilder directly).
    let urls = (self.return_url_builder)(billable);
    let req = CheckoutRequest {
        amount_cents: billable.amount_cents(),
        currency: billable.currency().to_string(),
        line_description: billable.checkout_line_description(),
        success_url: urls.success_url,
        cancel_url: urls.cancel_url,
        idempotency_key: format!("checkout-{}", row.id),
        connect_account_id: billable.connect_account_id(),
    };
    let resp = self.stripe.create_checkout_session(req).await
        .map_err(PaymentError::Stripe)?;

    // Step 3: Attach session_id + application_fee_cents (GuardedUpdate WHERE IS NULL).
    // On failure, leave row for phase-236 reaper (D-14).
    lifecycle::attach_session(
        row.id,
        &resp.intent.session_id,
        resp.application_fee_cents,
        &self.db,
    ).await?;

    Ok(CheckoutUrl(resp.intent.url))
}
```

**`request_refund` GuardedUpdate dedup pattern** (lifecycle.rs `mark_*` lines 67–127 is the template; RESEARCH.md Pattern 3):
```rust
pub async fn request_refund(
    &self,
    intent_id: i64,
    amount_cents: i64,
) -> Result<(), PaymentError> {
    // Load and validate preconditions (D-15).
    let row = Entity::find_by_id(intent_id)
        .one(&self.db)
        .await
        .map_err(PaymentError::Db)?
        .ok_or(PaymentError::NotFound)?;

    if row.status != PaymentIntentStatus::Paid {
        return Err(PaymentError::StatusPrecondition(
            "request_refund requires status = paid".to_string(),
        ));
    }
    let charge_id = row.charge_id.ok_or_else(|| {
        PaymentError::StatusPrecondition("charge_id must be set to request a refund".to_string())
    })?;

    // Atomic snapshot (GuardedUpdate WHERE refund_amount_cents IS NULL).
    // Returns Ok(false) = already in flight — no-op (D-15 dedup).
    let snapshot_ok = GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(intent_id))
        .filter(Column::RefundAmountCents.is_null())
        .set_value(Column::RefundAmountCents, Value::BigInt(Some(amount_cents)))
        .exec_at_most_one(&self.db)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;

    if !snapshot_ok {
        // Already in flight — do not call Stripe twice (D-15).
        return Ok(());
    }

    let idempotency_key = format!("refund-{intent_id}");
    self.stripe
        .create_refund(&charge_id, Some(amount_cents), &idempotency_key)
        .await
        .map_err(PaymentError::Stripe)
}
```

**Test harness pattern** (lifecycle.rs lines 166–397 is the direct template):
```rust
// ferro-payments/src/intent/lifecycle.rs lines 166-189 — fresh_db() + TestMigrator:
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, EntityTrait};
    use sea_orm_migration::MigratorTrait;
    use std::sync::{Arc, Mutex};
    use crate::migration::m20260617_create_payment_intents::Migration as CreateTable;

    struct TestMigrator;
    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateTable)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        TestMigrator::up(&conn, None).await.unwrap();
        conn
    }

    // MockStripeGateway — records calls, returns canned results (D-02).
    // Pattern: Mutex<Vec> for call log + Mutex<Option<Result>> for canned result.
    #[derive(Default)]
    struct MockStripeGateway {
        checkout_calls: Mutex<Vec<CheckoutRequest>>,
        canned_checkout: Mutex<Option<Result<CheckoutResponse, ferro_stripe::Error>>>,
        refund_calls: Mutex<Vec<(String, Option<i64>)>>,
        canned_refund: Mutex<Option<Result<(), ferro_stripe::Error>>>,
    }

    #[async_trait::async_trait]
    impl StripeGateway for MockStripeGateway {
        async fn create_checkout_session(
            &self,
            req: CheckoutRequest,
        ) -> Result<CheckoutResponse, ferro_stripe::Error> {
            self.checkout_calls.lock().unwrap().push(req);
            self.canned_checkout.lock().unwrap().take()
                .unwrap_or_else(|| Ok(fake_checkout_response()))
        }
        async fn create_refund(
            &self,
            charge_id: &str,
            amount_cents: Option<i64>,
            _key: &str,
        ) -> Result<(), ferro_stripe::Error> {
            self.refund_calls.lock().unwrap().push((charge_id.to_string(), amount_cents));
            self.canned_refund.lock().unwrap().take().unwrap_or(Ok(()))
        }
    }

    // Seed helpers mirror lifecycle.rs seed_reserved / seed_with_status pattern
    // (lifecycle.rs lines 192-223).
}
```

---

### `ferro-payments/src/intent/lifecycle.rs` — add `attach_session` (MODIFY)

**Analog:** `mark_paid` in the same file (lines 67–83) — exact template for a `GuardedUpdate` setting two columns guarded by an IS NULL filter.

**Pattern to copy** (lines 67–83 of lifecycle.rs, adapted for session attachment):
```rust
// Direct template: mark_paid (lifecycle.rs lines 67-83)
pub async fn mark_paid<C: ConnectionTrait>(id: i64, conn: &C) -> Result<bool, PaymentError> {
    let now = Utc::now();
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::Status.eq(PaymentIntentStatus::Reserved))  // ← guard
        .set_value(Column::Status, Value::String(Some(Box::new("paid".to_string()))))
        .set_value(Column::PaidAt, Value::ChronoDateTimeUtc(Some(Box::new(now))))
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
}
```

Apply the same shape to `attach_session`, with IS NULL guard on `StripeSessionId`:
```rust
/// Attach `stripe_session_id` and snapshot `application_fee_cents` after a
/// successful Stripe checkout session creation.
///
/// Guard: `WHERE stripe_session_id IS NULL` — idempotent for retries.
/// Returns `Ok(true)` on success, `Ok(false)` when session already attached.
pub async fn attach_session<C: ConnectionTrait>(
    id: i64,
    stripe_session_id: &str,
    application_fee_cents: Option<i64>,
    conn: &C,
) -> Result<bool, PaymentError> {
    GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::StripeSessionId.is_null())  // idempotency guard
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

**Column → Value type mapping** (from entity.rs lines 37–38 + lifecycle.rs Value usage):
- `stripe_session_id: Option<String>` → `Value::String(Some(Box::new(s)))` / `Value::String(None)`
- `application_fee_cents: Option<i64>` → `Value::BigInt(Some(f))` / `Value::BigInt(None)`
- `paid_at: Option<DateTimeUtc>` → `Value::ChronoDateTimeUtc(Some(Box::new(now)))` (reference from lifecycle.rs line 78)

---

### `ferro-payments/src/error.rs` (MODIFY — extend existing enum)

**Analog:** self — extend the existing 3-variant enum (lines 1–18 of the current file).

**Current state** (error.rs lines 1–18):
```rust
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("payment: not found")]
    NotFound,
    #[error("payment: status precondition not met: {0}")]
    StatusPrecondition(String),
    #[error("payment: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}
```

**`ferro_stripe::Error` structure** (ferro-stripe/src/error.rs lines 1–38 — the type used for `#[from]`):
```rust
// ferro-stripe/src/error.rs — verified variants:
pub enum Error {
    Config(String), Stripe(String), NoConnectAccount,
    WebhookVerification(String), EventAlreadyProcessed(String),
    MissingIdempotencyKey, ManualCaptureRequiresPaymentMode,
}
impl From<stripe::StripeError> for Error { ... }
```

**Extension pattern** — add three variants; `Stripe` uses `#[from]`, `Loader` does NOT (different error source), `AutoRefundTriggered` is a struct variant:
```rust
// Add AFTER existing three variants:

#[error("payment: stripe error: {0}")]
Stripe(#[from] ferro_stripe::Error),

// No #[from] — set manually via PaymentError::Loader(Box::new(err))
#[error("payment: loader error: {0}")]
Loader(Box<dyn std::error::Error + Send + Sync>),

// Defined here (D-18); only returned by webhook handlers in phase 235.
#[error("payment: auto-refund triggered: {reason:?}")]
AutoRefundTriggered { reason: AutoRefundReason },
```

**`AutoRefundReason` enum** (new type in the same file, after `PaymentError`):
```rust
#[derive(Debug)]
pub enum AutoRefundReason {
    LoaderError,
    BillableVanished,
    SideStateConflict,
}
```

---

### `ferro-payments/Cargo.toml` (MODIFY)

**Analog:** `ferro-stripe/Cargo.toml` lines 27 for path+version internal dep pattern:
```toml
# ferro-stripe/Cargo.toml line 27 — the path+version pattern:
ferro-queue = { path = "../ferro-queue", version = "0.2" }
```

**Edit to apply** — add to `[dependencies]` after the existing `ferro-orm` line:
```toml
ferro-stripe = { path = "../ferro-stripe", version = "0.9" }
```

**`test-helpers` feature gate analog** (ferro-stripe/Cargo.toml lines 33–35):
```toml
# ferro-stripe/Cargo.toml lines 33-35 — feature gate for test infrastructure:
[features]
test-helpers = []
```

For phase 234, the `MockStripeGateway` lives in `#[cfg(test)]` (not a `test-helpers` feature) per D-20 — no feature addition needed unless the planner decides otherwise.

---

### `ferro-payments/src/lib.rs` (MODIFY — add re-exports)

**Analog:** self — extend the existing `pub use` block (lines 1–29 of current file).

**Current re-export block** (lines 7–14):
```rust
pub use error::PaymentError;
pub use intent::entity::{ActiveModel, Column, Entity as PaymentIntentEntity, Model};
pub use intent::lifecycle::{
    create_reserved, find_active_for, find_by_stripe_session, mark_paid, mark_refunded,
    mark_released,
};
pub use intent::status::PaymentIntentStatus;
pub use migration::CreatePaymentIntentsTable;
```

**Extension pattern** — add new modules and re-exports after existing ones:
```rust
// New module declarations (after existing mod declarations):
pub mod billable;
pub mod loader;
pub mod service;

// New re-exports:
pub use billable::Billable;
pub use loader::BillableLoader;
pub use service::{
    AutoRefundReason, CheckoutRequest, CheckoutResponse, CheckoutUrl,
    PaymentService, ReturnUrls, StripeClientGateway, StripeGateway,
};
// PaymentError re-export already present; extended variants auto-available.
// Also re-export attach_session from lifecycle:
pub use intent::lifecycle::attach_session;
```

---

### `.github/workflows/publish.yml` (MODIFY — Wave 1c insertion)

**Analog:** The existing Wave 1b step (lines 236–263) and its index-wait (lines 265–268).

**Current Wave 1b** (lines 247, 265–268 — verified from file):
```yaml
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-payments ferro-projection ferro-deployments"
```

```yaml
- name: Wait for crates.io index update (Wave 1b)
  run: |
    echo "Waiting for crates.io to index Wave 1b crates..."
    sleep 30
```

**Edit 1:** Remove `ferro-payments` from `WAVE1B_CRATES` (line 247):
```yaml
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

**Edit 2:** Insert new Wave 1c step + index-wait AFTER the `Wait for crates.io index update (Wave 1b)` step and BEFORE `Publish Wave 2`. Copy the Wave 1b loop body exactly, changing only the crates variable name and echo strings:
```yaml
      - name: Publish Wave 1c (depends on Wave 1b only)
        run: |
          echo "Publishing Wave 1c crates..."
          WAVE1C_CRATES="ferro-payments"
          for crate in $WAVE1C_CRATES; do
            echo "Publishing $crate..."
            if OUTPUT=$(cargo publish -p $crate --no-verify 2>&1); then
              echo "$crate published successfully"
            else
              if echo "$OUTPUT" | grep -q "already exists\|already uploaded"; then
                echo "$crate already published, skipping"
              else
                echo "Failed to publish $crate"
                echo "$OUTPUT"
                exit 1
              fi
            fi
            sleep 5
          done

      - name: Wait for crates.io index update (Wave 1c)
        run: |
          echo "Waiting for crates.io to index Wave 1c crates..."
          sleep 30
```

---

## Shared Patterns

### `#[async_trait]` object-safe trait
**Source:** `ferro-stripe/src/idempotency.rs` lines 29–46
**Apply to:** `billable.rs`, `loader.rs`, `service.rs` (StripeGateway trait), `service.rs` (MockStripeGateway impl)
```rust
use async_trait::async_trait;  // crate already in ferro-payments Cargo.toml

#[async_trait]
pub trait SomeTrait: Send + Sync {
    async fn some_method(&self, arg: &str) -> Result<bool, Error>;
}
```

### `GuardedUpdate` + `exec_at_most_one` + `map_err` idiom
**Source:** `ferro-payments/src/intent/lifecycle.rs` lines 67–83 and 89–127
**Apply to:** `attach_session` (lifecycle.rs), `request_refund` (service.rs)
```rust
// The map_err idiom used everywhere in lifecycle.rs (e.g. line 82):
.exec_at_most_one(conn)
.await
.map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))
```

### `thiserror` enum extension
**Source:** `ferro-payments/src/error.rs` lines 1–18 (current) + `ferro-stripe/src/error.rs` lines 1–38 (for `#[from]` pattern)
**Apply to:** `error.rs`
- `#[from]` on `Db(sea_orm::DbErr)` is the exact template for `Stripe(ferro_stripe::Error)`
- `Loader` variant uses no `#[from]` — consumer wraps manually

### In-memory SQLite test harness
**Source:** `ferro-payments/src/intent/lifecycle.rs` lines 167–189
**Apply to:** `service.rs` `#[cfg(test)]` block
```rust
// lifecycle.rs lines 173-189 — TestMigrator + fresh_db():
struct TestMigrator;
#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateTable)]
    }
}
async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate up");
    conn
}
```

### Path+version internal dependency
**Source:** `ferro-stripe/Cargo.toml` line 27
**Apply to:** `ferro-payments/Cargo.toml`
```toml
ferro-queue = { path = "../ferro-queue", version = "0.2" }
# ↑ exact pattern — copy for ferro-stripe dep:
ferro-stripe = { path = "../ferro-stripe", version = "0.9" }
```

### `seed_with_status` raw SQL helper (for tests requiring non-lifecycle row states)
**Source:** `ferro-payments/src/intent/lifecycle.rs` lines 202–223
**Apply to:** `service.rs` test helpers (need `paid` rows with `charge_id` for `request_refund` tests)
```rust
// lifecycle.rs lines 202-223 — seed a row bypassing lifecycle guards:
async fn seed_with_status(conn: &sea_orm::DatabaseConnection, status: &str) -> i64 {
    conn.execute_unprepared(&format!(
        "INSERT INTO payment_intents \
         (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
          expires_at,reserved_at) \
         VALUES (1,'booking',99,500,'USD','{status}',\
         '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
    ))
    .await
    .expect("seed row");
    // ... last_insert_rowid() query to return id
}
```

For `request_refund` tests, extend this helper to also set `charge_id` and optionally `refund_amount_cents`.

---

## No Analog Found

All 8 files have close analogs. No files require falling back to RESEARCH.md patterns exclusively.

---

## Metadata

**Analog search scope:** `ferro-payments/src/`, `ferro-stripe/src/`, `ferro-orm/src/`, `.github/workflows/`
**Files read for pattern extraction:** 12
**Pattern extraction date:** 2026-06-17

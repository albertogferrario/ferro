# Phase 233: ferro-payments Crate Scaffold + PaymentIntent Entity + Migration — Pattern Map

**Mapped:** 2026-06-17
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-payments/Cargo.toml` | config | — | `ferro-reservation/Cargo.toml` | exact (same dep profile: sea-orm + sea-orm-migration + ferro-orm path dep + chrono/serde/thiserror) |
| `ferro-payments/src/lib.rs` | config | — | `ferro-audit/src/lib.rs` | exact (module declarations + pub use re-exports) |
| `ferro-payments/src/error.rs` | utility | — | `ferro-stripe/src/error.rs` | exact (thiserror derive, one enum per crate) |
| `ferro-payments/src/intent/entity.rs` | model | CRUD | `app/src/models/entities/api_keys.rs` + `ferro-audit/src/entity.rs` | exact (DeriveEntityModel, DateTimeUtc, Option<JsonValue>) |
| `ferro-payments/src/intent/status.rs` | model | — | RESEARCH.md Finding 1 (no workspace precedent for DeriveActiveEnum) | research-only |
| `ferro-payments/src/intent/lifecycle.rs` | service | CRUD | `ferro-orm/src/guarded.rs` (GuardedUpdate builder + tests) | role-match |
| `ferro-payments/src/intent/mod.rs` | config | — | `ferro-stripe/src/lib.rs` (mod declarations pattern) | role-match |
| `ferro-payments/src/migration/m20260617_create_payment_intents.rs` | migration | CRUD | `ferro-reservation/src/migration.rs` + `app/src/migrations/m20260228_create_api_keys_table.rs` + `ferro-mcp-oauth/src/migration.rs` | exact |
| `ferro-payments/src/migration/mod.rs` | config | — | `ferro-audit/src/lib.rs` migration export line | exact |
| `Cargo.toml` (workspace root) `members` array | config | — | existing `members` list in `Cargo.toml` lines 9-42 | exact (slot after `ferro-reservation`) |
| `.github/workflows/publish.yml` `WAVE1B_CRATES` | config | — | publish.yml line 247 `WAVE1B_CRATES` string | exact (append `ferro-payments` after `ferro-reservation`) |

---

## Pattern Assignments

### `ferro-payments/Cargo.toml` (config)

**Analog:** `ferro-reservation/Cargo.toml` (primary) + `ferro-stripe/Cargo.toml` (keywords/categories shape)

**Manifest pattern** (`ferro-reservation/Cargo.toml` lines 1-39):
```toml
[package]
name = "ferro-reservation"
version.workspace = true          # NOTE: ferro-payments uses "0.1.0" (D-14), not workspace
edition.workspace = true
license.workspace = true
description = "Generic hold/commit/release resource reservation kernel for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["reservation", "booking", "sea-orm", "concurrency", "ferro"]
categories = ["database", "asynchronous"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
sea-orm = "1.0"
sea-orm-migration = "1.0"         # REQUIRED — separate crate even though sea-orm pulls it
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
ferro-orm = { path = "../ferro-orm", version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

**Exact ferro-payments Cargo.toml** (from RESEARCH.md Finding — verified):
```toml
[package]
name = "ferro-payments"
version = "0.1.0"
edition.workspace = true
license.workspace = true
description = "Polymorphic payment intent layer for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["payments", "stripe", "sea-orm", "billing", "ferro"]
categories = ["database", "web-programming"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
sea-orm = "1.0"
sea-orm-migration = "1.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
async-trait = "0.1"
ferro-orm = { path = "../ferro-orm", version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

Key differences from ferro-reservation: `version = "0.1.0"` (explicit, not workspace); no `uuid`, no `ferro-events`, no `ferro-audit` deps; no `[features]` block in phase 233.

---

### `ferro-payments/src/lib.rs` (config — module declarations + re-exports)

**Analog:** `ferro-audit/src/lib.rs` (lines 54-75)

**Module + re-export pattern** (`ferro-audit/src/lib.rs` lines 54-75):
```rust
mod actor;
mod entity;
mod entry;
mod error;
mod migration;

pub use actor::AuditActor;
pub use entry::AuditEntry;
pub use error::AuditError;
pub use migration::Migration as CreateAuditLogTable;
pub use entity::Entity as AuditLogEntity;
```

**ferro-payments lib.rs pattern to follow:**
```rust
pub mod intent;
pub mod migration;
mod error;

pub use error::PaymentError;
pub use intent::entity::{ActiveModel, Column, Entity as PaymentIntentEntity, Model};
pub use intent::lifecycle::{create_reserved, find_active_for, find_by_stripe_session,
    mark_paid, mark_refunded, mark_released};
pub use intent::status::PaymentIntentStatus;
pub use migration::CreatePaymentIntentsTable;

// BillableKind newtype — minimal, no persistence logic
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillableKind(&'static str);

impl BillableKind {
    pub const fn new(s: &'static str) -> Self { Self(s) }
    pub fn as_str(&self) -> &'static str { self.0 }
}
```

---

### `ferro-payments/src/error.rs` (utility — thiserror error enum)

**Analog:** `ferro-stripe/src/error.rs` (lines 1-38)

**thiserror pattern** (`ferro-stripe/src/error.rs` lines 1-32):
```rust
/// Errors that can occur in ferro-stripe operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("stripe config error: {0}")]
    Config(String),

    /// Stripe API returned an error.
    #[error("stripe API error: {0}")]
    Stripe(String),

    /// No Connect account linked to this tenant.
    #[error("no Stripe Connect account linked to this tenant")]
    NoConnectAccount,
}
```

**ferro-orm error.rs pattern** for `#[from]` on DB variant (`ferro-orm/src/error.rs` lines 7-35):
```rust
#[derive(Debug, Error)]
pub enum GuardedError {
    #[error("guarded: predicate matched no rows")]
    NoRowsAffected,

    #[error("guarded: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}
```

**ferro-payments error.rs to implement** (phase 233 minimal set, D-13):
```rust
/// Errors that can occur in ferro-payments data layer operations.
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    /// The PaymentIntent or billable entity was not found.
    #[error("payment: not found")]
    NotFound,

    /// A state-transition was attempted from an invalid source status.
    /// Contains a human-readable description of the precondition that failed.
    #[error("payment: status precondition not met: {0}")]
    StatusPrecondition(String),

    /// Underlying database error.
    #[error("payment: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}
```

Note: `Stripe`, `Loader`, and `AutoRefundTriggered` variants are deferred to phase 234.

---

### `ferro-payments/src/intent/entity.rs` (model — DeriveEntityModel)

**Analog:** `app/src/models/entities/api_keys.rs` (lines 1-26) + `ferro-audit/src/entity.rs` (lines 1-66)

**DeriveEntityModel pattern** (`app/src/models/entities/api_keys.rs` lines 5-26):
```rust
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub scopes: Option<String>,
    pub last_used_at: Option<DateTimeUtc>,   // DateTimeUtc from sea_orm::entity::prelude
    pub expires_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**JSON column pattern** (`ferro-audit/src/entity.rs` lines 8-9, 43-44):
```rust
use serde_json::Value as JsonValue;

pub before: Option<JsonValue>,
pub after: Option<JsonValue>,
```

**ferro-payments entity.rs column map** (RESEARCH.md Finding 6 — all types verified):
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::intent::status::PaymentIntentStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payment_intents")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub tenant_id: i64,
    pub billable_kind: String,         // raw TEXT, no enum — D-05
    pub billable_id: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub status: PaymentIntentStatus,   // DeriveActiveEnum, TEXT column — D-04
    pub stripe_session_id: Option<String>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub application_fee_cents: Option<i64>,
    pub expires_at: DateTimeUtc,       // NOT NULL, set in Rust — D-06
    pub reserved_at: DateTimeUtc,
    pub paid_at: Option<DateTimeUtc>,
    pub released_at: Option<DateTimeUtc>,
    pub refunded_at: Option<DateTimeUtc>,
    pub refund_amount_cents: Option<i64>,
    pub metadata: Option<JsonValue>,   // json() column, nullable — D-07
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

`DateTimeUtc` = `chrono::DateTime<chrono::Utc>` re-exported from `sea_orm::entity::prelude`. No extra import needed — available via `use sea_orm::entity::prelude::*`.

---

### `ferro-payments/src/intent/status.rs` (model — DeriveActiveEnum)

**Analog:** No workspace precedent. `DeriveActiveEnum` is not used anywhere else in this workspace (confirmed by grep). Use RESEARCH.md Finding 1 as the template.

**DeriveActiveEnum exact syntax** (RESEARCH.md Finding 1 — verified against sea-orm 1.1.20 source and docs.rs):
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum PaymentIntentStatus {
    #[sea_orm(string_value = "reserved")]
    Reserved,
    #[sea_orm(string_value = "paid")]
    Paid,
    #[sea_orm(string_value = "released")]
    Released,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "refunded")]
    Refunded,
}
```

Constraints:
- `db_type` must be `"Text"` (not `"String(StringLen::None)"`) to match `.text().not_null()` in migration (D-04).
- `EnumIter` is required alongside `DeriveActiveEnum` — SeaORM's column mapper needs it to enumerate variants.
- All variants use `string_value` — cannot mix with `num_value`.
- The entity field type is `pub status: PaymentIntentStatus` (not `String`).

---

### `ferro-payments/src/intent/lifecycle.rs` (service — CRUD with GuardedUpdate)

**Analog:** `ferro-orm/src/guarded.rs` (lines 1-101) — GuardedUpdate builder API and usage examples.

**GuardedUpdate builder pattern** (`ferro-orm/src/guarded.rs` lines 21-101):
```rust
// Constructor
GuardedUpdate::new(entity)        // entity: E where E: EntityTrait

// Filter (AND-combined, multiple calls allowed)
.filter(col.eq(val))             // any IntoCondition
.filter(col.eq(val2))

// Set literal value
.set_value(col, Value::String(Some(Box::new("paid".to_string()))))

// Set expression (arithmetic, col references)
.set_expr(col, Expr::col(other_col).sub(n))

// Execute — use exec_at_most_one for lifecycle transitions (D-09, Pitfall 3)
.exec_at_most_one(&conn).await   // Ok(true)=updated, Ok(false)=no-op, Err=TooManyRows
.exec_one(&conn).await           // Err(NoRowsAffected) on 0 rows — do NOT use for lifecycle
```

**Test usage pattern** showing multi-column set (`ferro-orm/src/guarded.rs` lines 284-311):
```rust
GuardedUpdate::new(counters::Entity)
    .filter(counters::Column::Id.eq(1))
    .filter(counters::Column::Status.eq("pending"))
    .set_expr(
        counters::Column::Quantity,
        Expr::col(counters::Column::Quantity).sub(2),
    )
    .set_value(
        counters::Column::Status,
        Value::String(Some(Box::new("committed".to_string()))),
    )
    .exec_one(&conn)
    .await
    .expect("multi-column guarded update");
```

**ferro-payments lifecycle.rs pattern to follow** (for `mark_paid` — others are structurally identical):
```rust
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set};
use ferro_orm::{GuardedUpdate, Value};
use chrono::Utc;

use crate::intent::entity::{self, Column, Entity};
use crate::intent::status::PaymentIntentStatus;
use crate::error::PaymentError;

pub async fn mark_paid<C: ConnectionTrait>(
    id: i64,
    conn: &C,
) -> Result<bool, PaymentError> {
    let now = Utc::now();
    let updated = GuardedUpdate::new(Entity)
        .filter(Column::Id.eq(id))
        .filter(Column::Status.eq(PaymentIntentStatus::Reserved))
        .set_value(Column::Status, Value::String(Some(Box::new("paid".to_string()))))
        .set_value(Column::PaidAt, Value::ChronoDateTimeUtc(Some(Box::new(now))))
        .exec_at_most_one(conn)
        .await
        .map_err(|e| PaymentError::Db(sea_orm::DbErr::Custom(e.to_string())))?;
    Ok(updated)  // false = no-op (race no-op per D-09); caller may convert to StatusPrecondition
}
```

**`find_active_for` pattern** (D-11 — uses `is_in`):
```rust
pub async fn find_active_for<C: ConnectionTrait>(
    kind: &str,
    billable_id: i64,
    conn: &C,
) -> Result<Option<entity::Model>, PaymentError> {
    Entity::find()
        .filter(Column::BillableKind.eq(kind))
        .filter(Column::BillableId.eq(billable_id))
        .filter(Column::Status.is_in([
            PaymentIntentStatus::Reserved,
            PaymentIntentStatus::Paid,
        ]))
        .one(conn)
        .await
        .map_err(PaymentError::Db)
}
```

**Pitfall:** Always include at least one `.set_value()` call — an empty builder returns `GuardedError::EmptyUpdate` (tested in ferro-orm T-16-4). The `status` column is always set in every transition method, so this is naturally satisfied.

---

### `ferro-payments/src/migration/m20260617_create_payment_intents.rs` (migration — CRUD + cross-backend DDL)

**Analog:** `ferro-reservation/src/migration.rs` (manual MigrationName impl + table + indexes + test scaffold) + `app/src/migrations/m20260228_create_api_keys_table.rs` (DeriveIden + `big_integer().auto_increment().primary_key()` + `timestamp_with_time_zone()`) + `ferro-mcp-oauth/src/migration.rs` (`.unique()` index + multi-index pattern)

**Manual MigrationName impl** (`ferro-reservation/src/migration.rs` lines 21-26):
```rust
pub struct Migration;

impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260513_000001_create_reservations_table"
    }
}
```

**Table create + DeriveIden + primary key** (`app/src/migrations/m20260228_create_api_keys_table.rs` lines 1-80):
```rust
use sea_orm_migration::prelude::*;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKeys::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ApiKeys::Scopes).text().null())
                    .col(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_prefix")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::Prefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ApiKeys {
    Table,
    Id,
    Scopes,
    ExpiresAt,
}
```

**Unique index pattern** (`ferro-mcp-oauth/src/migration.rs` lines 53-63):
```rust
manager
    .create_index(
        Index::create()
            .name("idx_oauth_clients_client_id")
            .table(OauthClients::Table)
            .col(OauthClients::ClientId)
            .unique()
            .to_owned(),
    )
    .await
```

**execute_unprepared + DatabaseBackend branch pattern** (`ferro-mcp-oauth/src/migration.rs` lines 104-160 region — also RESEARCH.md Finding 4):
```rust
use sea_orm::DatabaseBackend;

// Inside up():
let db = manager.get_connection();
match manager.get_database_backend() {
    DatabaseBackend::Postgres | DatabaseBackend::Sqlite => {
        db.execute_unprepared(
            "CREATE UNIQUE INDEX uq_payment_intents_active \
             ON payment_intents (billable_kind, billable_id) \
             WHERE status IN ('reserved','paid')"
        ).await?;
    }
    DatabaseBackend::MySql => {
        db.execute_unprepared(
            "ALTER TABLE payment_intents \
             ADD COLUMN active_billable_key VARCHAR(600) \
             AS (CASE WHEN status IN ('reserved','paid') \
                      THEN CONCAT(billable_kind, '|', CAST(billable_id AS CHAR)) \
                      ELSE NULL END) STORED"
        ).await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX uq_payment_intents_active_mysql \
             ON payment_intents (active_billable_key)"
        ).await?;
    }
}
```

Note: `manager.get_connection()` returns `&DatabaseConnection`; `DatabaseConnection` implements `execute_unprepared(&str)`. This is the only API for raw DDL with no value bindings in a migration.

**Full migration DeriveIden enum** for payment_intents (all columns per RESEARCH.md Finding 6):
```rust
#[derive(DeriveIden)]
enum PaymentIntents {
    Table,
    Id,
    TenantId,
    BillableKind,
    BillableId,
    AmountCents,
    Currency,
    Status,
    StripeSessionId,
    PaymentIntentId,
    ChargeId,
    ApplicationFeeCents,
    ExpiresAt,
    ReservedAt,
    PaidAt,
    ReleasedAt,
    RefundedAt,
    RefundAmountCents,
    Metadata,
}
```

**Test scaffold pattern** (`ferro-reservation/src/migration.rs` lines 144-203 and `ferro-audit/src/migration.rs` lines 103-176 — both are exact templates):
```rust
#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    async fn name_exists(conn: &sea_orm::DatabaseConnection, name: &str, obj_type: &str) -> bool {
        let row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT name FROM sqlite_master WHERE type='{obj_type}' AND name='{name}'"),
            ))
            .await
            .expect("query sqlite_master");
        row.is_some()
    }

    #[tokio::test]
    async fn migration_creates_table_and_indexes() {
        let conn = fresh_db().await;
        assert!(name_exists(&conn, "payment_intents", "table").await);
        // plain indexes:
        assert!(name_exists(&conn, "idx_payment_intents_tenant_status", "index").await);
        assert!(name_exists(&conn, "idx_payment_intents_stripe_session_id", "index").await);
        assert!(name_exists(&conn, "idx_payment_intents_payment_intent_id", "index").await);
        // partial unique (SQLite path):
        assert!(name_exists(&conn, "uq_payment_intents_active", "index").await);
    }

    #[tokio::test]
    async fn migration_down_drops_table() {
        let conn = fresh_db().await;
        TestMigrator::down(&conn, Some(1)).await.expect("migrate down");
        assert!(!name_exists(&conn, "payment_intents", "table").await);
    }

    #[tokio::test]
    async fn partial_unique_rejects_second_active_row() {
        let conn = fresh_db().await;
        // INSERT first active row — succeeds
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ).await.expect("first insert");
        // INSERT second active row for the same billable — must fail
        let result = conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ).await;
        assert!(result.is_err(), "second active insert must violate partial unique index");
    }

    #[tokio::test]
    async fn partial_unique_allows_new_active_after_release() {
        let conn = fresh_db().await;
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','released',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ).await.expect("released row");
        // A new 'reserved' row for the same billable must succeed
        conn.execute_unprepared(
            "INSERT INTO payment_intents \
             (tenant_id,billable_kind,billable_id,amount_cents,currency,status,\
              expires_at,reserved_at) \
             VALUES (1,'order',42,1000,'EUR','reserved',\
             '2030-01-01T00:00:00Z','2026-06-17T00:00:00Z')"
        ).await.expect("new reserved after release");
    }
}
```

---

### `ferro-payments/src/migration/mod.rs` (config — migration module + public export)

**Analog:** `ferro-audit/src/lib.rs` line 67 and `ferro-reservation/src/lib.rs` migration export

**Pattern:**
```rust
mod m20260617_create_payment_intents;

pub use m20260617_create_payment_intents::Migration as CreatePaymentIntentsTable;

/// Convenience constructor — consumers pass the return value into their `Migrator`.
pub fn migration_create_payment_intents() -> Box<dyn sea_orm_migration::MigrationTrait> {
    Box::new(m20260617_create_payment_intents::Migration)
}
```

---

### `ferro-payments/src/intent/mod.rs` (config — submodule declarations)

**Analog:** `ferro-stripe/src/lib.rs` lines 43-53 (pub mod declarations)

**Pattern:**
```rust
pub mod entity;
pub mod lifecycle;
pub mod status;
```

---

### `Cargo.toml` (workspace root) — `members` array addition

**Analog:** `Cargo.toml` lines 9-42 (existing members list)

**Current `members` ending** (`Cargo.toml` lines 35-42):
```toml
    "ferro-orm",
    "ferro-audit",
    "ferro-migration",
    "ferro-reservation",
    "ferro-projection",
    "ferro-bundle",
    "ferro-deployments",
    "ferro-assets",
    "ferro-text",
]
```

**Change:** Add `"ferro-payments"` after `"ferro-reservation"` (topological grouping — depends on ferro-orm which is already a member):
```toml
    "ferro-reservation",
    "ferro-payments",     # ← insert here
    "ferro-projection",
```

---

### `.github/workflows/publish.yml` — `WAVE1B_CRATES` addition

**Analog:** `publish.yml` line 247 (current `WAVE1B_CRATES` string)

**Current Wave 1b line** (`publish.yml` line 247):
```bash
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

**Change:** Append `ferro-payments` after `ferro-reservation` (same rationale: depends on ferro-orm which is Wave 1a):
```bash
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-payments ferro-projection ferro-deployments"
```

Rationale confirmed by RESEARCH.md Finding 7: `ferro-orm` is Wave 1a; `ferro-payments` depends on `ferro-orm`; therefore Wave 1b. `ferro-reservation` uses the same pattern and is already in Wave 1b.

---

## Shared Patterns

### SeaORM Migration Structure
**Source:** `ferro-reservation/src/migration.rs` + `app/src/migrations/m20260228_create_api_keys_table.rs`
**Apply to:** `m20260617_create_payment_intents.rs`

Key invariants across all migrations:
1. Open with `use sea_orm_migration::prelude::*;` — this brings `Table`, `Index`, `ColumnDef`, `SchemaManager`, `MigrationTrait`, `DbErr`, `Expr`, `DeriveIden`, `DeriveMigrationName` into scope.
2. `DeriveIden` enum uses `Table` as the first variant (it maps to the table name string).
3. `manager.create_table(...).await?;` then `manager.create_index(...).await` (last one returns the `?` directly).
4. `down()` always calls `Table::drop().table(X::Table).to_owned()`.
5. `if_not_exists()` on every `Table::create()`.

### Error Handling in Lifecycle Methods
**Source:** `ferro-orm/src/error.rs` + `ferro-stripe/src/error.rs`
**Apply to:** `lifecycle.rs`, future `service.rs`

- All `sea_orm::DbErr` converts to `PaymentError::Db` via `#[from]` or `.map_err(PaymentError::Db)`.
- `GuardedError` (from `exec_at_most_one`) is NOT `#[from]`-convertible to `PaymentError` in phase 233 — map manually.
- `Ok(false)` from `exec_at_most_one` is a no-op by design (D-09); callers may choose to convert to `StatusPrecondition` if they need to distinguish.

### Test DB Setup
**Source:** `ferro-reservation/src/migration.rs` lines 144-163 + `ferro-audit/src/migration.rs` lines 103-127
**Apply to:** all `#[cfg(test)]` blocks in phase 233

Invariant: every test module that needs the DB calls `Database::connect("sqlite::memory:")` followed by `TestMigrator::up(&conn, None)`. The `fresh_db()` helper extracts this pattern — copy verbatim.

### `async_trait` Annotation
**Source:** `ferro-audit/src/migration.rs` line 21, `ferro-reservation/src/migration.rs` line 28
**Apply to:** all `impl MigrationTrait` and `impl MigratorTrait` blocks

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration { ... }
```

Always use `#[async_trait::async_trait]` (not `async fn` in traits directly) — sea-orm-migration 1.0 uses the async-trait macro, not RPITIT.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-payments/src/intent/status.rs` | model | — | `DeriveActiveEnum` has zero usages in this workspace. RESEARCH.md Finding 1 is the only template — use it verbatim. |

---

## Column Type Quick Reference (RESEARCH.md Finding 6 — verified)

| Column | Migration `.col()` call | Entity Field Type |
|--------|------------------------|-------------------|
| `id` | `.big_integer().not_null().auto_increment().primary_key()` | `i64` |
| `tenant_id`, `billable_id`, `amount_cents`, `application_fee_cents` (null), `refund_amount_cents` (null) | `.big_integer().not_null()` / `.null()` | `i64` / `Option<i64>` |
| `billable_kind`, `currency`, `stripe_session_id` (null), `payment_intent_id` (null), `charge_id` (null) | `.text().not_null()` / `.null()` | `String` / `Option<String>` |
| `status` | `.text().not_null()` | `PaymentIntentStatus` (DeriveActiveEnum) |
| `expires_at`, `reserved_at` | `.timestamp_with_time_zone().not_null()` | `DateTimeUtc` |
| `paid_at`, `released_at`, `refunded_at` | `.timestamp_with_time_zone().null()` | `Option<DateTimeUtc>` |
| `metadata` | `.json().null()` | `Option<serde_json::Value>` |

Note: `metadata` uses `.json()` not `.json_binary()` — safe because no btree index on metadata is created in phase 233. See RESEARCH.md Pitfall 2 for the distinction.

---

## Metadata

**Analog search scope:** `ferro-reservation/`, `ferro-audit/`, `ferro-stripe/`, `ferro-orm/`, `ferro-mcp-oauth/`, `app/src/migrations/`, `app/src/models/`, workspace `Cargo.toml`, `.github/workflows/publish.yml`
**Files read:** 15 source files
**Pattern extraction date:** 2026-06-17

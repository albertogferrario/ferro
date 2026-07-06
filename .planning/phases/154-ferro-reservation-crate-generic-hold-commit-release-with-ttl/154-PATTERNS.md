# Phase 154: ferro-reservation — Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 20 new/modified files
**Analogs found:** 20 / 20

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-reservation/Cargo.toml` | config | — | `ferro-audit/Cargo.toml` + `ferro-notifications/Cargo.toml` | exact |
| `ferro-reservation/README.md` | docs | — | `ferro-audit/README.md` (inferred shape) | role-match |
| `ferro-reservation/src/lib.rs` | public API facade | — | `ferro-audit/src/lib.rs` | exact |
| `ferro-reservation/src/error.rs` | error model | — | `ferro-audit/src/error.rs` + `ferro-orm/src/error.rs` | exact |
| `ferro-reservation/src/migration.rs` | migration | CRUD | `ferro-audit/src/migration.rs` | exact |
| `ferro-reservation/src/entity.rs` | model | CRUD | `ferro-audit/src/entity.rs` | exact |
| `ferro-reservation/src/resource.rs` | trait | request-response | `ferro-events/src/traits.rs` (async-trait shape) | role-match |
| `ferro-reservation/src/handle.rs` | value object | — | `ferro-audit/src/target.rs` (plain struct + serde) | role-match |
| `ferro-reservation/src/context.rs` | value object + builder | — | `ferro-audit/src/actor.rs` + `ferro-audit/src/entry.rs` (builder) | role-match |
| `ferro-reservation/src/event.rs` | event + trait impl | event-driven | `ferro-events/src/traits.rs` | exact |
| `ferro-reservation/src/kernel.rs` | service / orchestrator | request-response | `ferro-orm/src/guarded.rs` (builder + exec pattern) | role-match |
| `ferro-reservation/src/sweeper.rs` | batch job primitive | batch | `ferro-orm/src/guarded.rs` (exec_at_most_one pattern) | partial-match |
| `ferro-reservation/tests/concurrent_hold.rs` | integration test | CRUD | `ferro-orm/src/guarded.rs` `#[cfg(test)]` block | role-match |
| `ferro-reservation/tests/proptest_properties.rs` | property test | batch | `ferro-orm/src/guarded.rs` tests (tokio + SQLite harness) | role-match |
| `ferro-reservation/tests/integration_with_audit_and_events.rs` | integration test | event-driven | `ferro-audit/src/entry.rs` `#[cfg(test)]` inline Migrator | exact |
| `docs/src/database/reservations.md` | docs | — | inferred from `docs/src/database/audit-log.md` shape | role-match |
| root `Cargo.toml` members | workspace integration | — | existing `[workspace]` members list | exact |
| `CLAUDE.md` Workspace Structure table | docs | — | existing CLAUDE.md table rows | exact |
| `.github/workflows/publish.yml` WAVE1B_CRATES | config | — | existing `WAVE1B_CRATES` line | exact |
| `docs/src/SUMMARY.md` nav entry | docs | — | existing Database section entries | exact |

---

## Pattern Assignments

### `ferro-reservation/Cargo.toml` (config)

**Analogs:** `ferro-audit/Cargo.toml` (lines 1–25) and `ferro-notifications/Cargo.toml` (lines 1–35)

**Metadata block** — copy from `ferro-audit/Cargo.toml` lines 1–11, change name/description/keywords:
```toml
[package]
name = "ferro-reservation"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Generic hold/commit/release resource reservation kernel for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["reservation", "booking", "sea-orm", "concurrency", "ferro"]
categories = ["database", "asynchronous"]
readme = "README.md"
homepage = "https://ferro-rs.dev"
```

**Dependencies block** — extend `ferro-audit/Cargo.toml` lines 13–20 with `async-trait` and internal ferro-* path deps (pattern from `ferro-notifications/Cargo.toml` lines 28–30):
```toml
[dependencies]
sea-orm = "1.0"
sea-orm-migration = "1.0"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
async-trait = "0.1"
ferro-orm    = { path = "../ferro-orm",    version = "0.2" }
ferro-events = { path = "../ferro-events", version = "0.2" }
ferro-audit  = { path = "../ferro-audit",  version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
```

**Key notes:**
- No `[features]` block needed (unlike ferro-notifications which has `integration-tests`).
- `homepage` field is present in ferro-orm and ferro-audit; ferro-notifications omits it — include it (match the most recent sibling pair).

---

### `ferro-reservation/src/lib.rs` (public API facade)

**Analog:** `ferro-audit/src/lib.rs` (all 76 lines)

**Module declaration and pub-use facade** — mirror the exact shape of `ferro-audit/src/lib.rs` lines 54–76:
```rust
mod context;
mod entity;
mod error;
mod event;
mod handle;
mod kernel;
mod migration;
mod resource;
mod sweeper;

pub use context::ReservationContext;
pub use entity::{
    ActiveModel as ReservationActiveModel,
    Entity as ReservationEntity,
    Model as ReservationModel,
};
pub use error::ReservationError;
pub use event::{ReleaseReason, ReservationEvent};
pub use handle::ReservationHandle;
pub use kernel::ReservationKernel;
pub use migration::Migration as CreateReservationsTable;
pub use resource::Resource;
pub use sweeper::SweepReport;

// Re-export AuditActor so consumers building ReservationContext
// don't need a direct ferro-audit dependency for the common case.
pub use ferro_audit::AuditActor;
```

**Rustdoc tone** — mirror `ferro-orm/src/lib.rs` lines 1–36 and `ferro-audit/src/lib.rs` lines 1–52:
- Lead with *why* (the broken hand-rolled `read → check → write` pattern).
- One complete `rust,ignore` code example covering the full lifecycle: `hold → commit | release`.
- ASCII state diagram (four-node: `held → committed | released | expired`, sweeper arc from `held → expired`).
- Separate sections for the audit-emission guarantee and the event-bus best-effort semantics.
- Migration registration snippet (mirrors `ferro-audit/src/lib.rs` lines 40–52).

---

### `ferro-reservation/src/error.rs` (error model)

**Analogs:** `ferro-audit/src/error.rs` (lines 1–59) and `ferro-orm/src/error.rs` (lines 1–71)

**File-level doc comment** — copy `ferro-orm/src/error.rs` lines 1–5 pattern, changing the display prefix:
```rust
//! `ReservationError` — the single error type for the ferro-reservation crate.
//!
//! Every variant's `Display` impl prefixes `"reservation: …"` so production
//! log greps stay surgical (matches `"guarded: …"`, `"audit: …"`, `"config: …"`).
```

**Enum structure** — extend `ferro-audit/src/error.rs` lines 10–27 with domain variants:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ReservationError {
    #[error("reservation: insufficient capacity (requested {requested}, available {available} of {capacity})")]
    Insufficient { requested: u32, available: u32, capacity: u32 },

    #[error("reservation: id={id} not in expected state '{expected}'")]
    ConflictingState { id: Uuid, expected: &'static str },

    #[error("reservation: id={id} not found")]
    NotFound { id: Uuid },

    #[error("reservation: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("reservation: guarded update error: {0}")]
    Guarded(#[from] ferro_orm::GuardedError),

    #[error("reservation: audit error: {0}")]
    Audit(#[from] ferro_audit::AuditError),

    #[error("reservation: json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
```

**Tests block** — copy `ferro-orm/src/error.rs` lines 38–71 pattern: one `#[test]` per `Display` contract, one `From<X>` round-trip test per `#[from]` variant.

**Critical note on `Guarded` variant:** `#[from]` fires for `EmptyUpdate` and `TooManyRows` (programming bugs) but NOT for `NoRowsAffected` in kernel state-transition methods — those are caught before `?` and mapped to `ConflictingState`. The `#[from]` derive still applies to all variants; the explicit `map_err` in the kernel precedes `?` to override it for `NoRowsAffected`.

---

### `ferro-reservation/src/migration.rs` (migration)

**Analog:** `ferro-audit/src/migration.rs` (all 177 lines)

**Exact structural copy** — the entire file shape is identical; only the table name, column set, and index names differ.

**File-level doc comment** (mirror lines 1–14):
```rust
//! `CreateReservationsTable` — SeaORM migration that creates the `reservations`
//! table and its two composite indexes.
//!
//! Consumers register this migration in their own `Migrator`:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_audit::CreateAuditLogTable),
//!             Box::new(ferro_reservation::CreateReservationsTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```
```

**Migration struct + trait impl** (mirror lines 16–84):
```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Reservations::Table)
                .if_not_exists()
                .col(ColumnDef::new(Reservations::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Reservations::ResourceKind).string().not_null())
                .col(ColumnDef::new(Reservations::ResourceKey).json().not_null())
                .col(ColumnDef::new(Reservations::Window).json().null())
                .col(ColumnDef::new(Reservations::Quantity).integer().not_null())
                .col(ColumnDef::new(Reservations::Status).string().not_null())
                .col(ColumnDef::new(Reservations::ExpiresAt).timestamp().not_null())
                .col(
                    ColumnDef::new(Reservations::HeldAt)
                        .timestamp()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(ColumnDef::new(Reservations::CommittedAt).timestamp().null())
                .col(ColumnDef::new(Reservations::ReleasedAt).timestamp().null())
                .col(ColumnDef::new(Reservations::ReleaseReason).string().null())
                .col(ColumnDef::new(Reservations::TenantId).string().null())
                .to_owned(),
        ).await?;

        manager.create_index(Index::create()
            .name("idx_reservations_kind_key_window_status")
            .table(Reservations::Table)
            .col(Reservations::ResourceKind)
            .col(Reservations::ResourceKey)
            .col(Reservations::Window)
            .col(Reservations::Status)
            .to_owned()).await?;

        manager.create_index(Index::create()
            .name("idx_reservations_status_expires")
            .table(Reservations::Table)
            .col(Reservations::Status)
            .col(Reservations::ExpiresAt)
            .to_owned()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reservations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Reservations {
    Table, Id, ResourceKind, ResourceKey, Window,
    Quantity, Status, ExpiresAt, HeldAt, CommittedAt,
    ReleasedAt, ReleaseReason, TenantId,
}
```

**Tests block** (mirror `ferro-audit/src/migration.rs` lines 103–176): `TestMigrator` wrapping only `crate::migration::Migration`, SQLite `sqlite_master` assertions for the table and both indexes, and a `down()` verification. Test: `migration_creates_table_and_indexes`.

---

### `ferro-reservation/src/entity.rs` (model)

**Analog:** `ferro-audit/src/entity.rs` (all 67 lines)

**File-level doc** (mirror lines 1–6):
```rust
//! SeaORM `Entity` / `Model` / `ActiveModel` / `Column` / `Relation` for the
//! `reservations` table.
//!
//! Schema authority is `migration.rs` (`CreateReservationsTable`). This module's
//! `Model` shape must match the migration's column declarations exactly.
```

**Model struct** (mirror lines 14–66 with reservation columns):
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reservations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub resource_kind: String,
    pub resource_key: JsonValue,          // NOT NULL
    pub window: Option<JsonValue>,        // NULL when Window = ()
    pub quantity: i32,                    // i32 in DB; u32 at API boundary
    pub status: String,                   // "held" | "committed" | "released" | "expired"
    pub expires_at: DateTime,             // NOT NULL
    pub held_at: DateTime,               // NOT NULL, DB default CURRENT_TIMESTAMP
    pub committed_at: Option<DateTime>,   // NULL until commit
    pub released_at: Option<DateTime>,    // NULL until release
    pub release_reason: Option<String>,   // NULL until release
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**Key deviations from ferro-audit's entity:** `expires_at` is `DateTime` not `Option<DateTime>` (NOT NULL per schema D-39); `held_at` is `DateTime` not `Option<DateTime>` (DB default); `quantity` is `i32` (SeaORM INTEGER → i32; kernel casts to `u32` at API boundary); two additional timestamp columns (`committed_at`, `released_at`) following the same `Option<DateTime>` pattern as `before`/`after` in audit.

---

### `ferro-reservation/src/resource.rs` (trait)

**Analog:** `ferro-events/src/traits.rs` (lines 1–34 for the `async_trait` pattern)

**No exact codebase analog** — this is the first consumer-implemented async trait in the ferro-* library crates. The pattern for `#[async_trait]` on both definition and impl is verified from `ferro-events/src/traits.rs` and `ferro-events/src/lib.rs`.

**Trait definition shape:**
```rust
use async_trait::async_trait;
use sea_orm::ConnectionTrait;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;

use crate::error::ReservationError;

/// Consumer-implemented capacity model.
///
/// `Key` identifies a resource instance; `Window` scopes capacity to a
/// time range, seat category, or any other dimension. Use `Window = ()` for
/// non-windowed resources.
///
/// # Example
/// ```rust,ignore
/// #[async_trait::async_trait]
/// impl Resource for InventoryUnitResource {
///     type Key = ProductId;
///     type Window = ();
///     const KIND: &'static str = "inventory.unit";
///     // ...
/// }
/// ```
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
    type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;

    const KIND: &'static str;

    async fn capacity<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;

    async fn held<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;
}
```

**Tests block:** inline unit test with a `TestResource { capacity: u32 }` struct implementing `Resource` with `Window = ()` and `Key = String`. Verify `capacity()` and `held()` return the expected values against an in-memory SQLite connection. Both `#[async_trait]` annotations must appear (definition + impl).

---

### `ferro-reservation/src/handle.rs` (value object)

**Analog:** `ferro-audit/src/target.rs` (all 68 lines — plain struct, serde, constructor)

**Shape:**
```rust
/// Opaque token returned by `ReservationKernel::hold`.
///
/// Carries the persisted row's `id` plus a full snapshot of the hold-time
/// fields. Pass the handle to `commit`, `release`, or `extend`.
///
/// The `id` is the only field used as a primary key in subsequent calls;
/// the rest is reference data. The struct is `Serialize + Deserialize` so
/// callers can embed it in Stripe payment intent metadata, a queued-job
/// payload, or any other side-channel.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReservationHandle {
    pub id: Uuid,
    pub resource_kind: String,
    pub resource_key: serde_json::Value,
    pub window: Option<serde_json::Value>,
    pub quantity: u32,
    pub held_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub tenant_id: Option<String>,
}
```

**Tests block:** `handle_serde` — JSON round-trip via `serde_json::to_string` / `from_str`. Mirror the test style in `ferro-audit/src/target.rs` lines 30–67 (multiple small `#[test]` functions, no async).

---

### `ferro-reservation/src/context.rs` (value object + builder)

**Analog:** `ferro-audit/src/entry.rs` lines 59–120 (builder pattern) and `ferro-audit/src/actor.rs` (actor mapping)

**Consuming `AuditActor` directly** — `ReservationContext::actor` is `ferro_audit::AuditActor`. The constructors map to the actor variants:

```rust
pub struct ReservationContext {
    pub actor: ferro_audit::AuditActor,
    pub correlation_id: Option<uuid::Uuid>,
    pub tenant_id: Option<String>,
    pub reason: Option<String>,
}

impl ReservationContext {
    pub fn system() -> Self {
        Self { actor: ferro_audit::AuditActor::System, correlation_id: None,
               tenant_id: None, reason: None }
    }
    pub fn user(user_id: impl Into<String>) -> Self {
        Self { actor: ferro_audit::AuditActor::User(user_id.into()), .. Self::system() }
    }
    pub fn job(name: impl Into<String>) -> Self {
        Self { actor: ferro_audit::AuditActor::Job(name.into()), .. Self::system() }
    }
    pub fn anonymous() -> Self {
        Self { actor: ferro_audit::AuditActor::Anonymous, .. Self::system() }
    }
    // Builder methods — consuming self → Self (workspace convention)
    pub fn with_correlation(mut self, id: uuid::Uuid) -> Self { ... }
    pub fn with_tenant(mut self, t: impl Into<String>) -> Self { ... }
    pub fn with_reason(mut self, r: impl Into<String>) -> Self { ... }
}
```

**Builder method convention:** `mut self` consumed, returns `Self` — matches `ferro-audit/src/entry.rs` lines 76–120 (every setter takes `mut self`).

**Tests block:** `context_builder` — verify that `ReservationContext::user("u_1").with_correlation(id).with_tenant("t_1")` correctly populates all fields. No async. Mirror `ferro-audit/src/actor.rs` lines 52–90 style.

---

### `ferro-reservation/src/event.rs` (event + trait impl)

**Analog:** `ferro-events/src/traits.rs` lines 1–34 (Event trait); workspace serde conventions for the enum

**`ReservationEvent` definition:**
```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ReservationEvent {
    Held {
        id: uuid::Uuid,
        resource_kind: String,
        resource_key: serde_json::Value,
        window: Option<serde_json::Value>,
        quantity: u32,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    Committed { id: uuid::Uuid, resource_kind: String, resource_key: serde_json::Value },
    Released  { id: uuid::Uuid, resource_kind: String, resource_key: serde_json::Value,
                reason: ReleaseReason },
    Expired   { id: uuid::Uuid, resource_kind: String, resource_key: serde_json::Value },
}

impl ferro_events::Event for ReservationEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Held { .. }      => "ReservationHeld",
            Self::Committed { .. } => "ReservationCommitted",
            Self::Released { .. }  => "ReservationReleased",
            Self::Expired { .. }   => "ReservationExpired",
        }
    }
}
```

**`ReleaseReason` definition** (same file):
```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum ReleaseReason {
    UserCancelled,
    PaymentFailed,
    AdminOverride,
    Other(String),
}
```

**`Event` trait requirements** (from `ferro-events/src/traits.rs` line 34):
- `ReservationEvent` must be `Clone + Send + Sync + 'static`.
- `serde_json::Value`, `uuid::Uuid`, `chrono::DateTime<Utc>`, `ReleaseReason` are all `Clone + Send + Sync`.
- No `#[async_trait]` needed — `Event::name` is a sync method.

---

### `ferro-reservation/src/kernel.rs` (service / orchestrator)

**Analog:** `ferro-orm/src/guarded.rs` (lines 9–100 for the `GuardedUpdate` builder + exec pattern) and `ferro-audit/src/entry.rs` lines 125–185 (builder write + re-fetch pattern)

**Struct definition:**
```rust
pub struct ReservationKernel<R: Resource> {
    db: sea_orm::DatabaseConnection,
    resource: R,
}

impl<R: Resource> ReservationKernel<R> {
    pub fn new(db: sea_orm::DatabaseConnection, resource: R) -> Self {
        Self { db, resource }
    }
}
```

**`hold` method — GuardedUpdate call site pattern** (from `ferro-orm/src/guarded.rs` lines 60–68 + RESEARCH.md Pattern 1):

The INSERT path does not use `GuardedUpdate` — it uses SeaORM's `ActiveModel::insert`. The state-transition methods (`commit`, `release`, `extend`) all use the pattern below.

**`commit` state-transition core** (copy this pattern verbatim for every state-transition method):
```rust
// Map NoRowsAffected → ConflictingState BEFORE the ? operator (D-46)
GuardedUpdate::new(reservations::Entity)
    .filter(reservations::Column::Id.eq(handle.id))
    .filter(reservations::Column::Status.eq("held"))
    .set_value(
        reservations::Column::Status,
        Value::String(Some(Box::new("committed".to_string()))),
    )
    .set_value(
        reservations::Column::CommittedAt,
        Value::ChronoDateTimeUtc(Some(Box::new(chrono::Utc::now()))),
    )
    .exec_one(conn)
    .await
    .map_err(|e| match e {
        GuardedError::NoRowsAffected => ReservationError::ConflictingState {
            id: handle.id,
            expected: "held",
        },
        other => ReservationError::Guarded(other),
    })?;
```

**Audit emission pattern** (from `ferro-audit/src/entry.rs` lines 45–57 and RESEARCH.md Pattern 2 + Pitfall 4):
```rust
// Only call .correlation() / .tenant() when Some — empty string ≠ NULL
let mut builder = AuditEntry::record("reservation.committed")
    .actor(ctx.actor.clone())
    .target(AuditTarget::new("reservation", handle.id.to_string()))
    .before(json!({ "status": "held",      "quantity": handle.quantity }))
    .after( json!({ "status": "committed" }));
if let Some(cid) = ctx.correlation_id { builder = builder.correlation(cid); }
if let Some(tid) = &ctx.tenant_id     { builder = builder.tenant(tid); }
builder.write(conn).await.map_err(ReservationError::Audit)?;
```

**Event dispatch pattern** (from RESEARCH.md Pattern 3 + D-26):
```rust
// D-26: event dispatch failure is operational visibility, NOT a correctness error
if let Err(e) = ferro_events::dispatch(ReservationEvent::Committed {
    id: handle.id,
    resource_kind: handle.resource_kind.clone(),
    resource_key: handle.resource_key.clone(),
}).await {
    tracing::warn!(
        reservation_id = %handle.id,
        error = %e,
        "event dispatch failed after reservation.committed — state is committed"
    );
}
```

**Ordering within each method (invariant):** GuardedUpdate first → AuditEntry second → ferro_events::dispatch third. If audit fails, return `ReservationError::Audit` but state is already committed (D-30). If dispatch fails, log warn and return `Ok(())` (D-26).

---

### `ferro-reservation/src/sweeper.rs` (batch job primitive)

**Analog:** `ferro-orm/src/guarded.rs` lines 71–84 (`exec_at_most_one` — the sweeper uses this variant, not `exec_one`)

**`SweepReport` struct:**
```rust
pub struct SweepReport {
    pub expired_count: u32,
    pub scanned_at: chrono::DateTime<chrono::Utc>,
}
```

**`run_sweep_once` core pattern** (from RESEARCH.md Special Area 3):
```rust
// Uses self.db (owned DatabaseConnection) — no caller-supplied conn for sweeper
let expired_rows = reservations::Entity::find()
    .filter(reservations::Column::Status.eq("held"))
    .filter(reservations::Column::ExpiresAt.lt(chrono::Utc::now().naive_utc()))
    .limit(500)
    .all(&self.db)
    .await
    .map_err(ReservationError::Db)?;

for row in &expired_rows {
    let updated = GuardedUpdate::new(reservations::Entity)
        .filter(reservations::Column::Id.eq(row.id))
        .filter(reservations::Column::Status.eq("held"))
        .set_value(reservations::Column::Status,
                   Value::String(Some(Box::new("expired".to_string()))))
        .exec_at_most_one(&self.db)   // 0 rows = concurrent sweeper won; normal
        .await;

    match updated {
        Ok(true)  => { /* emit event + audit (AuditActor::System, D-23) */ }
        Ok(false) => { /* concurrent sweeper won; skip silently per D-24 */ }
        Err(e)    => { tracing::warn!(error = %e, "sweeper guarded update db error"); }
    }
}
```

**Audit in sweeper** (D-23): `AuditActor::System`, action `"reservation.expired"`, target `AuditTarget::new("reservation", row.id.to_string())`. Use `AuditEntry::record(...)` directly (no `ReservationContext` in sweeper — sweeper uses `AuditActor::System` unconditionally).

---

### `ferro-reservation/tests/concurrent_hold.rs` (integration test)

**Analog:** `ferro-orm/src/guarded.rs` lines 130–145 (inline `fresh_db` + `insert_row` harness) and `ferro-audit/src/entry.rs` lines 195–210 (TestMigrator + `fresh_db`)

**Test harness** — inline `TestMigrator` + `fresh_db`:
```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(ferro_reservation::migration::Migration)]
        // or vec![Box::new(ferro_reservation::CreateReservationsTable)]
    }
}

async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = sea_orm::Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```

**Concurrent hold test** (D-48): 20 tokio tasks, capacity=5, quantity=1 each. Assert exactly 5 `Ok(_)` results and 15 `Err(ReservationError::Insufficient { .. })`. Verify `SELECT COUNT(*) WHERE status='held'` = 5 afterward.

A minimal `TestResource` struct implementing `Resource` is defined inline in the test file (or a shared `test_helpers` module). `TestResource::capacity` returns the constant capacity; `TestResource::held` queries the `reservations` table directly via SeaORM.

---

### `ferro-reservation/tests/integration_with_audit_and_events.rs` (cross-crate integration test)

**Analog:** `ferro-audit/src/entry.rs` lines 195–210 (TestMigrator) — combined two-migration variant

**TestMigrator with both tables** (from RESEARCH.md Pattern 5 + Area 7):
```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_audit::CreateAuditLogTable),     // audit_log first
            Box::new(ferro_reservation::CreateReservationsTable), // reservations second
        ]
    }
}
```

**Test assertions** (D-50): after `hold` + `commit`:
1. Two `ReservationEvent` variants dispatched (`Held`, `Committed`) — verified via atomic counters in registered listeners.
2. `history_for_target(AuditTarget::new("reservation", id.to_string()), &conn)` returns two entries with matching `correlation_id`.
3. `reconstruct_state(&history)` returns `{"status": "committed"}` as the final state.

---

### `ferro-reservation/tests/proptest_properties.rs` (property tests)

**Analog:** `ferro-orm/src/guarded.rs` tests (tokio + SQLite harness); `proptest` is new to the workspace

**proptest + tokio pattern** (RESEARCH.md Special Area 5 — no native tokio support in `proptest!`):
```rust
proptest! {
    #[test]
    fn capacity_invariant_under_concurrent_holds(
        capacity in 1u32..=20u32,
        n_tasks in 1usize..=20usize,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            // fresh_db(), spin n_tasks holds, assert SUM(held) <= capacity
            prop_assert!(successes <= capacity as usize);
        });
    }
}
```

**Both properties** (D-49-P1 and D-49-P2) live in this file. Property 2 (state-machine validity via audit replay) uses `ferro_audit::history_for_target` + `ferro_audit::reconstruct_state` — requires `TestMigrator` with both tables.

---

## Shared Patterns

### GuardedUpdate state-transition pattern
**Source:** `ferro-orm/src/guarded.rs` lines 41–47 (`set_value`), lines 62–68 (`exec_one`)
**Apply to:** Every state-transition method in `kernel.rs` (`commit`, `release`, `extend`)

Critical invariant: call `map_err(|e| match e { GuardedError::NoRowsAffected => ConflictingState{..}, other => Guarded(other) })?` — never use bare `?` on a GuardedUpdate call in a transition method. `exec_at_most_one` is used ONLY in the sweeper.

### `Value::String` + `Value::ChronoDateTimeUtc` in `set_value`
**Source:** `ferro-orm/src/guarded.rs` lines 295–303 (multi_column_set_atomic test)
**Apply to:** Every `GuardedUpdate::set_value` call in `kernel.rs` and `sweeper.rs`

```rust
// String column:
Value::String(Some(Box::new("committed".to_string())))

// DateTime<Utc> column (ASSUMPTION A4 in RESEARCH.md — verify on first compile):
Value::ChronoDateTimeUtc(Some(Box::new(chrono::Utc::now())))
```

### Audit emission pattern with conditional optional fields
**Source:** `ferro-audit/src/entry.rs` lines 76–120 (builder setters) and RESEARCH.md Pitfall 4
**Apply to:** Every successful state transition in `kernel.rs` and every row expired in `sweeper.rs`

Never call `.correlation(id)` or `.tenant(t)` unless the `Option` is `Some`. Empty string is not NULL.

### Event dispatch after state commit (non-propagating)
**Source:** `ferro-events/src/dispatcher.rs` lines 265–267 (`dispatch` function)
**Apply to:** Every successful state transition in `kernel.rs` and `sweeper.rs`

`ferro_events::dispatch(event).await` returns `Result<(), ferro_events::Error>`. The `if let Err(e) = ...` + `tracing::warn!` pattern does NOT propagate the error. State is committed before this call; dispatch failure is operational visibility only (D-26).

### Error enum shape
**Source:** `ferro-orm/src/error.rs` lines 1–35 and `ferro-audit/src/error.rs` lines 1–27
**Apply to:** `error.rs`

`"reservation: …"` display prefix on every variant. One `#[from]` per transitive dependency error type. Domain-specific variants (`Insufficient`, `ConflictingState`, `NotFound`) before the `#[from]` variants.

### `with_*` consuming builder methods
**Source:** `ferro-audit/src/entry.rs` lines 76–120
**Apply to:** `ReservationContext` builder methods

All setters take `mut self` and return `Self`. No returning `&mut Self`. Matches workspace builder convention documented in CLAUDE.md.

### In-memory SQLite test harness (inline TestMigrator)
**Source:** `ferro-audit/src/entry.rs` lines 195–210 and `ferro-audit/src/migration.rs` lines 109–125
**Apply to:** All three test files (`concurrent_hold.rs`, `proptest_properties.rs`, `integration_with_audit_and_events.rs`)

`Database::connect("sqlite::memory:")` + inline `TestMigrator` + `MigratorTrait::up`. Do NOT depend on `framework` crate for the harness.

### `#[serde(rename_all = "snake_case", tag = "kind")]` on event enums
**Source:** workspace serde conventions (CLAUDE.md); applied in `ferro-audit/src/actor.rs` (snake_case kind strings)
**Apply to:** `ReservationEvent` and `ReleaseReason` enums in `event.rs`

---

## Files With No Close Codebase Analog

These files have a role in the project but no direct analog in the current codebase. The planner uses RESEARCH.md patterns instead of codebase excerpts.

| File | Role | Reason |
|------|------|--------|
| `ferro-reservation/README.md` | docs | No ferro-audit README.md to read; infer structure from ferro-audit/src/lib.rs rustdoc opening paragraphs and CONTEXT.md D-53 |
| `docs/src/database/reservations.md` | user docs | Analog docs page exists conceptually (`atomic-updates.md`, `audit-log.md`) but were not read; CONTEXT.md D-54 prescribes the full outline |

---

## Workspace Integration Files

These are single-line or table-row additions with exact patterns from the existing files.

### root `Cargo.toml` — `[workspace]` members
Add `"ferro-reservation"` to the members array adjacent to `"ferro-orm"` and `"ferro-audit"`.

### `CLAUDE.md` — Workspace Structure table
Add a row:
```
| `ferro-reservation` | Generic hold/commit/release reservation kernel | `src/lib.rs` |
```
Place it after the `ferro-audit` row.

### `.github/workflows/publish.yml` — `WAVE1B_CRATES`
Add `ferro-reservation` to the `WAVE1B_CRATES` environment variable alongside `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`. Exact format: comma-separated string matching the existing entries.

### `docs/src/SUMMARY.md` — Database section
Add `- [Reservations](database/reservations.md)` after the `audit-log.md` entry in the Database section.

### `CHANGELOG.md` — new section at top
New section titled `## ferro-reservation` (new crate), placed at the top per Phase 152 D-25 convention. Contents as specified in CONTEXT.md D-58.

---

## Metadata

**Analog search scope:** `ferro-orm/src/`, `ferro-audit/src/`, `ferro-events/src/`, `ferro-notifications/Cargo.toml`, `ferro-orm/Cargo.toml`
**Files scanned:** 14 source files + 3 Cargo.toml files
**Pattern extraction date:** 2026-05-13

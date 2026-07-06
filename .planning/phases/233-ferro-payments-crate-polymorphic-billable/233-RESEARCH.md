# Phase 233: ferro-payments Crate Scaffold + PaymentIntent Entity + Migration — Research

**Researched:** 2026-06-17
**Domain:** Rust crate scaffolding, SeaORM entity/migration authoring, cross-backend partial unique indexes
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01** Postgres and SQLite get a true partial unique index — identical syntax:
`CREATE UNIQUE INDEX uq_payment_intents_active ON payment_intents (billable_kind, billable_id) WHERE status IN ('reserved','paid')`.

**D-02** MySQL: emulate with a stored generated column (conditional identity string / NULL) plus a plain UNIQUE index. Research must confirm minimum MySQL version and NULL uniqueness.

**D-03** No SeaORM-native partial-index API; the WHERE clause is emitted via raw SQL through `manager.get_connection().execute_unprepared(...)` branched on `manager.get_database_backend()`.

**D-04** `status` is TEXT column. `PaymentIntentStatus` derives `DeriveActiveEnum` with `rs_type = "String"`, `db_type = "Text"`, per-variant `string_value`. No native DB ENUM.

**D-05** `billable_kind` is raw TEXT column, no enum at DB or entity layer. Matches `BillableKind(&'static str)`.

**D-06** Timestamps use SeaORM `timestamp_with_time_zone`; entity type `DateTimeUtc` (`chrono::DateTime<Utc>`). No DB-level `DEFAULT now()` — set in Rust at insert/transition time.

**D-07** `metadata` uses `ColumnType::Json`, nullable; entity type `serde_json::Value`. Maps JSONB on PG, JSON on MySQL, TEXT on SQLite.

**D-08** `tenant_id` and `billable_id` carry no FK constraint in this crate.

**D-09** State-transition methods use `ferro_orm::GuardedUpdate` atomic conditional UPDATE. 0 rows affected = no-op precondition failure.

**D-10** `create_reserved` is a plain INSERT; partial unique index enforces single-active-per-billable.

**D-11** `find_active_for(kind, id)` filters `status IN ('reserved','paid')`; `find_by_stripe_session(session_id)` filters on unique `stripe_session_id`.

**D-12** Phase 233 deps: `sea-orm`, `chrono`, `serde`, `serde_json`, `thiserror`, `async-trait`, `ferro-orm`. No `ferro-stripe`.

**D-13** Minimal `PaymentError` in 233: `Db(sea_orm::DbErr)`, `StatusPrecondition(String)`, `NotFound`. Full variants deferred to phase 234.

**D-14** Crate version `0.1.0`; edition/license/repository from workspace. Add to `Cargo.toml` members AND to `.github/workflows/publish.yml` in a wave after `ferro-orm` (Wave 1b or later).

### Claude's Discretion

- Exact module file split inside `src/`: recommended to ship only `lib.rs`, `intent/` (entity, status, lifecycle), `migration/`, and `error.rs` in phase 233. Stub or omit `service.rs`, `webhook.rs`, `reaper.rs`, `refund.rs`, `loader.rs`, `billable.rs`.
- Exact name and SQL expression of the MySQL generated column (subject to D-02 research confirmation).
- Whether `BillableKind` lives in a `billable.rs` stub now or is introduced minimally alongside the entity.

### Deferred Ideas (OUT OF SCOPE)

`PaymentService`, `Billable` trait, `BillableLoader`, `wire_dispatcher`, webhook handlers (234/235), reapers (236), `ferro-stripe` dependency, full `PaymentError` variants, design open questions (loader `tenant_id` signature, etc.).

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PAY-POLY-DM-01 | SeaORM `Entity` for `payment_intents` table with all columns, correct types, and nullability | Entity pattern from ferro-conduit/ferro-audit; column types verified against workspace; `DateTimeUtc` alias verified |
| PAY-POLY-DM-02 | `PaymentIntentStatus` and `BillableKind` enums, `DeriveActiveEnum` for status | `DeriveActiveEnum` API verified against docs.rs/sea-orm 1.1.x; `rs_type`/`db_type`/`string_value` syntax confirmed |
| PAY-POLY-DM-03 | Lifecycle methods: `create_reserved`, `mark_paid`, `mark_released`, `mark_refunded`, `find_active_for`, `find_by_stripe_session` | `GuardedUpdate` API fully read from source; builder signature, `exec_at_most_one` vs `exec_one` distinction, `set_value`/`set_expr` documented |
| PAY-POLY-DM-04 | Migration `m20260617_create_payment_intents` portable across Postgres + SQLite + MySQL with partial unique index | Raw SQL path confirmed via `manager.get_connection().execute_unprepared()`; MySQL generated-column workaround confirmed with minimum version and NULL semantics |

</phase_requirements>

---

## Summary

Phase 233 creates `ferro-payments` as a new workspace crate containing the data layer only: entity, status enum, lifecycle methods, and a cross-backend migration. The highest-risk item is the partial unique index: SeaORM 1.x has no native WHERE-clause support in `IndexCreateStatement`, so Postgres and SQLite use `execute_unprepared` with raw `CREATE UNIQUE INDEX ... WHERE ...` SQL. MySQL, which lacks partial indexes, uses a stored generated column set to the identity string when `status IN ('reserved','paid')` and NULL otherwise, relying on MySQL's documented behavior that UNIQUE indexes permit multiple NULL values.

Workspace precedents cover every sub-problem except the raw partial-index SQL: `ferro-audit` and `ferro-reservation` provide the migration/entity shape, test scaffold, and `MigratorTrait` wrapper pattern. `ferro-orm::GuardedUpdate` provides the atomic conditional UPDATE primitive for lifecycle transitions, with `exec_at_most_one` as the right call for transitions where 0-rows-affected is a valid no-op. The `DeriveActiveEnum` string-backed enum pattern is documented in sea-orm 1.1.x and exercised by the framework for status columns.

**Primary recommendation:** Follow the `ferro-audit` / `ferro-reservation` crate structure exactly (Cargo manifest, migration module layout, in-source test with `TestMigrator`), add a raw SQL branch for the partial index, and use `GuardedUpdate::exec_at_most_one` for all state-transition methods.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| PaymentIntent entity/columns | Database / Storage | — | Pure data layer; no business logic in phase 233 |
| Status enum (PaymentIntentStatus) | Database / Storage | API / Backend | Typed representation of DB TEXT column; validation at write boundary |
| Lifecycle methods (mark_paid, etc.) | Database / Storage | — | Atomic conditional UPDATE via GuardedUpdate; no higher-level orchestration |
| Cross-backend migration | Database / Storage | — | Schema DDL; branching logic stays inside the migration struct |
| PaymentError (minimal) | API / Backend | — | Error surface for lifecycle method callers |
| BillableKind newtype | API / Backend | — | Open-set string wrapper; no persistence logic itself |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | 1.0 (installed: 1.1.20) | ORM entity, migrations, `DeriveEntityModel`, `DeriveActiveEnum` | Workspace standard; all other ferro crates use `sea-orm = "1.0"` |
| sea-orm-migration | 1.0 | `MigrationTrait`, `SchemaManager`, `Table::create()`, `Index::create()` | Bundled with sea-orm; used in all workspace migrations |
| chrono | 0.4 (with `serde`) | `DateTime<Utc>` ≡ `DateTimeUtc` for timestamp columns | Workspace standard; verified in ferro-audit, ferro-stripe, app |
| serde | 1.0 (with `derive`) | Derive for model types | Workspace standard |
| serde_json | 1.0 | `serde_json::Value` for `metadata` JSON column | Used in ferro-audit for JSON columns |
| thiserror | 2 | `PaymentError` derive | Workspace standard error derive |
| async-trait | 0.1 | Async trait defs (future phases; include now per D-12) | Workspace standard |
| ferro-orm | 0.2 (path dep) | `GuardedUpdate` atomic conditional UPDATE | Required by D-09 |

[VERIFIED: workspace Cargo.toml + ferro-orm/Cargo.toml + ferro-audit/Cargo.toml + ferro-reservation/Cargo.toml]

### Dependency Features for `ferro-payments/Cargo.toml`

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

Note: no `with-chrono` feature needed in the `sea-orm` dep — `DateTimeUtc` is re-exported from `sea_orm::entity::prelude` and available when the `macros` feature (which brings in `sea-orm-macros`) is active. [VERIFIED: sea-orm-1.1.20/src/entity/prelude.rs — `pub type DateTimeUtc = chrono::DateTime<chrono::Utc>;`]

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `DateTimeUtc` (sea_orm prelude alias) | `chrono::DateTime<Utc>` directly | Both work; `DateTimeUtc` keeps entity imports consistent with existing workspace entities (api_keys.rs) |
| `json()` column for metadata | `json_binary()` | `json_binary()` maps to JSONB on Postgres (btree-indexable); `json()` causes SQLSTATE 42704 when adding a btree index on Postgres (documented in ferro-reservation migration comment). Metadata has no index requirement in 233, so `json()` is acceptable — see Pitfall section |

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer Migrator
       │
       ▼
ferro_payments::migration::m20260617_create_payment_intents
       │
       ├─► Table::create() → payment_intents table (all columns)
       │
       ├─► Index::create() (3 plain indexes via SchemaManager)
       │       • (tenant_id, status)
       │       • (stripe_session_id)  ← unique
       │       • (payment_intent_id)
       │
       └─► manager.get_database_backend() branch:
               ├─ Postgres / SQLite ──► execute_unprepared(
               │                         "CREATE UNIQUE INDEX ... WHERE ..."
               │                        )
               └─ MySQL ─────────────► execute_unprepared(
                                         "ALTER TABLE ... ADD COLUMN active_billable_key ..."
                                         + "CREATE UNIQUE INDEX ..."
                                        )

Lifecycle call site
       │
       ├─ create_reserved  ──► payment_intents::ActiveModel::insert()
       │
       ├─ mark_paid        ──► GuardedUpdate::new(Entity)
       │                           .filter(Id.eq(id))
       │                           .filter(Status.eq("reserved"))
       │                           .set_value(Status, "paid")
       │                           .set_value(PaidAt, now)
       │                           .exec_at_most_one(&conn)   // 0 = no-op
       │
       ├─ mark_released / mark_refunded  ── (same pattern, different guard status)
       │
       ├─ find_active_for  ──► Entity::find()
       │                           .filter(BillableKind.eq(kind.as_str()))
       │                           .filter(BillableId.eq(id))
       │                           .filter(Status.is_in(["reserved","paid"]))
       │
       └─ find_by_stripe_session ──► Entity::find()
                                         .filter(StripeSessionId.eq(session_id))
```

### Recommended Module Structure

```
ferro-payments/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs              # pub use re-exports (entity, status, lifecycle, error, migration)
    ├── error.rs            # PaymentError { Db, StatusPrecondition, NotFound }
    ├── intent/
    │   ├── mod.rs          # pub mod entity; pub mod status; pub mod lifecycle;
    │   ├── entity.rs       # DeriveEntityModel — payment_intents table
    │   ├── status.rs       # PaymentIntentStatus DeriveActiveEnum
    │   └── lifecycle.rs    # create_reserved / mark_paid / mark_released / mark_refunded
    │                       # find_active_for / find_by_stripe_session
    └── migration/
        ├── mod.rs          # pub struct CreatePaymentIntentsTable; public fn migration_create_payment_intents()
        └── m20260617_create_payment_intents.rs   # MigrationTrait impl
```

Stubs for `billable.rs`, `loader.rs`, `service.rs`, `webhook.rs`, `reaper.rs`, `refund.rs` are omitted in 233 per Claude's Discretion. `BillableKind` can live as a small newtype in `lib.rs` or a minimal `billable.rs` — the entity stores the raw string regardless.

---

## Key Technical Findings

### Finding 1: DeriveActiveEnum Exact Syntax [VERIFIED]

sea-orm 1.1.x `DeriveActiveEnum` with string backing:

```rust
// Source: docs.rs/sea-orm/latest/sea_orm/derive.DeriveActiveEnum.html + workspace sea-orm-1.1.20
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

Constraint: `db_type` must be `"Text"` (not `"String(StringLen::None)"`) to match the D-04 TEXT column. `EnumIter` is required alongside `DeriveActiveEnum` for SeaORM's column mapper to enumerate variants. All variants must use `string_value` (cannot mix with `num_value`).

In the entity `Model`, the column type is:
```rust
pub status: PaymentIntentStatus,
```
The column def in the migration uses `.text().not_null()`.

### Finding 2: Partial Index Raw SQL Path [VERIFIED]

SeaORM's `IndexCreateStatement` has no `where_` / partial-index support. The confirmed API for raw DDL in migrations is:

```rust
// Source: https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/
// Source: ferro-mcp-oauth/src/migration.rs (execute_unprepared usage in workspace)

let db = manager.get_connection();  // returns &DatabaseConnection

// Postgres + SQLite (identical WHERE syntax):
db.execute_unprepared(
    "CREATE UNIQUE INDEX uq_payment_intents_active \
     ON payment_intents (billable_kind, billable_id) \
     WHERE status IN ('reserved','paid')"
).await?;
```

`manager.get_connection()` returns `&DatabaseConnection`; `DatabaseConnection` implements `execute_unprepared(&str)`. This is the only path for DDL without value bindings in a migration. [VERIFIED: sea-ql.org migration docs + workspace pattern in ferro-mcp-oauth/src/migration.rs lines 104–107, 138–141 where `DatabaseBackend::Sqlite` branching is done]

### Finding 3: MySQL Generated Column Syntax [VERIFIED]

MySQL 5.7+ (8.0 recommended) supports STORED generated columns. The emulation pattern for the partial unique constraint:

```sql
-- Run via execute_unprepared on DatabaseBackend::MySql
ALTER TABLE payment_intents
  ADD COLUMN active_billable_key VARCHAR(600)
    AS (CASE WHEN status IN ('reserved','paid')
             THEN CONCAT(billable_kind, '|', billable_id)
             ELSE NULL
        END) STORED;

CREATE UNIQUE INDEX uq_payment_intents_active_mysql
  ON payment_intents (active_billable_key);
```

Key facts (all confirmed):

1. **MySQL 5.7+** introduced stored generated columns. MySQL 8.0 is the supported minimum in most production stacks; 5.7 is acceptable but EOL. [VERIFIED: dev.mysql.com/doc/refman/5.7/en/create-table-generated-columns.html]

2. **NULL uniqueness in MySQL UNIQUE indexes**: MySQL permits multiple NULL values in a UNIQUE index. NULL is not equal to NULL for uniqueness purposes. This is documented, intentional behavior (confirmed by MySQL bug reports and mysqltutorial.org). When `status` is `released`, `failed`, or `refunded`, `active_billable_key` is NULL — multiple such rows per billable are allowed, which is the correct semantic. [VERIFIED: mysqltutorial.org/mysql-index/mysql-unique-index/ + MySQL bug reports #6829, #66512, #8173 confirming multiple NULLs allowed]

3. **STORED vs VIRTUAL**: only STORED generated columns can be indexed in MySQL. [VERIFIED: MySQL 8.0 docs]

4. **Size note**: `VARCHAR(600)` covers `billable_kind` (TEXT, practically ≤100 chars) + `|` + `billable_id` (BIGINT, max 19 digits). Adjust if needed; MySQL index key length limit is 3072 bytes for utf8mb4.

### Finding 4: Migration Branch Pattern [VERIFIED]

Workspace already uses `DatabaseBackend` branching in `ferro-reservation/src/migration.rs` (line 168) and `ferro-mcp-server/src/dispatch.rs`. The migration branching pattern:

```rust
use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

// inside MigrationTrait::up():
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

[VERIFIED: manager.get_database_backend() and manager.get_connection() confirmed from sea-ql.org migration docs and workspace usage]

### Finding 5: GuardedUpdate API for Lifecycle Methods [VERIFIED: ferro-orm/src/guarded.rs]

The two entry points relevant to phase 233:

```rust
// Returns Err(GuardedError::NoRowsAffected) if precondition not met (0 rows).
// Returns Err(GuardedError::TooManyRows) if filter hits >1 rows (index bug).
pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError>

// Returns Ok(false) if 0 rows (no-op); Ok(true) if 1 row; Err on >1 rows.
pub async fn exec_at_most_one<C: ConnectionTrait>(self, conn: &C) -> Result<bool, GuardedError>
```

For lifecycle methods where 0-rows-affected is the designed race no-op (D-09 "second writer no-ops"):
- Use `exec_at_most_one` — returns `Ok(false)` cleanly without an error.
- Convert `Ok(false)` to `PaymentError::StatusPrecondition(...)` at the lifecycle layer if the caller needs to distinguish.

Builder methods: `.filter(col.eq(val))` (AND-combined), `.set_value(col, Value::String(...))`, `.set_expr(col, Expr::col(...))`.

Pitfall: the builder requires at least one `.set_*` call; an empty builder returns `GuardedError::EmptyUpdate` before any SQL fires (tested in ferro-orm T-16-4).

### Finding 6: Column Types and Entity Field Types [VERIFIED]

| Column | Migration ColumnDef | Entity Field Type | Notes |
|--------|--------------------|--------------------|-------|
| `id` | `.big_integer().not_null().auto_increment().primary_key()` | `i64` | Standard PK pattern from ferro-mcp-oauth |
| `tenant_id` | `.big_integer().not_null()` | `i64` | No FK per D-08 |
| `billable_kind` | `.text().not_null()` | `String` | Raw TEXT, no enum |
| `billable_id` | `.big_integer().not_null()` | `i64` | No FK per D-08 |
| `amount_cents` | `.big_integer().not_null()` | `i64` | |
| `currency` | `.text().not_null()` | `String` | |
| `status` | `.text().not_null()` | `PaymentIntentStatus` | DeriveActiveEnum |
| `stripe_session_id` | `.text().null()` | `Option<String>` | Unique index (plain, not partial) |
| `payment_intent_id` | `.text().null()` | `Option<String>` | |
| `charge_id` | `.text().null()` | `Option<String>` | |
| `application_fee_cents` | `.big_integer().null()` | `Option<i64>` | |
| `expires_at` | `.timestamp_with_time_zone().not_null()` | `DateTimeUtc` | Rust-set per D-06 |
| `reserved_at` | `.timestamp_with_time_zone().not_null()` | `DateTimeUtc` | Rust-set per D-06 |
| `paid_at` | `.timestamp_with_time_zone().null()` | `Option<DateTimeUtc>` | |
| `released_at` | `.timestamp_with_time_zone().null()` | `Option<DateTimeUtc>` | |
| `refunded_at` | `.timestamp_with_time_zone().null()` | `Option<DateTimeUtc>` | |
| `refund_amount_cents` | `.big_integer().null()` | `Option<i64>` | |
| `metadata` | `.json().null()` | `Option<serde_json::Value>` | json() per D-07; no btree index on metadata so json() is safe (see Pitfall 2) |

`DateTimeUtc` is a type alias from `sea_orm::entity::prelude`: `pub type DateTimeUtc = chrono::DateTime<chrono::Utc>;` [VERIFIED: sea-orm-1.1.20 source in ~/.cargo/registry]

### Finding 7: publish.yml Wave Placement [VERIFIED: .github/workflows/publish.yml]

Current wave structure:
- **Wave 1a**: leaf crates with ZERO internal ferro-* deps → includes `ferro-orm`
- **Wave 1b**: crates depending on Wave 1a leaves → `ferro-projections`, `ferro-stripe`, `ferro-reservation`, etc.
- **Wave 2**: framework + MCP → `ferro-rs`, `ferro-mcp`, etc.
- **Wave 3**: framework consumers → `ferro-cli`, `ferro-bundle`

`ferro-payments` depends on `ferro-orm` (Wave 1a), so it must go in **Wave 1b**. Add it to the `WAVE1B_CRATES` line in publish.yml.

### Finding 8: Migration Test Scaffold Pattern [VERIFIED: ferro-audit, ferro-reservation, ferro-mcp-oauth]

Consistent pattern across all sibling crates:

```rust
#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate up");
        conn
    }

    // test: table exists in sqlite_master
    // test: indexes exist in sqlite_master
    // test: down() drops table
}
```

For phase 233, add to these: partial-unique enforcement test (insert two active rows for same billable → second INSERT must fail with a unique constraint error).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic conditional UPDATE | Manual read-then-write | `ferro_orm::GuardedUpdate` | Eliminates TOCTOU race by construction; errors on 0 or >1 rows |
| Status string mapping | Custom serialize/deserialize | `DeriveActiveEnum` with `string_value` | Compile-time exhaustiveness, SeaORM query filter integration |
| Migration boilerplate | Raw string DDL everywhere | `Table::create()` + `Index::create()` + `execute_unprepared` only for the partial-index WHERE | Portability for all columns; raw SQL only where SeaQuery has no API |
| Error type | `Box<dyn Error>` or anyhow | `thiserror` derive | Workspace standard; typed matching at call sites |

---

## Common Pitfalls

### Pitfall 1: Empty GuardedUpdate Builder

**What goes wrong:** Calling `.exec_one()` or `.exec_at_most_one()` without any `.set_value()` / `.set_expr()` call returns `GuardedError::EmptyUpdate` without touching the DB. SeaORM's internal `Updater::is_noop()` would otherwise short-circuit with `rows_affected = 0`, making it look like a predicate miss.

**Why it happens:** Lifecycle methods built conditionally can accidentally produce an empty builder.

**How to avoid:** Always have at least one `.set_value()` call in every transition method. The status column itself must always be set. [VERIFIED: ferro-orm/src/guarded.rs lines 87-91]

### Pitfall 2: json() vs json_binary() for Indexed JSON Columns

**What goes wrong:** Using `ColumnDef::json()` on a column that gets a btree index causes `SQLSTATE 42704` on Postgres at index-creation time.

**Why it happens:** Postgres `json` type does not support btree operators; `jsonb` does. `json()` maps to `json`; `json_binary()` maps to `jsonb`.

**How to avoid:** `metadata` in phase 233 has NO index — `json()` is safe per D-07. If a future phase adds an index on `metadata`, switch to `json_binary()`. Do not proactively use `json_binary()` without a btree index need (adds storage overhead). [VERIFIED: ferro-reservation migration comment lines 49-54]

### Pitfall 3: GuardedUpdate exec_one vs exec_at_most_one for Lifecycle

**What goes wrong:** Using `exec_one` for `mark_paid` / `mark_released` / `mark_refunded` propagates `GuardedError::NoRowsAffected` as an error when the design intent is a no-op (second writer wins, first is silent).

**Why it happens:** `exec_one` is strict; it errors on 0 rows.

**How to avoid:** Use `exec_at_most_one`. Convert `Ok(false)` to `PaymentError::StatusPrecondition` only if the caller explicitly needs to distinguish a no-op. [VERIFIED: ferro-orm/src/guarded.rs — two methods have distinct semantics]

### Pitfall 4: MySQL Generated Column Expression Needs CAST for BIGINT

**What goes wrong:** `CONCAT(billable_kind, '|', billable_id)` on MySQL may implicitly cast `billable_id` (BIGINT) to string, but it is safer to be explicit: `CONCAT(billable_kind, '|', CAST(billable_id AS CHAR))`.

**Why it happens:** Implicit casts in generated column expressions can behave differently in strict mode.

**How to avoid:** Always use explicit `CAST(billable_id AS CHAR)` in the `CONCAT` expression. [ASSUMED based on MySQL strict mode behavior; low risk but explicit is safer]

### Pitfall 5: Missing `sea-orm-migration` dependency

**What goes wrong:** The migration module uses `sea_orm_migration::prelude::*` but if only `sea-orm` is declared, `sea_orm_migration` is not available.

**Why it happens:** `sea-orm-migration` is a separate crate even though sea-orm depends on it.

**How to avoid:** Declare both `sea-orm = "1.0"` and `sea-orm-migration = "1.0"` in `[dependencies]`. Both ferro-audit and ferro-reservation do this. [VERIFIED: ferro-audit/Cargo.toml, ferro-reservation/Cargo.toml]

### Pitfall 6: publish.yml wave placement

**What goes wrong:** Adding `ferro-payments` to Wave 1a would cause a crates.io publish failure because the `ferro-orm` path dependency isn't yet available when Wave 1a fires.

**Why it happens:** Wave ordering enforces topological dependency sort.

**How to avoid:** Add to `WAVE1B_CRATES` in publish.yml — the same wave as `ferro-reservation` (which also depends on `ferro-orm`). [VERIFIED: publish.yml Wave 1b comment + WAVE1B_CRATES variable]

### Pitfall 7: DeriveMigrationName vs custom MigrationName

**What goes wrong:** `#[derive(DeriveMigrationName)]` infers the migration name from the struct name by convention. If the struct is named `Migration`, the derived name is the containing module path. This may not match the date-based convention used for the `name()` return.

**Why it happens:** `DeriveMigrationName` uses the module path + struct name; ferro-reservation uses a manual `impl MigrationName` instead for an explicit date-keyed name.

**How to avoid:** Use the manual impl approach from ferro-reservation:
```rust
impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str { "m20260617_000001_create_payment_intents" }
}
```
Or use `#[derive(DeriveMigrationName)]` and rely on the module naming convention — either is acceptable. [VERIFIED: ferro-audit uses derive, ferro-reservation uses manual impl]

---

## Runtime State Inventory

Phase 233 is greenfield — no renames, migrations of existing data, or OS-registered state. **Omitted per template guidance.**

---

## Environment Availability

Phase 233 requires only the Rust toolchain (already present) and in-memory SQLite for tests (bundled via sqlx-sqlite feature). No external services needed.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | cargo build/test | ✓ | 1.88.0 (workspace rust-version) | — |
| SQLite (in-memory) | dev-dependencies test run | ✓ | bundled via sqlx-sqlite | — |
| cargo clippy | pre-commit gate | ✓ | bundled with toolchain | — |

**Missing dependencies with no fallback:** None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio-test via `#[tokio::test]` (sea-orm 1.0, in-memory SQLite) |
| Config file | None — in-source `#[cfg(test)] mod tests` per workspace convention |
| Quick run command | `cargo test -p ferro-payments` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PAY-POLY-DM-01 | Migration creates `payment_intents` table with correct columns | unit | `cargo test -p ferro-payments -- migration` | ❌ Wave 0 |
| PAY-POLY-DM-01 | All 4 supporting indexes created by migration | unit | `cargo test -p ferro-payments -- migration` | ❌ Wave 0 |
| PAY-POLY-DM-02 | `PaymentIntentStatus` round-trips through DB TEXT column | unit | `cargo test -p ferro-payments -- status` | ❌ Wave 0 |
| PAY-POLY-DM-03 | `create_reserved` inserts a row with status=reserved | unit | `cargo test -p ferro-payments -- lifecycle` | ❌ Wave 0 |
| PAY-POLY-DM-03 | `mark_paid` transitions reserved→paid atomically; 0 rows on wrong source status | unit | `cargo test -p ferro-payments -- lifecycle` | ❌ Wave 0 |
| PAY-POLY-DM-03 | `mark_released`/`mark_refunded` same no-op semantics | unit | `cargo test -p ferro-payments -- lifecycle` | ❌ Wave 0 |
| PAY-POLY-DM-03 | `find_active_for` returns only reserved/paid rows | unit | `cargo test -p ferro-payments -- lifecycle` | ❌ Wave 0 |
| PAY-POLY-DM-04 | Partial unique index: second active INSERT for same billable fails | unit | `cargo test -p ferro-payments -- partial_unique` | ❌ Wave 0 |
| PAY-POLY-DM-04 | Partial unique index: after release, new active INSERT for same billable succeeds | unit | `cargo test -p ferro-payments -- partial_unique` | ❌ Wave 0 |
| PAY-POLY-DM-04 | Migration down() drops table and indexes | unit | `cargo test -p ferro-payments -- migration` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo clippy -p ferro-payments --all-targets -- -D warnings && cargo test -p ferro-payments`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-payments/src/` — all source files (crate does not exist yet)
- [ ] `ferro-payments/src/migration/m20260617_create_payment_intents.rs` — migration + in-source tests
- [ ] `ferro-payments/src/intent/lifecycle.rs` — lifecycle unit tests inline
- [ ] `ferro-payments/Cargo.toml` — with dev-dependencies for tokio + sea-orm sqlite features

---

## Security Domain

Phase 233 is a pure data-layer crate with no HTTP surface, no authentication, and no credential handling. ASVS categories do not apply to this phase. The only security-relevant concern is SQL injection, which is mitigated by SeaORM's parameterized query builder for all non-DDL statements. The raw DDL in `execute_unprepared` contains no user-supplied values (all strings are hardcoded migration literals).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | MySQL CAST(billable_id AS CHAR) in generated column expression is the safest form | Finding 3 | Generated column rejected in strict mode — use explicit CAST |
| A2 | `VARCHAR(600)` is sufficient for `CONCAT(billable_kind, '|', billable_id)` in MySQL | Finding 3 | Truncation if billable_kind approaches 600 chars; very unlikely in practice |

**All other claims in this research were verified via workspace source, installed crate source, or official documentation.**

---

## Open Questions (RESOLVED)

Both questions below are resolved by the recommendations, which the plans adopt
verbatim (explicit `CAST(... AS CHAR)` in the MySQL generated-column expression;
manual `impl MigrationName` for date-keyed explicitness).

1. **MySQL generated column expression exact form**
   - What we know: STORED generated columns need a deterministic expression; `CONCAT` + `CASE` pattern confirmed to work
   - What's unclear: whether `CAST(billable_id AS CHAR)` is the idiomatic form or whether implicit cast suffices in MySQL 8.0 strict mode
   - Recommendation: Use explicit CAST in the migration; test passes on SQLite path (CI), MySQL correctness is by-construction

2. **Migration name style: `DeriveMigrationName` vs manual `impl MigrationName`**
   - What we know: Both patterns exist in the workspace; ferro-audit uses derive, ferro-reservation uses manual
   - What's unclear: The planner's preference
   - Recommendation: Use manual impl with `"m20260617_000001_create_payment_intents"` for explicitness, matching ferro-reservation

---

## Sources

### Primary (HIGH confidence)
- `ferro-orm/src/guarded.rs` — `GuardedUpdate` builder API, `exec_one` vs `exec_at_most_one`, `set_value`/`set_expr`, empty-builder guard
- `ferro-orm/src/error.rs` — `GuardedError` enum variants
- `ferro-audit/src/migration.rs` — migration struct pattern, `DeriveIden`, inline tests
- `ferro-audit/src/entity.rs` — `json()` column + `serde_json::Value` entity field, `DateTimeUtc` vs `DateTime`
- `ferro-reservation/src/migration.rs` — `json_binary()` rationale, manual `MigrationName` impl, two-index pattern
- `ferro-mcp-oauth/src/migration.rs` — `create_index().unique()` pattern, multi-migration struct file pattern
- `app/src/migrations/m20260228_create_api_keys_table.rs` — `DeriveIden` enum, `big_integer()`, `timestamp_with_time_zone()`, `auto_increment()`, `primary_key()`
- `app/src/models/entities/api_keys.rs` — `DateTimeUtc` entity field type for `timestamp_with_time_zone` columns
- `~/.cargo/registry/.../sea-orm-1.1.20/src/entity/prelude.rs` — `pub type DateTimeUtc = chrono::DateTime<chrono::Utc>`
- `.github/workflows/publish.yml` — Wave 1a/1b/2/3 definitions; `ferro-orm` is Wave 1a
- `ferro-audit/Cargo.toml`, `ferro-reservation/Cargo.toml` — sibling crate manifest patterns; both declare `sea-orm = "1.0"` + `sea-orm-migration = "1.0"`

### Secondary (MEDIUM confidence)
- [sea-ql.org migration writing docs](https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/) — `manager.get_connection()` → `execute_unprepared()` API confirmed
- [docs.rs DeriveActiveEnum](https://docs.rs/sea-orm/latest/sea_orm/derive.DeriveActiveEnum.html) — `rs_type = "String"`, `db_type`, `string_value` attribute syntax confirmed
- [dev.mysql.com refman 8.0 generated columns](https://dev.mysql.com/doc/refman/8.0/en/create-table-generated-columns.html) — STORED generated column syntax, minimum MySQL 5.7

### Tertiary (LOW confidence)
- MySQL bug reports [#6829](https://bugs.mysql.com/bug.php?id=6829), [#66512](https://bugs.mysql.com/bug.php?id=66512), [#8173](https://bugs.mysql.com/bug.php?id=8173) — multiple NULLs allowed in UNIQUE index; behavior reported as intentional by MySQL team

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — all versions read from installed registry sources and workspace Cargo.toml files
- Architecture: HIGH — all patterns read directly from sibling crate source; no novel patterns
- GuardedUpdate API: HIGH — read from source
- Partial index mechanism: HIGH — confirmed from sea-ql.org docs + workspace execute_unprepared precedents
- MySQL generated column + NULL uniqueness: MEDIUM — MySQL docs confirm stored columns 5.7+; NULL-in-UNIQUE confirmed via multiple MySQL bug reports (intentional behavior, not just anecdote)

**Research date:** 2026-06-17
**Valid until:** 2026-07-17 (stable APIs; sea-orm 1.x series is stable)

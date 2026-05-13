# Phase 153: ferro-audit — Research

**Researched:** 2026-05-13
**Domain:** Rust crate scaffolding / SeaORM migration + entity / structured audit log
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Ship as top-level `ferro-audit/` crate. No merge into `framework`.
- **D-02:** Thin and additive at v0. No subsumption of tracing, logging, or observability.
- **D-03:** No internal ferro-* dependencies. Depends on `sea-orm` and `sea-orm-migration` directly, NOT on `ferro-orm`.
- **D-04:** Wave 1a publish. External deps: `sea-orm` (1.0), `sea-orm-migration` (1.0), `thiserror` (2), `serde` + `serde_json`, `uuid` (with `serde` + `v4`), `chrono` (with `serde`).
- **D-05:** `AuditActor` enum: `User(String)`, `System`, `Job(String)`, `ApiClient(String)`, `Anonymous`. DB representation: `(actor_kind VARCHAR, actor_id VARCHAR NULL)`. `actor_kind` is the snake_case variant name.
- **D-06:** No current-actor pickup from request. Caller passes `AuditActor` explicitly.
- **D-07:** `AuditTarget` struct with `kind: String` and `id: String`. Constructor `AuditTarget::new(kind, id)`. `From<(impl Into<String>, impl ToString)>` tuple impl.
- **D-08:** Dotted-namespace convention for `action` and `target.kind`. Not enforced at compile time.
- **D-09:** Builder API `AuditEntry::record(action).actor(…).target(…).before(…).after(…).reason(…).correlation(…).tenant(…).write(&conn).await?`. No macro façade in v0.
- **D-10:** `action` is the only required field. `actor` defaults to `AuditActor::System`. Missing `target` writes successfully with `tracing::warn!`; does NOT error.
- **D-11:** `before` and `after` are `Option<serde_json::Value>`.
- **D-12:** `correlation_id` is `Option<Uuid>`. Caller-supplied.
- **D-13:** `tenant_id` is `Option<String>`.
- **D-14:** `async fn write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError>`. Returns persisted entry with generated `id: Uuid` and DB-stamped `created_at`.
- **D-15:** `AuditError` enum: `MissingAction`, `Db(#[from] sea_orm::DbErr)`, `Json(#[from] serde_json::Error)`. Display prefix `"audit: …"`.
- **D-16:** Missing target is NOT an error. Missing action IS.
- **D-17:** JSON serialization errors propagate as `AuditError::Json`.
- **D-18:** `pub use migration::Migration as CreateAuditLogTable;` at crate root. Consumers add it to their `Migrator`.
- **D-19:** Schema: `id (UUID PK) | tenant_id (VARCHAR NULL) | actor_kind (VARCHAR NOT NULL) | actor_id (VARCHAR NULL) | action (VARCHAR NOT NULL) | target_kind (VARCHAR NULL) | target_id (VARCHAR NULL) | before (JSON NULL) | after (JSON NULL) | reason (VARCHAR NULL) | correlation_id (UUID NULL) | created_at (TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)`.
- **D-20:** Indexes: `idx_audit_target (tenant_id, target_kind, target_id, created_at)` and `idx_audit_actor (tenant_id, actor_kind, actor_id, created_at)`.
- **D-21:** `id` is `Uuid`, client-generated UUIDv4 at `write()` time.
- **D-22:** `created_at` set by DB (`CURRENT_TIMESTAMP` default) — no app-clock assignment.
- **D-23:** Query helpers: `history_for_target`, `recent_by_actor`, `recent`. No pagination in v0.
- **D-24:** `reconstruct_state(entries: &[AuditEntry]) -> Option<serde_json::Value>`. Shallow JSON object merge. Pure function.
- **D-25:** No streaming / pagination. Public `Entity` re-export for SeaORM-native queries.
- **D-26:** `prune_older_than(cutoff: DateTime<Utc>, conn: &C) -> Result<u64, AuditError>`.
- **D-27:** No auto-enforcement. Document 1–3 year recommendation.
- **D-28:** No concurrency contract beyond atomic single-row INSERT.
- **D-29:** No deduplication.
- **D-30:** Unit tests in `#[cfg(test)] mod tests` covering 9 named scenarios.
- **D-31:** One integration test `tests/replay_round_trip.rs`.
- **D-32:** No property-based tests (Phase 154 carries that budget).
- **D-33:** No Postgres CI tests.
- **D-34:** SQLite in-memory test harness re-derived inline (no `framework` dep).
- **D-35:** Module rustdoc on `lib.rs` with canonical example.
- **D-36:** User doc page `docs/src/database/audit-log.md`.
- **D-37:** No new MCP tools.
- **D-38:** Workspace version bump — see Version Drift note below.
- **D-39:** Add `ferro-audit` to Wave 1a in `.github/workflows/publish.yml`.
- **D-40:** CHANGELOG entry under `ferro-audit`.

### Claude's Discretion

- Internal module layout of `ferro-audit/src/` (likely `lib.rs` + `actor.rs` + `target.rs` + `entry.rs` + `error.rs` + `migration.rs`, but planner is free to consolidate)
- Whether SeaORM `Entity` / `Model` / `ActiveModel` types live in `entity` submodule or in `entry.rs`
- Exact JSON-merge implementation in `reconstruct_state`
- Whether `pub use migration::Migration as CreateAuditLogTable` is at crate root or via `pub mod migration { … }`
- Exact rustdoc prose and code-block formatting
- Test file names within `ferro-audit/tests/`

### Deferred Ideas (OUT OF SCOPE)

- `audit_log!` macro façade
- Automatic `correlation_id` pickup from `tracing` span / task-local
- `from_request(&Request) -> AuditActor`
- Ferro-events `AuditEntryRecorded` emission on every write
- MCP tools to query the audit log
- Distributed audit-stream / log shipping
- Postgres CI integration tests
- Property-based tests
- PII redaction / GDPR right-to-erasure tooling
- Deep-merge `reconstruct_state` variant
- Pagination helpers on query API

</user_constraints>

---

## Summary

`ferro-audit` is a new Wave 1a leaf crate — greenfield, no existing code in the workspace. The entire public surface is specified in CONTEXT.md (D-01..D-40); research confirms every technical premise and fills in the Cargo.toml feature flags, SeaORM entity patterns, JSON/UUID cross-dialect behavior, test harness shape, and publish-workflow wiring.

The structural template is `ferro-orm` (Phase 152): same crate scaffolding convention, same error-naming discipline (`"audit: …"` mirrors `"guarded: …"`), same Wave 1a Cargo.toml shape, same `framework`-independent inline test harness. The departure point is that `ferro-audit` adds a SeaORM migration (which `ferro-orm` does not) and a full entity definition for `audit_log` (which `ferro-orm` does not need, since it only mutates consumer entities).

Key confirmed facts: SeaORM's `json()` column type maps to TEXT in SQLite and `json` in Postgres — `serde_json::Value` round-trips identically on both backends without consumer-side configuration. The `uuid()` column type maps to TEXT (`uuid_text`) in SQLite and native `uuid` in Postgres — `uuid::Uuid::new_v4()` requires the `uuid = { version = "1", features = ["v4"] }` dep (the workspace uses `uuid = { version = "1", features = ["v4"] }` in `framework/`; `ferro-audit` declares it independently at the same version pin). The workspace `sea-orm` resolves to `1.1.19`; `sea-orm-migration` resolves to `1.1.19`. `chrono` with `serde` feature is sufficient for `DateTime<Utc>` storage; the workspace already uses `chrono = { version = "0.4", features = ["serde"] }` in both `framework/` and `ferro-wallet/`.

**Version drift alert:** CONTEXT D-38 references a `0.2.25 → 0.2.26` bump. The workspace `Cargo.toml` currently reads `version = "0.2.30"`. The CONTEXT was written before Phase 152 executed its own version bump. The planner must derive the actual next version from `Cargo.toml` at execution time, not from the static D-38 text. As of this research the current version is `0.2.30`; the bump for Phase 153 will be to `0.2.31`.

**Primary recommendation:** Mirror `ferro-orm`'s crate scaffold exactly; the only new structural element is the SeaORM migration module and the entity definition. Keep the entity in `entity.rs` (separate from `entry.rs`) to keep the file small and the SeaORM derive-heavy code isolated.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Audit log write (`AuditEntry::record(…).write()`) | Library crate (`ferro-audit`) | Consumer application | ferro-audit is pure Rust library; consumer app owns the call site and passes the connection |
| Schema / migration | Library crate (`ferro-audit`) | Consumer application's `Migrator` | ferro-audit ships the migration struct; consumer wires it into their migrator ordering |
| Query helpers | Library crate (`ferro-audit`) | — | `history_for_target`, `recent_by_actor`, `recent` live inside the crate |
| Replay reconstruction | Library crate (`ferro-audit`) | — | `reconstruct_state` is a pure function, no DB call; lives in the crate |
| Retention enforcement | Consumer application | — | `prune_older_than` helper is in the crate; scheduling is caller-driven (`ferro-queue` cron job in the app) |
| Actor identity resolution | Consumer application | — | ferro-audit takes `AuditActor` as a value; no request context, no DI |
| Correlation ID population | Consumer application | — | Caller-supplied `Option<Uuid>`; no framework-level plumbing in v0 |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm` | 1.0 (resolves to 1.1.19 in lock) | ORM backend for `audit_log` entity + queries | Already workspace dep; Wave 1a requires direct dep, not via ferro-orm |
| `sea-orm-migration` | 1.0 (resolves to 1.1.19 in lock) | Ships `CreateAuditLogTable` migration struct for consumers | Same version as framework; standard migration crate |
| `thiserror` | 2 | `AuditError` derive | Workspace standard; `ferro-orm`, `ferro-wallet`, every leaf crate uses thiserror 2 |
| `serde` | 1 | Derive + `serde_json::Value` serialization | Workspace standard |
| `serde_json` | 1 | `before` / `after` JSON column type | Workspace standard; required for `serde_json::Value` |
| `uuid` | 1 (with `v4` + `serde` features) | `id` and `correlation_id` UUIDv4 generation | Framework uses `uuid = { version = "1", features = ["v4"] }` |
| `chrono` | 0.4 (with `serde` feature) | `created_at: DateTime<Utc>` column type | Framework and ferro-wallet both use `chrono = { version = "0.4", features = ["serde"] }` |

### Supporting (dev-dependencies only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tokio` | 1 (features: `["full"]`) | Async test runtime | All `#[tokio::test]` unit and integration tests |
| `sea-orm` (dev) | 1.0 (features: `["sqlx-sqlite", "runtime-tokio-native-tls", "macros"]`) | In-memory SQLite for tests; `Schema::new()` for table creation | Required for test harness; NOT in `[dependencies]` |

**Version verification:** [VERIFIED: Cargo.lock] `sea-orm` = `1.1.19`, `sea-orm-migration` = `1.1.19`, `uuid` = `1.19.0`.

**Installation (Cargo.toml for `ferro-audit`):**

```toml
[package]
name = "ferro-audit"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Append-only structured before/after audit log for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["audit", "sea-orm", "database", "history", "ferro"]
categories = ["database"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
sea-orm = "1.0"
sea-orm-migration = "1.0"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```

Note: `tracing` is required for the `tracing::warn!` call when `target` is missing at `write()` time (D-10). `tracing` is a zero-cost dep when no subscriber is attached. [VERIFIED: codebase grep — `tracing` already in workspace via `sea-orm`'s own dep graph]

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer App Call Site
    │
    │  AuditEntry::record("inventory.stock.adjust")
    │      .actor(AuditActor::User(user_id))
    │      .target(AuditTarget::new("inventory.unit", id))
    │      .before(json!({ "quantity": old }))
    │      .after(json!({ "quantity": new }))
    │      .write(&conn)
    │      .await?
    ▼
ferro-audit Builder (AuditEntryBuilder)
    │  • Validates: action is non-empty → else AuditError::MissingAction
    │  • Warns: target is None → tracing::warn!
    │  • Generates: id = Uuid::new_v4()
    │  • Serializes: before/after to serde_json::Value (already Value; no-op)
    │  • Does NOT set created_at (DB default handles it)
    ▼
SeaORM INSERT (audit_log::ActiveModel)
    │  • Single INSERT INTO audit_log (…) VALUES (…)
    │  • created_at assigned by DB CURRENT_TIMESTAMP default
    ▼
Database (SQLite dev / Postgres prod)
    │  audit_log table with idx_audit_target + idx_audit_actor
    ▼
Return: AuditEntry (populated with id + created_at from DB re-read or insert response)

─────────────────────────────────────────────────────────

Query Path:
    Consumer Call Site
        │  AuditEntry::history_for_target(&target, &conn).await?
        │  AuditEntry::recent_by_actor(&actor, &conn, limit).await?
        │  AuditEntry::recent(&conn, limit).await?
        ▼
    SeaORM SELECT (audit_log::Entity)
        │  • Hits idx_audit_target or idx_audit_actor
        ▼
    Vec<AuditEntry> → reconstruct_state(&entries) → Option<serde_json::Value>
```

### Recommended Project Structure

```
ferro-audit/
├── Cargo.toml               # wave-1a leaf, no internal ferro-* deps
├── README.md                # one-paragraph crate summary
├── src/
│   ├── lib.rs               # module-level rustdoc + pub use re-exports
│   ├── actor.rs             # AuditActor enum + DB serialization helpers
│   ├── target.rs            # AuditTarget struct + From impl
│   ├── entry.rs             # AuditEntryBuilder, write(), query helpers, reconstruct_state
│   ├── entity.rs            # SeaORM Model / ActiveModel / Column / Entity for audit_log
│   ├── error.rs             # AuditError thiserror derive
│   └── migration.rs         # MigrationTrait impl (the CreateAuditLogTable struct)
└── tests/
    └── replay_round_trip.rs # D-31 integration test
```

### Pattern 1: SeaORM Entity Definition (DeriveEntityModel)

The `ferro-orm` concurrent_decrement test shows the exact pattern. For `ferro-audit`, the entity uses UUID primary key, nullable JSON, and nullable UUID columns.

```rust
// Source: ferro-orm/tests/concurrent_decrement.rs + SeaORM docs
// ferro-audit/src/entity.rs

use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Option<String>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub before: Option<JsonValue>,
    pub after: Option<JsonValue>,
    pub reason: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub created_at: DateTime,          // chrono::NaiveDateTime per SeaORM column type mapping
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

Note: SeaORM's `DateTime` type alias maps to `chrono::NaiveDateTime` (wall clock, no tz) in entity models. The `created_at` column is DB-stamped via `DEFAULT CURRENT_TIMESTAMP`; the application does not set it. After INSERT, the builder re-fetches the row or reads the returned ActiveModel to populate `created_at` in the returned `AuditEntry`.

### Pattern 2: SeaORM Migration (MigrationTrait)

```rust
// Source: app/src/migrations/m20260228_create_api_keys_table.rs
// ferro-audit/src/migration.rs

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLog::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuditLog::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuditLog::TenantId).string().null())
                    .col(ColumnDef::new(AuditLog::ActorKind).string().not_null())
                    .col(ColumnDef::new(AuditLog::ActorId).string().null())
                    .col(ColumnDef::new(AuditLog::Action).string().not_null())
                    .col(ColumnDef::new(AuditLog::TargetKind).string().null())
                    .col(ColumnDef::new(AuditLog::TargetId).string().null())
                    .col(ColumnDef::new(AuditLog::Before).json().null())
                    .col(ColumnDef::new(AuditLog::After).json().null())
                    .col(ColumnDef::new(AuditLog::Reason).string().null())
                    .col(ColumnDef::new(AuditLog::CorrelationId).uuid().null())
                    .col(
                        ColumnDef::new(AuditLog::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_target")
                    .table(AuditLog::Table)
                    .col(AuditLog::TenantId)
                    .col(AuditLog::TargetKind)
                    .col(AuditLog::TargetId)
                    .col(AuditLog::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_actor")
                    .table(AuditLog::Table)
                    .col(AuditLog::TenantId)
                    .col(AuditLog::ActorKind)
                    .col(AuditLog::ActorId)
                    .col(AuditLog::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLog {
    Table,
    Id,
    TenantId,
    ActorKind,
    ActorId,
    Action,
    TargetKind,
    TargetId,
    Before,
    After,
    Reason,
    CorrelationId,
    CreatedAt,
}
```

`DeriveMigrationName` derives the migration name from the struct's file path (e.g. `migration.rs` → `m{timestamp}_migration` or a descriptive name from the source path). Because `ferro-audit` ships only one migration and the struct is named `Migration`, the migration name in the `schema_migrations` table will be derived automatically. No manual timestamped filename is required (unlike the `m20251208_…` style in app migrations — that style is for CLI-generated migrations). [VERIFIED: app/src/migrations/mod.rs pattern — library crates can use any struct name + `DeriveMigrationName`]

### Pattern 3: In-Memory SQLite Test Harness (no framework dep)

```rust
// Source: ferro-orm/src/guarded.rs tests + concurrent_decrement.rs

async fn fresh_db() -> sea_orm::DatabaseConnection {
    use sea_orm::{Database, DatabaseBackend, Schema, ConnectionTrait};

    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");

    // Run the ferro-audit migration against the in-memory DB
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;
    #[async_trait::async_trait]
    impl sea_orm_migration::MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }
    TestMigrator::up(&conn, None).await.expect("run migration");
    conn
}
```

This is the lightest variant — runs the actual `CreateAuditLogTable` migration, so the test exercises the migration SQL. No `framework` dep, no `TestContainer`, no `DbConnection` wrapper. [VERIFIED: ferro-orm/src/guarded.rs `fresh_db()` function]

### Pattern 4: `reconstruct_state` Shallow Merge

Shallow JSON object merge: iterate `entries` in order, for each entry with a non-None `after`, merge its keys into a running `serde_json::Map`. Keys present in newer `after` overwrite older ones; absent keys from older entries are preserved. Non-object `after` values (arrays, primitives) replace the state wholesale.

```rust
// ferro-audit/src/entry.rs

pub fn reconstruct_state(entries: &[AuditEntry]) -> Option<serde_json::Value> {
    use serde_json::{Map, Value};

    let mut state: Map<String, Value> = Map::new();
    let mut seen_any = false;

    for entry in entries {
        if let Some(Value::Object(after_map)) = &entry.after {
            for (k, v) in after_map {
                state.insert(k.clone(), v.clone());
            }
            seen_any = true;
        } else if let Some(v) = &entry.after {
            // Non-object after: replace state wholesale
            return Some(v.clone());
        }
    }

    if seen_any { Some(Value::Object(state)) } else { None }
}
```

### Anti-Patterns to Avoid

- **`created_at` assigned by application clock:** The `created_at` column uses `DEFAULT CURRENT_TIMESTAMP` and must NOT be set in the `ActiveModel` at insert time. This ensures ordering correctness across multiple app servers.
- **`before` / `after` set to `json!("null")` instead of `None`:** Use `Option<serde_json::Value>` with `None` for absent JSON. `json!("null")` produces `Value::String("null")`, not SQL NULL.
- **Using `json_binary()` column type instead of `json()`:** `json_binary()` maps to `jsonb` on Postgres and `jsonb` on SQLite (as of SQLite 3.45.0). For v0 and the test suite target (SQLite 3.x), use `json()` which maps to TEXT in SQLite and `json` in Postgres — both have identical round-trip behavior for `serde_json::Value`. [VERIFIED: sea-query docs + SeaORM column type docs]
- **Storing `AuditActor` as a single serialized string:** The schema uses two columns (`actor_kind`, `actor_id`) for query-ability. Do not serialize the enum to a single JSON field.
- **`Uuid::new_v4()` without the `v4` feature:** Will fail to compile. The `uuid` dep in `ferro-audit` must include `features = ["v4", "serde"]`. [VERIFIED: framework/Cargo.toml uses `uuid = { version = "1", features = ["v4"] }`]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON column storage + round-trip | Custom `BLOB` serialization | SeaORM `ColumnType::Json` + `serde_json::Value` | Cross-dialect (SQLite TEXT / Postgres json) handled by sea-query; `serde_json::Value` is the native Rust type |
| UUID generation | Custom random bytes | `uuid::Uuid::new_v4()` | RFC 4122 compliance, no collision risk, already in workspace dep graph |
| Migration schema management | Raw `CREATE TABLE` SQL in tests | SeaORM migration via `MigratorTrait::up()` | Tests exercise the real migration path; no schema drift between tests and production |
| DateTime UTC storage | Storing Unix timestamp integers | SeaORM `timestamp()` + `chrono::NaiveDateTime` | SeaORM handles SQLite TEXT serialization; no custom parsing needed |
| Composite index creation | Post-migration raw SQL | `Index::create().name(…).col(…).col(…)` via `SchemaManager` | Cross-dialect index DDL handled by sea-query |

**Key insight:** `ferro-audit` is a data-centric crate; every "custom" approach for JSON / UUID / DateTime storage introduces a dialect-specific foot-gun. The SeaORM + sea-query layer is precisely designed to abstract these away.

---

## Common Pitfalls

### Pitfall 1: `created_at` Returned as Default After INSERT

**What goes wrong:** SeaORM's `ActiveModel::insert()` returns the `ActiveModel` after INSERT, but on SQLite with `DEFAULT CURRENT_TIMESTAMP`, the `created_at` column may come back as `NotSet` or a zero value because SQLite doesn't return the server-generated default in the INSERT response.

**Why it happens:** SeaORM's SQLite driver doesn't execute a `RETURNING created_at` clause; it uses `last_insert_rowid()` to re-fetch. UUID primary keys with `auto_increment = false` can break this re-fetch.

**How to avoid:** After `insert()`, explicitly re-fetch the row by `id` to populate `created_at`:
```rust
let model = audit_log::Entity::find_by_id(new_id)
    .one(conn)
    .await?
    .ok_or(AuditError::Db(DbErr::RecordNotFound("audit_log".to_string())))?;
```
Or use `insert_and_return()` if available for the backend. The re-fetch pattern is safer and cross-dialect.

**Warning signs:** Tests show `created_at` as `0001-01-01T00:00:00` or a Unix epoch value.

### Pitfall 2: `DeriveMigrationName` and Migration Name Collisions

**What goes wrong:** If two crates both ship a migration struct named `Migration` (without a timestamped prefix), the `sea_migrations` table entry name may collide or mislead.

**Why it happens:** `DeriveMigrationName` uses the Rust module path. Since `ferro-audit`'s migration lives in `ferro_audit::migration::Migration`, the derived name will be `migration` — which is distinct from any app migration names (which use `m20251208_…` prefixes). No collision with app migrations.

**How to avoid:** Leave `DeriveMigrationName` as-is for `ferro-audit`. The derived name will be the snake_case module path component (`migration`), which is unique when combined with the crate namespace. [ASSUMED — based on sea-orm-migration `DeriveMigrationName` behavior; pattern verified in app migrations]

**Warning signs:** `MigratorTrait::up()` returns `DbErr::Migration("already applied")` when the name unexpectedly matches.

### Pitfall 3: JSON `Option<serde_json::Value>` vs SQL NULL

**What goes wrong:** SeaORM's `ActiveModel` uses `ActiveValue::Set(None::<serde_json::Value>)` to write SQL NULL to a `json` column. If the field is set to `ActiveValue::Set(Value::Null)` (the JSON null value, not Rust `None`), the column stores the string `"null"` instead of SQL NULL.

**Why it happens:** `serde_json::Value::Null` serializes as the JSON string `"null"`, not as an absent value.

**How to avoid:** Use `ActiveValue::Set(None::<serde_json::Value>)` when the audit entry has no `before` or `after`. Do not pass `json!(null)` — that produces `Value::Null`. [VERIFIED: sea-orm docs on nullable column handling + SeaQL/sea-orm issues]

**Warning signs:** `history_for_target` returns entries where `before` / `after` are `Some(Value::Null)` instead of `None`.

### Pitfall 4: UUID Column Type in SQLite Stored as TEXT

**What goes wrong:** SeaORM's `.uuid()` column maps to `uuid_text` in SQLite — stored as a VARCHAR text representation (e.g., `"550e8400-e29b-41d4-a716-446655440000"`). A filter like `.filter(Column::Id.eq(my_uuid))` must pass the UUID as a string value, not as bytes.

**Why it happens:** SQLite has no native UUID type; sea-query uses TEXT storage. SeaORM handles this automatically for entity queries, but hand-rolled `sea_query::Value::Uuid(…)` in tests can trip up if the wrong variant is used.

**How to avoid:** Use SeaORM entity queries (`.find_by_id(uuid)`) rather than raw sea-query `Value::Uuid`. SeaORM handles the TEXT conversion automatically. [VERIFIED: sea-query docs — `Uuid` ColumnType maps to `uuid_text` in SQLite]

**Warning signs:** `find_by_id(uuid)` returns `None` even though the row was inserted.

### Pitfall 5: `async_trait` Required for `MigrationTrait`

**What goes wrong:** `impl MigrationTrait for Migration` requires `#[async_trait::async_trait]` on the impl block. Missing this attribute produces a confusing compiler error about async functions in trait impls.

**Why it happens:** `sea-orm-migration` uses the `async_trait` crate for the `MigrationTrait` async methods. `async_trait` is a dev-dependency of `sea-orm-migration` itself, so it's transitively available, but the attribute must be applied explicitly.

**How to avoid:** Always annotate `impl MigrationTrait` with `#[async_trait::async_trait]`. Pattern copied verbatim from `app/src/migrations/m20260228_create_api_keys_table.rs`. [VERIFIED: app migration files]

---

## Code Examples

### AuditActor DB Serialization

```rust
// ferro-audit/src/actor.rs

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditActor {
    User(String),
    System,
    Job(String),
    ApiClient(String),
    Anonymous,
}

impl AuditActor {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::System => "system",
            Self::Job(_) => "job",
            Self::ApiClient(_) => "api_client",
            Self::Anonymous => "anonymous",
        }
    }

    pub fn id(&self) -> Option<&str> {
        match self {
            Self::User(id) | Self::Job(id) | Self::ApiClient(id) => Some(id.as_str()),
            Self::System | Self::Anonymous => None,
        }
    }
}
```

### Consumer Integration (what D-18 enables)

```rust
// In consumer app's src/migrations/mod.rs

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ferro_audit::CreateAuditLogTable),   // ferro-audit's migration
            Box::new(m20260101_create_inventory_table::Migration),
        ]
    }
}
```

### lib.rs Public Surface

```rust
// ferro-audit/src/lib.rs — final public API shape

mod actor;
mod entry;
mod entity;
mod error;
mod migration;
mod target;

pub use actor::AuditActor;
pub use entry::{AuditEntry, reconstruct_state};
pub use error::AuditError;
pub use migration::Migration as CreateAuditLogTable;
pub use target::AuditTarget;

// Entity re-export for SeaORM-native consumer queries (D-25)
pub use entity::Entity as AuditLogEntity;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `sea-orm-migration` with only `#[async_trait]` from `async-trait` crate | `async_trait::async_trait` attribute from the re-exported crate via `sea_orm_migration::prelude::*` | sea-orm 1.0 | `use sea_orm_migration::prelude::*` pulls in the attribute; no separate `async-trait` dep needed |
| `ColumnDef.json_binary()` for cross-dialect JSON | `ColumnDef.json()` for TEXT-stored JSON (compatible with both SQLite and Postgres `json` type) | sea-orm 1.0 | `json()` is the safe cross-dialect choice; `json_binary()` produces JSONB on Postgres and SQLite 3.45+ JSONB on-disk format |
| Manual UUID string conversion in migrations | `ColumnDef.uuid()` maps to `uuid_text` in SQLite automatically | sea-query | SeaORM/sea-query handles the TEXT ↔ Uuid round-trip; no manual `to_string()` or parsing in entity code |

**Deprecated / outdated:**
- `thiserror = "1.0"`: framework uses `thiserror = "1.0"`, but `ferro-orm` and `ferro-wallet` use `thiserror = "2"`. `ferro-audit` should use `thiserror = "2"` (current leaf-crate standard). [VERIFIED: ferro-orm/Cargo.toml and ferro-wallet/Cargo.toml]

---

## Runtime State Inventory

> `ferro-audit` is a greenfield crate. No rename/refactor scope. Omitted per execution-flow rule.

---

## Environment Availability

> This phase creates a new Rust crate within an existing workspace. External dependencies are the Rust toolchain and Cargo registry. No external services, databases, or CLIs beyond the build toolchain are required during development. SQLite in-memory is used for tests (no daemon).

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | 1.88.0 (workspace `rust-version`) | — |
| Cargo registry (crates.io) | New dep resolution | ✓ | — | — |
| sqlite3 (via SQLx) | Test suite | ✓ | Bundled in `sqlx` feature `sqlx-sqlite` | — |
| Personal crates.io token | First-publish bootstrap | Must be confirmed by operator | — | Cannot publish new crate without it |

**Missing dependencies with no fallback:**
- Personal crates.io publish token for first-bootstrap (CI token is publish-update only). This is a human-action checkpoint in the release plan, not a blocker for development or CI testing.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio::test` + inline `#[cfg(test)]` modules |
| Config file | none — cargo's built-in test runner |
| Quick run command | `cargo test -p ferro-audit` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

All test requirements come from CONTEXT.md (D-30 + D-31). No external REQUIREMENTS.md IDs apply.

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-30-1 | Builder happy path: `write()` returns persisted entry with non-nil `id` and `created_at` | unit | `cargo test -p ferro-audit happy_path` | Wave 0 |
| D-30-2 | Missing `action` → `AuditError::MissingAction` | unit | `cargo test -p ferro-audit missing_action` | Wave 0 |
| D-30-3 | Missing `target` writes successfully; `tracing::warn!` emitted | unit | `cargo test -p ferro-audit missing_target_writes` | Wave 0 |
| D-30-4 | `before` / `after` JSON round-trip | unit | `cargo test -p ferro-audit json_roundtrip` | Wave 0 |
| D-30-5 | `AuditActor::System` / `Anonymous` persist `actor_id = NULL` | unit | `cargo test -p ferro-audit actor_null_id` | Wave 0 |
| D-30-6 | `history_for_target` ordering (`created_at ASC`) | unit | `cargo test -p ferro-audit history_ordering` | Wave 0 |
| D-30-7 | `recent_by_actor` ordering (`DESC`) + `limit` | unit | `cargo test -p ferro-audit recent_by_actor` | Wave 0 |
| D-30-8 | `prune_older_than` returns count + deletes only old rows | unit | `cargo test -p ferro-audit prune_older_than` | Wave 0 |
| D-30-9 | `reconstruct_state` on empty → `None`; on sequence → correct object | unit | `cargo test -p ferro-audit reconstruct_state` | Wave 0 |
| D-31 | Full lifecycle: insert sequence → `history_for_target` → `reconstruct_state` equals expected | integration | `cargo test -p ferro-audit --test replay_round_trip` | Wave 0 |

### Sampling Rate

- Per task commit: `cargo test -p ferro-audit`
- Per wave merge: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- Phase gate: full suite green before verification

### Wave 0 Gaps

- [ ] `ferro-audit/src/` — entire crate (no source files exist yet)
- [ ] `ferro-audit/tests/replay_round_trip.rs` — D-31 integration test
- [ ] `ferro-audit/Cargo.toml` — new crate manifest
- [ ] `ferro-audit/README.md` — crate README

*(All gaps are expected for a greenfield crate phase. Wave 0 = crate scaffold plan.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | ferro-audit writes whatever the caller passes; access enforcement is the caller's responsibility |
| V5 Input Validation | yes (partial) | `action` non-empty check; `before`/`after` are caller-supplied `serde_json::Value` — no schema validation in v0 |
| V6 Cryptography | no | `Uuid::new_v4()` uses `getrandom`-backed CSPRNG; no hand-rolled crypto |

### Known Threat Patterns for Audit Log Crates

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Audit log tampering (UPDATE / DELETE of existing rows) | Tampering | Append-only design; no `UPDATE` or `DELETE` exposed in the public API (only `prune_older_than` which is explicit + caller-driven) |
| PII injection into `before` / `after` JSON | Information Disclosure | Caller responsibility; document that GDPR-sensitive data must be redacted before passing to builder |
| Unbounded log growth | Denial of Service | `prune_older_than` helper; document recommended retention policy |
| Missing `action` string causing uninterpretable rows | Repudiation | Validated at write time → `AuditError::MissingAction` |

---

## Key Research Findings (Numbered)

### F-01: `sea-orm` version 1.1.19, `sea-orm-migration` 1.1.19 (not 1.0.x)
[VERIFIED: Cargo.lock] Both packages resolve to 1.1.19. Cargo.toml should pin `"1.0"` (minor-compatible); the lock file handles the actual version. No action needed.

### F-02: SeaORM `json()` column — correct cross-dialect choice
[VERIFIED: SeaORM column-types docs + sea-query docs] `ColumnDef::json()` maps to:
- SQLite: `json_text` (stored as TEXT, transparently parsed by sea-query)
- Postgres: `json` type
Both backends read back as `serde_json::Value` without consumer configuration. `json_binary()` maps to JSONB on Postgres and SQLite 3.45+ JSONB on-disk — avoid for v0 (wider dialect compatibility with `json()`).

### F-03: `uuid()` column — TEXT in SQLite, native in Postgres
[VERIFIED: SeaORM column-types docs] SeaORM entity queries handle TEXT ↔ `uuid::Uuid` conversion automatically in both dialects. Callers use `uuid::Uuid` everywhere; no manual stringification.

### F-04: `chrono::NaiveDateTime` for SeaORM `DateTime` entity field
[VERIFIED: SeaORM column-types docs] `ColumnType::DateTime` → Rust `chrono::NaiveDateTime` (no tz). The `created_at` column is set by the DB (`DEFAULT CURRENT_TIMESTAMP`); the entity stores `NaiveDateTime`. If tz-aware storage is needed later, switch to `DateTimeWithTimeZone` → `DateTime<FixedOffset>`, but v0 uses naive datetime (consistent with app migrations using `.timestamp()`).

### F-05: `sea-orm` feature flags in `[dependencies]` vs `[dev-dependencies]`
[VERIFIED: ferro-orm/Cargo.toml] `sea-orm = "1.0"` in `[dependencies]` (no feature flags — SeaORM's default features include `chrono`, `json`, `uuid` transitively). `sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }` in `[dev-dependencies]` to enable the SQLite backend for tests. This pattern mirrors `ferro-orm/Cargo.toml` exactly.

### F-06: `tracing` is already in the workspace dep graph via `sea-orm`
[VERIFIED: Cargo.lock] `tracing` is a dep of `sea-orm` (listed in the lock file). `ferro-audit` should declare `tracing = "0.1"` explicitly in `[dependencies]` for the `tracing::warn!` call — relying on transitive availability is fragile and would cause clippy warnings about unused imports if sea-orm's dep graph changes.

### F-07: Version is `0.2.30`, not `0.2.25` as D-38 states
[VERIFIED: Cargo.toml + git log] D-38 references a stale pre-Phase-152-execution version. The current workspace version is `0.2.30`. The Phase 153 bump will be to `0.2.31`. Planner must not use the D-38 version number literally.

### F-08: Wave 1a `WAVE1A_CRATES` string to update in publish.yml
[VERIFIED: .github/workflows/publish.yml line 201] Current string:
```
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm"
```
Add `ferro-audit` to this string. Position: end of the list (after `ferro-orm`), or alphabetically between `ferro-broadcast` and `ferro-cache`. Order does not matter for Wave 1a (no internal deps between these crates).

### F-09: `docs/SUMMARY.md` nav entry pattern
[VERIFIED: docs/src/SUMMARY.md] Current database section:
```markdown
- [Database](features/database.md)
- [Atomic Updates](database/atomic-updates.md)
```
Add below `atomic-updates.md`:
```markdown
- [Audit Log](database/audit-log.md)
```

### F-10: Workspace members list — `ferro-audit` not yet present
[VERIFIED: Cargo.toml `[workspace.members]`] The crate is not yet in the workspace. Wave 0 plan must add `"ferro-audit"` to `[workspace.members]`.

### F-11: `CLAUDE.md` workspace structure table — needs `ferro-audit` row
[VERIFIED: CLAUDE.md] The table lists every crate. Phase 153 must add a row for `ferro-audit` with purpose `"Append-only structured before/after audit log with replay"`.

### F-12: `created_at` population after INSERT
[ASSUMED] SeaORM's INSERT on SQLite with UUID PK + `DEFAULT CURRENT_TIMESTAMP` column may not return the server-generated `created_at` value. The safest approach is a post-insert `find_by_id(new_id).one(conn)` re-fetch. This adds one SELECT round-trip but guarantees correct `created_at` in the returned `AuditEntry`. An alternative is to set `created_at` in the application using `chrono::Utc::now().naive_utc()` — simpler but breaks the D-22 invariant. The re-fetch is the correct implementation.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `DeriveMigrationName` on a struct named `Migration` in `migration.rs` produces a unique migration name that does not collide with app migration names | Pitfall 2 | Migration table entry name collision would prevent consumer from running the migration alongside app migrations; mitigation: use a prefixed struct name (e.g. `CreateAuditLogTable`) as the `Migration` alias in `lib.rs`, but the internal struct name can stay `Migration` |
| A2 | A post-INSERT `find_by_id(new_id)` re-fetch is necessary to populate `created_at` correctly from the DB default | F-12 / Pitfall 1 | If SeaORM's SQLite INSERT does return the default value, the re-fetch is a harmless extra round-trip; if it does not return it and we skip the re-fetch, `created_at` is wrong |
| A3 | `tracing = "0.1"` is the correct version pin for the `tracing::warn!` call | Standard Stack | If workspace resolves a different major, clippy or the compiler will catch it; low risk |
| A4 | `AuditEntry::write()` returning `Result<AuditEntry, AuditError>` correctly means a re-fetch by `id` is needed (vs using `insert_and_return()`) | Architecture Patterns | If SeaORM 1.1 `insert_and_return()` works on SQLite with UUID PK, the re-fetch can be skipped; test T-D-30-1 will surface this |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **`created_at` after INSERT on SQLite with UUID PK**
   - What we know: SeaORM SQLite driver uses `last_insert_rowid()` to re-fetch after INSERT; UUID PKs with `auto_increment = false` may not work with `last_insert_rowid()`.
   - What's unclear: Whether `insert_and_return()` is available and works cross-dialect for UUID PKs in SeaORM 1.1.
   - Recommendation: Implement the re-fetch pattern (`find_by_id(new_id).one(conn).await?`) and let the tests (T-D-30-1) confirm it works. If `insert_and_return()` works, it can replace the re-fetch in a follow-up.

2. **`DeriveMigrationName` output for `ferro_audit::migration::Migration`**
   - What we know: The macro derives the name from the Rust source path / module path.
   - What's unclear: Exact string produced (`migration`, `ferro_audit__migration__migration`, or something else).
   - Recommendation: The `schema_migrations` table entry name is an implementation detail; consumers don't need to know it. The migration executes correctly regardless of the stored name. Planner can ignore this — tests will confirm by running `MigratorTrait::up()` without error.

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: ferro-orm/Cargo.toml] — Wave 1a Cargo.toml shape, dev-dependency sea-orm features
- [VERIFIED: ferro-orm/src/guarded.rs] — `fresh_db()` inline test harness pattern (no framework dep)
- [VERIFIED: ferro-orm/tests/concurrent_decrement.rs] — integration test shape, `Schema::new()` pattern
- [VERIFIED: app/src/migrations/m20260228_create_api_keys_table.rs] — `DeriveMigrationName`, `MigrationTrait` impl pattern, `Index::create()` pattern
- [VERIFIED: app/src/migrations/mod.rs] — consumer `MigratorTrait` registration pattern
- [VERIFIED: framework/Cargo.toml] — `uuid = { version = "1", features = ["v4"] }`, `chrono = { version = "0.4", features = ["serde"] }`, `sea-orm = { version = "1.0", features = ["sqlx-postgres", "sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }`
- [VERIFIED: Cargo.toml] — workspace version `0.2.30`; `ferro-audit` not yet in `[workspace.members]`
- [VERIFIED: Cargo.lock] — `sea-orm` 1.1.19, `sea-orm-migration` 1.1.19, `uuid` 1.19.0
- [VERIFIED: .github/workflows/publish.yml line 201] — exact `WAVE1A_CRATES` string
- [VERIFIED: docs/src/SUMMARY.md] — nav entry pattern for database section
- [CITED: https://www.sea-ql.org/SeaORM/docs/generate-entity/column-types/] — JSON → `json_text` (SQLite) / `json` (Postgres); UUID → `uuid_text` (SQLite) / `uuid` (Postgres); DateTime → `chrono::NaiveDateTime`
- [CITED: https://docs.rs/sea-query/latest/sea_query/] — `ColumnDef::json()`, `ColumnDef::uuid()`, `ColumnDef::json_binary()` API signatures

### Secondary (MEDIUM confidence)
- [WebSearch + SeaQL GitHub discussion] — `json()` vs `json_binary()` cross-dialect behavior; `json_binary()` produces JSONB on Postgres

### Tertiary (LOW confidence)
- [ASSUMED: A1] — `DeriveMigrationName` name uniqueness in consumer context
- [ASSUMED: A2] — post-INSERT re-fetch necessity for UUID PK + DB-default `created_at`

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — exact Cargo.toml shape verified from sibling crate; dep versions verified from Cargo.lock
- Architecture: HIGH — migration pattern, entity model, test harness verified from existing crate code
- Pitfalls: MEDIUM — UUID PK + `created_at` INSERT behavior (A2) is assumed; all others verified from SeaORM docs or existing patterns
- Release wiring: HIGH — exact WAVE1A_CRATES string verified from publish.yml; docs/SUMMARY.md entry pattern verified

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 (SeaORM 1.1 is stable; 30-day window)

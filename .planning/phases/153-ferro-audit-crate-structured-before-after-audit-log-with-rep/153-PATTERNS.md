# Phase 153: ferro-audit — Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 19 new / modified files
**Analogs found:** 18 / 19 (one greenfield with closest partial match noted)

---

## File Classification

| New / Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------------|------|-----------|----------------|---------------|
| `ferro-audit/Cargo.toml` | config | — | `ferro-orm/Cargo.toml` | exact |
| `ferro-audit/src/lib.rs` | config | — | `ferro-orm/src/lib.rs` | exact |
| `ferro-audit/src/error.rs` | utility | — | `ferro-orm/src/error.rs` | exact |
| `ferro-audit/src/actor.rs` | model | — | `ferro-orm/src/error.rs` (enum + match arms) | role-match |
| `ferro-audit/src/target.rs` | model | — | (struct shape; no direct twin) | partial |
| `ferro-audit/src/entry.rs` | service | CRUD | `ferro-orm/src/guarded.rs` (consuming builder) | role-match |
| `ferro-audit/src/entity.rs` | model | CRUD | inline entity in `ferro-orm/src/guarded.rs` tests | role-match |
| `ferro-audit/src/migration.rs` | migration | — | `app/src/migrations/m20260228_create_api_keys_table.rs` | exact |
| `ferro-audit/src/query.rs` | service | CRUD | `framework/src/database/query_builder.rs` | role-match |
| `ferro-audit/src/replay.rs` | utility | transform | `ferro-json-ui/src/data.rs` (`Value::Object` iteration) | partial |
| `ferro-audit/src/prune.rs` | service | CRUD | `framework/src/database/model.rs` (DELETE returning rows_affected) | partial |
| `ferro-audit/tests/replay_round_trip.rs` | test | — | `ferro-orm/tests/concurrent_decrement.rs` | exact |
| `ferro-audit/README.md` | config | — | `ferro-orm/README.md` | exact |
| `docs/src/database/audit-log.md` | config | — | `docs/src/database/atomic-updates.md` | exact |
| `docs/src/SUMMARY.md` | config | — | current `docs/src/SUMMARY.md` (add one line) | exact |
| `Cargo.toml` (workspace) | config | — | current `Cargo.toml` (add member + bump version) | exact |
| `.github/workflows/publish.yml` | config | — | current `publish.yml` line 201 (`WAVE1A_CRATES`) | exact |
| `README.md` (workspace root) | config | — | current `README.md` "What's included" list | exact |
| `CLAUDE.md` | config | — | current `CLAUDE.md` Workspace Structure table | exact |
| `CHANGELOG.md` | config | — | `CHANGELOG.md` `## ferro-orm` section | exact |

---

## Pattern Assignments

### `ferro-audit/Cargo.toml` (config)

**Analog:** `ferro-orm/Cargo.toml`

**Full file excerpt** (lines 1-20):
```toml
[package]
name = "ferro-orm"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Atomic conditional updates and ORM primitives for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["orm", "sea-orm", "atomic", "concurrency", "ferro"]
categories = ["database"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

[dependencies]
sea-orm = "1.0"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```

**Pattern:** Copy this shape verbatim. Swap `name`, `description`, `keywords`. `ferro-audit` needs additional deps beyond `sea-orm` and `thiserror` — add `sea-orm-migration = "1.0"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `uuid = { version = "1", features = ["v4", "serde"] }`, `chrono = { version = "0.4", features = ["serde"] }`, `tracing = "0.1"` to `[dependencies]`. The `[dev-dependencies]` `tokio` entry keeps `"full"` only (drop `"test-util"` — audit tests use `#[tokio::test]` without `test-util`).

---

### `ferro-audit/src/lib.rs` (config)

**Analog:** `ferro-orm/src/lib.rs`

**Full file excerpt** (lines 1-47):
```rust
//! # ferro-orm
//!
//! Atomic conditional updates and ORM primitives for the Ferro framework.
//!
//! `GuardedUpdate<E>` compiles to a single `UPDATE … WHERE …` SQL statement,
//! …
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_orm::{GuardedUpdate, ColumnTrait};
//! …
//! ```

mod error;
mod guarded;

pub use error::GuardedError;
pub use guarded::GuardedUpdate;

// Targeted re-exports — consumers calling the builder need these.
// Do NOT add a wildcard re-export of `sea_orm` (D-03).
pub use sea_orm::sea_query::{Expr, IntoCondition, SimpleExpr, Value};
pub use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait};
```

**Pattern:** Module-level rustdoc opens with *why* first (what problem does the crate solve, who uses it), then shows the one-call example. Module declarations match the file layout. `pub use` re-exports bring the public API to the crate root. No blanket `pub use sea_orm::*`. For `ferro-audit`, the re-exports will be `AuditActor`, `AuditTarget`, `AuditEntry`, `AuditError`, `CreateAuditLogTable`, `reconstruct_state`, and `AuditLogEntity`.

---

### `ferro-audit/src/error.rs` (utility)

**Analog:** `ferro-orm/src/error.rs`

**Full file excerpt** (lines 1-35):
```rust
//! `GuardedError` — the single error type for the ferro-orm crate.
//!
//! Every variant's `Display` impl prefixes `"guarded: …"` so production
//! log greps stay surgical (matches the workspace convention used by
//! `WalletError` with `"wallet: …"`, `ConfigError` with `"config: …"`).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardedError {
    #[error("guarded: predicate matched no rows")]
    NoRowsAffected,

    #[error(
        "guarded: predicate matched {affected} rows (expected 1) — likely an index/uniqueness bug"
    )]
    TooManyRows { affected: u64 },

    #[error("guarded: no columns to set — builder is empty")]
    EmptyUpdate,

    #[error("guarded: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_no_rows_affected_displays_message() {
        assert_eq!(
            GuardedError::NoRowsAffected.to_string(),
            "guarded: predicate matched no rows"
        );
    }
    // … (pattern: one display-assertion test per variant)
}
```

**Pattern:** One `thiserror`-derived enum, `"audit: …"` Display prefix instead of `"guarded: …"`. The three `ferro-audit` variants are `MissingAction`, `Db(#[from] sea_orm::DbErr)`, and `Json(#[from] serde_json::Error)`. Inline `#[cfg(test)]` module with one `assert_eq!(err.to_string(), "audit: …")` per variant — copy the test structure directly.

---

### `ferro-audit/src/actor.rs` (model)

**Analog:** `ferro-orm/src/error.rs` (enum + `match self {}` pattern)

**Pattern to copy — enum declaration with match-arm DB serialization:**
```rust
// From ferro-orm/src/error.rs — enum + match pattern
#[derive(Debug, Error)]
pub enum GuardedError {
    NoRowsAffected,
    TooManyRows { affected: u64 },
    EmptyUpdate,
    Db(#[from] sea_orm::DbErr),
}
```

**Actor-specific pattern (from RESEARCH.md §Code Examples):**
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

**Pattern:** Plain enum (no `thiserror`, no `serde` on the enum itself — DB serialization is via the two `kind()` / `id()` helper methods). `kind()` returns the snake_case string stored in `actor_kind` column. `id()` returns `None` for `System` and `Anonymous` (written as SQL NULL); other variants return `Some(&str)`. The `User(id) | Job(id) | ApiClient(id)` multi-arm pattern collapses the three tuple-variant arms. Add `#[cfg(test)]` inline tests asserting `kind()` and `id()` for each variant.

---

### `ferro-audit/src/target.rs` (model)

**Analog:** No exact twin. Closest structural reference is the `AuditActor` pattern above (stringly-typed, domain-agnostic wrapper).

**Pattern (from CONTEXT.md D-07):**
```rust
// ferro-audit/src/target.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditTarget {
    pub kind: String,
    pub id: String,
}

impl AuditTarget {
    pub fn new(kind: impl Into<String>, id: impl ToString) -> Self {
        Self {
            kind: kind.into(),
            id: id.to_string(),
        }
    }
}

impl<K: Into<String>, I: ToString> From<(K, I)> for AuditTarget {
    fn from((kind, id): (K, I)) -> Self {
        Self::new(kind, id)
    }
}
```

**Pattern:** Plain struct, two `String` fields. `new()` takes `impl Into<String>` + `impl ToString` (same ergonomic pattern used throughout the framework for stringly-keyed primitives). `From<(K, I)>` tuple impl for convenience. No generics on the struct itself — domain-agnostic by being string-based, not by being generic.

---

### `ferro-audit/src/entry.rs` (service, CRUD / write path)

**Analog:** `ferro-orm/src/guarded.rs`

**Builder struct pattern** (lines 14-18 of `guarded.rs`):
```rust
pub struct GuardedUpdate<E: EntityTrait> {
    entity: E,
    filters: Condition,
    sets: Vec<(E::Column, SimpleExpr)>,
}
```

**Consuming-builder method chain pattern** (lines 30-45 of `guarded.rs`):
```rust
pub fn filter<F: IntoCondition>(mut self, f: F) -> Self {
    self.filters = self.filters.add(f.into_condition());
    self
}

pub fn set_expr(mut self, col: E::Column, expr: SimpleExpr) -> Self {
    self.sets.push((col, expr));
    self
}
```

**Async exec entry point + generic `<C: ConnectionTrait>`** (lines 62-68 of `guarded.rs`):
```rust
pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError> {
    match self.exec_raw(conn).await? {
        0 => Err(GuardedError::NoRowsAffected),
        1 => Ok(()),
        n => Err(GuardedError::TooManyRows { affected: n }),
    }
}
```

**Pattern for `ferro-audit/src/entry.rs`:** Define `AuditEntryBuilder` (or `AuditEntry` as the builder directly). Each setter takes `mut self -> Self`. `write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError>` mirrors `exec_one`. The returned `AuditEntry` is the persisted struct populated via a post-INSERT `find_by_id(new_id).one(conn).await?` re-fetch (see RESEARCH.md Pitfall 1 / F-12). `AuditEntry::record(action)` is the entry point that returns the builder with `actor` defaulted to `AuditActor::System` and all other fields as `None`.

**Entry point pattern:**
```rust
impl AuditEntry {
    pub fn record(action: impl Into<String>) -> AuditEntryBuilder {
        AuditEntryBuilder {
            action: action.into(),
            actor: AuditActor::System,  // D-10 default
            target: None,
            before: None,
            after: None,
            reason: None,
            correlation_id: None,
            tenant_id: None,
        }
    }
}
```

---

### `ferro-audit/src/entity.rs` (model, SeaORM DeriveEntityModel)

**Analog:** Inline `counters` entity in `ferro-orm/src/guarded.rs` tests (lines 112-128):
```rust
mod counters {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "counters")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub quantity: i32,
        pub status: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
```

**Pattern for `ferro-audit/src/entity.rs`:**
```rust
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
    pub created_at: DateTime,   // chrono::NaiveDateTime via SeaORM alias
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**Key deviations from `counters`:** `auto_increment = false` on the UUID primary key (RESEARCH F-03). `Option<JsonValue>` for `before` / `after` columns. `DateTime` (SeaORM alias for `chrono::NaiveDateTime`) for `created_at` — not set by application, set by DB default (RESEARCH F-04). `PartialEq` only (not `Eq`) because `serde_json::Value` is not `Eq`.

---

### `ferro-audit/src/migration.rs` (migration)

**Analog:** `app/src/migrations/m20260228_create_api_keys_table.rs`

**Full file excerpt** (all 80 lines):
```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
                    .col(ColumnDef::new(ApiKeys::Name).string().not_null())
                    // … nullable columns:
                    .col(ColumnDef::new(ApiKeys::Scopes).text().null())
                    // … timestamp with default:
                    .col(
                        ColumnDef::new(ApiKeys::CreatedAt)
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
    Table, Id, Name, Prefix, HashedKey, Scopes, LastUsedAt, ExpiresAt, RevokedAt, CreatedAt,
}
```

**Pattern for `ferro-audit/src/migration.rs`:** Copy the structure verbatim. Key differences for audit: `id` uses `.uuid().not_null().primary_key()` (no `.auto_increment()`); `before` / `after` use `.json().null()`; two composite indexes (`idx_audit_target`, `idx_audit_actor`) instead of one — chain a second `manager.create_index(…).await?` before the final `manager.create_index(…).await`. The `DeriveIden` enum covers all 12 `audit_log` columns. The `#[async_trait::async_trait]` attribute on `impl MigrationTrait` is mandatory (RESEARCH Pitfall 5).

---

### `ferro-audit/src/query.rs` (service, CRUD / read path)

**Analog:** `framework/src/database/query_builder.rs`

**SeaORM filter + order + limit pattern** (lines 86-157 of `query_builder.rs`):
```rust
pub fn filter<F>(mut self, filter: F) -> Self
where
    F: sea_orm::sea_query::IntoCondition,
{
    self.select = self.select.filter(filter);
    self
}

pub fn order_by_asc<C>(mut self, col: C) -> Self
where
    C: ColumnTrait,
{
    self.select = self.select.order_by(col, Order::Asc);
    self
}

pub fn order_by_desc<C>(mut self, col: C) -> Self
where
    C: ColumnTrait,
{
    self.select = self.select.order_by(col, Order::Desc);
    self
}

pub fn limit(mut self, limit: u64) -> Self {
    self.select = self.select.limit(limit);
    self
}
```

**Pattern for `ferro-audit/src/query.rs`:** These are standalone `async fn` helpers on `AuditEntry`, not a builder — they take the filter argument directly and execute against a `<C: ConnectionTrait>`. Use `Entity::find()` as the starting point, chain `.filter(Column::TargetKind.eq(…))`, `.filter(Column::TargetId.eq(…))`, `.order_by(Column::CreatedAt, Order::Asc)`, `.all(conn)`. The `recent_by_actor` helper filters by `Column::ActorKind.eq(actor.kind())` and optionally `Column::ActorId.eq(actor.id())` where `actor.id()` is `Some`. The `prune_older_than` function (implemented in `prune.rs`) uses `Entity::delete_many().filter(Column::CreatedAt.lt(cutoff)).exec(conn).await` and returns `result.rows_affected`.

---

### `ferro-audit/src/replay.rs` (utility, transform)

**Analog:** `ferro-json-ui/src/data.rs` (closest existing `serde_json::Value::Object(map)` iteration)

**`Value::Object` iteration pattern** (lines 29-31 of `data.rs`):
```rust
match current {
    Value::Object(map) => {
        current = map.get(segment)?;
    }
    // …
}
```

**Pattern for `ferro-audit/src/replay.rs`:**
```rust
// ferro-audit/src/replay.rs
use serde_json::{Map, Value};

pub fn reconstruct_state(entries: &[AuditEntry]) -> Option<Value> {
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

**Pattern:** Pure function (no `conn`, no `async`). `Map::new()` accumulates the running state; keys from newer `after` overwrite older ones. Non-object `after` values replace the state wholesale and return immediately. Empty slice or all-None `after` fields return `None`. This is greenfield logic — no existing analog in the codebase does the fold — but the `Value::Object(map)` destructure pattern is established in `ferro-json-ui/src/data.rs`.

---

### `ferro-audit/src/prune.rs` (service, CRUD / delete path)

**Analog:** `framework/src/database/model.rs` lines 236-246

**DELETE returning rows_affected pattern:**
```rust
async fn delete_by_pk<K>(id: K) -> Result<u64, FrameworkError>
where
    K: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
{
    let db = DB::connection()?;
    let result = Self::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|e| FrameworkError::database(e.to_string()))?;
    Ok(result.rows_affected)
}
```

**Pattern for `ferro-audit/src/prune.rs`:** Use `Entity::delete_many()` instead of `delete_by_id` (bulk delete by time range, not by PK). Filter with `.filter(Column::CreatedAt.lt(cutoff))`. Return `result.rows_affected as u64`. Signature: `pub async fn prune_older_than<C: ConnectionTrait>(cutoff: chrono::NaiveDateTime, conn: &C) -> Result<u64, AuditError>`. The `chrono::NaiveDateTime` type matches the entity's `created_at` field type (SeaORM `DateTime` alias; RESEARCH F-04).

---

### `ferro-audit/tests/replay_round_trip.rs` (test, integration)

**Analog:** `ferro-orm/tests/concurrent_decrement.rs`

**Integration test structure** (lines 1-30):
```rust
use ferro_orm::{GuardedError, GuardedUpdate};
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseBackend,
    EntityTrait, Schema, Set,
};

mod counters {
    use sea_orm::entity::prelude::*;
    // … inline entity definition
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ten_tasks_against_capacity_three_exactly_three_succeed() {
    let conn = /* connect to sqlite::memory: */;
    // schema setup → seed → action → assert
}
```

**Pattern for `tests/replay_round_trip.rs`:** Single `#[tokio::test]` (default single-threaded flavor — no concurrency needed for replay). Use `sqlite::memory:` via `Database::connect`. Run the migration via `TestMigrator::up(&conn, None)` (inline `struct TestMigrator` implementing `MigratorTrait` with `vec![Box::new(crate::migration::Migration)]`). Insert a lifecycle sequence: `AuditEntry::record("inventory.unit.created").after(json!({…})).write(&conn)`, then two `.record("inventory.unit.updated").before(…).after(…).write(&conn)`, then a status-change entry. Assert that `AuditEntry::history_for_target(&target, &conn).await` returns entries ordered `ASC`. Assert `reconstruct_state(&entries) == Some(expected_final_json)`. The test proves the design promise in one readable scenario.

---

### `ferro-audit/README.md` (config)

**Analog:** `ferro-orm/README.md`

**Full file (11 lines):**
```markdown
# ferro-orm

Atomic conditional updates and ORM primitives for the Ferro framework.

The crate exposes `GuardedUpdate<E>` — a chainable builder that compiles to a single `UPDATE … WHERE …` SQL statement, …

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-orm

License: MIT
```

**Pattern:** One paragraph of prose (what + how in one sentence). Status line. Docs link. License. Nothing else — no badges, no API tables, no examples. Mirror this format exactly, replacing the crate name, concept, and docs link.

---

### `docs/src/database/audit-log.md` (config)

**Analog:** `docs/src/database/atomic-updates.md`

**Structure to mirror:**
1. H1 title (`# Audit Log`)
2. Opening paragraph: what the crate is for (forensic / regulatory / replay) in 2–3 sentences
3. `## The Anti-Pattern` — what the caller would write without this crate
4. `## The Replacement` — one-call API with the canonical builder example
5. `## API` — subsections for each public symbol (`AuditEntry::record`, builder methods, query helpers, `reconstruct_state`, `prune_older_than`)
6. `## AuditActor` — variant table
7. `## AuditTarget` — struct fields, dotted-namespace convention
8. `## Schema` — the `audit_log` table columns + indexes
9. `## Replay` — how `history_for_target` + `reconstruct_state` compose; shallow-merge semantics; warning on deep-merge
10. `## Retention and Pruning` — `prune_older_than`, 1–3 year recommendation, GDPR note
11. `## Errors` — variant table (matching the error-table style in `atomic-updates.md`)
12. `## Postgres vs SQLite` — dialect-agnostic note

**Pattern:** Tone matches `atomic-updates.md` — matter-of-fact, no marketing language. Code blocks use `rust,ignore`. Every API method has a one-line description. The worked example is the inventory-decrement scenario from CONTEXT.md §specifics.

---

### `docs/src/SUMMARY.md` (config, nav)

**Analog:** Current `docs/src/SUMMARY.md` lines 33-34:
```markdown
- [Database](features/database.md)
- [Atomic Updates](database/atomic-updates.md)
```

**Pattern:** Add one line immediately after `Atomic Updates`:
```markdown
- [Audit Log](database/audit-log.md)
```

---

### `Cargo.toml` (workspace root, members + version)

**Analog:** Current `Cargo.toml` lines 1-30:
```toml
[workspace]
resolver = "2"
members = [
    "framework",
    …
    "ferro-orm",
]

[workspace.package]
version = "0.2.30"
```

**Pattern:** Add `"ferro-audit"` to `members` list (after `"ferro-orm"`). Bump `version = "0.2.31"`. No other changes.

---

### `.github/workflows/publish.yml` (config, Wave 1a)

**Analog:** `publish.yml` line 201:
```
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm"
```

**Pattern:** Append `ferro-audit` at the end of the string:
```
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit"
```

---

### `README.md` (workspace root, "What's included" section)

**Analog:** Current `README.md` lines 60-72, specifically:
```markdown
- **Atomic conditional updates** — race-free counter mutations and state transitions (`ferro-orm`)
```

**Pattern:** Add one bullet in the list referencing `ferro-audit`. Position after the `ferro-orm` bullet. The existing ORM bullet is a good model — one sentence, feature description in bold, crate name in parentheses at the end.

---

### `CLAUDE.md` Workspace Structure table

**Analog:** Current `CLAUDE.md` lines 58-59:
```markdown
| `ferro-orm` | Atomic conditional updates and ORM primitives (`GuardedUpdate`) | `src/lib.rs` |
| `app` | Sample application | Reference implementation |
```

**Pattern:** Insert a row for `ferro-audit` between `ferro-orm` and `app`:
```markdown
| `ferro-audit` | Append-only structured before/after audit log with replay | `src/lib.rs` |
```

---

### `CHANGELOG.md` (new section)

**Analog:** `CHANGELOG.md` lines 6-48 (`## ferro-orm` section):
```markdown
## ferro-orm

### [0.2.30] — 2026-05-13

Initial release. Phase 152 — `ferro-orm` crate (atomic conditional UPDATE
primitive for race-free counter mutations and state transitions).
Milestone v11.11.

#### Added

- New crate `ferro-orm` exposing the `GuardedUpdate<E>` builder …
- `GuardedError` — `NoRowsAffected | TooManyRows | EmptyUpdate | Db(…)`
  Display prefix `"guarded: …"`.
- Targeted re-exports of SeaORM symbols …
- Workspace member registered in `Cargo.toml`; auto-publish Wave 1a slot …
- New documentation page `docs/src/database/atomic-updates.md` …
```

**Pattern:** New `## ferro-audit` section at the top of `CHANGELOG.md` (before `## ferro-orm`). Heading `### [0.2.31] — 2026-05-13`. Introductory sentence: "Initial release. Phase 153 — `ferro-audit` crate (…). Milestone v11.11." Then `#### Added` with one bullet per major public symbol: `AuditEntry::record(…).write()`, `AuditActor`, `AuditTarget`, query helpers (`history_for_target`, `recent_by_actor`, `recent`), `reconstruct_state`, `prune_older_than`, `CreateAuditLogTable` migration, workspace member + Wave 1a slot, doc page.

---

## Shared Patterns

### Consuming-builder with `mut self -> Self` setters
**Source:** `ferro-orm/src/guarded.rs` lines 30-45
**Apply to:** `ferro-audit/src/entry.rs` (all builder setter methods)
```rust
pub fn filter<F: IntoCondition>(mut self, f: F) -> Self {
    self.filters = self.filters.add(f.into_condition());
    self
}
```
Every setter takes `mut self`, mutates one field, returns `Self`. No `&mut self` references — the builder is moved through the chain.

### Generic `<C: ConnectionTrait>` on async DB methods
**Source:** `ferro-orm/src/guarded.rs` line 62
**Apply to:** `ferro-audit/src/entry.rs` (`write`), `ferro-audit/src/query.rs` (all three helpers), `ferro-audit/src/prune.rs`
```rust
pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError> {
```
Never hardcode `DatabaseConnection` — always use `<C: ConnectionTrait>` so callers can pass a `&DatabaseTransaction` inside a broader transaction.

### `"audit: …"` Display prefix on error variants
**Source:** `ferro-orm/src/error.rs` lines 14-34 (`"guarded: …"` pattern)
**Apply to:** All three `AuditError` variants in `ferro-audit/src/error.rs`
```rust
#[error("guarded: predicate matched no rows")]
#[error("guarded: db error: {0}")]
```
Replace `"guarded:"` with `"audit:"`. Identical structural convention.

### Post-INSERT `find_by_id` re-fetch for DB-defaulted columns
**Source:** RESEARCH.md §Pitfall 1 + §F-12 (no existing analog in codebase — this is a confirmed-needed pattern without a current example)
**Apply to:** `ferro-audit/src/entry.rs` (`write` function)
```rust
let model = audit_log::Entity::find_by_id(new_id)
    .one(conn)
    .await?
    .ok_or(AuditError::Db(DbErr::RecordNotFound("audit_log".to_string())))?;
```
Required because SeaORM's SQLite INSERT with UUID PK + `DEFAULT CURRENT_TIMESTAMP` does not return the server-generated `created_at`. Re-fetch by `id` after INSERT.

### Inline `fresh_db()` test harness (no framework dep)
**Source:** `ferro-orm/src/guarded.rs` lines 130-140
**Apply to:** All `#[cfg(test)]` blocks in `ferro-audit/src/` and `ferro-audit/tests/`
```rust
async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");
    // run migration inline
    conn
}
```
`ferro-audit` re-derives this harness inline using its own `migration::Migration` (not `Schema::create_table_from_entity` like `ferro-orm` — the audit tests exercise the real migration SQL). Pattern: inline `struct TestMigrator` implementing `MigratorTrait` with `vec![Box::new(migration::Migration)]`, then `TestMigrator::up(&conn, None).await`.

### `tracing::warn!` for soft-failure diagnostics
**Source:** CONTEXT.md D-10 (no existing analog in `ferro-*` leaf crates; established pattern in `framework/`)
**Apply to:** `ferro-audit/src/entry.rs` (`write` function, missing-target branch)
```rust
if self.target.is_none() {
    tracing::warn!(
        action = %self.action,
        "audit entry written without a target — history_for_target will not find this entry"
    );
}
```
`tracing::warn!` takes structured fields (`key = %value`) before the message string. This is the established `tracing` macro call convention throughout the framework.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-audit/src/replay.rs` (the fold logic itself) | utility | transform | No existing crate accumulates a running `serde_json::Map` via fold. The `Value::Object(map)` destructure pattern exists in `ferro-json-ui/src/data.rs` but the fold-accumulate pattern is greenfield. Planner should use RESEARCH.md §Pattern 4 as the authoritative implementation. |

---

## Metadata

**Analog search scope:** `ferro-orm/`, `ferro-wallet/`, `ferro-json-ui/`, `framework/src/database/`, `app/src/migrations/`, `docs/src/database/`, `CHANGELOG.md`, `README.md`, `CLAUDE.md`, `.github/workflows/publish.yml`, `Cargo.toml`
**Files scanned:** 18 source files read
**Pattern extraction date:** 2026-05-13

---
phase: 155
plan: "03"
subsystem: ferro-projection
tags: [seaorm, migration, entity, composite-pk, sqlite, persistence]
dependency_graph:
  requires: [155-02]
  provides: [projection_snapshots table schema, SeaORM Entity/Model/ActiveModel surface]
  affects: [ferro-projection/src/migration.rs, ferro-projection/src/entity.rs, ferro-projection/src/lib.rs]
tech_stack:
  added: []
  patterns: [SeaORM composite primary key (first workspace use), sqlite_master smoke test, composite-PK tuple lookup]
key_files:
  created: []
  modified:
    - ferro-projection/src/migration.rs
    - ferro-projection/src/entity.rs
decisions:
  - "Composite PK declared in migration via .primary_key(Index::create().col(A).col(B)) — NOT two individual .primary_key() calls on each column (those would create two single-column PKs, which is a schema error)"
  - "Composite PK signaled to DeriveEntityModel by annotating BOTH projection_name and key fields with #[sea_orm(primary_key, auto_increment = false)] — SeaORM generates the composite PrimaryKey impl automatically"
  - "Manual MigrationName impl (not DeriveMigrationName) for explicit ident control: m20260514_000001_create_projection_snapshots_table"
  - "lib.rs re-exports required no changes — DeriveEntityModel-generated Entity/ActiveModel/Model symbols match the stub names the plan-01 re-exports expected"
metrics:
  duration: "~8 minutes"
  completed: "2026-05-14"
  tasks_completed: 3
  files_modified: 2
---

# Phase 155 Plan 03: SeaORM Persistence Layer for ferro-projection Summary

SeaORM `projection_snapshots` table with composite `(projection_name, key)` PK — the first workspace use of this pattern — proven against SQLite via two unit tests.

## What Was Built

### Files Overwritten

**`ferro-projection/src/migration.rs`** (replaces plan-01 stub):
- `pub struct Migration` + manual `MigrationName` impl returning `"m20260514_000001_create_projection_snapshots_table"`
- `MigrationTrait::up` creates `projection_snapshots` with 5 columns: `projection_name VARCHAR NOT NULL`, `key VARCHAR NOT NULL`, `state JSON NOT NULL`, `version BIGINT NOT NULL`, `updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP`
- Composite PK via `.primary_key(Index::create().col(ProjectionSnapshots::ProjectionName).col(ProjectionSnapshots::Key))` — no per-column `.primary_key()` calls
- `DeriveIden` enum `ProjectionSnapshots` with 6 variants: `Table, ProjectionName, Key, State, Version, UpdatedAt`
- `MigrationTrait::down` drops the table
- Test: `migration_creates_projection_snapshots_table` (sqlite_master smoke test)

**`ferro-projection/src/entity.rs`** (replaces plan-01 stub):
- `#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]` Model on `projection_snapshots`
- EXACTLY 2 `#[sea_orm(primary_key, auto_increment = false)]` annotations: one on `projection_name: String`, one on `key: String`
- `state: JsonValue`, `version: i64`, `updated_at: DateTime`
- Empty `Relation` enum + default `ActiveModelBehavior`
- Test: `round_trip_with_composite_pk` — insert ActiveModel, find via `Entity::find_by_id((name, key))` tuple, assert all 5 fields
- Test: `duplicate_composite_pk_is_constraint_violation` — second insert with same `(name, key)` returns `Err`, proving the DB-level constraint fires

### File Verified Unchanged

**`ferro-projection/src/lib.rs`**: `pub use entity::{ActiveModel as ProjectionSnapshotActiveModel, Entity as ProjectionSnapshotEntity, Model as ProjectionSnapshotModel}` — DeriveEntityModel-generated symbols share names with the plan-01 stubs, so the re-exports resolved correctly without edits.

## Composite-PK Pattern

This plan establishes the first workspace use of SeaORM composite primary keys:

**At the migration level:**
```rust
.primary_key(
    Index::create()
        .col(ProjectionSnapshots::ProjectionName)
        .col(ProjectionSnapshots::Key),
)
```

**At the entity level (both columns):**
```rust
#[sea_orm(primary_key, auto_increment = false)]
pub projection_name: String,

#[sea_orm(primary_key, auto_increment = false)]
pub key: String,
```

**Lookup form:**
```rust
Entity::find_by_id((name_value, key_value)).one(&conn).await?
```

## Migration Ident

`"m20260514_000001_create_projection_snapshots_table"` — one day after Phase 154's `"m20260513_..."` per design timeline convention.

## Tests Added

| Test | File | What It Proves |
|------|------|----------------|
| `migration_creates_projection_snapshots_table` | migration.rs | Table exists in sqlite_master after up() |
| `round_trip_with_composite_pk` | entity.rs | Insert + composite-PK tuple lookup + all 5 fields round-trip |
| `duplicate_composite_pk_is_constraint_violation` | entity.rs | DB-level PK constraint fires on duplicate (projection_name, key) |

## Test Results

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  - error::tests::db_from_sea_orm_dberr
  - error::tests::broadcast_display
  - error::tests::events_display
  - error::tests::json_from_serde_json_error
  - error::tests::state_not_found_display
  - migration::tests::migration_creates_projection_snapshots_table
  - entity::tests::duplicate_composite_pk_is_constraint_violation
  - entity::tests::round_trip_with_composite_pk
```

`cargo clippy -p ferro-projection --all-targets -- -D warnings` exits 0 with no warnings.

## Risk R3 Closed

RESEARCH.md §Risks R3 ("Composite PK — no workspace precedent") is closed. The `duplicate_composite_pk_is_constraint_violation` test is the proving artefact: the constraint fires at the DB level, the composite PK declaration syntax is verified against SQLite, and `DeriveEntityModel`'s tuple-lookup code generation works correctly.

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | c50956af | feat(155-03): replace migration.rs stub with full CreateProjectionSnapshotsTable body |
| 2 | a0621dd5 | feat(155-03): replace entity.rs stub with full SeaORM Model (composite PK + JSON state) |
| 3 | (verification only — no file changes) | cargo build + cargo test --lib (8 passed) + clippy (-D warnings) all exit 0 |

## Self-Check: PASSED

- ferro-projection/src/migration.rs: FOUND
- ferro-projection/src/entity.rs: FOUND
- Commit c50956af: FOUND
- Commit a0621dd5: FOUND
- 8 tests passing: CONFIRMED
- clippy -D warnings exit 0: CONFIRMED

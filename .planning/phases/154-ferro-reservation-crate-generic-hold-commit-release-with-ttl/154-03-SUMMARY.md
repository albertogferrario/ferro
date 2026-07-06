---
phase: 154
plan: "03"
subsystem: ferro-reservation
tags: [sea-orm, migration, entity, schema, sqlite, test]
dependency_graph:
  requires: [154-01, 154-02]
  provides: [CreateReservationsTable migration, ReservationEntity/Model/ActiveModel, reservations schema]
  affects: [154-04, 154-05]
tech_stack:
  added: []
  patterns:
    - SeaORM DeriveEntityModel with UUID PK and JSON nullable columns
    - Inline TestMigrator + sqlite::memory: test harness
    - serde_json::Value as JsonValue explicit import (not re-exported from sea_orm prelude in this context)
key_files:
  created: []
  modified:
    - ferro-reservation/src/migration.rs
    - ferro-reservation/src/entity.rs
    - ferro-reservation/src/lib.rs
decisions:
  - quantity stored as i32 in entity (SeaORM INTEGER mapping); kernel will cast to u32 at ReservationHandle API boundary per RESEARCH Pitfall 6
  - JsonValue import must be explicit (serde_json::Value as JsonValue) — sea_orm::entity::prelude::* does not re-export it in this crate build context
  - Stale planning comment in lib.rs replaced with neutral description after work completed
metrics:
  duration_seconds: 170
  completed: 2026-05-13T21:05:12Z
  tasks_completed: 3
  tasks_total: 3
  files_modified: 3
---

# Phase 154 Plan 03: Migration + Entity Full Body Summary

Full `CreateReservationsTable` SeaORM migration and `reservations` SeaORM entity replacing plan 01 stubs; lib.rs re-exports expanded to the three-symbol form.

## What Was Built

### Task 1: migration.rs — Full CreateReservationsTable body

`ferro-reservation/src/migration.rs` replaced the `pub struct Migration;` stub with a complete `MigrationTrait` impl:

- `up()` creates the `reservations` table with exactly 12 columns per D-39:
  `id` (UUID PK, client-generated D-41), `resource_kind` (VARCHAR NOT NULL),
  `resource_key` (JSON NOT NULL), `window` (JSON NULL), `quantity` (INTEGER NOT NULL),
  `status` (VARCHAR NOT NULL — D-16 stringly-typed), `expires_at` (TIMESTAMP NOT NULL),
  `held_at` (TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP — D-42),
  `committed_at` (TIMESTAMP NULL), `released_at` (TIMESTAMP NULL),
  `release_reason` (VARCHAR NULL), `tenant_id` (VARCHAR NULL — D-36)
- `up()` creates 2 composite indexes per D-40:
  `idx_reservations_kind_key_window_status` on `(resource_kind, resource_key, window, status)` and
  `idx_reservations_status_expires` on `(status, expires_at)`
- `down()` drops the table
- `DeriveIden` enum `Reservations` with `Table` + 12 column variants (13 total)
- Inline `#[cfg(test)]` block with 2 `#[tokio::test]` functions against in-memory SQLite:
  `migration_creates_reservations_table_and_indexes` and `migration_down_drops_table`

Commit: `bba94e03`

### Task 2: entity.rs — Full DeriveEntityModel body

`ferro-reservation/src/entity.rs` replaced the `pub struct Entity;` stub with a complete
`DeriveEntityModel` block matching the migration schema column-for-column:

- `#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]` on `pub struct Model`
- `#[sea_orm(table_name = "reservations")]` annotation
- All 12 fields with exact names and SeaORM-mapped types:
  - `id: Uuid` with `#[sea_orm(primary_key, auto_increment = false)]`
  - `resource_kind: String`, `resource_key: JsonValue`, `window: Option<JsonValue>`
  - `quantity: i32` (NOT u32 — SeaORM INTEGER maps to i32; Pitfall 6)
  - `status: String`, `expires_at: DateTime`, `held_at: DateTime`
  - `committed_at: Option<DateTime>`, `released_at: Option<DateTime>`
  - `release_reason: Option<String>`, `tenant_id: Option<String>`
- Empty `pub enum Relation {}` and `impl ActiveModelBehavior for ActiveModel {}`
- Inline round-trip test: insert via ActiveModel, fetch by id, assert all 12 fields

Commit: `420163ec`

### Task 3: lib.rs — Entity re-export expansion

`ferro-reservation/src/lib.rs` expanded from:
```rust
pub use entity::Entity as ReservationEntity;
```
to:
```rust
pub use entity::{
    ActiveModel as ReservationActiveModel, Entity as ReservationEntity, Model as ReservationModel,
};
```

Also removed stale planning comment that referenced future work now completed.

Commit: `e9ad6953`

## Test Results

```
cargo test -p ferro-reservation --lib

running 10 tests
test error::tests::insufficient_display ... ok
test error::tests::not_found_display ... ok
test error::tests::audit_from_ferro_audit_error ... ok
test error::tests::guarded_from_ferro_orm_error ... ok
test error::tests::conflicting_state_display ... ok
test error::tests::json_from_serde_json_error ... ok
test error::tests::db_from_sea_orm_dberr ... ok
test migration::tests::migration_creates_reservations_table_and_indexes ... ok
test entity::tests::model_round_trips_through_active_model ... ok
test migration::tests::migration_down_drops_table ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- 7 error tests (from plan 01)
- 2 migration tests (new in this plan)
- 1 entity round-trip test (new in this plan)

## Schema Alignment

| Column | Migration type | Entity field | Nullable |
|--------|---------------|--------------|----------|
| id | UUID PK | `id: Uuid` | NOT NULL |
| resource_kind | VARCHAR NOT NULL | `resource_kind: String` | NOT NULL |
| resource_key | JSON NOT NULL | `resource_key: JsonValue` | NOT NULL |
| window | JSON NULL | `window: Option<JsonValue>` | NULL |
| quantity | INTEGER NOT NULL | `quantity: i32` | NOT NULL |
| status | VARCHAR NOT NULL | `status: String` | NOT NULL |
| expires_at | TIMESTAMP NOT NULL | `expires_at: DateTime` | NOT NULL |
| held_at | TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP | `held_at: DateTime` | NOT NULL |
| committed_at | TIMESTAMP NULL | `committed_at: Option<DateTime>` | NULL |
| released_at | TIMESTAMP NULL | `released_at: Option<DateTime>` | NULL |
| release_reason | VARCHAR NULL | `release_reason: Option<String>` | NULL |
| tenant_id | VARCHAR NULL | `tenant_id: Option<String>` | NULL |

Column count: 12. Index count: 2. Alignment: exact.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing JsonValue import in entity.rs**
- **Found during:** Task 2 first compile
- **Issue:** `sea_orm::entity::prelude::*` does not re-export `JsonValue` in the ferro-reservation build context (contrast with ferro-audit which imports `serde_json::Value as JsonValue` explicitly)
- **Fix:** Added `use serde_json::Value as JsonValue;` after the prelude import
- **Files modified:** `ferro-reservation/src/entity.rs`
- **Commit:** `420163ec`

**2. [Rule 1 - Cleanup] Stale planning comment in lib.rs**
- **Found during:** Task 3 edit
- **Issue:** Comment still referenced "Plan 154-03 expands entity.rs to a full DeriveEntityModel block" — accurate before but misleading after the work completes
- **Fix:** Replaced with neutral description "SeaORM entity re-exports for consumers who need native SeaORM query access."
- **Files modified:** `ferro-reservation/src/lib.rs`
- **Commit:** `e9ad6953`

**3. [Rule 1 - Formatting] cargo fmt normalization on migration.rs and entity.rs**
- **Found during:** Task 3 fmt --check gate
- **Issue:** rustfmt collapsed some multi-line ColumnDef chains to single lines and reformatted the long assert_eq! call in entity tests
- **Fix:** Ran `cargo fmt -p ferro-reservation`; all diffs are whitespace/line-width only, no semantic change
- **Commit:** `e9ad6953`

## Requirements Addressed

D-16 (status as VARCHAR NOT NULL), D-36 (tenant_id Option<String>),
D-38 (CreateReservationsTable public re-export), D-39 (all 12 columns),
D-40 (both composite indexes), D-41 (UUID PK client-generated),
D-42 (held_at DEFAULT CURRENT_TIMESTAMP), D-47 (per-task unit tests),
D-52 (inline SQLite test harness, no framework dep)

## Known Stubs

None — no placeholder values or TODO markers in any of the three files.
The entity Model fields are all concrete typed columns; no hardcoded empty data flows to any rendering surface.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| ferro-reservation/src/migration.rs exists | FOUND |
| ferro-reservation/src/entity.rs exists | FOUND |
| ferro-reservation/src/lib.rs exists | FOUND |
| Commit bba94e03 exists | FOUND |
| Commit 420163ec exists | FOUND |
| Commit e9ad6953 exists | FOUND |
| cargo test --lib: 10 passed | PASSED |

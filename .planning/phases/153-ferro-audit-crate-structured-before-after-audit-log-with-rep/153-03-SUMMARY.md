---
phase: 153
plan: 03
subsystem: ferro-audit
tags: [rust, sea-orm, migration, entity, audit-log, schema]
dependency_graph:
  requires: [153-01, 153-02]
  provides: [ferro-audit/src/entity.rs, ferro-audit/src/migration.rs]
  affects: [ferro-audit/src/lib.rs (re-exports now resolve to real types)]
tech_stack:
  added: []
  patterns: [DeriveEntityModel, MigrationTrait, DeriveIden, DeriveRelation, MigratorTrait-unit-test]
key_files:
  created: []
  modified:
    - ferro-audit/src/entity.rs
    - ferro-audit/src/migration.rs
decisions:
  - "Used .json() not .json_binary() for before/after columns — cross-dialect compatibility (SQLite TEXT / Postgres json) per RESEARCH F-02"
  - "#[async_trait::async_trait] applied to impl MigrationTrait block — required by sea-orm-migration 1.1.19 per RESEARCH Pitfall 5"
  - "created_at uses .timestamp().not_null().default(Expr::current_timestamp()) — DB-stamped per D-22; application never sets it"
  - "UUID PK uses .uuid().not_null().primary_key() with no .auto_increment() — client-generated UUIDv4 per D-21"
  - "DeriveEntityModel derives PartialEq only (not Eq) — serde_json::Value does not implement Eq"
  - "id column written on one line to satisfy rustfmt (formatter requires it vs multi-line expansion)"
metrics:
  duration: "~3 minutes"
  completed: "2026-05-13"
  tasks: 2
  files: 2
requirements_addressed: [D-18, D-19, D-20, D-21, D-22]
---

# Phase 153 Plan 03: SeaORM Entity + Migration + Unit Test — Summary

SeaORM `DeriveEntityModel` entity and `MigrationTrait` migration for the `audit_log` table, overwriting the plan 153-01 stubs. The migration unit test `migration_creates_table_and_indexes` proves the schema is created and dropped correctly against in-memory SQLite. The schema-push gate for the phase is now closed.

## What Was Built

### Files Modified (2)

| File | Contents |
|------|----------|
| `ferro-audit/src/entity.rs` | Full `DeriveEntityModel` model for `audit_log`: 12 fields matching D-19, UUID PK (`auto_increment = false`), `Option<JsonValue>` for before/after, `DateTime` (chrono::NaiveDateTime) for created_at, empty `Relation` enum, default `ActiveModelBehavior` |
| `ferro-audit/src/migration.rs` | Full `MigrationTrait` impl: 12 columns via SeaORM schema builder (no raw SQL), two composite indexes (`idx_audit_target`, `idx_audit_actor`), `DeriveIden` enum with 13 variants, `down()` drops table, inline `migration_creates_table_and_indexes` unit test |

## Verification Results

| Command | Result |
|---------|--------|
| `cargo build -p ferro-audit` | exit 0 |
| `cargo test -p ferro-audit migration_creates_table_and_indexes` | 1/1 pass |
| `cargo test -p ferro-audit` | 13/13 pass (12 prior + 1 new) |
| `cargo clippy -p ferro-audit --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo fmt --all -- --check` | exit 0 |

## Schema Verification

The migration unit test verifies:
1. `audit_log` table exists in `sqlite_master` after `up()`
2. `idx_audit_target` index exists in `sqlite_master` after `up()`
3. `idx_audit_actor` index exists in `sqlite_master` after `up()`
4. `audit_log` table is absent from `sqlite_master` after `down()`

## Key Decisions

### `.json()` not `.json_binary()` for before/after

`ColumnDef::json()` maps to `json_text` (TEXT) in SQLite and `json` in Postgres. Both backends round-trip `serde_json::Value` transparently without consumer-side configuration. `json_binary()` produces JSONB on Postgres and SQLite 3.45+ JSONB on-disk format — avoided per RESEARCH F-02 anti-pattern callout for v0 cross-dialect compatibility.

### `#[async_trait::async_trait]` on impl MigrationTrait

Required by `sea-orm-migration 1.1.19` — the crate uses `async_trait` for `MigrationTrait`'s async methods. The attribute is re-exported via `sea_orm_migration::prelude::*` so no separate `async-trait` dep is needed. Pattern copied from `app/src/migrations/m20260228_create_api_keys_table.rs` per RESEARCH Pitfall 5.

### DeriveMigrationName name (Open Question 2 resolution)

The derived migration name is an implementation detail of `sea-orm-migration`. The unit test confirms the migration executes correctly regardless. No name collision is possible because `ferro-audit` ships only one migration and consumer app migrations use timestamped `m20…` prefixes.

## Deviations from Plan

### Auto-fix [Rule 1 - Style] rustfmt required id column on single line

- **Found during:** Post-commit `cargo fmt --all -- --check`
- **Issue:** The id column definition was written as a multi-line chain (`.uuid()` / `.not_null()` / `.primary_key()` each on its own line), which `rustfmt` collapses to a single line
- **Fix:** Applied `rustfmt` formatting — the three methods are now on one line: `ColumnDef::new(AuditLog::Id).uuid().not_null().primary_key()`
- **Commit:** `8e11fea1`

This also means the plan's `<verify>` grep `grep -q '.uuid().not_null().primary_key()'` now passes on the formatted file.

## Commits

| Hash | Message |
|------|---------|
| `3cf6dbc7` | feat(153-03): implement SeaORM entity for audit_log table |
| `767b3fee` | feat(153-03): implement CreateAuditLogTable migration + unit test |
| `8e11fea1` | style(153-03): apply rustfmt to migration.rs (id column single-line) |

## Stub Tracking

No stubs remain in the files modified by this plan. Both `entity.rs` and `migration.rs` previously had `#![allow(dead_code)]` placeholder bodies; those are now fully replaced. The `lib.rs` re-exports `pub use entity::Entity as AuditLogEntity` and `pub use migration::Migration as CreateAuditLogTable` now resolve to real types.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. The migration creates a DB table — this is within the planned threat surface (T-153-02). All DDL uses SeaORM's `Table::create()` / `Index::create()` schema builder; no raw SQL strings are constructed (T-153-02 mitigation confirmed).

## Self-Check: PASSED

- [x] `ferro-audit/src/entity.rs` — found, contains `DeriveEntityModel`, `table_name = "audit_log"`, 12 fields
- [x] `ferro-audit/src/migration.rs` — found, contains `impl MigrationTrait for Migration`, both indexes, unit test
- [x] Commit `3cf6dbc7` — present in git log
- [x] Commit `767b3fee` — present in git log
- [x] Commit `8e11fea1` — present in git log
- [x] 13 tests pass (including `migration_creates_table_and_indexes`)
- [x] clippy -D warnings clean
- [x] fmt clean

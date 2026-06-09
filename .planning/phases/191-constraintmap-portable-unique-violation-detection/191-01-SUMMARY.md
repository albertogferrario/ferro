---
phase: 191-constraintmap-portable-unique-violation-detection
plan: "01"
subsystem: framework/validation
tags: [constraint-map, unique-violation, sea-orm, validation, defensive-layer]
dependency_graph:
  requires: [190-async-rule-infrastructure-unique-rule]
  provides: [ConstraintMap, MapConstraintExt]
  affects: [framework/src/validation/, framework/src/lib.rs]
tech_stack:
  added: []
  patterns: [consuming-builder, extension-trait, sea-orm-sql_err-portable-gate, bifurcated-identity]
key_files:
  created:
    - framework/src/validation/constraint_map.rs
  modified:
    - framework/src/validation/mod.rs
    - framework/src/lib.rs
decisions:
  - "`SqlxError` re-export from sea_orm used in place of `sqlx::Error` (sqlx is not a direct dependency — accessed via sea_orm::SqlxError)"
metrics:
  duration: "176s"
  completed: "2026-06-09"
  tasks_completed: 3
  files_changed: 3
---

# Phase 191 Plan 01: ConstraintMap + MapConstraintExt Summary

## One-Liner

`ConstraintMap` defensive layer: portable UNIQUE-violation detection via `sql_err()` with bifurcated Postgres/SQLite identity, falling through unchanged on any non-match.

## What Was Built

`framework/src/validation/constraint_map.rs` — a new module implementing:

- **`ConstraintMap`** struct with consuming `.on(pg_constraint, field, message)` and `.sqlite(table_col)` builder methods (derives `Clone` + `Default`). Entries hold both Postgres constraint name and optional SQLite `table.column` discriminator so one registration covers both backends.
- **`ConstraintMap::try_map(&self, err: DbErr) -> Result<ValidationError, DbErr>`** — the load-bearing method:
  1. Portable type gate: `err.sql_err()` matching `SqlErr::UniqueConstraintViolation(_)`; all other variants fall through `Err(err)` unchanged.
  2. Postgres identity: `DatabaseError::constraint()` on the `Box<dyn DatabaseError>` trait object — no downcast, no `#[cfg]` guard required.
  3. SQLite identity: `msg.split(": ").nth(1)` on the `UniqueConstraintViolation` payload — defensive, never panics on unexpected format (T-191-01 mitigation).
  4. First-match entry lookup: `pg_hit || sqlite_hit`; on match returns `Ok(ValidationError)` with the entry's field + message.
  5. No match: `Err(err)` by move, unchanged (SC2 contract).
- **`MapConstraintExt<T>` trait** on `Result<T, DbErr>` with `map_constraint(map, data, url)` — eliminates closure ladders at the call site, reusing `ValidationError::with_old_input().into_action_error()` (Phase 190 surfacing chain, zero new redirect code).
- **4 inline non-DB unit tests**: `non_unique_dberr_passes_through_unchanged`, `empty_map_passes_through_any_dberr_unchanged`, `builder_chains_on_and_sqlite_without_panic`, `sqlite_no_op_without_prior_on`.

Wired through `framework/src/validation/mod.rs` (module declaration + `pub use`) and `framework/src/lib.rs` (crate-root `pub use` block). Both `ferro_rs::ConstraintMap` and `ferro_rs::MapConstraintExt` resolve at crate root.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `sqlx::Error` not available as top-level name**

- **Found during:** Task 1 first compile attempt
- **Issue:** The match arm `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))` uses `sqlx::Error` which requires `sqlx` as a direct dependency. `sqlx` is only a transitive dep via sea-orm; it is not in `framework/Cargo.toml`.
- **Fix:** sea-orm re-exports `sqlx::error::Error` as `sea_orm::SqlxError` (confirmed in sea-orm-1.1.19/src/error.rs line `pub use sqlx::error::Error as SqlxError`). Replaced `sqlx::Error::Database(e)` with `SqlxError::Database(e)` and added `SqlxError` to the `use sea_orm::{...}` import. No `Cargo.toml` change needed.
- **Files modified:** `framework/src/validation/constraint_map.rs` (import line + pattern)
- **Commits:** c7313de2

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1+2  | c7313de2 | feat(191-01): ConstraintMap struct, builder, try_map, and MapConstraintExt |
| 3    | 8d554581 | feat(191-01): wire ConstraintMap + MapConstraintExt through mod.rs and lib.rs |

## Verification Results

All acceptance criteria verified:

- `cargo build -p ferro-rs` — green
- `cargo test -p ferro-rs --lib validation::constraint_map` — 4/4 tests green
- SC5 audit: `! grep -nE '^[^/]*("pages"|"slug"|"_unique")' framework/src/validation/constraint_map.rs` — clean (consumer literals only in `///` doc-comment lines)
- All 15 grep acceptance criteria: PASS

## Known Stubs

None. The implementation is complete. DB-backed SQLite integration tests (VALID-05 SC3 SQLite, SC4 TOCTOU simulation) are deferred to Plan 02 by plan design — this is not a stub, it is an intentional plan boundary.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The `ConstraintMap` operates entirely within the existing error-handling chain. No new threat surface beyond what is already covered in the plan's threat model.

## Self-Check: PASSED

- `framework/src/validation/constraint_map.rs` — FOUND
- `framework/src/validation/mod.rs` — modified with constraint_map declaration and re-export — FOUND
- `framework/src/lib.rs` — modified with ConstraintMap + MapConstraintExt in pub use block — FOUND
- Commits c7313de2, 8d554581 — FOUND (`git log --oneline -5`)

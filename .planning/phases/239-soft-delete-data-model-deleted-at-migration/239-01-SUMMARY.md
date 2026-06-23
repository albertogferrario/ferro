---
phase: 239-soft-delete-data-model-deleted-at-migration
plan: "01"
subsystem: app/migrations + app/models/entities
tags: [migration, sea-orm, soft-delete, schema, orders]
dependency_graph:
  requires: []
  provides: [deleted_at column on orders table, orders entity Model with deleted_at field]
  affects: [app/src/migrations, app/src/models/entities/orders.rs, app/src/bootstrap.rs, app/src/tests]
tech_stack:
  added: []
  patterns: [sea-orm additive ALTER TABLE migration, nullable timestamp column, Option<String> entity field]
key_files:
  created:
    - app/src/migrations/m20260623_add_deleted_at_to_orders.rs
  modified:
    - app/src/migrations/mod.rs
    - app/src/models/entities/orders.rs
    - app/src/bootstrap.rs
    - app/src/tests/mcp_tenant_isolation.rs
    - app/src/tests/mcp_write_dispatch.rs
    - app/src/tests/single_source.rs
    - app/src/tests/visual_action.rs
decisions:
  - "deleted_at typed as Option<String> to match created_at: String convention in the orders entity"
  - "Column is .timestamp().null() with no default — only valid SQLite ADD COLUMN form without NOT NULL"
  - "DeriveIden enum contains only Table + DeletedAt — not the full column list"
metrics:
  duration: ~25 minutes
  completed: "2026-06-23"
  tasks: 2
  files: 7
---

# Phase 239 Plan 01: Soft-delete Data Model — deleted_at Migration — Summary

**One-liner:** Additive `ALTER TABLE orders ADD COLUMN deleted_at TIMESTAMP NULL` migration + orders entity field, enabling nullable soft-delete substrate across SQLite and Postgres.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Author + register additive deleted_at migration | `a2e38073` | m20260623_add_deleted_at_to_orders.rs, migrations/mod.rs |
| 2 | Sync orders entity Model + fix seed/test fixtures | `0322a9e5` | orders.rs, bootstrap.rs, 4 test files |

## Verification Results

- `cargo build -p app` exits 0 (both tasks)
- `cargo clippy --all --all-targets -- -D warnings` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo test --all-features` exits 0 — all tests pass
- SQLite `db:migrate` applies clean: `DATABASE_URL=sqlite:///tmp/test_239_migrate.db cargo run -p app -- db:migrate` → `Migrations completed successfully!` (exit 0)
- `m20260611_create_orders_table.rs` byte-identical to HEAD (`git diff --quiet` exits 0)
- `grep -c 'm20260623_add_deleted_at_to_orders' app/src/migrations/mod.rs` → 2 (mod line + Box::new line)
- No `not_null` or `.default(` in the new migration file

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `deleted_at` field in existing `OrderActive` struct constructors**

- **Found during:** Task 2 — `cargo clippy --all --all-targets` revealed 8 `OrderActive` struct literals missing `deleted_at` (4 in bootstrap.rs, 1 in each of 4 test files)
- **Issue:** `E0063: missing field 'deleted_at' in initializer of entities::orders::ActiveModel` — adding the field to the Model makes all existing struct-literal constructors fail to compile
- **Fix:** Added `deleted_at: Set(None)` to all 8 `OrderActive` struct literals in:
  - `app/src/bootstrap.rs` (4 seed orders)
  - `app/src/tests/mcp_tenant_isolation.rs` (loop seed)
  - `app/src/tests/mcp_write_dispatch.rs` (loop seed)
  - `app/src/tests/single_source.rs` (loop seed)
  - `app/src/tests/visual_action.rs` (loop seed)
- **Commit:** `0322a9e5`

## Known Stubs

None. The migration and entity field are complete and wired. No placeholders exist in the delivered files.

## Threat Flags

No new threat surface introduced. This plan is schema-only DDL with no new network endpoints, auth paths, or file access patterns. The T-239-MIG-01/02 threats from the plan's threat model are fully mitigated:
- T-239-MIG-01 (nullable no-default): accepted — NULL = not deleted, correct semantics
- T-239-MIG-02 (idempotency): mitigated — unique `20260623` stem, existing create-migration untouched

## Notes

- **Postgres path:** Not verified in this execution environment — recorded as a manual/CI-matrix verification item. The `sea-orm` `ALTER TABLE ... ADD COLUMN ... NULL` form without a default is Postgres-portable; CI Postgres matrix (if configured) will cover this path.
- **`db:sync` vs manual edit:** The PATTERNS.md documents that `ferro db:sync` after migration produces the identical `Option<String>` field. The manual edit was performed because `db:sync` requires a running app with a migrated database, and the field produced is byte-identical to what `db:sync` generates.

## Self-Check

Files created:
- `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` — FOUND
- `.planning/phases/239-soft-delete-data-model-deleted-at-migration/239-01-SUMMARY.md` — FOUND (this file)

Commits:
- `a2e38073` — FOUND (feat(239-01): add deleted_at migration for orders table)
- `0322a9e5` — FOUND (feat(239-01): sync orders entity + seed/test fixtures with deleted_at)

## Self-Check: PASSED

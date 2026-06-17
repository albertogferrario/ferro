---
phase: 233-ferro-payments-crate-polymorphic-billable
plan: "02"
subsystem: ferro-payments
tags: [sea-orm, entity, migration, partial-index, sqlite, payments]
dependency_graph:
  requires: [ferro-payments crate skeleton (Plan 01), ferro-orm (GuardedUpdate)]
  provides: [PaymentIntent entity, payment_intents migration, partial unique index]
  affects: [ferro-payments/src/intent/, ferro-payments/src/migration/, ferro-payments/src/lib.rs]
tech_stack:
  added: [sea-orm-migration (cross-backend DDL), DeriveEntityModel, DeriveIden, execute_unprepared, tokio (dev)]
  patterns: [manual MigrationName impl, cross-backend partial unique via execute_unprepared, TestMigrator inline test scaffold]
key_files:
  created:
    - ferro-payments/src/intent/entity.rs
    - ferro-payments/src/migration/m20260617_create_payment_intents.rs
    - ferro-payments/src/migration/mod.rs
  modified:
    - ferro-payments/src/intent/mod.rs
    - ferro-payments/src/intent/status.rs
    - ferro-payments/src/lib.rs
decisions:
  - "Manual MigrationName impl returning 'm20260617_000001_create_payment_intents' (matches ferro-reservation pattern, explicit date key)"
  - "PaymentIntentStatus derives Serialize+Deserialize (required by entity Model which derives both; status.rs updated from Plan 01)"
  - "Partial unique via execute_unprepared branched on get_database_backend() — SQLite/PG get true WHERE-clause partial index; MySQL gets stored generated column + plain UNIQUE"
  - "metadata uses .json().null() (not json_binary) — no btree index on metadata in this phase (D-07 / Pitfall 2)"
  - "Empty Relation enum on entity — no FK on tenant_id/billable_id per D-08"
metrics:
  duration_minutes: 4
  completed_date: "2026-06-17"
  tasks_completed: 2
  tasks_total: 2
  files_created: 3
  files_modified: 3
---

# Phase 233 Plan 02: PaymentIntent Entity + Migration Summary

**One-liner:** `payment_intents` SeaORM entity (19 columns, no FKs, status as DeriveActiveEnum) + cross-backend migration with `WHERE`-clause partial unique index on `(billable_kind, billable_id)` enforced at the DB level and proven against in-memory SQLite via four inline tests.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | PaymentIntent entity + module/lib re-exports | 00bf2891 | ferro-payments/src/intent/entity.rs, intent/mod.rs, intent/status.rs, lib.rs |
| 2 | Cross-backend migration + partial unique index + inline tests | 22d76410 | ferro-payments/src/migration/m20260617_create_payment_intents.rs, migration/mod.rs, lib.rs |

## Verification Results

- `cargo build -p ferro-payments`: exit 0
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: exit 0 (no warnings)
- `cargo fmt -p ferro-payments -- --check`: exit 0
- `cargo test -p ferro-payments`: 5 passed, 0 failed
  - `status_string_values_round_trip` (from Plan 01)
  - `migration_creates_table_and_indexes`
  - `migration_down_drops_table`
  - `partial_unique_rejects_second_active_row`
  - `partial_unique_allows_new_active_after_release`
- Migration source contains `execute_unprepared` (8 occurrences), `get_database_backend` (1), `WHERE status IN ('reserved','paid')` (1), `CAST(billable_id AS CHAR)` (1)

## Decisions Made

- Manual `impl MigrationName` with `"m20260617_000001_create_payment_intents"` chosen over `#[derive(DeriveMigrationName)]` for explicit date-keyed naming (mirrors ferro-reservation, clearer when listed in consumer migrators).
- `PaymentIntentStatus` updated to derive `Serialize` + `Deserialize` — the entity `Model` struct derives both, which propagates the requirement to all field types including the status enum. This is a minor additive change to Plan 01's `status.rs`.
- `metadata` column uses `.json().null()` (not `.json_binary()`) — no index planned on this column in phase 233, so `json()` is safe and avoids unnecessary JSONB overhead on Postgres (Pitfall 2).
- Partial unique index name `uq_payment_intents_active` (SQLite/PG path) and `uq_payment_intents_active_mysql` (MySQL path) follow the naming conventions from the design spec.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] PaymentIntentStatus missing Serialize + Deserialize**
- **Found during:** Task 1, first `cargo build`
- **Issue:** Entity `Model` derives `#[derive(Serialize, Deserialize)]`, which propagates Serialize/Deserialize bounds to all field types. `PaymentIntentStatus` (from Plan 01) only derived `EnumIter + DeriveActiveEnum`, causing 4 compile errors.
- **Fix:** Added `Serialize, Deserialize` to the `#[derive(...)]` on `PaymentIntentStatus` in `intent/status.rs` and added `use serde::{Deserialize, Serialize};`.
- **Files modified:** ferro-payments/src/intent/status.rs
- **Commit:** 00bf2891 (included in Task 1 commit)

**2. [Rule 1 - Bug] rustfmt wrapping in name_exists format! call**
- **Found during:** Task 2, pre-commit `cargo fmt --check`
- **Issue:** The `format!("SELECT name FROM sqlite_master WHERE type='{obj_type}' AND name='{name}'")` string was written across multiple lines; rustfmt collapsed it to one line.
- **Fix:** Ran `cargo fmt -p ferro-payments` to auto-apply.
- **Files modified:** ferro-payments/src/migration/m20260617_create_payment_intents.rs
- **Commit:** 22d76410 (included in Task 2 commit after fix)

## Known Stubs

None — all entity columns are fully defined and the migration creates the complete schema. The `lifecycle` module (Plan 03) and `PaymentService` (Plan 234) are explicitly out of scope for this plan and not stubs here.

## Threat Flags

None beyond what is documented in the plan's threat model:
- T-233-03 (Tampering/Injection): all `execute_unprepared` strings are static migration literals with no user-supplied values — documented inline in the migration source.
- T-233-04 (data integrity): the partial unique index is the DB-level guard; proven by `partial_unique_rejects_second_active_row` and `partial_unique_allows_new_active_after_release`.
- T-233-05 (metadata PII): accepted — convention documented in entity field doc comment.

## Self-Check

- ferro-payments/src/intent/entity.rs: FOUND
- ferro-payments/src/migration/m20260617_create_payment_intents.rs: FOUND
- ferro-payments/src/migration/mod.rs: FOUND
- Commit 00bf2891: FOUND
- Commit 22d76410: FOUND

## Self-Check: PASSED

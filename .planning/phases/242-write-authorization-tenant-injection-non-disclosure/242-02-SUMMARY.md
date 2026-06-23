---
phase: 242
plan: 02
slug: write-authorization-tenant-injection-non-disclosure
subsystem: framework::write
tags: [crud, tenant, security, sql, non-disclosure]
requirements: [CRUD-05]

dependency_graph:
  requires: [tenant_column derivation in CrudPlan (Plan 01)]
  provides: [execute_crud_plan binds runtime tenant_id when tenant_column is Some]
  affects: [dispatch_write (calls execute_crud_plan), any caller relying on CRUD SQL output]

tech_stack:
  added: []
  patterns:
    - AND <tenant_column> = ? appended to UPDATE WHERE predicate (Update arm, Delete arm, post-update SELECT)
    - tenant column appended to INSERT col_names + ph_parts (Create arm); placeholder index = columns.len() + 1 because created_at is a SQL literal that does not consume a slot
    - sea_orm::Value::BigInt(Some(tenant_id)) as the bound value at all four injection sites
    - Non-disclosure via existing rows_affected()==0 -> WriteError::RecordNotFound path (D-08); no new error kind

key_files:
  modified:
    - framework/src/write/mod.rs

decisions:
  - D-05: execute_crud_plan binds runtime tenant_id when tenant_column is Some; three SQL verbs and post-update SELECT are all updated
  - D-08: cross-tenant predicate produces 0 rows -> existing RecordNotFound path; no new error variant or code path added
  - "Create arm: created_at is a SQL literal (NOT a bound parameter), so the tenant placeholder index is columns.len() + 1, not +2"
  - "Pitfall 5 (post-update SELECT): SELECT after UPDATE also carries AND <tenant_column> = ? to prevent a concurrent cross-tenant race from returning foreign data"

metrics:
  duration_seconds: 678
  completed_date: "2026-06-24"
  tasks_completed: 2
  files_modified: 1
---

# Phase 242 Plan 02: Bind tenant_id into execute_crud_plan SQL Summary

`execute_crud_plan` now enforces tenant isolation at the SQL level. When `tenant_column` is `Some`, the runtime `tenant_id` is injected as a bound parameter into the INSERT for Create and into the WHERE predicate for Update, Delete, and the post-update SELECT. Cross-tenant and soft-deleted targets collapse to the existing `WriteError::RecordNotFound` path with no new error kind.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Bind tenant_id into execute_crud_plan SQL (Create/Update/Delete + post-update SELECT) | 14e8c6e7 | framework/src/write/mod.rs |
| 2 | sqlite-in-memory dispatch tests — injection, non-disclosure, happy path | f7874385 | framework/src/write/mod.rs |

## What Was Built

### Task 1: Implementation

Three changes to `execute_crud_plan` in `framework/src/write/mod.rs`:

**Signature:** `_tenant_id: i64` renamed to `tenant_id: i64`; all three match arms changed from `tenant_column: _` to `tenant_column` so the value is bound, not ignored.

**Create arm** (lines ~287-363): when `tenant_column` is `Some`, appends the column name to `col_names`, appends `placeholder(backend, columns.len() + 1)` to `ph_parts` (critical: `created_at` is a SQL literal that does NOT consume a placeholder index), and pushes `sea_orm::Value::BigInt(Some(tenant_id))` to `values`.

**Update arm** (lines ~366-424): when `tenant_column` is `Some`, extends the WHERE clause with `AND <tenant_column> = ?` at index `patch.len() + 2` (id is `patch.len() + 1`), and pushes `tenant_id` to values. The **post-update SELECT** also carries the tenant predicate at index 2 (Pitfall 5 mitigation: prevents a concurrent cross-tenant reassignment race from returning foreign data between UPDATE and SELECT).

**Delete arm** (lines ~427-510): when `tenant_column` is `Some`, extends the WHERE clause with `AND <tenant_column> = ?` at index 2 (id is 1), and pushes `tenant_id` to values.

**Non-disclosure mechanism (D-08):** the existing `rows_affected() == 0 → Err(WriteError::RecordNotFound)` checks in Update and Delete are the sole non-disclosure path. The tenant predicate makes a foreign-tenant row produce 0 affected rows, which falls through to `RecordNotFound` — indistinguishable from a genuinely missing row. No new error variant or code path added.

### Task 2: Tests

Five new `#[tokio::test]` functions in `mod tests` (require `--all-features` to run due to `confirmation` feature):

- `crud_create_injects_tenant`: CREATE with `tenant_id=7` → raw SELECT confirms the row's `tenant_id` column == 7.
- `crud_update_tenant_predicate`: same-tenant update (row tenant=7, call tenant_id=7) → Ok, status mutated.
- `crud_delete_tenant_predicate`: same-tenant delete (row tenant=7, call tenant_id=7) → Ok, `deleted_at` set.
- `crud_cross_tenant_update_not_found`: row `tenant_id=2`, call `tenant_id=7` → `Err(RecordNotFound)`; post-call SELECT asserts status is **unchanged** (non-leakage verified).
- `crud_cross_tenant_delete_not_found`: row `tenant_id=2`, call `tenant_id=7` → `Err(RecordNotFound)`; post-call SELECT asserts `deleted_at` is **still NULL** (non-leakage verified).

The existing `crud_update_soft_deleted_not_found` continues to pass, confirming the same `RecordNotFound` envelope covers all three non-disclosure cases (cross-tenant, soft-deleted, missing).

All 22 `write::tests` pass (5 new + 17 pre-existing).

## Verification

```
cargo test -p ferro-rs --lib --all-features -- write::tests
# 22 tests: 22 passed, 0 failed

cargo clippy -p ferro-rs --all-features --all-targets -- -D warnings
# Finished (no warnings)

cargo fmt --all -- --check
# (no diff)
```

Placeholder-index audit (all correct):
- Create tenant: `columns.len() + 1` (created_at is literal, does not shift the index)
- Update tenant: `patch.len() + 2` (id is `patch.len() + 1`)
- Update post-SELECT tenant: `2` (id is `1`)
- Delete tenant: `2` (id is `1`)

## Deviations from Plan

None — plan executed exactly as written. The TDD structure was split (Task 1 = implementation, Task 2 = tests) per plan design rather than a pure RED/GREEN cycle, because the implementation must exist for the SQL-in-memory tests to have anything to execute.

## Known Stubs

None. `tenant_id` is fully bound into the SQL for all three verbs when `tenant_column` is `Some`. The non-disclosure path is complete (uses the existing `RecordNotFound` envelope). Plans 03 and 04 cover the upstream authorization gate (`write_authorized`) and the app-level host wiring.

## Threat Flags

No new threat surface. The mitigation for T-242-02 (tenant column injection) and T-242-03 (information disclosure via distinct cross-tenant error) is complete:
- T-242-02: tenant_id is a bound `sea_orm::Value::BigInt`, never string-interpolated.
- T-242-03: cross-tenant rows produce 0 affected rows → `RecordNotFound`; no distinct signal; tests assert the row is left untouched.
- Post-update SELECT race (Pitfall 5): SELECT also carries `AND <tenant_column> = ?`, closing the concurrent reassignment window.

## Self-Check: PASSED

- `framework/src/write/mod.rs` — modified, present
- Commit `14e8c6e7` exists (implementation)
- Commit `f7874385` exists (tests)
- `grep -c "_tenant_id" mod.rs` = 0
- `grep -c "tenant_column: _" mod.rs` = 0
- `grep -c "sea_orm::Value::BigInt(Some(tenant_id))" mod.rs` = 8 (>= 4 required)
- `grep -c "placeholder(backend, columns.len() + 1)" mod.rs` = 1 (the Create tenant index)
- `cargo test -p ferro-rs --lib --all-features -- write::tests` = 22 passed, 0 failed
- `cargo clippy -p ferro-rs --all-features --all-targets -- -D warnings` = no warnings

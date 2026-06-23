---
phase: 239-soft-delete-data-model-deleted-at-migration
plan: "03"
subsystem: ferro-mcp-server/dispatch
tags: [soft-delete, dispatch, predicate-injection, security, SC#3]

dependency_graph:
  requires: ["239-02"]
  provides: ["soft-delete IS NULL predicate in dispatch", "resolved_table() wired"]
  affects: ["ferro-mcp-server/src/dispatch.rs"]

tech_stack:
  added: []
  patterns:
    - "IS NULL predicate injected into shared WHERE clause with no idx increment"
    - "Explicit soft_delete_column.is_some() gate — not service.deletable"
    - "resolved_table() replaces inline format! derivation"

key_files:
  modified:
    - ferro-mcp-server/src/dispatch.rs

decisions:
  - "Gate on soft_delete_column.is_some() (explicit opt-in only) — not deletable, not unconditional"
  - "IS NULL pushes no bound value and does not increment idx — LIMIT/OFFSET indices unchanged"
  - "resolved_table() called directly; inline format! + TODO removed (D-08)"
  - "soft_delete_excluded test uses its own in-memory DB; setup_orders_db() is untouched"

metrics:
  duration: "~22 minutes"
  completed: "2026-06-23"
  tasks_completed: 2
  files_modified: 1
---

# Phase 239 Plan 03: Soft-delete dispatch predicate + SC#3 test Summary

**One-liner:** `deleted_at IS NULL` gated predicate injected into the shared WHERE clause in `dispatch()`, with `resolved_table()` wired and the SC#3 unit test proving exclusion by construction for both row set and total count.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Wire resolved_table() + inject deleted_at IS NULL predicate | `af15216a` | `ferro-mcp-server/src/dispatch.rs` |
| 2 | Add soft_delete_excluded unit test (SC#3 / T-239-02) | `6e6a2f81` | `ferro-mcp-server/src/dispatch.rs` |
| - | Fix cargo fmt (assert_eq! inline style) | `66f5e06b` | `ferro-mcp-server/src/dispatch.rs` |

## What Was Built

**Task 1 — Production edits in `dispatch()`:**

1. Replaced the inline `format!("{}s", service.name.to_lowercase())` table derivation (and its TODO comment) with `service.resolved_table()` — behavior is identical for existing projections; the accessor now owns the defaulting logic.

2. Injected the soft-delete predicate block after the tenant predicate block, before `where_str` assembly:
   ```rust
   if service.soft_delete_column.is_some() {
       let col = service.resolved_soft_delete_column();
       where_clauses.push(format!("\"{col}\" IS NULL"));
       // No values.push() — IS NULL takes no bound parameter.
       // idx is NOT incremented: LIMIT/OFFSET placeholders keep correct indices on Postgres.
   }
   ```
   Because `where_clauses` is assembled into `where_str` before both the COUNT and DATA queries, the predicate covers both automatically — soft-deleted rows are invisible to both the row oracle and the count oracle.

**Task 2 — `soft_delete_excluded` test (SC#3):**

Self-contained in-memory SQLite DB with `deleted_at TEXT NULL` column. Seeds 1 active row (Alice, `deleted_at NULL`) and 1 soft-deleted row (Bob, `deleted_at = '2026-06-23 12:00:00'`). Asserts `result.rows.len() == 1`, `rows[0]["customer_name"] == "Alice"`, and `result.total == 1`. The existing `setup_orders_db()` helper is unchanged.

## Decisions Made

- **Gate signal is `soft_delete_column.is_some()`** — the explicit `.soft_delete_column(...)` opt-in only. Not `service.deletable` (a projection can be `.deletable(true)` on a table that has no soft-delete column → SQL error). Not unconditional (projections without the column would SQL-error).
- **No `values.push()` and no `idx += 1`** after the IS NULL block — `IS NULL` is a literal clause with no bound parameter; incrementing `idx` would shift LIMIT/OFFSET from `$N/$N+1` to `$N+1/$N+2` on Postgres.
- **`resolved_soft_delete_column()`** called (not a hardcoded `"deleted_at"`) — a projection with `.soft_delete_column("removed_at")` gets the correct column name.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed `uninlined_format_args` clippy lint**
- **Found during:** Task 2, clippy gate
- **Issue:** `format!("\"{}\" IS NULL", col)` triggers `clippy::uninlined_format_args` under `-D warnings`
- **Fix:** Changed to `format!("\"{col}\" IS NULL")`
- **Files modified:** `ferro-mcp-server/src/dispatch.rs`
- **Commit:** `6e6a2f81` (included in Task 2 commit)

**2. [Rule 1 - Style] Fixed `cargo fmt` formatting in test**
- **Found during:** Phase gate `cargo fmt --all -- --check`
- **Issue:** Multi-line `assert_eq!(result.total, 1, ...)` was reformatted by rustfmt to inline
- **Fix:** `cargo fmt --all` applied; committed separately as `66f5e06b`

## Verification

- `cargo test -p ferro-mcp-server soft_delete` → `test dispatch::tests::soft_delete_excluded ... ok`
- `cargo test -p ferro-mcp-server` → 33 unit + 14 integration tests, all green
- `cargo clippy -p ferro-mcp-server --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean
- `cargo clippy --all --all-targets -- -D warnings` → full workspace clean
- TODO `ServiceDef.table` removed; `resolved_table()` at line 122; IS NULL block at line 173 uses `resolved_soft_delete_column()`, gated on `soft_delete_column.is_some()` only, no `values.push`, no `idx` increment.

## Known Stubs

None. The predicate is fully wired and regression-pinned by the unit test.

## Threat Flags

None. The changes close T-239-02 (soft-deleted row visibility) and T-239-COL-01 (SQL error on missing column) per the plan's threat register. No new network endpoints or trust boundaries introduced.

## Self-Check: PASSED

- `ferro-mcp-server/src/dispatch.rs` contains `let table = service.resolved_table();` ✓
- `ferro-mcp-server/src/dispatch.rs` contains `if service.soft_delete_column.is_some() {` ✓
- `ferro-mcp-server/src/dispatch.rs` contains `async fn soft_delete_excluded()` ✓
- Commits `af15216a`, `6e6a2f81`, `66f5e06b` exist in git log ✓
- All 33+14 ferro-mcp-server tests green ✓
- Full workspace clippy clean ✓

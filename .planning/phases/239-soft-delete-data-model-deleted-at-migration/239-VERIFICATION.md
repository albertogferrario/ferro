---
phase: 239-soft-delete-data-model-deleted-at-migration
verified: 2026-06-23T00:00:00Z
status: human_needed
score: 4/4
overrides_applied: 0
human_verification:
  - test: "Run `db:migrate` against a live Postgres instance after applying Phase 239 commits. Confirm the migration applies clean and the `deleted_at` column is nullable (no NOT NULL constraint, no default) on the `orders` table."
    expected: "Migration completes with exit 0; `\information_schema.columns` shows `deleted_at` with `is_nullable = YES` and `column_default = NULL` in Postgres."
    why_human: "No Postgres instance is available in this execution environment. The SQLite path was verified by the executor (exit 0). The sea-orm `ColumnDef::new(...).timestamp().null()` form is documented as backend-portable, but Postgres application cannot be confirmed programmatically here."
---

# Phase 239: Soft-delete Data Model Verification Report

**Phase Goal:** Establish the soft-delete data substrate every CRUD read/update/delete path depends on — a nullable `deleted_at` column on soft-deletable tables plus the `field->column` binding the kernel needs — so a deleted row becomes invisible by construction rather than by ad-hoc filtering. `created_at`-on-create and the tenant-column-as-server-injected contract are fixed here at the data layer.

**Verified:** 2026-06-23
**Status:** human_needed (4/4 automated truths verified; 1 human item: Postgres migration path)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A backend-portable migration adds a nullable `deleted_at` to the soft-deletable table(s); a fresh `db:migrate` applies clean on SQLite | VERIFIED | `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` exists with `.timestamp().null()`, no `.not_null()`, no `.default(`. Registered last in `app/src/migrations/mod.rs` as `Box::new(m20260623_add_deleted_at_to_orders::Migration)`. Executor confirmed `DATABASE_URL=sqlite:///tmp/test_239_migrate.db cargo run -p app -- db:migrate` exits 0. Original `m20260611_create_orders_table.rs` is unmodified (append-only invariant intact). |
| 1b | Postgres `db:migrate` path applies clean | HUMAN NEEDED | Cannot verify without a live Postgres instance. Classified as human_needed (see section below). |
| 2 | The `table()`/`soft_delete_column()` binding resolves a projection's field set to its concrete columns (default `deleted_at`, explicit override honored) | VERIFIED | `resolved_table()` at `ferro-projections/src/service.rs:215` — default `format!("{}s", self.name.to_lowercase())`, explicit via `self.table`. `resolved_soft_delete_column()` at line 223 — default `"deleted_at"`, explicit via `self.soft_delete_column.as_deref()`. 5 table tests confirmed passing live: `resolved_table_default`, `resolved_table_default_lowercases`, `resolved_table_explicit_override`, `resolved_soft_delete_column_default`, `resolved_soft_delete_column_explicit_override`. `cargo test -p ferro-projections resolved_` → 5 passed. |
| 3 | A row with a non-null `deleted_at` is excluded from a baseline read query in a unit test (the `deleted_at IS NULL` predicate is enforced at the data layer, not per-tool) | VERIFIED | `ferro-mcp-server/src/dispatch.rs` line 173: `if service.soft_delete_column.is_some()` gates `where_clauses.push(format!("\"{col}\" IS NULL"))`. No `values.push()`, no `idx += 1`. The WHERE clause is assembled once (line 180) and reused by both COUNT and DATA queries. `soft_delete_excluded` test at line 381 seeds 1 active row + 1 soft-deleted row, asserts `result.rows.len() == 1` and `result.total == 1`. `cargo test -p ferro-mcp-server soft_delete` → `test dispatch::tests::soft_delete_excluded ... ok` (1 passed). The old inline `format!("{}s", service.name.to_lowercase())` and its `TODO: ServiceDef.table` comment are replaced by `service.resolved_table()` at line 122. |
| 4 | `created_at` is set on insert and the tenant column is identified as server-injected (never an agent input) at the schema-derivation boundary | VERIFIED | `created_at` has `DEFAULT current_timestamp` at the DB level in `m20260611_create_orders_table.rs` line 25-28 (`.timestamp().not_null().default(Expr::current_timestamp())`). `is_server_injected_field()` at `ferro-projections/src/service.rs:236` returns `true` for `FieldMeaning::Identifier`, `FieldMeaning::CreatedAt`, and when `field.name == self.tenant_column` (reads `self.tenant_column` dynamically — no hardcoded identity). 4 table tests confirmed passing live: `server_injected_identifier`, `server_injected_created_at`, `server_injected_tenant_column`, `server_injected_false_for_regular_field`. `cargo test -p ferro-projections server_injected` → 4 passed. |

**Score:** 4/4 automated truths verified (1 human item for Postgres path of SC#1)

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` | Additive ALTER TABLE migration adding `deleted_at` to orders | VERIFIED | File exists. Contains `add_column`, `Orders::DeletedAt`, `.timestamp().null()`, `drop_column`. No `.not_null()`, no `.default(`. |
| `app/src/migrations/mod.rs` | Registration of the new migration last in the `migrations()` vec | VERIFIED | `mod m20260623_add_deleted_at_to_orders;` on line 15. `Box::new(m20260623_add_deleted_at_to_orders::Migration)` on line 33 (last entry). `grep -c 'm20260623_add_deleted_at_to_orders' mod.rs` = 2. |
| `app/src/models/entities/orders.rs` | Entity Model with `deleted_at: Option<String>` | VERIFIED | Field present at lines 19-20: `#[sea_orm(column_name = "deleted_at")] pub deleted_at: Option<String>,`. |
| `ferro-projections/src/service.rs` | `resolved_table`, `resolved_soft_delete_column`, `is_server_injected_field` accessors + table tests | VERIFIED | All three accessors at lines 215, 223, 236. Tests at lines 2043-2118. |
| `ferro-mcp-server/src/dispatch.rs` | `resolved_table()` wiring + `deleted_at IS NULL` predicate gated on `soft_delete_column.is_some()` + `soft_delete_excluded` test | VERIFIED | `let table = service.resolved_table()` at line 122. IS NULL block at lines 168-178 gated on `service.soft_delete_column.is_some()`. `async fn soft_delete_excluded` at line 381. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `app/src/migrations/mod.rs` | `m20260623_add_deleted_at_to_orders::Migration` | `Box::new(...)` in `migrations()` vec, last entry | WIRED | Line 33 confirmed. |
| `ServiceDef::resolved_table` | `format!("{}s", self.name.to_lowercase())` | default branch when `self.table` is None | WIRED | Line 218: `unwrap_or_else(|| format!("{}s", self.name.to_lowercase()))`. Byte-identical to the prior inline derivation in dispatch.rs. |
| `ServiceDef::is_server_injected_field` | `FieldMeaning::Identifier \| FieldMeaning::CreatedAt` + `tenant_column` match | `matches!` + `self.tenant_column.as_deref()` compare | WIRED | Lines 236-245. No hardcoded column name — reads `self.tenant_column`. |
| `dispatch()` table derivation | `service.resolved_table()` | replaces inline `format!` + TODO removed | WIRED | Line 122. TODO gone (grep confirms 0 matches for `TODO.*ServiceDef.table`). |
| `dispatch()` WHERE assembly | `deleted_at IS NULL` predicate | `where_clauses.push(...)` gated on `soft_delete_column.is_some()` | WIRED | Lines 173-178. Covers both COUNT and DATA queries via shared `where_str`. No `values.push`, no `idx` increment. |

---

## Data-Flow Trace (Level 4)

Not applicable: this phase delivers a data substrate (migration, resolver accessors, classifier predicate), not a component that renders dynamic data. The `soft_delete_excluded` test serves as the data-flow proof — a seeded row with non-null `deleted_at` is verifiably excluded from both `result.rows` and `result.total`.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC#2 resolver — 5 tests | `cargo test -p ferro-projections resolved_` | 5 passed, 0 failed | PASS |
| SC#4 classifier — 4 tests | `cargo test -p ferro-projections server_injected` | 4 passed, 0 failed | PASS |
| SC#3 soft-delete exclusion | `cargo test -p ferro-mcp-server soft_delete` | `dispatch::tests::soft_delete_excluded ... ok` (1 passed) | PASS |

---

## Requirements Coverage

Phase 239 is a foundation phase with no uniquely-owned v1 requirements. The four roadmap success criteria (SC#1–SC#4) are the verification contract, all covered above.

---

## Anti-Patterns Found

No blockers or warnings. Checked:

- Migration: no `TODO`, no `FIXME`, no placeholder comments. Column is `.timestamp().null()` with no default.
- `service.rs` accessors: pure `&self` predicates, no side effects, no hardcoded identity strings in production code.
- `dispatch.rs` soft-delete block: no `values.push()`, no `idx += 1`, gated correctly on `soft_delete_column.is_some()` (not on `deletable`). The old `TODO: ServiceDef.table` comment is fully removed.
- `orders.rs` entity: `deleted_at: Option<String>` field present with correct attribute. All existing seed/test `OrderActive` constructors updated with `deleted_at: Set(None)`.

---

## Human Verification Required

### 1. Postgres migration path (SC#1 partial)

**Test:** On a machine with Postgres running, checkout Phase 239 commits, set `DATABASE_URL=postgres://...` and run `cargo run -p app -- db:migrate` (or the equivalent ferro CLI command). Inspect the `orders` table with `\d orders` (psql) or `information_schema.columns`.

**Expected:** Migration applies cleanly (exit 0). The `deleted_at` column shows `data_type = timestamp without time zone` (or `timestamp`), `is_nullable = YES`, `column_default = NULL`.

**Why human:** No Postgres instance is available in this verification environment. The executor reported the sea-orm `ALTER TABLE ... ADD COLUMN ... TIMESTAMP NULL` form is Postgres-portable and the SQLite path was confirmed clean. A CI Postgres matrix would cover this automatically if configured.

---

## Context Decisions Honored (D-01..D-11)

| Decision | Status | Evidence |
|----------|--------|---------|
| D-01: `deleted_at` on `orders` only | HONORED | Only `m20260623_add_deleted_at_to_orders.rs` created. No other table touched. |
| D-02: New standalone additive migration, never edit shipped migration | HONORED | New file created; `m20260611_create_orders_table.rs` unmodified (6 commits verified). |
| D-03: Register in `mod.rs` after existing `m20260614_*` entries | HONORED | Last entry in `migrations()` vec. |
| D-04: Nullable timestamp, backend-portable | HONORED | `.timestamp().null()` with no default. |
| D-05: `deleted_at IS NULL` in shared WHERE clause builder | HONORED | Lines 168-178 in dispatch.rs, before `where_str` assembly. |
| D-06: Use resolved column name (not hardcoded) | HONORED | `service.resolved_soft_delete_column()` called in predicate block. |
| D-07: `resolved_table()` + `resolved_soft_delete_column()` on `ServiceDef` | HONORED | Both at lines 215 and 223. |
| D-08: Wire `resolved_table()` into dispatch.rs, remove TODO | HONORED | `let table = service.resolved_table()` at line 122; TODO gone. |
| D-09: Table tests for resolver (default vs explicit) | HONORED | 5 tests at lines 2043-2079. |
| D-10: `created_at` via DB `DEFAULT current_timestamp` | HONORED | `m20260611_create_orders_table.rs` line 28: `.default(Expr::current_timestamp())`. |
| D-11: `is_server_injected_field()` classifier (Identifier + CreatedAt + tenant column) | HONORED | Lines 236-245; 4 tests at lines 2081-2118; reads `self.tenant_column` dynamically. |

---

## Gaps Summary

No gaps. All four success criteria are verified against the actual codebase, not just SUMMARY claims. The sole open item is the Postgres path of SC#1, which is a human-verification item (no Postgres instance locally), not a code gap.

---

_Verified: 2026-06-23_
_Verifier: Claude (gsd-verifier)_

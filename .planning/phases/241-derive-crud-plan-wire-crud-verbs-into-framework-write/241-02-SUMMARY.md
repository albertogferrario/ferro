---
phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write
plan: "02"
subsystem: framework::write
tags: [crud, dispatch_write, kernel, sql-executor, soft-delete, confirmation, idempotency, audit]
dependency_graph:
  requires:
    - CrudPlan enum (ferro-projections, Plan 01)
    - dispatch_write kernel (framework/src/write/mod.rs, pre-existing)
    - WriteDispatcher / OverrideFn / idempotency / audit infrastructure (pre-existing)
  provides:
    - execute_crud_plan free function (CREATE / UPDATE / soft-DELETE SQL interpreter)
    - dispatch_write with crud_plan: Option<&CrudPlan> parameter
    - WriteError::CrudVerbNotEnabled and WriteError::RecordNotFound variants
    - 8 sqlite-in-memory CRUD dispatch tests (VALIDATION rows #6–#13)
  affects:
    - ferro-mcp-server (call sites patched with , None; error.rs From impl extended)
    - app (call sites patched with , None in controller + tests)
    - framework (Plan 03 framing call site will pass Some(&plan))
tech_stack:
  added: []
  patterns:
    - sea_orm::Statement::from_sql_and_values parameterized SQL (all values bound, never interpolated)
    - INSERT + last_insert_rowid() for SQLite; INSERT RETURNING * for Postgres
    - crud_plan: Option<&CrudPlan> trailing parameter — None for transitions, Some for CRUD
    - "{channel}.crud.{name}" audit prefix (distinct from ".action." for queryability)
key_files:
  created: []
  modified:
    - framework/src/write/mod.rs
    - ferro-mcp-server/src/error.rs
    - ferro-mcp-server/src/write_dispatch.rs
    - app/src/controllers/visual_action.rs
    - app/src/tests/single_source.rs
    - app/src/tests/visual_action.rs
decisions:
  - RETURNING * avoided for SQLite; used INSERT + last_insert_rowid() + SELECT * instead (portability — safer than relying on sqlite 3.35+ in all envs)
  - RETURNING * used for Postgres (standard; single round-trip)
  - Audit prefix chosen as "{channel}.crud.{name}" (D-08 discretion) for log queryability
  - WriteError::CrudVerbNotEnabled mapped to Error::ActionNotFound in ferro-mcp-server; RecordNotFound mapped to Error::Validation
  - crud_plan placed as the LAST positional parameter so all existing call sites only append ", None"
  - row_to_json() helper added privately in write/mod.rs (i64 → f64 → bool → String fallback chain)
metrics:
  duration_minutes: 45
  tasks_completed: 2
  files_modified: 6
  completed_date: "2026-06-23"
---

# Phase 241 Plan 02: CRUD kernel extension Summary

Generic CRUD SQL executor wired into the single `dispatch_write` kernel: one new `Option<&CrudPlan>` parameter, one `||` seam extension for soft-delete confirmation, one executor-call branch. Guards, idempotency, audit, and override hook run identically for CRUD and transition verbs. 8 sqlite-in-memory tests prove all SQL shapes plus the confirmation gate, soft-delete addressability, override hook reuse, and idempotency.

## What Was Built

### `execute_crud_plan` (framework/src/write/mod.rs)

```rust
async fn execute_crud_plan(
    plan: &CrudPlan,
    _tenant_id: i64,  // unused in 241; Phase 242 binds it for tenant predicates
    db: &DatabaseConnection,
) -> WriteResult<Value>
```

Three SQL shapes, all values bound via `sea_orm::Value` through `Statement::from_sql_and_values`:

| Variant | SQL shape | Returns |
|---------|-----------|---------|
| `Create` | `INSERT INTO {table} ({cols}, created_at) VALUES ({ph}, datetime('now')/NOW())` + `SELECT * WHERE id = last_insert_rowid()` (SQLite) / `RETURNING *` (Postgres) | inserted record with `id` |
| `Update` | `UPDATE {table} SET {patch} WHERE {id_col}=? AND {soft_delete_col} IS NULL` | updated record; `RecordNotFound` if 0 rows |
| `Delete` | `UPDATE {table} SET {soft_delete_col}=datetime('now')/NOW() WHERE {id_col}=? AND {soft_delete_col} IS NULL` | `{"id": …, "deleted": true}`; `RecordNotFound` if 0 rows |

`created_at` is **not** in `CrudPlan::Create.columns` (Plan 01 contract) — the executor injects it as a SQL literal (`datetime('now')` / `NOW()`), never as a bound parameter.

### Private helpers added

- `placeholder(backend, index)` — `?` for SQLite, `$N` for Postgres (copied from dispatch.rs)
- `json_to_sea_value(val)` — serde_json::Value → sea_orm::Value coercion (copied from dispatch.rs)
- `row_to_json(row)` — QueryResult → serde_json::Value (i64 → f64 → bool → String fallback chain)

### `dispatch_write` extension

**Final signature (authoritative for Plan 03 framing call site):**

```rust
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
    crud_plan: Option<&CrudPlan>,   // ← NEW: None for transitions, Some(&plan) for CRUD
) -> WriteResult<Value>
```

Step 3 seam extension:
```rust
#[cfg(feature = "confirmation")]
{
    let is_destructive = action.transition_trigger.is_some()
        || matches!(crud_plan, Some(CrudPlan::Delete { .. }));
    if is_destructive && !is_confirmed {
        return Err(WriteError::ConfirmationRequired(action.name.clone()));
    }
}
```

Step 4 branch:
```rust
let result = if let Some(plan) = crud_plan {
    execute_crud_plan(plan, tenant_id, db).await?
} else {
    (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?
};
```

Step 6 audit prefix:
```rust
let audit_action = if crud_plan.is_some() {
    format!("{channel}.crud.{}", &action.name)   // e.g. "mcp.crud.create_order"
} else {
    format!("{channel}.action.{}", &action.name) // unchanged for transitions
};
```

### New WriteError variants

```rust
#[error("crud verb not enabled: {0}")]
CrudVerbNotEnabled(String),

#[error("record not found or already deleted")]
RecordNotFound,
```

Mapped in `ferro-mcp-server/src/error.rs`:
- `CrudVerbNotEnabled(m)` → `Error::ActionNotFound(m)`
- `RecordNotFound` → `Error::Validation("record not found or already deleted")`

### Call sites patched (all append `, None`)

- `ferro-mcp-server/src/write_dispatch.rs`: lines 220, 532, 942 (test)
- `app/src/controllers/visual_action.rs`: line 71
- `app/src/tests/visual_action.rs`: line 244
- `app/src/tests/single_source.rs`: line 276 (also added `#[cfg(feature = "confirmation")] false,` which was missing)
- `framework/src/write/mod.rs` tests: all 9 existing call sites

### setup_db extension

```rust
db.execute(Statement::from_string(
    DatabaseBackend::Sqlite,
    "CREATE TABLE IF NOT EXISTS orders (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        status TEXT NOT NULL DEFAULT 'draft',
        amount TEXT,
        created_at TEXT DEFAULT (datetime('now')),
        deleted_at TEXT
    )".to_string(),
)).await.expect("create orders table");
```

### 8 CRUD dispatch tests (VALIDATION rows #6–#13)

| Test | VALIDATION # | Proves |
|------|-------------|--------|
| `crud_create_inserts_row` | #6 / SC#1 | INSERT runs; returned record has `id`; row count = 1 |
| `crud_update_patches_row` | #7 / SC#2 | UPDATE changes only supplied fields; unchanged field stays |
| `crud_update_soft_deleted_not_found` | #8 / SC#2+CRUD-03 | UPDATE on soft-deleted row → `RecordNotFound` |
| `crud_delete_sets_deleted_at` | #9 / SC#2+CRUD-03 | `deleted_at` set; row physically present; `deleted:true` returned |
| `crud_deleted_row_hidden_from_list` | #10 / CRUD-03 | `deleted_at IS NULL` filter returns 0 rows after soft-delete |
| `crud_delete_requires_confirmation` | #11 / CRUD-03 | Bare delete (is_confirmed=false) → `ConfirmationRequired` |
| `crud_override_replaces_generic` | #12 / SC#3 | Override hook fires after generic create; row still inserted |
| `crud_create_idempotent` | #13 / CRUD-06 | Second create with same key returns stored result; 1 row in DB |

## Deviations from Plan

### Auto-fix: SQLite INSERT strategy

The plan said "try RETURNING * first; fall back if tests fail." SQLite's `RETURNING` support requires 3.35.0+ but the workspace SQLite version in tests may vary. Chose the safer path proactively: use INSERT + `last_insert_rowid()` + SELECT for SQLite, `RETURNING *` for Postgres. No test failures occurred; this was a preemptive correctness choice, not a reaction to failure.

### Auto-fix: ferro-mcp-server/src/error.rs From impl

The plan listed only `framework/src/write/mod.rs` as a modified file. The two new `WriteError` variants required extending the `From<WriteError>` impl in `ferro-mcp-server/src/error.rs` to keep the match exhaustive. Added as Rule 3 (blocking issue — compile error).

### Auto-fix: app call sites

The plan listed only `framework/src/write/mod.rs`. The `dispatch_write` signature change required patching 3 additional call sites in `app/`. Added as Rule 3 (blocking issue). Also discovered `app/src/tests/single_source.rs` was calling `dispatch_write` without the `#[cfg(feature = "confirmation")] is_confirmed` parameter — added it.

## Known Stubs

None. `execute_crud_plan` is a complete implementation for Phase 241 scope. The `_tenant_id` parameter is intentionally unused (D-09 — Phase 242 fills the tenant predicate slot).

## Threat Flags

No new security surfaces beyond those declared in the plan's threat model.

All five STRIDE threats from plan 02:
- T-241-05 (SQL injection): all values bound via `sea_orm::Value`; column/table identifiers from `CrudPlan` only — confirmed by grep (`DELETE FROM` = 0, all SQL uses `Statement::from_sql_and_values`)
- T-241-06 (soft-deleted row addressability): Update and Delete both emit `AND {soft_delete_column} IS NULL`; proven by `crud_update_soft_deleted_not_found` and `crud_deleted_row_hidden_from_list`
- T-241-07 (delete without confirmation): seam extension proven by `crud_delete_requires_confirmation`
- T-241-08 (sensitive field in audit): Plan 01 excluded Sensitive fields from `CrudPlan::Create.columns`; Delete returns minimal `{id, deleted}` object
- T-241-09 (cross-tenant idempotency replay): reused unchanged (scoped by `(tenant_id, idempotency_key)`)

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `framework/src/write/mod.rs` | FOUND |
| `ferro-mcp-server/src/error.rs` | FOUND |
| `ferro-mcp-server/src/write_dispatch.rs` | FOUND |
| `app/src/controllers/visual_action.rs` | FOUND |
| `app/src/tests/single_source.rs` | FOUND |
| `app/src/tests/visual_action.rs` | FOUND |
| Commit `c243a431` | FOUND |
| `grep -c "fn dispatch_write" framework/src/write/mod.rs` == 1 | CONFIRMED |
| `grep -c "async fn execute_crud_plan" framework/src/write/mod.rs` == 1 | CONFIRMED |
| `grep -c "DELETE FROM" framework/src/write/mod.rs` == 0 | CONFIRMED |
| `grep -c "IS NULL" framework/src/write/mod.rs` >= 2 | CONFIRMED (11) |
| All 8 crud_ tests pass | CONFIRMED |
| `cargo fmt --all -- --check` exits 0 | CONFIRMED |
| `cargo clippy -p ferro-rs --all-targets -- -D warnings` exits 0 | CONFIRMED |
| `cargo clippy -p ferro-mcp-server -p app --all-targets -- -D warnings` exits 0 | CONFIRMED |
| `cargo test -p ferro-rs --all-features` 612 pass, 0 fail | CONFIRMED |

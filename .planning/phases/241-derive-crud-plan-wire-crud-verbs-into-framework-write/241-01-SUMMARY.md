---
phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write
plan: "01"
subsystem: ferro-projections
tags: [crud, projections, executor, schema-only, derive]
dependency_graph:
  requires: []
  provides:
    - CrudPlan enum (schema-only, serde_json::Value payloads)
    - CrudVerb enum (Create/Update/Delete)
    - TenantColumn struct (tenant-predicate extension point for Phase 242)
    - derive_crud_plan pure function
    - VerbNotEnabled error variant
  affects:
    - framework (Plan 02 consumes CrudPlan to generate SQL)
    - ferro-mcp-server (Plan 03 calls derive_crud_plan)
tech_stack:
  added: []
  patterns:
    - TransitionPlan / derive_transition_plan analog (same file, same shape)
    - schema-only boundary (serde_json::Value, no sea-orm in ferro-projections)
    - thiserror variant style (VerbNotEnabled)
key_files:
  created: []
  modified:
    - ferro-projections/src/executor.rs
    - ferro-projections/src/error.rs
    - ferro-projections/src/lib.rs
decisions:
  - CrudPlan derives PartialEq but NOT Eq (serde_json::Value is not Eq due to floats); CrudVerb and TenantColumn keep Eq
  - created_at omitted from CrudPlan::Create.columns; executor injects server-side (no magic sentinel)
  - tenant_column: Option<TenantColumn> = None on all variants (D-09 extension point for Phase 242)
  - Status = initial_state pushed into columns by derive_crud_plan when StateMachine declared; excluded from user inputs via is_write_excluded_field(field, has_sm)
  - CrudVerb placed in ferro-projections (same module as TransitionPlan); dependency direction preserved (framework depends on ferro-projections, not reverse)
metrics:
  duration_minutes: 4
  tasks_completed: 2
  files_modified: 3
  completed_date: "2026-06-23"
---

# Phase 241 Plan 01: CrudPlan + derive_crud_plan Summary

Pure, serializable CRUD write plan type and derivation function added to `ferro-projections`, mirroring the `TransitionPlan`/`derive_transition_plan` convention exactly — schema-only, no sea-orm, no I/O.

## What Was Built

### Types added to `ferro-projections/src/executor.rs`

**`CrudVerb`** — `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]`
```
pub enum CrudVerb { Create, Update, Delete }
```
`Eq` survives because CrudVerb holds no `serde_json::Value`.

**`TenantColumn`** — `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]`
```
pub struct TenantColumn { pub column: String }
```
Tenant-predicate extension point; Phase 242 sets `Some(TenantColumn { column })` without reworking the plan struct.

**`CrudPlan`** — `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]`
Note: **`Eq` was dropped** because `serde_json::Value` is not `Eq` (floats). `PartialEq` is retained, which is sufficient for test assertions and structural equality checks.

Variant field shapes (authoritative for Plan 02 executor and Plan 03 framing):

```rust
CrudPlan::Create {
    table: String,
    columns: Vec<(String, serde_json::Value)>,  // user fields + Status=initial (if SM); NO created_at
    tenant_column: Option<TenantColumn>,          // None in Phase 241
}

CrudPlan::Update {
    table: String,
    id_column: String,
    id_value: serde_json::Value,
    patch: Vec<(String, serde_json::Value)>,     // only supplied writable fields (patch semantics)
    soft_delete_column: String,                   // always present; executor emits AND col IS NULL
    tenant_column: Option<TenantColumn>,          // None in Phase 241
}

CrudPlan::Delete {
    table: String,
    id_column: String,
    id_value: serde_json::Value,
    soft_delete_column: String,                   // executor sets col = datetime('now')/NOW()
    tenant_column: Option<TenantColumn>,          // None in Phase 241
}
```

### created_at contract (chosen: omit-from-plan)

`created_at` is **not** present in `CrudPlan::Create.columns`. The framework executor (Plan 02) injects it as a server-side SQL literal (`datetime('now')` on SQLite, `NOW()` on Postgres). This keeps the plan free of magic sentinels and matches DB DEFAULT behavior. Wave 2 (Plan 02) must honor this contract.

### VerbNotEnabled error

Added `VerbNotEnabled(String)` to `ferro-projections/src/error.rs` following the thiserror style. Message: `"crud verb not enabled for service: {0}"`. Returned when `svc.creatable`/`updatable`/`deletable` is false for the requested verb.

### derive_crud_plan function

```rust
pub fn derive_crud_plan(
    svc: &crate::ServiceDef,
    verb: CrudVerb,
    inputs: &serde_json::Value,
) -> Result<CrudPlan, crate::Error>
```

Pure, side-effect-free, no I/O. Reuses:
- `svc.resolved_table()` — backing table name
- `svc.resolved_soft_delete_column()` — soft-delete column (default `"deleted_at"`)
- `svc.is_write_excluded_field(field, has_sm)` — column-set gate (Identifier, CreatedAt, UpdatedAt, Sensitive, list, Status-when-SM)

Column selection gates (T-241-01 / T-241-02 mitigations): `is_write_excluded_field` excludes Sensitive and server-injected fields so they never enter the derived plan.

### lib.rs re-exports

```rust
pub use executor::{
    derive_crud_plan, derive_transition_plan, CrudPlan, CrudVerb, TenantColumn, TransitionPlan,
};
```

## Tests (6 new tests, all passing)

| Test | Validates |
|------|-----------|
| `derive_crud_plan_create` | Create plan: correct columns, Status=initial (SM), id/created_at excluded, tenant_column=None |
| `derive_crud_plan_create_no_sm_status_included` | Without SM, Status IS writable (exclude_sm_status=false path) |
| `derive_crud_plan_update` | Update plan: patch semantics, soft_delete_column="deleted_at", Status excluded with SM, tenant_column=None |
| `derive_crud_plan_delete` | Delete plan: soft_delete_column="deleted_at", tenant_column=None |
| `derive_crud_plan_verb_not_enabled` | All three verbs return VerbNotEnabled when flags are false |
| `crud_plan_serde_round_trip` | All three variants serialize/deserialize cleanly with assert_eq! |

Total: 283 tests pass (all existing + 6 new).

## Deviations from Plan

None — plan executed exactly as written.

The plan noted "DROP `Eq` from the CrudPlan derive line if it fails to compile" — this was applied proactively (serde_json::Value is not Eq, as documented in RESEARCH.md). `CrudVerb` and `TenantColumn` retain `Eq` as specified.

## Known Stubs

None. This plan is schema-only with no UI rendering or data source wiring.

## Threat Flags

None. All four STRIDE threats (T-241-01 through T-241-04) from the plan's threat model are addressed:
- T-241-01/T-241-02: `is_write_excluded_field` gate excludes Sensitive/server-injected fields from derived columns (asserted by `derive_crud_plan_create` test)
- T-241-03: `is_server_injected_field` excludes tenant column; `derive_crud_plan` never reads `inputs["tenant_id"]`; `tenant_column: None` in all plans
- T-241-04: `CrudPlan::Update` and `Delete` unconditionally carry `soft_delete_column` (asserted by `derive_crud_plan_update` and `derive_crud_plan_delete` tests)

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `ferro-projections/src/executor.rs` | FOUND |
| `ferro-projections/src/error.rs` | FOUND |
| `ferro-projections/src/lib.rs` | FOUND |
| `241-01-SUMMARY.md` | FOUND |
| Commit `140fa95f` | FOUND |
| 283 tests pass | CONFIRMED |
| `cargo fmt --all -- --check` exits 0 | CONFIRMED |
| `cargo clippy -p ferro-projections --all-targets -- -D warnings` exits 0 | CONFIRMED |
| `grep -c "sea_orm" executor.rs` == 0 | CONFIRMED |

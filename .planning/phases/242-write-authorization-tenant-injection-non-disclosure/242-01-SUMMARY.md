---
phase: 242
plan: 01
slug: write-authorization-tenant-injection-non-disclosure
subsystem: ferro-projections
tags: [crud, tenant, derivation, security]
requirements: [CRUD-05]

dependency_graph:
  requires: []
  provides: [tenant_column derivation in CrudPlan for all three CRUD verbs]
  affects: [framework::write::execute_crud_plan (Plan 02 consumer)]

tech_stack:
  added: []
  patterns:
    - svc.tenant_column.as_ref().map(|col| TenantColumn { column: col.clone() }) at all three CrudPlan construction sites

key_files:
  modified:
    - ferro-projections/src/executor.rs

decisions:
  - D-04: derive_crud_plan reads svc.tenant_column (a static projection declaration) to fill tenant_column; never reads from agent inputs
  - D-06: CrudPlan carries only the column NAME; runtime tenant_id is never stored in the serializable plan

metrics:
  duration_seconds: 780
  completed_date: "2026-06-24"
  tasks_completed: 2
  files_modified: 1
---

# Phase 242 Plan 01: Fill tenant_column in derive_crud_plan Summary

`derive_crud_plan` now emits `Some(TenantColumn { column })` for tenant-scoped projections across all three CRUD verbs (Create, Update, Delete), replacing the hardcoded `None` from Phase 241.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| RED | Failing tenant_column tests for all three verbs | deed2315 | ferro-projections/src/executor.rs |
| GREEN | Fill tenant_column in all three derive_crud_plan arms | ab8c63bd | ferro-projections/src/executor.rs |

## What Was Built

Three `CrudPlan` construction sites in `derive_crud_plan` (lines ~244, ~269, ~286 of `executor.rs`) previously hardcoded `tenant_column: None`. Each now reads `svc.tenant_column` via:

```rust
tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn {
    column: col.clone(),
}),
```

This is the Wave 1 prerequisite for Plan 02 (framework kernel tenant injection). The `CrudPlan` carries only the column name — the runtime `tenant_id` value is bound in Plan 02 from auth context (D-06).

## Tests Added

Three new test functions, each asserting both the Some and None cases:

- `derive_crud_plan_create_tenant_column` — Create verb, with/without tenant column
- `derive_crud_plan_update_tenant_column` — Update verb, with/without tenant column
- `derive_crud_plan_delete_tenant_column` — Delete verb, with/without tenant column

Helper: `crud_order_service_with_tenant()` — adds `.tenant_column("tenant_id")` to the existing CRUD order service fixture.

All 6 tenant_column tests pass (3 new + 3 pre-existing service tests).

## Deviations from Plan

None — plan executed exactly as written. The two tasks were collapsed into a single TDD RED/GREEN cycle (tests written first in a single pass, implementation followed).

## Known Stubs

None. The tenant_column slot is now properly filled by derivation. Plan 02 consumes this as input to SQL injection.

## Threat Flags

No new threat surface introduced. The derivation reads only `svc.tenant_column` (a static projection declaration at registration time), never agent inputs. T-242-02 (elevation of privilege via derive path) is fully mitigated: the column name comes from the service schema, not the write payload.

## Self-Check: PASSED

- `ferro-projections/src/executor.rs` — modified, present
- Commit `deed2315` exists (RED: failing tests)
- Commit `ab8c63bd` exists (GREEN: implementation)
- `grep -c "svc.tenant_column.as_ref().map" executor.rs` = 3
- `grep "tenant_column: None" executor.rs` = no output (zero matches)
- `cargo test -p ferro-projections tenant_column` = 6 passed, 0 failed
- `cargo test -p ferro-projections --all-features` = all passed
- fmt clean, clippy clean

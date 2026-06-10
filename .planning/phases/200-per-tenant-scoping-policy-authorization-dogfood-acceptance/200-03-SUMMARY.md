---
phase: 200
plan: "03"
subsystem: app-data-layer
tags: [migrations, sea-orm, multi-tenant, data-substrate]
dependency_graph:
  requires: [200-02]
  provides: [tenants-table, orders-table, users-tenant-fk, Tenant-model, Order-model]
  affects: [200-04, 200-05]
tech_stack:
  added: []
  patterns: [sea-orm-migration, AlterTable, ForeignKey::create, FerroModel-derive]
key_files:
  created:
    - app/src/migrations/m20260611_create_tenants_table.rs
    - app/src/migrations/m20260611_add_tenant_id_to_users.rs
    - app/src/migrations/m20260611_create_orders_table.rs
    - app/src/models/entities/tenants.rs
    - app/src/models/entities/orders.rs
    - app/src/models/tenants.rs
    - app/src/models/orders.rs
  modified:
    - app/src/migrations/mod.rs
    - app/src/models/entities/mod.rs
    - app/src/models/entities/users.rs
    - app/src/models/mod.rs
decisions:
  - "orders columns match projection field names verbatim (id, customer_name, total, status, created_at, tenant_id) per dispatch SELECT * contract"
  - "tenants PK is big_integer (i64) for alignment with TenantContext.id: i64 in framework"
  - "migration registration order: tenants → add_tenant_id_to_users → orders (FK constraint safe)"
metrics:
  duration: "144s"
  completed_date: "2026-06-10"
  tasks_completed: 2
  files_changed: 11
---

# Phase 200 Plan 03: Two-Tenant Data Substrate Summary

Three migrations and five model files that stand up the multi-tenant data layer in the sample `app` (D-07 fixture prerequisite). SC-1 tenant isolation becomes provable once Plans 04/05 wire scoping middleware and dispatch.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Tenants, orders, users.tenant_id migrations + mod.rs | 1b80e3e4 | 3 new migration files + migrations/mod.rs |
| 2 | Tenant/Order entities + wrappers; users.tenant_id; re-exports | 231c166d | entities/tenants.rs, entities/orders.rs, tenants.rs, orders.rs, users.rs, entities/mod.rs, models/mod.rs |

## Verification

`cargo build -p app` produces exactly one error — the known mcp.rs arity mismatch from plan 200-02 (`handle_tools_call` missing 4th argument). No errors from this plan's additions.

All 12 acceptance criteria pass:
- migrations contain correct Iden names (`Tenants::Table`, `Orders::CustomerName`, `ForeignKey::create`, `Users::TenantId`)
- entities carry correct `table_name` attributes and column types
- `orders.tenant_id` is `i64` (NOT NULL); `users.tenant_id` is `Option<i64>` (nullable)
- `Tenant::find_by_slug` and `Tenant::find_by_id` implemented
- all modules registered in `entities/mod.rs` and `models/mod.rs`

Migration registration order in `migrations/mod.rs`:
1. create_users_table
2. create_todos_table
3. create_api_keys_table
4. create_oauth_clients_table
5. **create_tenants_table** ← FK target must exist first
6. **add_tenant_id_to_users** ← alters users after tenants exists
7. **create_orders_table** ← FK to tenants; added last

## Key Design Decisions

1. **orders columns = projection field names verbatim** — dispatch uses `SELECT *` over `format!("{}s", name)` = `orders`; a column name drift (e.g. `name` instead of `customer_name`) would silently break the read path (Pitfall 4 from RESEARCH.md).

2. **tenants.id is `big_integer` / `i64`** — aligns with `TenantContext { id: i64, ... }` in the framework, which `DbTenantLookup` (Plan 04) maps to directly.

3. **FK safe order** — `orders.tenant_id` references `tenants.id`; `tenants` must be created before `orders` migration runs. The `AlterTable` adding `users.tenant_id` runs between them (no FK constraint on users.tenant_id in this migration, nullable column only).

## Deviations from Plan

None — plan executed exactly as written. Migration shapes copied verbatim from PATTERNS.md. Entity shapes match todos.rs analog exactly.

## Threat Surface Scan

T-200-SCHEMA (Tampering): `orders.tenant_id` is NOT NULL with a FK to `tenants.id` — an order cannot exist without a valid tenant. Mitigated as specified.

T-200-COLMATCH (correctness→disclosure): Column names match projection field names verbatim — `customer_name` exactly as declared in `order.rs` lines 15–16. Verified by acceptance grep.

No new threat surface beyond what the plan's threat model covers.

## Known Stubs

None — this plan is pure data layer. No UI components, no rendering paths, no placeholder values in flowing data.

## Self-Check: PASSED

Files created:
- app/src/migrations/m20260611_create_tenants_table.rs: FOUND
- app/src/migrations/m20260611_add_tenant_id_to_users.rs: FOUND
- app/src/migrations/m20260611_create_orders_table.rs: FOUND
- app/src/models/entities/tenants.rs: FOUND
- app/src/models/entities/orders.rs: FOUND
- app/src/models/tenants.rs: FOUND
- app/src/models/orders.rs: FOUND

Commits:
- 1b80e3e4: FOUND (Task 1 — migrations)
- 231c166d: FOUND (Task 2 — models)

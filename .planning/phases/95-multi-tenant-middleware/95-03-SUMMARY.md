---
phase: 95-multi-tenant-middleware
plan: "03"
subsystem: framework/tenant
tags: [multi-tenancy, query-scoping, from-request, documentation]
dependency_graph:
  requires: [95-01, 95-02]
  provides: [TenantScope, TenantContext-FromRequest, public-tenant-api, multi-tenancy-docs]
  affects: [framework/src/tenant/, framework/src/lib.rs, docs/src/]
tech_stack:
  added: []
  patterns: [TDD-RED-GREEN, Scope-trait, FromRequest-trait, task-local-context]
key_files:
  created:
    - framework/src/tenant/scope.rs
    - docs/src/features/multi-tenancy.md
  modified:
    - framework/src/tenant/mod.rs
    - framework/src/lib.rs
    - docs/src/SUMMARY.md
decisions:
  - "TenantScope tests use sea_orm::DbBackend::Sqlite + Statement.values to verify filter without DB connection"
  - "FromRequest tests use TCP loopback (matching middleware test pattern) since Request has no default constructor"
  - "SQL assertion checks Statement.values for BigInt(Some(id)) since SQLite uses ? placeholders, not inlined values"
metrics:
  duration: "~10 minutes"
  completed: "2026-03-11"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 3
---

# Phase 95 Plan 03: TenantScope, FromRequest, Re-exports, and Documentation Summary

TenantScope implementing Scope<E> trait with task-local context reads, TenantContext FromRequest extractor, complete public API re-exports, and 253-line multi-tenancy documentation.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | TenantScope + TenantContext FromRequest | 888215a | framework/src/tenant/scope.rs, framework/src/tenant/mod.rs |
| 2 | Framework re-exports and documentation | dbe8266 | framework/src/lib.rs, docs/src/features/multi-tenancy.md, docs/src/SUMMARY.md |

## What Was Built

### TenantScope (framework/src/tenant/scope.rs)

Generic `TenantScope<C: ColumnTrait>` implementing `Scope<E>` for all SeaORM entities. Reads current tenant from task-local context via `current_tenant()` and applies `.filter(column.eq(tenant_id))`. Panics with a clear message if called outside `TenantMiddleware` scope — intentional, as this is a programming error.

```rust
let posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
    .all()
    .await?;
```

### TenantContext FromRequest (framework/src/tenant/mod.rs)

`#[async_trait] impl FromRequest for TenantContext` reads from task-local context and returns `Ok(ctx)` when a tenant is active, or `Err(FrameworkError::domain(..., 400))` when called outside middleware scope. The request argument is ignored since tenant context is in task-local storage.

```rust
#[handler]
pub async fn dashboard(tenant: TenantContext) -> Response {
    Ok(json!({"tenant": tenant.name}))
}
```

### Framework Re-exports (framework/src/lib.rs)

All tenant types accessible from the crate root:

```rust
pub use tenant::{
    current_tenant, DbTenantLookup, HeaderResolver, JwtClaimResolver, PathResolver,
    SubdomainResolver, TenantContext, TenantFailureMode, TenantLookup, TenantMiddleware,
    TenantResolver, TenantScope,
};
```

### Documentation (docs/src/features/multi-tenancy.md)

253-line guide covering:
- Shared-schema multi-tenancy overview
- Quick start with SubdomainResolver
- All 4 resolver strategies (Subdomain, Header, Path, JWT) with when/why
- Handler extraction via `TenantContext` parameter
- Query scoping with `TenantScope`
- Failure modes table (NotFound/Forbidden/Allow)
- Custom `TenantLookup` implementation
- Safety notes on unscoped queries and cache key prefixing

## Tests Written (TDD)

6 behavioral tests across 2 test modules:

**scope.rs tests:**
- `tenant_scope_apply_adds_tenant_id_filter` — verifies SQL has WHERE + bound value 42
- `tenant_scope_panics_outside_middleware_scope` — verifies panic message
- `tenant_scope_is_generic_over_column_type` — verifies works with any ColumnTrait
- `concurrent_tasks_get_isolated_tenant_scopes` — multi-thread isolation

**mod.rs tests:**
- `from_request_returns_ok_when_tenant_context_is_set` — returns Ok(ctx) inside scope
- `from_request_returns_400_error_when_no_tenant_context` — returns 400 outside scope

Total tenant tests: 38 (was 28 before this plan, +10 new tests across scope and mod)

## Deviations from Plan

### Auto-fixed Issues

**[Rule 1 - Bug] SQL assertion approach for parameterized queries**
- **Found during:** Task 1 test implementation
- **Issue:** SQLite uses `?` placeholders, not inlined values — `sql.contains("42")` was always false
- **Fix:** Used `Statement.values` to inspect bound parameter values as `Value::BigInt(Some(42))` instead of string matching on SQL
- **Files modified:** framework/src/tenant/scope.rs
- **Commit:** 888215a (inline fix before commit)

**[Rule 2 - Missing] Test imports cleanup**
- **Found during:** Task 1 clippy run
- **Issue:** Redundant `DeriveEntityModel`, `DeriveRelation`, `EnumIter`, `EntityTrait as _` imports in test module — `post` sub-module uses `use sea_orm::entity::prelude::*` which covers these
- **Fix:** Removed redundant imports, kept only `DbBackend, QueryTrait, Value`
- **Files modified:** framework/src/tenant/scope.rs
- **Commit:** 888215a (inline fix before commit)

## Self-Check: PASSED

All files found:
- FOUND: framework/src/tenant/scope.rs
- FOUND: framework/src/tenant/mod.rs
- FOUND: framework/src/lib.rs
- FOUND: docs/src/features/multi-tenancy.md
- FOUND: docs/src/SUMMARY.md

All commits found:
- 888215a: feat(95-03): implement TenantScope and TenantContext FromRequest extractor
- dbe8266: feat(95-03): add tenant re-exports and multi-tenancy documentation

Key patterns verified:
- `impl<E, C> Scope<E> for TenantScope<C>` in scope.rs
- `pub use tenant::` in lib.rs
- 38 tenant tests pass, 0 failures

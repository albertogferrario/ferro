---
phase: 95-multi-tenant-middleware
plan: 01
subsystem: middleware
tags: [multi-tenancy, tokio, task-local, moka, cache, async-trait]

# Dependency graph
requires: []
provides:
  - TenantContext struct with id, slug, name, plan fields
  - TenantFailureMode enum (NotFound, Forbidden, Allow, Custom)
  - current_tenant() task-local accessor returning Option<TenantContext>
  - tenant_scope() and with_tenant_scope() for middleware lifecycle
  - TenantLookup trait (object-safe, find_by_slug, find_by_id, invalidate)
  - DbTenantLookup with moka cache (5-min TTL, 10k capacity, pluggable finders)
  - FromRequest impl for TenantContext returning 400 when no middleware
affects:
  - 95-02 (TenantMiddleware implementations that use these traits)
  - 95-03 (re-exports from framework/src/lib.rs)
  - 96-stripe-integration (plan field on TenantContext)
  - 97-tenant-aware-background-jobs (current_tenant() in job context)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - tokio::task_local! for per-request task-local context (mirrors session/lang patterns)
    - Pluggable async finder closures injected into DbTenantLookup for decoupled caching
    - pub(crate) scope helpers with #[allow(dead_code)] for forward-declared utilities

key-files:
  created:
    - framework/src/tenant/mod.rs
    - framework/src/tenant/context.rs
    - framework/src/tenant/lookup.rs
  modified:
    - framework/src/lib.rs

key-decisions:
  - "plan field on TenantContext is Option<String> — nullable until Phase 96 Stripe adds billing plans"
  - "DbTenantLookup takes boxed async finder closures — decouples caching from user DB schema, users inject own query"
  - "tenant_scope() and with_tenant_scope() are pub(crate) with #[allow(dead_code)] — forward-declared for Plan 02 middleware"
  - "TenantResolver::resolve() takes &Request (non-consuming) — allows header/host inspection without consuming body"

patterns-established:
  - "task-local tenant context mirrors session/lang patterns exactly (tokio::task_local! + try_with + try_read + clone)"
  - "DbTenantLookup slug cache key is the slug string; id cache key is i64::to_string()"

requirements-completed: [MT-05, MT-09]

# Metrics
duration: 4min
completed: 2026-03-11
---

# Phase 95 Plan 01: Core Tenant Types Summary

**TenantContext struct with task-local storage, TenantFailureMode enum, TenantLookup trait with moka-cached DbTenantLookup — the complete type foundation for multi-tenant middleware**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-11T00:54:52Z
- **Completed:** 2026-03-11T00:58:37Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- TenantContext struct with id (i64), slug, name, plan (Option<String>) fields — Debug, Clone, Serialize derived
- TenantFailureMode enum: NotFound (404), Forbidden (403), Allow (pass-through), Custom(Box<dyn Fn>) with manual Debug/Clone impls
- TENANT_CONTEXT tokio::task_local! with current_tenant(), tenant_scope(), with_tenant_scope() — exact mirror of session/lang pattern
- TenantLookup trait: object-safe, find_by_slug and find_by_id methods, default no-op invalidate()
- DbTenantLookup: moka cache 5-min TTL, 10k capacity, pluggable async finder closures for user DB queries
- FromRequest impl for TenantContext: reads task-local, returns 400 domain error when not in middleware scope
- 16 unit tests across context, lookup, and mod modules — all pass

## Task Commits

Each task was committed atomically:

1. **Task 1: TenantContext, task-local context, TenantFailureMode** - `3652070` (feat)
2. **Task 2: TenantLookup trait and DbTenantLookup with moka cache** - `d854038` (feat)

**Plan metadata:** `e3fb416` (docs: complete tenant core types plan)

## Files Created/Modified

- `framework/src/tenant/mod.rs` - TenantContext, TenantFailureMode, module declarations, re-exports, FromRequest impl, 2 tests
- `framework/src/tenant/context.rs` - TENANT_CONTEXT task_local, current_tenant(), tenant_scope(), with_tenant_scope(), 7 tests
- `framework/src/tenant/lookup.rs` - TenantLookup trait, DbTenantLookup with moka cache, 7 tests
- `framework/src/lib.rs` - Added `pub mod tenant;` declaration

## Decisions Made

- `plan` field on `TenantContext` is `Option<String>` — many tenants have no billing plan until Stripe integration (Phase 96)
- `DbTenantLookup` takes boxed async closure finders rather than coupling to a specific SeaORM entity — users inject their own DB query while getting caching for free
- `tenant_scope()` and `with_tenant_scope()` are `pub(crate)` with `#[allow(dead_code)]` — they will be consumed by `TenantMiddleware` in Plan 02
- `TenantResolver::resolve()` takes `&Request` (non-consuming) so resolvers can inspect host/headers without consuming the request body

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All foundational types defined and tested — Plan 02 (TenantMiddleware concrete resolvers) can proceed immediately
- `tenant_scope()` and `with_tenant_scope()` ready for middleware implementation
- `TenantLookup` trait object ready for Plan 02 middleware to reference `Arc<dyn TenantLookup>`

---
*Phase: 95-multi-tenant-middleware*
*Completed: 2026-03-11*

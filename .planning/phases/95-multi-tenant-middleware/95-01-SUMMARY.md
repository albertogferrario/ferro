---
phase: 95-multi-tenant-middleware
plan: 01
subsystem: middleware
tags: [multi-tenancy, tokio, task-local, moka, cache, async-trait]

# Dependency graph
requires: []
provides:
  - TenantContext struct with id, slug, name, plan fields
  - TenantFailureMode enum (NotFound, Forbidden, Allow)
  - current_tenant() task-local accessor returning Option<TenantContext>
  - tenant_scope() and with_tenant_scope() for middleware lifecycle
  - TenantResolver trait (object-safe, async resolve from &Request)
  - TenantLookup trait (object-safe, find_by_slug and find_by_id)
  - DbTenantLookup with moka cache (5-min TTL, 10k capacity, pluggable finders)
affects:
  - 95-02 (TenantMiddleware implementations that use these traits)
  - 95-03 (re-exports from framework/src/lib.rs)
  - 95-04 (TenantExtractor uses TenantContext)
  - 96-stripe-integration (plan field on TenantContext)
  - 98-tenant-aware-background-jobs (current_tenant() in job context)

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
    - framework/src/tenant/resolver.rs
    - framework/src/tenant/lookup.rs
  modified:
    - framework/src/lib.rs

key-decisions:
  - "plan field on TenantContext is Option<String> — many tenants have no billing plan until Phase 96 Stripe"
  - "DbTenantLookup takes boxed async finder closures rather than SeaORM entity — decouples caching from user DB schema"
  - "tenant_scope() and with_tenant_scope() marked #[allow(dead_code)] since Plan 02 middleware consumes them"
  - "TenantResolver receives &Request (non-consuming) to allow header/host inspection without consuming the request body"

patterns-established:
  - "task-local tenant context mirrors session/lang patterns exactly (tokio::task_local! + try_with + try_read + clone)"
  - "DbTenantLookup slug cache key is the slug string; id cache key is i64::to_string()"

requirements-completed: [MT-01, MT-02, MT-09]

# Metrics
duration: 4min
completed: 2026-03-11
---

# Phase 95 Plan 01: Core Tenant Types Summary

**TenantContext, task-local context, TenantResolver/TenantLookup traits, and moka-cached DbTenantLookup providing the complete type vocabulary for multi-tenant middleware**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-11T00:54:52Z
- **Completed:** 2026-03-11T00:58:37Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- TenantContext struct with id (i64), slug, name, plan (Option<String>) fields — Serialize-derived
- TenantFailureMode enum: NotFound (404), Forbidden (403), Allow (pass-through)
- TENANT_CONTEXT task_local with current_tenant(), tenant_scope(), with_tenant_scope() — exact mirror of session/lang pattern
- TenantResolver trait: object-safe, async resolve(&Request) -> Option<TenantContext>
- TenantLookup trait: object-safe, find_by_slug and find_by_id methods
- DbTenantLookup: moka cache 5-min TTL, 10k capacity, pluggable async finder closures for user DB queries
- 13 unit tests across context, resolver, and lookup modules — all pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Core types — TenantContext, TenantFailureMode, task-local context** - `3652070` (feat)
2. **Task 2: TenantResolver trait + TenantLookup trait with DbTenantLookup** - `d854038` (feat)

**Plan metadata:** (docs commit below)

_Note: TDD tasks implemented inline — types defined and tests written together since types were needed to compile tests._

## Files Created/Modified

- `framework/src/tenant/mod.rs` - TenantContext, TenantFailureMode, module declarations and re-exports
- `framework/src/tenant/context.rs` - TENANT_CONTEXT task_local, current_tenant(), tenant_scope(), with_tenant_scope(), 7 tests
- `framework/src/tenant/resolver.rs` - TenantResolver trait definition, 1 object-safety test
- `framework/src/tenant/lookup.rs` - TenantLookup trait, DbTenantLookup with moka cache, 5 tests
- `framework/src/lib.rs` - Added `pub mod tenant;` declaration

## Decisions Made

- `plan` field on `TenantContext` is `Option<String>` — many tenants won't have a billing plan until Stripe integration (Phase 96)
- `DbTenantLookup` takes boxed async closure finders rather than coupling to a specific SeaORM entity — users inject their own DB query while getting caching for free
- `tenant_scope()` and `with_tenant_scope()` are `pub(crate)` with `#[allow(dead_code)]` — they will be consumed by `TenantMiddleware` in Plan 02
- `TenantResolver::resolve()` takes `&Request` (non-consuming) so resolvers can inspect host/headers without consuming the request body

## Deviations from Plan

None - plan executed exactly as written.

Minor fix applied: removed unused `MockResolver` struct from resolver.rs tests (dead_code lint under -D warnings) and added `#[allow(dead_code)]` to `tenant_scope()`/`with_tenant_scope()` which are forward-declared for Plan 02 middleware use. Both are within the deviation rule bounds (Rule 2 — prevent compile warnings from being errors).

## Issues Encountered

- cargo fmt reformatted type alias declarations in lookup.rs (`SlugFinder`, `IdFinder`) — applied formatting before commit
- `pub mod tenant` insertion order in lib.rs was adjusted by rustfmt to alphabetical position

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All foundational types defined and tested — Plan 02 (TenantMiddleware concrete resolvers) can proceed immediately
- `tenant_scope()` and `with_tenant_scope()` ready for middleware implementation
- `TenantResolver` and `TenantLookup` trait objects ready for Plan 02 concrete implementations (SubdomainResolver, HeaderResolver, PathResolver, JwtResolver)

---
*Phase: 95-multi-tenant-middleware*
*Completed: 2026-03-11*

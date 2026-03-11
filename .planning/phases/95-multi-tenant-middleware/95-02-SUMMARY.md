---
phase: 95-multi-tenant-middleware
plan: 02
subsystem: middleware
tags: [multi-tenancy, middleware, async-trait, hyper, jwt, subdomain, header, path]

# Dependency graph
requires:
  - phase: 95-01
    provides: TenantContext, TenantFailureMode, TenantResolver trait, TenantLookup trait, tenant_scope, with_tenant_scope
provides:
  - TenantMiddleware implementing Middleware trait with builder API
  - SubdomainResolver extracting tenant slug from Host header with port stripping and configurable base domain parts
  - HeaderResolver extracting tenant slug from configurable HTTP header
  - PathResolver extracting tenant slug from route path parameter
  - JwtClaimResolver extracting tenant_id from serde_json::Value in request extensions
affects:
  - 95-03 (re-exports from framework/src/lib.rs)
  - 95-04 (TenantExtractor can rely on current_tenant() being set by TenantMiddleware)
  - 96-stripe-integration (middleware pipeline complete for multi-tenant requests)
  - 98-tenant-aware-background-jobs (current_tenant() set by middleware, available in downstream)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Resolver chain pattern: Vec<Box<dyn TenantResolver>> tried in order, first Some wins
    - Task-local tenant context set via tenant_scope/with_tenant_scope during Next call
    - JwtClaimResolver reads serde_json::Value from request extensions (upstream JWT middleware inserts claims)
    - Test requests created via TCP loopback (hyper::server::conn::http1) to get real hyper::body::Incoming

key-files:
  created:
    - framework/src/tenant/middleware.rs
  modified:
    - framework/src/tenant/resolver.rs
    - framework/src/tenant/mod.rs
    - framework/src/http/response.rs

key-decisions:
  - "TCP loopback for test requests: hyper::body::Incoming cannot be constructed without a real connection, so tests use tokio::net::TcpListener pattern matching rate_limit tests"
  - "#[derive(Debug)] added to HttpResponse: required for Result::unwrap()/unwrap_err() in tests — correct correctness fix"
  - "JwtClaimResolver reads serde_json::Value from request extensions: no JWT infrastructure exists in framework, upstream middleware must insert parsed claims"
  - "PathResolver uses req.param().ok() to convert Result to Option: req.param() returns Result<&str, ParamError> not Option<&str>"

patterns-established:
  - "Resolver chain: middleware holds Vec<Box<dyn TenantResolver>>, iterates calling resolve(), breaks on first Some"
  - "Task-local scope lifecycle: tenant_scope() creates Arc<RwLock<Option<TenantContext>>>, write guard sets tenant, with_tenant_scope() wraps next() call"

requirements-completed: [MT-01, MT-04, MT-05, MT-07, MT-08]

# Metrics
duration: 9min
completed: 2026-03-11
---

# Phase 95 Plan 02: TenantMiddleware and Concrete Resolvers Summary

**TenantMiddleware implementing Middleware trait with resolver chain and failure modes, plus four concrete resolver strategies (Subdomain, Header, Path, JWT) each delegating to TenantLookup for DB verification**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-11T01:01:54Z
- **Completed:** 2026-03-11T01:10:46Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- TenantMiddleware with consuming builder API (resolver/on_failure), resolver chain runs in order first match wins, stores tenant in task-local context via with_tenant_scope()
- Three failure modes: NotFound (404 JSON), Forbidden (403 JSON), Allow (pass-through with None tenant)
- SubdomainResolver: extracts subdomain from Host header, strips port, skips requests with only base domain parts
- HeaderResolver: reads configurable header name as slug
- PathResolver: reads named route parameter as slug (handles Result->Option conversion correctly)
- JwtClaimResolver: reads serde_json::Value from request extensions, extracts i64 field as tenant_id
- 20 total new tests (9 middleware + 11 resolver) — all pass

## Task Commits

Each task was committed atomically:

1. **Task 1: TenantMiddleware with resolver chain and failure modes** - `03e8a46` (feat)
2. **Task 2: Four concrete resolver implementations** - `de9cd7b` (feat)

**Plan metadata:** (docs commit below)

_Note: TDD tasks implemented with tests and implementation together — type signatures needed to compile tests_

## Files Created/Modified

- `framework/src/tenant/middleware.rs` - TenantMiddleware struct with Middleware impl, 9 tests
- `framework/src/tenant/resolver.rs` - SubdomainResolver, HeaderResolver, PathResolver, JwtClaimResolver, 11 tests
- `framework/src/tenant/mod.rs` - Added middleware module, re-exported TenantMiddleware and 4 resolver types
- `framework/src/http/response.rs` - Added #[derive(Debug)] to HttpResponse

## Decisions Made

- `PathResolver` uses `req.param(&self.param_name).ok()?` — `req.param()` returns `Result<&str, ParamError>`, not `Option<&str>` as the plan assumed; `.ok()` converts cleanly
- `JwtClaimResolver` reads `serde_json::Value` from request extensions — no existing JWT claims infrastructure in framework; documents that upstream JWT middleware must insert claims
- `#[derive(Debug)]` added to `HttpResponse` — required for `Result::unwrap()` in tests; useful derive that was missing
- Test requests created via TCP loopback — `hyper::body::Incoming` has no default constructor, matching the pattern already used in rate_limit tests

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] HttpResponse lacked Debug impl**
- **Found during:** Task 1 (writing middleware tests)
- **Issue:** `Result::unwrap()`/`unwrap_err()` requires `Debug` on the error type; tests failed to compile
- **Fix:** Added `#[derive(Debug)]` to `HttpResponse` in `framework/src/http/response.rs`
- **Files modified:** `framework/src/http/response.rs`
- **Verification:** Compiler error resolved; all 9 middleware tests pass
- **Committed in:** `03e8a46` (Task 1 commit)

**2. [Rule 1 - Bug] PathResolver: req.param() returns Result not Option**
- **Found during:** Task 2 (implementing PathResolver)
- **Issue:** Plan spec said `req.param(&self.param_name)` returns `Option<&str>` but actual signature is `Result<&str, ParamError>`
- **Fix:** Used `.ok()?` to convert `Result` to `Option` before passing to `find_by_slug`
- **Files modified:** `framework/src/tenant/resolver.rs`
- **Verification:** PathResolver tests 7 and 8 both pass
- **Committed in:** `de9cd7b` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes necessary for correct compilation and behavior. No scope creep.

## Issues Encountered

- `hyper::body::Incoming` cannot be instantiated directly — switched to TCP loopback test request pattern (matching `rate_limit.rs` existing pattern in the codebase)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- TenantMiddleware and all 4 resolver strategies implemented and tested
- Re-exports added to `framework/src/tenant/mod.rs` — ready for Plan 03 to expose via `framework/src/lib.rs`
- `current_tenant()` accessible in downstream handlers when middleware is active
- JwtClaimResolver is documented: upstream JWT middleware must insert `serde_json::Value` claims

---
*Phase: 95-multi-tenant-middleware*
*Completed: 2026-03-11*

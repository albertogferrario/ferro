---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
plan: "04"
subsystem: app-middleware-routing
tags: [tenant-scoping, bearer-auth, gate-authorization, mcp, middleware, seed]
dependency_graph:
  requires: ["200-01", "200-02", "200-03"]
  provides: ["bearer-auth-middleware", "session-user-tenant-resolver", "db-tenant-lookup", "mcp-middleware-stack", "authorize-middleware", "dogfood-seed"]
  affects: ["200-05", "200-06", "200-07"]
tech_stack:
  added: []
  patterns:
    - "OnceLock global for sharing DbTenantLookup between bootstrap and route registration"
    - "BearerAuthMiddleware inserts serde_json::Value; JwtClaimResolver reads same TypeId"
    - "SessionUserTenantResolver reads Auth::id() → User.tenant_id at /authorize time"
    - "Gate::define with downcast_ref pattern for typed ability check"
    - "Idempotent seed guarded by tenants table count check"
key_files:
  created:
    - app/src/middleware/bearer_auth.rs
    - app/src/tenant_resolver.rs
    - app/src/tenant_lookup.rs
  modified:
    - app/src/bootstrap.rs
    - app/src/routes.rs
    - app/src/projections/order.rs
    - app/src/middleware/mod.rs
    - app/src/main.rs
    - app/src/models/users.rs
    - app/src/controllers/mcp.rs
decisions:
  - "OnceLock for global tenant_lookup avoids routes! macro signature changes"
  - "SessionUserTenantResolver reads Auth::id() at /authorize time — JWT doesn't exist yet"
  - "BearerAuthMiddleware expected_tenant=None — TenantMiddleware owns tenant resolution"
  - "TenantFailureMode::Allow on /authorize — unauthenticated visitors must reach login redirect"
  - "TenantFailureMode::Forbidden on /mcp — validated token with unknown tenant = 403, not 404"
  - "find_by_id added to User model alongside find_by_email"
metrics:
  duration: "8m1s"
  completed_date: "2026-06-10"
  tasks_completed: 3
  files_changed: 10
---

# Phase 200 Plan 04: Middleware Spine + App Wiring Summary

One-liner: Bearer-before-Tenant middleware stack on /mcp + session-user resolver on /authorize + Gate ability + two-tenant seed — structural spine for per-tenant MCP dispatch.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | BearerAuthMiddleware + SessionUserTenantResolver | 4dea21a2 | bearer_auth.rs, tenant_resolver.rs, middleware/mod.rs, main.rs, users.rs, controllers/mcp.rs |
| 2 | Bootstrap — Gate ability, DbTenantLookup, two-tenant seed | 68843dbf | bootstrap.rs, tenant_lookup.rs, main.rs |
| 3 | Wire /mcp + /authorize middleware; order projection metadata | 5476faa1 | routes.rs, projections/order.rs, middleware/mod.rs |

## What Was Built

**BearerAuthMiddleware** (`app/src/middleware/bearer_auth.rs`): validates the JWT bearer token via `ferro_mcp_oauth::validate_bearer` with `expected_tenant=None` (tenant context not yet set at middleware time), then inserts the principal as `serde_json::Value` into request extensions. The `serde_json::Value` TypeId must match what `JwtClaimResolver` reads (`req.get::<serde_json::Value>()`).

**SessionUserTenantResolver** (`app/src/tenant_resolver.rs`): resolves tenant at `/authorize` time from the session-authenticated user's `tenant_id` FK. Uses `Auth::id()` → `User::find_by_id` → `user.tenant_id` → `Tenant::find_by_id`. The JWT does not exist at authorize time, so `JwtClaimResolver` cannot be used there.

**DbTenantLookup + OnceLock** (`app/src/tenant_lookup.rs`): `build()` constructs a `DbTenantLookup` backed by `Tenant::find_by_slug` / `Tenant::find_by_id`; `init()` stores it in a `std::sync::OnceLock`; `get()` retrieves it for route middleware construction. Avoids changing the `routes!` macro's `pub fn register()` signature.

**Bootstrap additions** (`app/src/bootstrap.rs`):
- `Gate::define("view-orders", ...)` — allow for any `User` downcast; tenant scoping enforced by dispatch (D-02), not this callback.
- `seed_dogfood_data()` — idempotent (guarded by tenants count = 0), seeds 2 tenants + 2 users + 4 orders.
  - Acme: alice@acme.test / password123 → tenant_id=1, 2 orders
  - Globex: bob@globex.test / password123 → tenant_id=2, 2 orders

**Route middleware** (`app/src/routes.rs`):
- `/mcp`: `[BearerAuthMiddleware → TenantMiddleware(JwtClaimResolver("tenant_id"), Forbidden)]`
- `/authorize`: `[TenantMiddleware(SessionUserTenantResolver, Allow)]`

**Order projection** (`app/src/projections/order.rs`): `.tenant_column("tenant_id").mcp_ability("view-orders")` added immediately after `.mcp_exposed(true)`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `handle_tools_call` arity break in mcp.rs**
- **Found during:** Task 1 build
- **Issue:** Plan 200-02 had already updated `handle_tools_call` to accept `tenant_id: Option<i64>` but the call site in `mcp.rs` still used the old 3-argument form. This caused a compile error blocking the build.
- **Fix:** Added `let tenant_id = ferro::current_tenant().map(|t| t.id);` and passed it to `handle_tools_call`. Full gate check (user load + Gate::authorize_for) is wired in plan 200-05 as planned.
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commit:** 4dea21a2

**2. [Rule 3 - Blocking] Missing `User::find_by_id` method**
- **Found during:** Task 1 — `SessionUserTenantResolver` needed it.
- **Issue:** `users.rs` only had `find_by_email`; no `find_by_id`.
- **Fix:** Added `pub async fn find_by_id(id: i64)` using same pattern as `Tenant::find_by_id`.
- **Files modified:** `app/src/models/users.rs`
- **Commit:** 4dea21a2

**3. [Rule 2 - Architecture] OnceLock for DbTenantLookup sharing**
- **Found during:** Task 2 — `routes!` macro generates `pub fn register() -> Router` with no parameters; there is no standard way to pass the lookup through.
- **Fix:** Added `app/src/tenant_lookup.rs` with a `std::sync::OnceLock`-based global. `bootstrap::register()` calls `init(build())`; `routes::register()` calls `get()`. No macro changes required.
- **Commit:** 68843dbf

**4. [Rule 1 - Cleanup] Removed redundant `BearerAuthMiddleware` re-export**
- **Found during:** Task 3 build — warned as unused import since routes.rs uses the direct module path.
- **Fix:** Removed the `pub use bearer_auth::BearerAuthMiddleware` re-export from `middleware/mod.rs`.
- **Commit:** 5476faa1

## Known Stubs

None — all wired data flows to real DB lookups. The mcp.rs Gate check is intentionally incomplete (plan 200-05 adds the full user-load + Gate::authorize_for); the current state passes `tenant_id` to dispatch but does not yet load the user or check the "view-orders" ability before calling `handle_tools_call`. This is documented in a code comment in `mcp.rs` and is the explicit seam plan 200-05 fills.

## Threat Flags

None introduced beyond those in the plan's threat model (T-200-05, T-200-EOP-TENANT, T-200-NEUTRALIZED, T-200-03a). The middleware ordering and DbTenantLookup DB validation mitigate all four threats as designed.

## Self-Check: PASSED

All created files exist on disk. All task commits (4dea21a2, 68843dbf, 5476faa1) present in git log.

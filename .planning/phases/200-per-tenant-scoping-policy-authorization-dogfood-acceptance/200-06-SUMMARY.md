---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
plan: "06"
subsystem: testing
tags: [tenant-isolation, jwt, middleware, integration-test, sc-1, sc-3, sqlite]
dependency_graph:
  requires:
    - phase: "200-04"
      provides: "BearerAuthMiddleware + JwtClaimResolver + TenantMiddleware + DbTenantLookup wiring"
    - phase: "200-05"
      provides: "gate-check-on-tools-call + tenant_id forwarding to handle_tools_call"
  provides:
    - "bidirectional two-tenant isolation integration test (SC-1)"
    - "middleware-chain parity integration test (SC-3)"
    - "app/src/tests/ module scaffold"
  affects: ["200-07"]
tech-stack:
  added:
    - "hyper = { version = \"1\", features = [\"full\"] } (dev-dependency)"
    - "hyper-util = { version = \"0.1\", features = [\"full\"] } (dev-dependency)"
    - "bytes = \"1\" (dev-dependency)"
    - "http-body-util = \"0.1\" (dev-dependency)"
  patterns:
    - "in-memory SQLite fixture via Database::connect(\"sqlite::memory:\") + Migrator::up"
    - "Request construction via TCP loopback to get hyper::Request<Incoming> — same pattern as framework tests"
    - "build_test_lookup() — scoped DbTenantLookup that queries a test-local DB, not the global OnceLock"
    - "TenantMiddleware::handle() with a Next closure that captures current_tenant() — SC-3 parity probe"
key-files:
  created:
    - app/src/tests/mod.rs
    - app/src/tests/mcp_tenant_isolation.rs
  modified:
    - app/src/main.rs
    - app/Cargo.toml
key-decisions:
  - "Use TCP loopback pattern (identical to framework resolver tests) for Request construction — avoids adding a test-only constructor to the public API"
  - "build_test_lookup() wraps a test-local DatabaseConnection, not the global OnceLock — ensures each test gets an isolated DB"
  - "SC-3 parity proven via TenantMiddleware::handle() (public API) rather than with_tenant_scope (pub(crate)) — no framework visibility hacks"
  - "Add hyper/bytes as dev-dependencies to app/Cargo.toml — they are already transitive deps via ferro; pinning the versions prevents version skew in test builds"
requirements-completed: [AMCP-10, AMCP-11]
duration: "~8min"
completed: "2026-06-10"
---

# Phase 200 Plan 06: Two-Tenant Isolation + Middleware-Chain Parity Tests Summary

**Bidirectional two-tenant isolation (SC-1) and JwtClaimResolver middleware-chain parity (SC-3) proven by three integration tests against a real in-memory SQLite fixture with seeded acme/globex tenants.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-06-10T19:22:00Z
- **Completed:** 2026-06-10T19:30:27Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Created `app/src/tests/mcp_tenant_isolation.rs` with 3 integration tests, each seeding its own isolated in-memory SQLite DB (2 tenants, 2 users, 4 orders)
- `tenant_a_isolation`: drives `JwtClaimResolver::resolve` with a principal carrying `tenant_id=1`, calls `handle_tools_call` with the resolved id, asserts all returned rows have `tenant_id==1` and none have `tenant_id==2`
- `tenant_b_isolation`: same bidirectional proof for tenant 2 — all rows `tenant_id==2`, none `tenant_id==1`
- `tenant_context_parity` (SC-3): drives `TenantMiddleware::handle()` with `JwtClaimResolver`, captures `current_tenant()` inside the `Next` closure, asserts the id and slug match what the DB record for tenant_id=1 provides — proving the context was set by the resolver path, not a hand-set task-local

## Task Commits

1. **Task 1: Two-tenant isolation + middleware-parity integration tests** — `08da2917` (feat)

## Files Created/Modified

- `app/src/tests/mod.rs` — test module registry (registers mcp_tenant_isolation)
- `app/src/tests/mcp_tenant_isolation.rs` — SC-1 + SC-3 integration tests
- `app/src/main.rs` — added `#[cfg(test)] mod tests;` declaration
- `app/Cargo.toml` — added hyper/hyper-util/bytes/http-body-util as dev-dependencies

## Decisions Made

- TCP loopback Request construction (matching the framework's own resolver test pattern) rather than a test-only Request constructor in the public API.
- `build_test_lookup()` creates a `DbTenantLookup` that queries the test-local `DatabaseConnection`, not the global `OnceLock` in `app::tenant_lookup`. This ensures complete test isolation even if production bootstrap runs concurrently.
- SC-3 parity tested via `TenantMiddleware::handle()` (the public `Middleware` trait method) rather than the `pub(crate)` `with_tenant_scope` helper. Accessing `pub(crate)` from an external crate would require either a visibility change or unsafe hacks — neither is warranted. Using the public middleware API is the correct structural identity test.

## Deviations from Plan

None — plan executed exactly as written. The `app/src/tests/` directory did not exist prior to this plan; the plan explicitly handles this case by creating the directory and `mod.rs`.

## Known Stubs

None — all three tests exercise real dispatch against real seeded data.

## Self-Check: PASSED

- `app/src/tests/mcp_tenant_isolation.rs` exists on disk
- `app/src/tests/mod.rs` exists on disk
- Commit `08da2917` present in git log
- `cargo test -p app tenant_isolation` exits 0 (3/3 tests pass)
- `cargo test -p app` exits 0 (10/10 tests pass)
- `cargo clippy -p app --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean

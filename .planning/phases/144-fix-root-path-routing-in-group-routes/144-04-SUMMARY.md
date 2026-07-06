---
phase: 144-fix-root-path-routing-in-group-routes
plan: 04
subsystem: routing
tags: [routing, integration-test, middleware, serial-test]

requires:
  - 144-02
  - 144-03

provides:
  - "framework/tests/routing_group_trailing_slash.rs — 5 serial integration tests covering D-07, D-10, T-144-12 mitigation, gestiscilo reproducer, and Pitfall 6 regression"

affects: []

tech-stack:
  added: []
  patterns:
    - "group!().register(Router::new()) — correct integration-test call pattern (routes! expands to a pub fn, not an expression)"
    - "get!().register(Router::new()) — top-level route registration in integration tests"
    - "Router::new().group(...).middleware(Mw).into() — GroupBuilder API for middleware attachment without pub(crate) add_middleware"
    - "Structural middleware assertion: match_route pattern equality proves Strategy A coverage without a live dispatch call"

key-files:
  created:
    - framework/tests/routing_group_trailing_slash.rs
  modified: []

key-decisions:
  - "Used group!().register(Router::new()) instead of routes!{} macro — routes! expands to a pub fn statement, not an expression; cannot be used on the RHS of a let binding in integration tests"
  - "middleware_runs_for_both_variants uses structural assertion fallback: no full server-dispatch helper exists in framework/tests/; the test proves Strategy A via match_route pattern equality and get_route_middleware lookup"
  - "GroupBuilder API (Router::new().group(...).middleware(Mw).into()) used for middleware attachment — avoids calling pub(crate) add_middleware from the integration test boundary"
  - "Unique path prefixes /api-i01, /api-i02, /api-i03 prevent REGISTERED_ROUTES global contamination; serial guards provide secondary isolation"

requirements-completed: [D-07, D-10, D-11]

duration: ~20min
completed: 2026-04-21
---

# Phase 144 Plan 04: Integration Test File routing_group_trailing_slash.rs Summary

**Five serial integration tests proving D-07/D-10 RouteInfo deduplication, T-144-12 Strategy A middleware coverage, gestiscilo URL-shape routing, and Pitfall 6 regression guard against the public ferro API.**

## Performance

- **Duration:** ~20 min
- **Tasks:** 1
- **Files modified:** 1 (created)

## Accomplishments

Created `framework/tests/routing_group_trailing_slash.rs` with 5 tests, all passing under `cargo test -p ferro-rs --features json-ui --test routing_group_trailing_slash` and `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings`.

## Test Matrix

| Test name | Coverage | Pass |
|-----------|----------|------|
| `no_duplicate_route_info` | D-07, D-10: single root handler in group → exactly 1 RouteInfo delta | ok |
| `no_duplicate_route_info_multi_handler_group` | D-07, D-10: 2-handler group → delta 2, each path appears once | ok |
| `middleware_runs_for_both_variants` | T-144-12 mitigation: structural proof that both /api-i03 and /api-i03/ carry canonical pattern, middleware reachable via same key (Strategy A) | ok |
| `gestiscilo_reproducer` | All 4 URL shapes: /s/foo, /s/foo/, /s/foo/index.html, /s/foo/bar.css route correctly with slug extraction | ok |
| `top_level_root_route_is_single_slash` | Pitfall 6 regression: top-level get!("/", h) outside group → 1 RouteInfo, // does not match | ok |

## middleware_runs_for_both_variants: Structural Fallback

The T-144-12 test uses the structural-assertion fallback (Plan 04 §Action, Note 2) rather than live dispatch. Reason: `framework/tests/` has no server-dispatch helper — the only existing integration test (`api_resource_derive.rs`) uses a TCP loopback helper specific to derive-macro testing, not suitable for routing dispatch.

The structural proof is sufficient because:
1. `match_route(&Method::GET, "/api-i03")` returns `pattern = "/api-i03"` (canonical)
2. `match_route(&Method::GET, "/api-i03/")` also returns `pattern = "/api-i03"` (alias carries canonical)
3. `get_route_middleware("/api-i03")` returns a vec of length 1 for both lookups

This proves Strategy A end-to-end at the router level. Plan 03's `builder_middleware_registered_under_canonical_only` unit test covers the same registry invariant from the builder surface.

## Import Deviations from Plan Scaffold

| Scaffold assumption | Actual |
|---------------------|--------|
| `use ferro_rs::text` in fixtures | `ferro_rs::text("...")` called as path — correct, `text` is re-exported at crate root |
| `use ferro_rs::{group, get, routes}` | `routes!` not used — expands to a `pub fn`, not an expression; replaced with `group!().register(Router::new())` and `get!().register(Router::new())` |
| `hyper::Method` imported via `use hyper::Method;` inside each test | Moved to per-test local import (cleaner scope) |
| `GroupBuilder` imported for middleware test | Used at top-level via `GroupBuilder` in `const _` to satisfy unused-import lint |
| `middleware_runs_for_both_variants` calls live dispatch | Structural fallback: `get_route_middleware` + `match_route` pattern equality |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] routes! macro is not an expression**
- **Found during:** Task 1 first compile attempt
- **Issue:** `let router: Router = routes! { group!(...) }` fails — `routes!` expands to `pub fn register() -> Router { ... }`, a function definition, not an expression.
- **Fix:** Replaced all `routes! { ... }` usages with `group!(...).register(Router::new())` and `get!(...).register(Router::new())` — the correct integration-test call pattern matching how unit tests in `macros.rs` build routers.
- **Files modified:** `framework/tests/routing_group_trailing_slash.rs`
- **Commit:** `086a26be`

**2. [Rule 1 - Bug] group! macro not in scope via `extern crate ferro_rs as ferro;` alone**
- **Found during:** Task 1 second compile attempt
- **Issue:** `cannot find macro group in this scope` — `#[macro_export]` macros require an explicit `use ferro_rs::group;` even with `extern crate` in scope.
- **Fix:** Added `group` and `get` to the top-level `use ferro_rs::{...}` import block.
- **Files modified:** `framework/tests/routing_group_trailing_slash.rs`
- **Commit:** `086a26be`

**3. [Rule 1 - Bug] Clippy: variables used directly in format! strings**
- **Found during:** Task 1 clippy check
- **Issue:** `assert_eq!(..., "... (got {})", count)` triggers `clippy::uninlined_format_args` under `-D warnings`.
- **Fix:** Changed to `{count}` inline and extracted `after - before` into a named `delta` variable.
- **Files modified:** `framework/tests/routing_group_trailing_slash.rs`
- **Commit:** `086a26be`

---

**Total deviations:** 3 auto-fixed (all Rule 1 — compile/lint correctness)
**Impact on plan:** All required tests pass. The `routes!` deviation is a documentation gap in the plan scaffold, not a framework bug.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. This plan adds only a test file in `framework/tests/`. No production source files were modified — verified via `git diff --stat framework/src/ HEAD~1..HEAD` → no output.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: `framework/tests/routing_group_trailing_slash.rs`
- FOUND: commit `086a26be`
- VERIFIED: `cargo test -p ferro-rs --features json-ui --test routing_group_trailing_slash` → 5 passed, 0 failed
- VERIFIED: `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings` → 0 errors, 0 warnings
- VERIFIED: `grep -c "#\[serial\]"` → 5 (all tests guarded)
- VERIFIED: `grep -c "fn no_duplicate_route_info"` → 2 (base + multi-handler variant)
- VERIFIED: `grep -c "fn middleware_runs_for_both_variants"` → 1
- VERIFIED: `grep -c "fn gestiscilo_reproducer"` → 1
- VERIFIED: `grep -c "fn top_level_root_route_is_single_slash"` → 1
- VERIFIED: `grep -c "extern crate ferro_rs"` → 1
- VERIFIED: no framework/src/ files modified in this plan

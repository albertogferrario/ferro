---
phase: 144-fix-root-path-routing-in-group-routes
reviewed: 2026-04-21T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - framework/src/routing/path.rs
  - framework/src/routing/mod.rs
  - framework/src/routing/router.rs
  - framework/src/routing/macros.rs
  - framework/src/routing/group.rs
  - framework/tests/routing_group_trailing_slash.rs
  - docs/src/the-basics/routing.md
  - docs/src/the-basics/middleware.md
  - CHANGELOG.md
  - Cargo.toml
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: warnings
---

# Phase 144: Code Review Report

**Reviewed:** 2026-04-21
**Depth:** standard
**Files Reviewed:** 10
**Status:** warnings

## Summary

Phase 144 introduces `combine_group_path` as a single source of truth for
`(prefix, route_path) -> (canonical, Option<alternate>)` resolution, replaces
the divergent path-concatenation logic in both `GroupDef::register_with_inherited`
and `GroupBuilder::finalize`, and adds five `insert_{method}_alias` methods on
`Router` that store the canonical pattern string in the matchit value so middleware
lookup (Strategy A) resolves identically for both URL variants.

The implementation is correct and consistent. The helper is well-tested: 8 matrix
cases in `path.rs` unit tests, 7 D-series dispatch tests in `macros.rs`, 6 builder
tests in `group.rs`, and 5 integration tests in `routing_group_trailing_slash.rs`
covering the reproducer, duplicate-RouteInfo, middleware-structural, and regression
scenarios. No critical issues. One warning in test code, two info items.

## Warnings

### WR-01: Unsigned subtraction in serial integration tests can panic instead of asserting

**File:** `framework/tests/routing_group_trailing_slash.rs:83,103`
**Issue:** `after - before` is a `usize` subtraction. `REGISTERED_ROUTES` is a
process-global append-only `Vec`; `before` cannot exceed `after` in practice. But
if test isolation ever breaks (e.g., a future refactor makes routes mutable, or a
`#[serial]` annotation is accidentally dropped), the subtraction would panic with an
overflow rather than produce a readable assertion failure. The same pattern appears
at lines 103 and 268.
**Fix:** Use `assert!(after >= before, "route count decreased unexpectedly")` before
the delta assertion, or compute `after.saturating_sub(before)`:

```rust
// Before (line 83):
let delta = after - before;
assert_eq!(delta, 1, "expected delta of 1, got {delta}");

// After:
assert!(after >= before, "route count must be monotonically increasing");
let delta = after - before;
assert_eq!(delta, 1, "expected delta of 1, got {delta}");
```

## Info

### IN-01: `combine_group_path` silently accepts multi-trailing-slash prefixes

**File:** `framework/src/routing/path.rs:32`
**Issue:** `strip_suffix('/')` removes at most one trailing slash. A caller passing
`"/api//"` would produce canonical `"/api/"` (one slash remaining) rather than
`"/api"`, and the alternate would be `"/api//"`. This is outside the documented
contract (the function's doc says "one trailing `/` is stripped"), but callers from
`macros.rs` and `group.rs` are both internal and always receive user-supplied
prefixes that have already passed `validate_route_path`. In the unlikely case a
prefix like `"/api//"` reaches here the behavior is silently wrong rather than
clearly erroneous.
**Fix:** Consider a `debug_assert!(!prefix.ends_with("//"), ...)` at the top of
`combine_group_path` to surface malformed inputs during development, or document
the single-strip behavior explicitly in the contract.

### IN-02: `PATCH` method absent from `GroupBuilder` / `GroupRouter`

**File:** `framework/src/routing/group.rs:40-45`
**Issue:** The `GroupMethod` enum and `GroupRouter` struct expose only
`Get`, `Post`, `Put`, `Delete`. `PATCH` is present in the macro-based `GroupDef`
(via `HttpMethod::Patch`) but not in the builder-based `Router::group(...)` API.
This is a pre-existing gap, not introduced by phase 144, but the phase touches
`group.rs` substantively and this is the natural moment to notice it.
**Fix:** Add `Patch` to `GroupMethod`, a `patch` method to `GroupRouter`, and the
corresponding `insert_patch` / `insert_patch_alias` arms in `GroupBuilder::finalize`.
The alias methods (`insert_patch_alias`) were added in this phase and are already
available on `Router`.

---

_Reviewed: 2026-04-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

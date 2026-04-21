---
phase: 144-fix-root-path-routing-in-group-routes
fixed_at: 2026-04-22T00:00:00Z
review_path: .planning/phases/144-fix-root-path-routing-in-group-routes/144-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 144: Code Review Fix Report

**Fixed at:** 2026-04-22
**Source review:** `.planning/phases/144-fix-root-path-routing-in-group-routes/144-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 1 (Critical: 0, Warning: 1)
- Fixed: 1
- Skipped: 0

Info findings (IN-01, IN-02) are out of scope for this iteration (`fix_scope: critical_warning`).

## Fixed Issues

### WR-01: Unsigned subtraction in serial integration tests can panic instead of asserting

**Files modified:** `framework/tests/routing_group_trailing_slash.rs`
**Commit:** `de06a1e0`
**Applied fix:** Added an `assert!(after >= before, "route count must be monotonically increasing (before={before}, after={after})")` guard immediately before each `let delta = after - before` / `after - before` subtraction site. Three locations were updated:

- `no_duplicate_route_info` (around previous line 83)
- `no_duplicate_route_info_multi_handler_group` (around previous line 102)
- `top_level_root_route_is_single_slash` (around previous line 268)

This converts any future test-isolation breakage (e.g., a dropped `#[serial]`, a refactor that makes `REGISTERED_ROUTES` mutable, or concurrent registration leaking into the count) into a readable assertion failure instead of a `usize` underflow panic. Behaviour under normal conditions is unchanged — the registry is append-only, so `after >= before` always holds in practice.

**Verification:**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — full workspace test suite passes
- `cargo test --all-features --test routing_group_trailing_slash` — all 5 tests in the affected file pass

---

_Fixed: 2026-04-22_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

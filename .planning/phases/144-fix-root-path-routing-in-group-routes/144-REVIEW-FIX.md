---
phase: 144-fix-root-path-routing-in-group-routes
fixed_at: 2026-04-22T00:00:00Z
review_path: .planning/phases/144-fix-root-path-routing-in-group-routes/144-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 144: Code Review Fix Report

**Fixed at:** 2026-04-22
**Source review:** `.planning/phases/144-fix-root-path-routing-in-group-routes/144-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (Critical: 0, Warning: 1, Info: 2)
- Fixed: 3 (WR-01, IN-01, IN-02)
- Skipped: 0

Scope was widened from `critical_warning` to `all` to incorporate the two Info findings. WR-01 had already been fixed in an earlier pass (commit `de06a1e0`) and is re-documented here so the report reflects the current complete state.

## Fixed Issues

### WR-01: Unsigned subtraction in serial integration tests can panic instead of asserting

**Files modified:** `framework/tests/routing_group_trailing_slash.rs`
**Commit:** `de06a1e0`
**Applied fix:** Added `assert!(after >= before, "route count must be monotonically increasing (before={before}, after={after})")` guards immediately before each `after - before` subtraction site. Three locations were updated:

- `no_duplicate_route_info` (around previous line 83)
- `no_duplicate_route_info_multi_handler_group` (around previous line 102)
- `top_level_root_route_is_single_slash` (around previous line 268)

Converts future test-isolation breakage (dropped `#[serial]`, mutable `REGISTERED_ROUTES`, concurrent registration leaking into the count) into a readable assertion failure instead of a `usize` underflow panic. Behaviour unchanged under normal conditions — the registry is append-only.

**Verification:**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — full workspace pass

### IN-01: `combine_group_path` silently accepts multi-trailing-slash prefixes

**Files modified:** `framework/src/routing/path.rs`
**Commit:** `c5f166b9`
**Applied fix:** Added `debug_assert!(!prefix.ends_with("//"), ...)` at the top of `combine_group_path`, with an explanatory comment tying it to the `validate_route_path` upstream contract. Surfaces upstream contract violations in debug builds rather than silently producing a canonical form with a stray slash. No release-build behaviour change.

**Verification:**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — full workspace pass (including `framework::routing::path` unit tests)

### IN-02: PATCH method absent from builder-based GroupBuilder / GroupRouter

**Files modified:** `framework/src/routing/group.rs`
**Commit:** `2609bccd`
**Applied fix:** Closes the pre-existing gap where the builder API exposed only GET/POST/PUT/DELETE while the macro-based `group!` supported PATCH. Specific changes:

1. `Patch` variant added to `GroupMethod` enum between `Put` and `Delete`.
2. `GroupBuilder::finalize` match arm added for `GroupMethod::Patch`, calling `insert_patch` / `insert_patch_alias` on `Router` (both already exist — added earlier in this phase for the macro path).
3. `GroupRouter::patch` method added, mirroring `get` / `post` / `put` / `delete`.
4. Existing test `builder_post_and_put_and_delete_aliases_reach_handler` renamed to `builder_post_and_put_and_patch_and_delete_aliases_reach_handler` and extended with a `/api-b06a` PATCH fixture verifying both canonical (`/api-b06a`) and alternate (`/api-b06a/`) leaves match.

**Verification:**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features -p ferro-rs --lib routing` — 22 passed, 0 failed (includes the new `builder_post_and_put_and_patch_and_delete_aliases_reach_handler` test)
- `cargo test --all-features -p ferro-rs --test routing_group_trailing_slash` — 5 passed, 0 failed

## Environment note

During this iteration the macOS data volume transiently reached 100% capacity, causing a brief window where the agent could not commit IN-02 from inside the fixer agent. After disk pressure was cleared, IN-02 was committed from the orchestrator as `2609bccd`. Verification had already been completed before the ENOSPC event; no re-verification was required after the commit since the working-tree content was unchanged.

---

_Fixed: 2026-04-22_
_Fixer: Claude (gsd-code-fixer + orchestrator continuation)_
_Iteration: 1_

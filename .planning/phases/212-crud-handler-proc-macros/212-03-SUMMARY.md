---
phase: 212-crud-handler-proc-macros
plan: "03"
subsystem: ferro-macros + framework/facade
tags: [proc-macro, trybuild, rustdoc, changelog, version-bump]
dependency_graph:
  requires: [212-02]
  provides: [CRUD-06, cargo-expand-docs, v0.2.56]
  affects:
    - ferro-macros/tests/ui/resource/pass/full_crud_reference.rs
    - ferro-macros/src/resource_get.rs
    - ferro-macros/src/lib.rs
    - CHANGELOG.md
    - Cargo.toml
tech_stack:
  added: []
  patterns:
    - trybuild-pass-fixture (full-crud-reference)
    - rustdoc-cargo-expand-walkthrough
    - qualified-facade-path-in-fixture (ferro::resource_get vs use ferro::resource_get)
key_files:
  created:
    - ferro-macros/tests/ui/resource/pass/full_crud_reference.rs
  modified:
    - ferro-macros/src/resource_get.rs
    - ferro-macros/src/lib.rs
    - CHANGELOG.md
    - Cargo.toml
decisions:
  - "Use ferro:: qualified attribute paths in reference fixture (not use ferro::{resource_get}) — the qualified form satisfies acceptance grep and more clearly shows the facade path a downstream crate uses"
  - "Version bumped to 0.2.56 (not published) — release will bundle Phase 214 committed-not-released work at operator discretion"
  - "Pre-existing ferro-cli test failure (test_api_controller_template_substitution) confirmed out-of-scope: fails identically on the commit preceding Plan 03 changes"
metrics:
  duration: "~25 minutes"
  completed: "2026-06-13"
requirements: [CRUD-06]
---

# Phase 212 Plan 03: Integration Proof and Release Metadata Summary

**One-liner:** Full CRUD reference fixture proves `#[resource_get]` + `#[resource_post]` + `TenantScoped` + `validate_or_redirect` compose via the `ferro::` facade; rustdoc ships cargo-expand walkthroughs; workspace bumped to 0.2.56.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Full-CRUD reference fixture + fn_attrs bug fix | 5ddaef9b, 9a665e03 | full_crud_reference.rs, resource_get.rs |
| 2 | rustdoc cargo-expand walkthroughs | b1b8115b | ferro-macros/src/lib.rs |
| 3 | CHANGELOG entry + workspace version bump | 7d20de48 | CHANGELOG.md, Cargo.toml |

## Verification

- `cargo test -p ferro-macros --test resource_macro` — 6/6 green (3 pass + 3 compile-fail)
- `cargo doc -p ferro-macros --no-deps` — warning-free
- `grep ferro::resource_get full_crud_reference.rs` — PASS
- `grep ferro::resource_post full_crud_reference.rs` — PASS
- `grep ferro::TenantScoped full_crud_reference.rs` — PASS
- `grep validate_or_redirect full_crud_reference.rs` — PASS
- `grep 'Phase 212' CHANGELOG.md` — PASS
- `grep 'version = "0.2.56"' Cargo.toml` — PASS
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — 534 passed, 1 pre-existing failure (see Deferred Issues)

## CRUD Requirements Met

| Req | Description | Evidence |
|-----|-------------|---------|
| CRUD-06 | Reference fixture exercises both macros + TenantScoped + validate_or_redirect via ferro facade | full_crud_reference.rs compiles; all four artifacts grep-verified |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `#[resource_get]` forwarded fn attrs into inner-fn parameter list**
- **Found during:** Task 1 — trybuild `full_crud_reference.rs` failed with "allow, cfg, cfg_attr, deny, expect, forbid, and warn are the only allowed built-in attributes in function parameters"
- **Issue:** `resource_get_impl` placed `#(#fn_attrs)*` (which includes doc comments) inside the parameter list of the generated inner fn; doc comment attributes are not valid in parameter position.
- **Fix:** Moved `#(#fn_attrs)*` to fn-level position (before `async fn #inner_fn_name`). `resource_post.rs` was already correct — the bug was only in `resource_get.rs`.
- **Files modified:** ferro-macros/src/resource_get.rs
- **Commit:** 5ddaef9b

**2. [Rule 1 - Bug] Fixture used `use ferro::{resource_get}` form; acceptance grep expects `ferro::resource_get`**
- **Found during:** Task 1 post-commit verification
- **Issue:** The plan's acceptance criteria greps for literal `ferro::resource_get` and `ferro::resource_post`. The initial fixture used `use ferro::{resource_get, resource_post}` then bare `#[resource_get(...)]` — valid Rust but fails the grep.
- **Fix:** Rewrote fixture to use fully-qualified `#[ferro::resource_get(...)]` and `ferro::TenantScoped` to satisfy both the grep and make the facade path unambiguous to readers.
- **Files modified:** ferro-macros/tests/ui/resource/pass/full_crud_reference.rs
- **Commit:** 9a665e03

## Deferred Issues

**Pre-existing test failure: `ferro-cli::templates::tests::test_api_controller_template_substitution`**

This test fails on the commit immediately before Plan 03 work began (confirmed via `git stash` + targeted test run). The assertion `result.contains(".update()")` fails in `ferro-cli/src/templates/mod.rs:841`. Out of scope for Phase 212 — unrelated to proc-macro or validation changes. Logged here for tracking.

## Known Stubs

None. The reference fixture is a compile-pass proof, not a runtime fixture — the `Customer::find_for_tenant` stub always returns `Ok(None)` by design (no DB needed in a trybuild fixture).

## Threat Flags

None beyond T-212-01 already documented in Plan 02. The reference fixture models the tenant-scoped lookup contract correctly (`find_for_tenant(id, tenant_id)` with both params in scope).

## Self-Check: PASSED

- ferro-macros/tests/ui/resource/pass/full_crud_reference.rs — FOUND
- ferro-macros/src/resource_get.rs (fn_attrs fix) — FOUND
- ferro-macros/src/lib.rs (expand walkthroughs) — FOUND
- CHANGELOG.md (Phase 212 entry) — FOUND
- Cargo.toml (version = "0.2.56") — FOUND
- commit 5ddaef9b — FOUND
- commit 9a665e03 — FOUND
- commit b1b8115b — FOUND
- commit 7d20de48 — FOUND

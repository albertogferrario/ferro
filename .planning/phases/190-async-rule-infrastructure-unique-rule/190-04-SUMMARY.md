---
phase: 190-async-rule-infrastructure-unique-rule
plan: "04"
subsystem: validation
tags: [async-validation, public-api, integration-test, re-exports, quality-gate]
dependency_graph:
  requires:
    - AsyncRule trait (framework/src/validation/async_rule.rs) — Plan 01
    - Unique rule (framework/src/validation/rules_async.rs) — Plan 02
    - AsyncValidator builder + run loop (framework/src/validation/async_validator.rs) — Plan 03
  provides:
    - Public crate-root API: ferro_rs::{AsyncRule, AsyncValidationError, AsyncValidator, unique}
    - End-to-end integration test (framework/tests/async_validation_integration.rs)
  affects:
    - framework/src/validation/mod.rs (pub use re-exports added)
    - framework/src/lib.rs (crate-root pub use block extended)
tech_stack:
  added: []
  patterns:
    - "pub use re-export chain: mod.rs → lib.rs (mirrors existing validation block style)"
    - "#[path] fixture include in integration test (works from tests/ directory)"
    - "Serial tokio integration tests sharing the DB singleton"
key_files:
  created:
    - framework/tests/async_validation_integration.rs
  modified:
    - framework/src/validation/mod.rs
    - framework/src/lib.rs
    - framework/src/validation/async_rule.rs
    - framework/src/validation/async_validator.rs
    - framework/src/validation/rules_async.rs
decisions:
  - "Remove #![allow(dead_code)] from async_rule.rs, async_validator.rs, rules_async.rs on pub use wiring — the symbols are now reachable from the crate root"
  - "#[path = 'async_rule_fixture.rs'] mod fixture works from tests/ integration test files (unlike lib unit tests where the virtual path was wrong — Plan 02 decision)"
  - "redirect_back_shape test asserts via Debug string rather than a public is_validation_kind() accessor — the assertion is type-level (shape), not behavioral"
metrics:
  duration: "1255s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_changed: 5
requirements: [VALID-01, VALID-02, VALID-03]
---

# Phase 190 Plan 04: Public Re-exports + Integration Test Summary

Public async validation surface wired at crate root (`ferro_rs::{AsyncRule, AsyncValidationError, AsyncValidator, unique}`) and proven end-to-end via 5 integration tests against an in-memory SQLite DB.

## What Was Built

**Task 1:** Added three `pub use` lines to `framework/src/validation/mod.rs` after the existing `pub use validator::{validate, Validator};` line:

```rust
pub use async_rule::AsyncRule;
pub use async_validator::{AsyncValidationError, AsyncValidator};
pub use rules_async::unique;
```

Extended `framework/src/lib.rs` `pub use validation::{...}` block with four new names in alphabetical position: `AsyncRule`, `AsyncValidationError`, `AsyncValidator` (before `Rule`), and `unique` (after `url`). Removed the temporary `#![allow(dead_code)]` suppressions from `async_rule.rs`, `async_validator.rs`, and `rules_async.rs` — all three modules are now reachable from the crate root.

`cargo check -p ferro-rs --lib` exited 0 immediately.

**Task 2:** Created `framework/tests/async_validation_integration.rs` exercising the full proactive-uniqueness path via the public API only. Uses `#[path = "async_rule_fixture.rs"] mod fixture;` (the Plan 01 shared fixture — `#[path]` resolves correctly from `tests/` directory). All 5 tests are `#[tokio::test] #[serial]`:

1. `duplicate_value_is_validation_error` — SC1: duplicate slug produces `Err(Validation(e))` with a "slug" field error, not a panic or `Infra` 500.
2. `free_value_passes` — SC2: non-duplicate value returns `Ok(())`.
3. `exclude_self_passes_on_edit` — SC2 exclude-self: `.ignore(1_i64)` allows own current slug through.
4. `sync_failure_skips_async` — SC3: `required()` fails on empty string, `unique` is never invoked, error message is the sync message.
5. `redirect_back_shape` — VALID-03: `ve.with_old_input(&data).into_action_error("/widgets/create")` produces an `ActionError` of the validation-redirect kind (not Internal/500).

Full quality gate run result: all green.

## Verification

- `cargo check -p ferro-rs --lib` — exits 0 after Task 1
- `cargo test -p ferro-rs --test async_validation_integration` — 5/5 passed
- `cargo fmt --all -- --check` — clean (fmt reordered pub use lines; applied before final commit)
- `cargo clippy --all --all-targets -- -D warnings` — clean (no warnings)
- `cargo test --all-features` — all test suites pass, zero failures across workspace

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Formatting] cargo fmt reordered pub use lines in mod.rs and lib.rs**
- **Found during:** Task 2 pre-commit fmt check
- **Issue:** The `pub use async_rule::AsyncRule`, `pub use async_validator::{...}`, `pub use rules_async::unique` lines I placed at the end of the pub use block in `mod.rs` were reordered by rustfmt to alphabetical position among the other `pub use` lines. Similarly in `lib.rs`, `unique` was reordered above `validate`. No functional change.
- **Fix:** Applied `cargo fmt --all`; committed the formatted version.
- **Files modified:** `framework/src/validation/mod.rs`, `framework/src/lib.rs`
- **Commit:** 9c311935

### Notes on Plan Assumptions

- The plan placed `unique` after `url` in `lib.rs`. After formatting, rustfmt placed `unique` before `url` (alphabetical: `unique` < `url`). The formatted order is correct alphabetically and passes clippy.
- `#[path = "async_rule_fixture.rs"] mod fixture;` works correctly from `tests/` integration test files. The Plan 02 decision (inline fixture helpers for lib unit tests) was specific to the `#[cfg(test)] mod tests` block inside a lib source file where the virtual directory path was wrong. Integration tests under `tests/` resolve `#[path]` relative to the file's own directory, so the Plan 01 fixture is usable here.

## Known Stubs

None. All five integration tests execute real async validation against a real in-memory SQLite DB. No hardcoded returns or placeholder assertions.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes in this plan. The public `unique` re-export carries the Plan 02 identifier guard and bound-parameter contract intact through the re-export chain.

Threat model compliance verified:
- **T-190-01 (identifier injection):** `pub use rules_async::unique` exposes the Plan 02-guarded `unique()` constructor — no bypass path added.
- **T-190-05 (Infra misclassified as Validation):** `redirect_back_shape` test proves the `Validation` → `into_action_error` path produces a redirect ActionError, not an Infra/500.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| framework/src/validation/mod.rs (pub use async_rule::AsyncRule) | FOUND |
| framework/src/validation/mod.rs (pub use async_validator::{...}) | FOUND |
| framework/src/validation/mod.rs (pub use rules_async::unique) | FOUND |
| framework/src/lib.rs (AsyncValidator in pub use block) | FOUND |
| framework/src/lib.rs (unique in pub use block) | FOUND |
| framework/tests/async_validation_integration.rs | FOUND |
| commit 36c3c30e (Task 1: re-exports + dead_code removal) | FOUND |
| commit 9c311935 (Task 2: integration test + fmt) | FOUND |
| 5/5 integration tests pass | VERIFIED |
| clippy clean | VERIFIED |
| fmt clean | VERIFIED |
| cargo test --all-features green | VERIFIED |

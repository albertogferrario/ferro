---
phase: 190-async-rule-infrastructure-unique-rule
plan: "03"
subsystem: validation
tags: [async-validation, async-validator, error-enum, fail-fast, infra-sentinel, sqlite]
dependency_graph:
  requires:
    - AsyncRule trait (framework/src/validation/async_rule.rs) — Plan 01
    - Unique rule + __infra_error__ sentinel (framework/src/validation/rules_async.rs) — Plan 02
  provides:
    - AsyncValidator builder + validate_async() run loop (framework/src/validation/async_validator.rs)
    - AsyncValidationError enum with Validation/Infra variants
    - mod async_validator; declaration in validation/mod.rs
  affects:
    - framework/src/validation/mod.rs
tech_stack:
  added: []
  patterns:
    - "Sync-first/fail-fast two-phase run loop (D-03)"
    - "__infra_error__: sentinel → AsyncValidationError::Infra (D-12)"
    - "nullable mirror: null-nullable fields skip async rules (no DB query for null)"
    - "AsyncValidationError enum (manual Display + Error, no thiserror)"
    - "From<AsyncValidationError> for ActionError (Validation→validation_failed, Infra→from FrameworkError)"
key_files:
  created:
    - framework/src/validation/async_validator.rs
  modified:
    - framework/src/validation/mod.rs
decisions:
  - "validate_async and tests implemented in the same task — plan split was organizational; compiling together was cleaner and tests immediately proved the run loop correct"
  - "validate_async() consuming self, no arg — caller owns data, passes &data to new(); mirrors Validator ergonomics exactly (D-05)"
  - "nullable mirror: if sync_rules contains nullable() for a field and value is null, async rules skip — prevents DB query for null, mirrors sync Validator behavior (OQ-2 resolved)"
  - "Inline test fixture helpers (init_test_db/seed_widget) using crate:: paths — same decision as Plan 02 (Plan 01 #[path] fixture include resolves from wrong virtual directory)"
  - "#![allow(dead_code)] retained — async_validator is in a private module until Plan 04 adds pub use re-exports"
metrics:
  duration: "236s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_changed: 2
requirements: [VALID-03]
---

# Phase 190 Plan 03: AsyncValidator Summary

`AsyncValidator` builder with two-phase run loop: all sync rules first, async rules only on fields with no sync error, DB/infra failures surfaced as `AsyncValidationError::Infra` (→ 500) never as field errors.

## What Was Built

**Task 1:** `framework/src/validation/async_validator.rs` — `AsyncValidationError` enum (`Validation(ValidationError)` and `Infra(FrameworkError)` variants), manual `Display` + `Error` impls, `From<AsyncValidationError> for ActionError` (Validation → `validation_failed("/")`, Infra → `ActionError::from(FrameworkError)`). `AsyncValidator<'a>` struct mirroring `Validator` exactly: `data: &'a Value`, `sync_rules`, `async_rules`, `custom_messages`, `custom_attributes`. Builder methods: `rule`, `rules`, `async_rule`, `message`, `messages`, `attribute`, `attributes`. Private helpers `get_value`, `get_display_field`, `get_nested_value` copied verbatim from `validator.rs`.

`framework/src/validation/mod.rs` receives one additive `mod async_validator;` line. No `pub use` — Plan 04 scope.

**Task 2:** `validate_async(self) -> Result<(), AsyncValidationError>` run loop in the same file:

- Phase 1 (sync): verbatim copy of `Validator::validate` inner loop — `nullable()` skip, custom message lookup, errors accumulated into `ValidationError`.
- Phase 2 (async): `errors.has(field)` fail-fast guard before each field's async rules. Nullable mirror: if the field's sync rules contain `nullable()` and the value is null, the async phase also skips (no DB query). For each async rule: `__infra_error__:` prefix detection returns `AsyncValidationError::Infra(FrameworkError::database(rest))` immediately; all other `Err(msg)` accumulate as field errors.

Seven tests in `#[cfg(test)] mod tests`:
- `async_validator_all_pass` — sync + OkRule → Ok(()).
- `async_validator_sync_first` — required() fails → CountingRule counter stays 0.
- `async_validator_skips_async_on_sync_error` — same, checks Validation error shape.
- `async_validator_infra_error_shape` — InfraRule returns `__infra_error__: boom` → Err(Infra), field map never contains the sentinel.
- `async_validator_nullable_skips_async` — nullable() + null value → CountingRule counter stays 0.
- `async_validator_validation_failure_shape` — FailRule → Err(Validation) with field present.
- `async_validator_unique_duplicate_is_validation` — `#[serial]`, real SQLite DB, seed_widget("taken"), AsyncValidator with `unique("widgets","slug")` → Err(Validation) with "slug" field.

## Verification

- `cargo test -p ferro-rs --lib validation::` — 69 passed, 0 failed (includes all 7 new async_validator tests + carried-over Plans 01/02 tests)
- `cargo clippy -p ferro-rs --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `framework/src/validation/validator.rs` — zero diff across all commits (SC4 confirmed via `git diff`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Formatting] Apply cargo fmt to new file**
- **Found during:** Pre-commit gate
- **Issue:** Long async fn signatures in test `impl AsyncRule` blocks and one `.async_rule(...)` call exceeded line limit.
- **Fix:** Applied `cargo fmt --all`.
- **Files modified:** `framework/src/validation/async_validator.rs`
- **Commit:** e1e28e32

### Structural Deviation: Both Tasks Implemented in One File Creation

The plan asked to create the struct/builders in Task 1 and `validate_async` in Task 2. The file was written with both in a single pass during Task 1 because the TDD `behavior` spec in Task 2 was clear enough to implement and test immediately. Task 2 became "run tests and verify", not "add new code". Tests proved the implementation correct on first run (all 7 passed). This is not a correctness concern — the acceptance criteria for both tasks are fully satisfied.

## Known Stubs

None. `AsyncValidator::validate_async` runs real sync rules and real async rules against a real DB (for the `#[serial]` test). No hardcoded counters, no placeholder returns.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns. `async_validator.rs` introduces no SQL and no end-user-reachable input path.

Threat model compliance:
- **T-190-02 (Tampering/DoS — infra error masked as validation):** `__infra_error__:` sentinel intercepted and returned as `AsyncValidationError::Infra(FrameworkError)` (→ 500). Proven by `async_validator_infra_error_shape`.
- **T-190-04 (Information disclosure — infra detail leaked into field error):** Infra messages stripped of sentinel prefix, wrapped in `FrameworkError::database`, never added to field error map. Proven by same test.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| framework/src/validation/async_validator.rs | FOUND |
| framework/src/validation/mod.rs (mod async_validator; present) | FOUND |
| commit f3ab4193 (Task 1: struct/builders) | FOUND |
| commit e1e28e32 (Task 2: run loop + tests + fmt) | FOUND |
| 7 async_validator tests pass | VERIFIED (69/69 total) |
| validator.rs byte-unchanged (SC4) | VERIFIED (git diff empty) |
| clippy clean | VERIFIED |
| fmt clean | VERIFIED |

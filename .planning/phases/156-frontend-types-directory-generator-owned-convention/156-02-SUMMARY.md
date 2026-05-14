---
phase: 156
plan: "02"
subsystem: ferro-cli/doctor
tags: [doctor, convention-check, frontend-types, ferro-cli]
dependency_graph:
  requires: []
  provides: [FrontendTypesConventionCheck, doctor-11-checks]
  affects: [ferro-cli/src/doctor/checks, ferro-cli/src/doctor/registry.rs, ferro-cli/src/doctor/check.rs]
tech_stack:
  added: []
  patterns: [DoctorCheck-trait, check_impl-free-fn, TempDir-tests, GENERATED_ALLOWLIST-constant]
key_files:
  created:
    - ferro-cli/src/doctor/checks/frontend_types_convention.rs
  modified:
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/registry.rs
    - ferro-cli/src/doctor/check.rs
decisions:
  - No category() override — defaults to CheckCategory::General (convention check, not deploy-specific)
  - Severity hard-capped at Warn — IO errors return Ok("unreadable (skipped)"), never Error
  - GENERATED_ALLOWLIST uses set-membership semantics: any subset of allowed files passes
  - Alphabetical placement in mod.rs: between docker_template_drift and generated_artifacts
metrics:
  completed_date: "2026-05-14"
  tasks: 2
  files_created: 1
  files_modified: 3
---

# Phase 156 Plan 02: FrontendTypesConventionCheck Doctor Check Summary

**One-liner:** New `ferro doctor` check detecting hand-written files in `frontend/src/types/` (the generator-owned directory), registered as check #11 in `default_checks()`.

## What Was Built

Added `FrontendTypesConventionCheck` to the `ferro doctor` check registry. The check reads `frontend/src/types/` one level deep, filters against a two-element allowlist (`inertia-props.ts`, `routes.ts` — the only files `ferro generate-types` produces), and returns `Warn` listing each hand-written filename with a redirect to `frontend/src/lib/types/`. Absent directory and unreadable directory both return `Ok` — the check is purely advisory.

**Task 1:** Created `ferro-cli/src/doctor/checks/frontend_types_convention.rs` with `FrontendTypesConventionCheck` struct, `DoctorCheck` impl, `check_impl` free function, and 6 unit tests.

**Task 2:** Wired into `mod.rs` (`pub mod` + `pub use`), `registry.rs` (import + `Box::new` at position 11, docstring updated, test renamed to `_eleven_` with count 11), and `check.rs` (`general_names` slice extended with `"frontend_types_convention"`).

## Verification

```
cargo fmt --all -- --check      # exit 0
cargo clippy --all --all-targets -- -D warnings  # exit 0, zero warnings
cargo test -p ferro-cli --lib doctor::  # 48 passed, 0 failed
```

Specific tests confirmed passing:
- `doctor::checks::frontend_types_convention::tests` — 6 tests (absent dir, only generated, only routes.ts, hand-written warn, mixed warn)
- `doctor::registry::tests::default_checks_returns_eleven_in_declared_order` — len 11, correct order
- `doctor::registry::tests::deploy_category_filter_returns_two` — unchanged, still 2 deploy checks
- `doctor::check::tests::non_deploy_checks_return_general_category` — includes new check in general_names

## Deviations from Plan

None — plan executed exactly as written. The check file content, allowlist, struct name, test names, and wiring all match the plan specification verbatim.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes. The check performs one `std::fs::read_dir` on a fixed path relative to the project root (`frontend/src/types`). IO errors fall through to `Ok("unreadable (skipped)")` per T-156-02-01. No new threat surface beyond what the plan's threat model documents.

## Self-Check: PASSED

Files exist:
- `ferro-cli/src/doctor/checks/frontend_types_convention.rs` — FOUND (147 lines)
- `ferro-cli/src/doctor/checks/mod.rs` — FOUND, contains `pub mod frontend_types_convention` and `pub use frontend_types_convention::FrontendTypesConventionCheck`
- `ferro-cli/src/doctor/registry.rs` — FOUND, contains `Box::new(FrontendTypesConventionCheck)` and `assert_eq!(checks.len(), 11)`
- `ferro-cli/src/doctor/check.rs` — FOUND, contains `"frontend_types_convention"` in `general_names`

Commits exist:
- `7a927397` — test(156-02): add failing tests for FrontendTypesConventionCheck
- `24d29ef7` — feat(156-02): register FrontendTypesConventionCheck in doctor registry

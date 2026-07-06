---
phase: 128-deploy-preflight
plan: 01
subsystem: infra
tags: [rust, ferro-cli, doctor, deploy, refactor]

# Dependency graph
requires:
  - phase: 124-doctor-introspection-and-ci-scaffold
    provides: DoctorCheck trait and check registry
  - phase: 122-deploy-scaffold-core-rewrite
    provides: deploy module and rewrite_ferro_version helper
provides:
  - CheckCategory enum (General, Deploy) exported from doctor::check
  - DoctorCheck::category() default method returning General
  - pub(crate) read_path_dep_version helper in deploy::mod
affects: [128-02, 128-03, ferro-mcp deploy_check tool]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Category filter via default trait method — new checks override category() to return Deploy"
    - "Shared version-resolution helper in deploy::mod, called from both doctor checks and deploy rewriter"

key-files:
  created: []
  modified:
    - ferro-cli/src/doctor/check.rs
    - ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs
    - ferro-cli/src/deploy/mod.rs
    - ferro-cli/src/deploy/rewrite_ferro_version.rs

key-decisions:
  - "CheckCategory as enum (not tag set) — only two variants needed, default impl covers all nine existing checks with zero edits"
  - "Shared read_path_dep_version uses toml::Value (consistent with staleness check) rather than toml_edit — read-only path, no formatting preservation needed"

patterns-established:
  - "Category filter: default category() = General; deploy checks override to Deploy; filter applied at call site not registry"
  - "Version helper: pub(crate) fn in deploy::mod is the single source; both doctor checks and the rewriter call it"

requirements-completed: [REPORT-04, REPORT-13, REPORT-17]

# Metrics
duration: 5min
completed: 2026-04-09
---

# Phase 128 Plan 01: Wave 0 Foundation Summary

**CheckCategory enum + category() default on DoctorCheck trait; single read_path_dep_version helper in deploy::mod replacing two private duplicates**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-09T03:48:29Z
- **Completed:** 2026-04-09T03:53:16Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `CheckCategory` enum with `General` and `Deploy` variants to `doctor::check`
- Added `category()` default method to `DoctorCheck` trait returning `General` — all nine existing checks compile unchanged
- Extracted `read_path_dep_version` to `pub(crate)` helper in `deploy/mod.rs`, eliminating duplicates in `cargo_docker_toml_staleness.rs` and `rewrite_ferro_version.rs`
- Added test `default_category_is_general_for_all_registry_checks` verifying all registry checks return General

## Task Commits

1. **Task 1: Add CheckCategory enum + category() default to DoctorCheck trait** - `65bf976a` (feat)
2. **Task 2: Extract read_path_dep_version to deploy/mod.rs** - `ec620555` (refactor)

## Files Created/Modified

- `ferro-cli/src/doctor/check.rs` — Added CheckCategory enum and category() default method + test
- `ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs` — Removed private read_path_dep_version, use shared helper via crate::deploy
- `ferro-cli/src/deploy/mod.rs` — Added pub(crate) read_path_dep_version shared helper
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — Removed private read_path_dep_version, use super::read_path_dep_version

## Decisions Made

- Used `CheckCategory` enum (not trait method returning `Option<&str>` or tag set) — minimal surface, exactly two variants needed for this phase, default impl is zero-code-change for existing checks.
- Shared helper uses `toml::Value` (not `toml_edit`) — read-only path, no formatting concerns; matches the existing pattern in `cargo_docker_toml_staleness.rs`.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

- Plan 02 (D-02 filter mechanism): `CheckCategory::Deploy` is defined; `ferry doctor --deploy` flag can filter `default_checks()` by `category() == Deploy`.
- Plan 03 (ferro_version_skew check): `crate::deploy::read_path_dep_version` is available for the new check without duplication.
- All nine existing checks compile and pass unchanged.

---
*Phase: 128-deploy-preflight*
*Completed: 2026-04-09*

---
phase: 128-deploy-preflight
plan: 02
subsystem: infra
tags: [rust, ferro-cli, doctor, deploy, checks, preflight]

# Dependency graph
requires:
  - phase: 128-deploy-preflight
    plan: 01
    provides: CheckCategory enum + read_path_dep_version helper
  - phase: 124-doctor-introspection-and-ci-scaffold
    provides: DoctorCheck trait and check registry
provides:
  - CopyDirsDockerignoreCollisionCheck (CheckCategory::Deploy)
  - FerroVersionSkewCheck (CheckCategory::Deploy)
  - cargo_docker_toml_staleness categorized as Deploy
  - default_checks() with 11 entries
  - ferro doctor --deploy filter flag
affects: [128-03, ferro-mcp deploy_check tool]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "check_impl separation: pub(crate) fn check_impl(root) callable from tests without constructing struct"
    - "deploy filter: deploy_only flag filters default_checks() by CheckCategory::Deploy at call site"
    - "DriftKind enum: classifies version pairs as None/Patch/MajorMinor for clear match arms"

key-files:
  created:
    - ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs
    - ferro-cli/src/doctor/checks/ferro_version_skew.rs
  modified:
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs
    - ferro-cli/src/doctor/registry.rs
    - ferro-cli/src/commands/doctor.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/doctor/check.rs

key-decisions:
  - "deploy filter at call site: all_checks filtered by category() == Deploy in commands/doctor.rs, not in registry"
  - "DriftKind internal enum: cleaner than ad-hoc string classification; not exposed in public API"
  - "Updated check.rs test: renamed default_category_is_general_for_all_registry_checks to non_deploy_checks_return_general_category to correctly reflect new reality"

requirements-completed: [REPORT-03, REPORT-04, REPORT-13, REPORT-17]

# Metrics
duration: ~5min
completed: 2026-04-09
---

# Phase 128 Plan 02: Two new deploy checks + --deploy flag Summary

**Two deploy-specific doctor checks (`copy_dirs_dockerignore_collision`, `ferro_version_skew`) plus `--deploy` filter flag on `ferro doctor`; all three checks categorized as `CheckCategory::Deploy` and registered in `default_checks()` at positions 7-8 of 11.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-04-09T03:55:06Z
- **Completed:** 2026-04-09T03:59:58Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Created `copy_dirs_dockerignore_collision.rs`: flags `copy_dirs` entries excluded by `.dockerignore`; skips gracefully when either file is absent; returns `Error` with details on collision
- Created `ferro_version_skew.rs` (killer feature): detects version drift between local `ferro*` path deps and `Cargo.docker.toml`; `Error` on major/minor drift, `Warn` on patch-only drift
- Extended `CargoDockerTomlStalenessCheck::category()` to return `CheckCategory::Deploy`
- Updated `default_checks()` from 9 to 11 checks in canonical order; renamed ordering test to `default_checks_returns_eleven_in_declared_order`
- Added `deploy_category_filter_returns_three` test verifying the deploy filter returns exactly the right 3 checks
- Added `--deploy` flag to `ferro doctor` CLI; filters `default_checks()` by `CheckCategory::Deploy` at call site

## Task Commits

1. **Task 1: add copy_dirs_dockerignore_collision deploy check** — `d088d26f` (feat)
2. **Task 2: add ferro_version_skew deploy check (killer feature)** — `040c8cc6` (feat)
3. **Task 3: register deploy checks, add --deploy flag** — `c7b16d29` (feat)

## Files Created/Modified

- `ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs` — NEW: deploy preflight D-04 check
- `ferro-cli/src/doctor/checks/ferro_version_skew.rs` — NEW: deploy preflight D-05 check (killer feature)
- `ferro-cli/src/doctor/checks/mod.rs` — Added two new module declarations and pub use re-exports
- `ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs` — Added category() override returning Deploy
- `ferro-cli/src/doctor/registry.rs` — Updated imports, default_checks() to 11 entries, updated tests
- `ferro-cli/src/commands/doctor.rs` — Added deploy_only param, apply filter when true
- `ferro-cli/src/main.rs` — Added --deploy arg to Doctor variant and dispatch arm
- `ferro-cli/src/doctor/check.rs` — Updated test name/logic to reflect new Deploy checks

## Decisions Made

- Deploy filter is applied at the call site in `commands/doctor.rs` (not in registry) — consistent with the pattern from 128-01's design; registry remains the single source of truth.
- `DriftKind` as a private enum (not `&str` or `i8`) — improves readability of the `classify` + `match` arms without leaking abstraction.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated stale `default_category_is_general_for_all_registry_checks` test in check.rs**
- **Found during:** Task 3
- **Issue:** Plan 01 added a test asserting all registry checks return `General`. Plan 02 intentionally changes `cargo_docker_toml_staleness` to return `Deploy`, so the test broke.
- **Fix:** Renamed to `non_deploy_checks_return_general_category` and updated to check only the 8 General checks by name.
- **Files modified:** `ferro-cli/src/doctor/check.rs`
- **Commit:** `c7b16d29`

**2. [Rule 2 - Format] Applied rustfmt to ferro_version_skew.rs**
- **Found during:** Task 3 overall verification
- **Issue:** `cargo fmt --all -- --check` flagged a method chain formatting difference in `ferro_version_skew.rs`.
- **Fix:** Ran `cargo fmt --all` to auto-format.
- **Files modified:** `ferro-cli/src/doctor/checks/ferro_version_skew.rs`
- **Commit:** `c7b16d29` (formatted before commit)

## Known Stubs

None — all checks are fully wired with real file I/O and version resolution.

---

## Self-Check: PASSED

- `copy_dirs_dockerignore_collision.rs`: FOUND
- `ferro_version_skew.rs`: FOUND
- Commit `d088d26f`: FOUND
- Commit `040c8cc6`: FOUND
- Commit `c7b16d29`: FOUND

---

*Phase: 128-deploy-preflight*
*Completed: 2026-04-09*

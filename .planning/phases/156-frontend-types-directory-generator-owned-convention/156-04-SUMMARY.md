---
phase: 156
plan: "04"
subsystem: ferro-cli/commands/docker_init,ferro-cli/doctor/checks/docker_template_drift
tags: [docker, types-gen, resolve_ferro_version, call-site-wiring]
dependency_graph:
  requires: [03]
  provides: [docker_init.resolve_ferro_version_wired, docker_template_drift.resolve_ferro_version_wired, smoke_tests]
  affects:
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/doctor/checks/docker_template_drift.rs
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/tests/gestiscilo_fixture.rs
tech_stack:
  added: []
  patterns: [caller-resolved-version, CLI-flag-override-pattern]
key_files:
  created: []
  modified:
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/doctor/checks/docker_template_drift.rs
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/tests/gestiscilo_fixture.rs
decisions:
  - "resolve_ferro_version promoted from pub(crate) to pub — required for external integration test in tests/ to call it directly"
  - "ferro_version_flag parameter renamed from _ferro_version_flag (underscore-unused) to ferro_version_flag (active); CLI override wired via unwrap_or_else chain"
metrics:
  duration: "~7 minutes"
  completed: "2026-05-14"
  tasks: 2
  files: 4
---

# Phase 156 Plan 04: Wire resolve_ferro_version Call Sites Summary

Closed the loop on the Dockerfile types-gen fix by replacing all three `env!("CARGO_PKG_VERSION")` placeholders (introduced by Plan 03) with real `resolve_ferro_version` calls. The rendered Dockerfile now pins `cargo install ferro-cli` to the exact version the project compiles against, not the doctor binary's own version.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Wire resolve_ferro_version in docker_init + CLI flag override + smoke tests | ec5e6612 | ferro-cli/src/commands/docker_init.rs |
| 2 | Wire resolve_ferro_version in docker_template_drift + gestiscilo_fixture | 25fe408b | ferro-cli/src/doctor/checks/docker_template_drift.rs, ferro-cli/src/templates/docker.rs, ferro-cli/tests/gestiscilo_fixture.rs |

## What Was Built

**`docker_init.rs` call site wired** — `ferro_version_flag` parameter (renamed from `_ferro_version_flag`) is now active: CLI `--ferro-version <X>` overrides the resolver. When no flag is provided, `resolve_ferro_version(&root)` reads the project's Cargo.lock. The `DockerContext { ferro_version, ... }` literal uses the resolved value.

**`docker_template_drift.rs` call site wired** — `check_impl` now calls `resolve_ferro_version(root)` so the drift check reconstructs an expected Dockerfile using the same version that `docker:init` would render. Before this plan, the drift check always compared against the doctor binary's own version, producing false positives on any project with a different ferro-rs version in its Cargo.lock.

**`resolve_ferro_version` promoted to `pub`** — The function was `pub(crate)` in Plan 03. The external integration test `tests/gestiscilo_fixture.rs` cannot call `pub(crate)` symbols. Promoting to `pub` matches the visibility pattern of all other exported functions in `ferro-cli/src/templates/docker.rs` and is the correct fix (versus keeping the placeholder in the external test).

**Two new smoke tests in `docker_init.rs`:**
- `dockerfile_pins_to_cargo_lock_ferro_version` — creates a temp project with `ferro-rs = "9.9.9"` in Cargo.lock, asserts rendered Dockerfile contains `--version 9.9.9`
- `dockerfile_falls_back_to_env_version_when_no_cargo_lock` — creates a temp project without Cargo.lock, asserts rendered Dockerfile pins to `env!("CARGO_PKG_VERSION")`

## Test Results

- `cargo test -p ferro-cli --lib commands::docker_init` — 4 tests pass (2 footer + 2 new smoke)
- `cargo test -p ferro-cli --lib doctor::checks::docker_template_drift` — 4 tests pass
- `cargo test -p ferro-cli --lib templates::docker` — 28 tests pass (Plan 03 tests unaffected)
- `cargo test -p ferro-cli --all-features` — all 497 lib + integration tests pass

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] gestiscilo_fixture.rs also needed resolve_ferro_version**

- **Found during:** Task 2 clippy run (`--all-targets` caught the external test crate)
- **Issue:** `tests/gestiscilo_fixture.rs` still had the Plan 03 placeholder `// Phase 156 Plan 04 will replace this with resolve_ferro_version(&root)`. The plan's success criteria requires "No `// Phase 156 Plan 04 will replace` comments remain anywhere in the workspace".
- **Fix:** Updated the import and replaced `env!("CARGO_PKG_VERSION").to_string()` with `resolve_ferro_version(&root)` in `build_docker_context()`.
- **Files modified:** `ferro-cli/tests/gestiscilo_fixture.rs`
- **Commit:** 25fe408b

**2. [Rule 3 - Blocking] resolve_ferro_version visibility: pub(crate) not accessible from tests/**

- **Found during:** Task 2 clippy run after patching gestiscilo_fixture.rs
- **Issue:** External test crates (in `ferro-cli/tests/`) cannot call `pub(crate)` symbols from the library. Calling `resolve_ferro_version` from `gestiscilo_fixture.rs` failed with E0603 (private function).
- **Fix:** Promoted `resolve_ferro_version` from `pub(crate)` to `pub`, matching the visibility of all other exported functions in `docker.rs` (`read_rust_channel`, `render_dockerfile`, etc.). Also removed the `#[allow(dead_code)]` attribute (now unused since both call sites are wired).
- **Files modified:** `ferro-cli/src/templates/docker.rs`
- **Commit:** 25fe408b

## Known Stubs

None. All Plan 03 placeholders have been replaced with real resolution calls.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- `ferro-cli/src/commands/docker_init.rs` — exists; imports `resolve_ferro_version`; `execute` uses `ferro_version_flag` (active); `DockerContext { ferro_version, ... }` wired; 2 new smoke tests present; no Plan 04 comment
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — exists; imports `resolve_ferro_version`; `check_impl` uses `ferro_version: resolve_ferro_version(root),`; no Plan 04 comment; test fixtures retain `"0.0.0-test"` deterministic value
- `ferro-cli/src/templates/docker.rs` — `resolve_ferro_version` is `pub` (promoted from `pub(crate)`); `#[allow(dead_code)]` removed
- `ferro-cli/tests/gestiscilo_fixture.rs` — imports `resolve_ferro_version`; uses `resolve_ferro_version(&root)`; no Plan 04 comment
- Workspace-wide grep for "Plan 04 will replace" returns zero matches in ferro-cli source
- Commit `ec5e6612` — present in git log (Task 1)
- Commit `25fe408b` — present in git log (Task 2)
- All 497 ferro-cli tests pass

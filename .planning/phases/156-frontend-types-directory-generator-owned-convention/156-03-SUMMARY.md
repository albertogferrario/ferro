---
phase: 156
plan: "03"
subsystem: ferro-cli/templates/docker
tags: [docker, types-gen, renderer, tdd]
dependency_graph:
  requires: []
  provides: [DockerContext.ferro_version, TYPES_GEN_STAGE_BODY, FRONTEND_STAGE_WITH_TYPES_COPY_BODY, resolve_ferro_version]
  affects: [ferro-cli/src/commands/docker_init.rs, ferro-cli/src/doctor/checks/docker_template_drift.rs, ferro-cli/tests/gestiscilo_fixture.rs]
tech_stack:
  added: []
  patterns: [pure-renderer, token-substitution-chain, TDD-RED-GREEN]
key_files:
  created: []
  modified:
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/doctor/checks/docker_template_drift.rs
    - ferro-cli/tests/gestiscilo_fixture.rs
decisions:
  - "resolve_ferro_version marked #[allow(dead_code)] with comment; Plan 04 wires the call sites"
  - "gestiscilo_fixture.rs integration test also required patching (not noted in plan — Rule 3 auto-fix)"
  - "FRONTEND_STAGE_BODY deleted per Phase 130/156 no-dead-code convention; verified zero external references"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-14"
  tasks: 2
  files: 4
---

# Phase 156 Plan 03: Dockerfile types-gen Stage + resolve_ferro_version Summary

Extended the Dockerfile renderer to emit a `types-gen` Rust stage that regenerates `frontend/src/types/` inside the Docker build context, ensuring `docker build` no longer fails at `tsc` due to the gitignored types directory.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add DockerContext.ferro_version + token substitution + types-gen stage + new tests | 7e35576c | ferro-cli/src/templates/docker.rs |
| 2 | Patch the two existing DockerContext call sites with placeholder ferro_version | 7ec75d32 | ferro-cli/src/commands/docker_init.rs, ferro-cli/src/doctor/checks/docker_template_drift.rs, ferro-cli/tests/gestiscilo_fixture.rs |

## What Was Built

**`DockerContext.ferro_version: String`** — new field carrying the pinned ferro-cli version for the `types-gen` stage's `cargo install` command.

**`TYPES_GEN_STAGE_BODY`** — new stage constant emitted before the frontend-builder when `has_frontend == true`:
```
FROM rust:{{RUST_IMAGE_TAG}} AS types-gen
WORKDIR /app
RUN cargo install ferro-cli --version {{FERRO_VERSION}} --locked
COPY . .
RUN ferro generate-types
```

**`FRONTEND_STAGE_WITH_TYPES_COPY_BODY`** — updated frontend stage with `COPY --from=types-gen /app/frontend/src/types ./src/types` positioned immediately before `RUN npm run build`.

**`resolve_ferro_version(project_root: &Path) -> String`** — `pub(crate)` helper that parses `Cargo.lock` for the `ferro-rs` package version, falling back to `env!("CARGO_PKG_VERSION")` when absent.

**Token substitution chain** — `{{FERRO_VERSION}}` added after `{{RUST_IMAGE_TAG}}` in `render_dockerfile`; `{{FRONTEND_STAGE}}` first ensures `TYPES_GEN_STAGE_BODY`'s `{{RUST_IMAGE_TAG}}` is present before the `{{RUST_IMAGE_TAG}}` pass.

## Test Results

28 docker template tests pass (19 existing + 9 new):
- `types_gen_stage_present_when_has_frontend`
- `types_gen_stage_absent_when_no_frontend`
- `copy_from_types_gen_before_npm_build`
- `ferro_version_token_resolved`
- `types_gen_stage_uses_same_rust_image_tag`
- `no_unresolved_tokens_with_frontend_stage`
- `resolve_ferro_version_reads_cargo_lock`
- `resolve_ferro_version_falls_back_when_lockfile_absent`
- `resolve_ferro_version_falls_back_when_ferro_rs_absent`

489 ferro-cli lib tests pass total.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] gestiscilo_fixture.rs integration test also required `ferro_version` patch**

- **Found during:** Task 2 (clippy `--all-targets` revealed the integration test crate)
- **Issue:** `ferro-cli/tests/gestiscilo_fixture.rs` constructs `DockerContext` directly in `build_docker_context()` — not mentioned in the plan's Task 2 file list
- **Fix:** Added `ferro_version: env!("CARGO_PKG_VERSION").to_string()` with the Plan 04 placeholder comment
- **Files modified:** `ferro-cli/tests/gestiscilo_fixture.rs`
- **Commit:** 7ec75d32

**2. [Rule 1 - Dead code] FRONTEND_STAGE_BODY deleted**

- **Found during:** Task 1 review
- **Issue:** After replacing `FRONTEND_STAGE_BODY.to_string()` with the two-stage format, `FRONTEND_STAGE_BODY` became unused. Keeping it would fail clippy `-D warnings` (dead_code lint). The plan mentioned preferring deletion if no external references exist.
- **Fix:** Confirmed `grep -rn FRONTEND_STAGE_BODY ferro-cli/` shows only `docker.rs` (now cleaned up). Deleted the constant.
- **Files modified:** `ferro-cli/src/templates/docker.rs`
- **Commit:** 7e35576c

**3. [Rule 1 - Dead code] resolve_ferro_version needs #[allow(dead_code)]**

- **Found during:** Task 1 clippy run
- **Issue:** `resolve_ferro_version` is declared `pub(crate)` but neither call site uses it yet (both use `env!("CARGO_PKG_VERSION")` as placeholders per plan). Clippy `-D warnings` rejects unused functions.
- **Fix:** Added `#[allow(dead_code)]` with comment `// Plan 04 wires the two call sites; suppress dead-code until that lands.`
- **Files modified:** `ferro-cli/src/templates/docker.rs`
- **Commit:** 7e35576c

## Known Stubs

- `docker_init.rs`: `ferro_version: env!("CARGO_PKG_VERSION").to_string()` — intentional placeholder; Plan 04 replaces with `resolve_ferro_version(&root)`
- `docker_template_drift.rs`: same placeholder in `check_impl` — intentional; Plan 04
- `gestiscilo_fixture.rs`: same placeholder in `build_docker_context` — intentional; Plan 04

These stubs do not block this plan's goal (renderer correctness). They are the documented pre-condition for Plan 04.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes beyond what was specified in the plan's threat model.

## Self-Check: PASSED

- `ferro-cli/src/templates/docker.rs` — exists, contains all required constants and functions
- `ferro-cli/src/commands/docker_init.rs` — exists, contains `ferro_version` placeholder
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — exists, contains `ferro_version` placeholder in both production path and both test fixtures
- `ferro-cli/tests/gestiscilo_fixture.rs` — exists, contains `ferro_version` placeholder
- Commit `7e35576c` — present in git log
- Commit `7ec75d32` — present in git log
- 28 docker template tests pass, 489 ferro-cli lib tests pass

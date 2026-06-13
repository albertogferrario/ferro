---
phase: "214"
plan: "02"
subsystem: ferro-cli
tags: [ci, scaffold, testing, template-drift]
dependency_graph:
  requires: [214-01]
  provides: [scaffold-compile-gate, post-publish-smoke]
  affects: [framework/src/validation, ferro-cli/templates, .github/workflows]
tech_stack:
  added: []
  patterns: [cargo-patch-crates-io, docker-build-arg, github-actions-needs]
key_files:
  created:
    - ferro-cli/tests/benchmark_new_project.rs (scaffold_builds_against_workspace_ferro test)
  modified:
    - ferro-cli/src/templates/scaffold.rs
    - ferro-cli/src/templates/auth.rs
    - ferro-cli/src/templates/files/backend/bootstrap.rs.tpl
    - ferro-cli/src/templates/files/backend/controllers/auth.rs.tpl
    - ferro-cli/src/templates/files/backend/controllers/profile.rs.tpl
    - ferro-cli/src/templates/files/backend/controllers/settings.rs.tpl
    - ferro-cli/src/commands/make_scaffold.rs
    - framework/src/validation/validator.rs
    - .github/workflows/ci.yml
    - .github/workflows/publish.yml
    - ferro-cli/tests/fixtures/benchmark/Dockerfile
decisions:
  - "[patch.crates-io] uses absolute path from CARGO_MANIFEST_DIR parent so the test works from any working directory"
  - "scaffold-smoke CI job runs cargo test -p ferro-cli (not --all-features) to avoid asm/nasm codec flags on CI runners"
  - "post-publish smoke waits up to 6 minutes polling crates.io before building the Docker image"
  - "Dockerfile ARG FERRO_VERSION defaults to the current pinned version so local manual runs still work"
  - "Validator::with_error() added to framework for cross-field validation error pre-seeding (used by auth templates)"
metrics:
  duration: "~90 minutes"
  completed: "2026-06-13"
  tasks_completed: 3
  files_changed: 12
---

# Phase 214 Plan 02: CI Scaffold Compile Guard Summary

Two-layer permanent CI guard ensuring a non-compiling scaffold can never ship: a workspace-level compile test (fast feedback on every PR) and a post-publish Docker smoke test (end-to-end validation against the published binary).

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | scaffold_builds_against_workspace_ferro test + template drift fixes | daf72308 |
| 2 | scaffold-smoke job in ci.yml | 1371bbb7 |
| 3 | Dockerfile ARG FERRO_VERSION + post-publish-scaffold-smoke in publish.yml | ea52eb2f |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 51 compile errors in scaffolded app detected by the test**

- **Found during:** Task 1 — the new test immediately caught template drift
- **Issue:** Templates emitted API calls that don't exist on published ferro-rs
- **Fixes applied:**
  - `bootstrap.rs.tpl`: `ferro::Queue, ferro::QueueConfig` → `ferro::queue::{Queue, QueueConfig}`
  - `controllers/profile.rs.tpl`: `Auth::user_id()` → `Auth::id()`, `FrameworkError::AuthenticationRequired` → `FrameworkError::Unauthorized`, `FrameworkError::NotFound` → `FrameworkError::model_not_found()`, `User::find_by_id()` → `user::Entity::find_by_pk()`, inline ModelMut-based updates replacing nonexistent helper methods
  - `controllers/settings.rs.tpl`: same `Auth::user_id()` / `AuthenticationRequired` fixes
  - `controllers/auth.rs.tpl`: added `database::{Model as DatabaseModel, ModelMut}` + `sea_orm::Set` imports; replaced `user.update_password()` with inline hashing + ActiveModel update
  - `templates/auth.rs` (API auth controller): replaced `DB::connection()` + `Entity::insert().exec_with_returning()` with `Entity::insert_one()` via `ModelMut`
  - `templates/scaffold.rs` (all controller templates): replaced all `req.db()` + raw SeaORM calls with `DatabaseModel`/`ModelMut` trait methods; fixed `req.param("id").unwrap_or_default() as i64` → `req.param_as::<i64>("id")`; fixed FK fetch code; fixed insert/update/delete patterns
  - `commands/make_scaffold.rs`: removed `Eq` from model `#[derive(...)]` (float fields don't implement `Eq`); changed `update_fields` generation from `.set_field()` chain to `active.field = Set(...)` statements
- **Files modified:** 8 template/source files
- **Commit:** daf72308

**2. [Rule 2 - Missing functionality] Validator::with_error() absent from framework**

- **Found during:** Task 1 — auth templates use `validator.with_error()` for cross-field checks
- **Fix:** Added `with_error(field, message)` method to `Validator<'a>` in `framework/src/validation/validator.rs`; pre-seeds errors before rule evaluation so they appear in the final `ValidationError` alongside rule-based errors
- **Files modified:** `framework/src/validation/validator.rs`
- **Commit:** daf72308

## Known Stubs

None — the guard is a test + CI job, no UI or data stubs.

## CI Notes

The `workflow` scope is not available on the CI token, so `.github/workflows/ci.yml` and `.github/workflows/publish.yml` changes must be pushed manually (standard `git push`). This is a known constraint documented in project memory.

The `scaffold-smoke` CI job uses `cargo test -p ferro-cli` without `--all-features` to avoid the asm/nasm codec dependency that would fail on GitHub runners without `nasm` installed.

## Self-Check

Files exist:
- ferro-cli/tests/benchmark_new_project.rs: FOUND (modified in place)
- .github/workflows/ci.yml: FOUND
- .github/workflows/publish.yml: FOUND
- ferro-cli/tests/fixtures/benchmark/Dockerfile: FOUND

Commits exist: daf72308, 1371bbb7, ea52eb2f — all verified in git log.

Test result: `scaffold_builds_against_workspace_ferro` passed (1 passed, 0 failed).

## Self-Check: PASSED

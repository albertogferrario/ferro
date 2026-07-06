---
phase: 214-scaffold-library-parity
verified: 2026-06-13T00:00:00Z
status: passed
score: 10/10
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 214: Scaffold↔Library Parity & CI Smoke Test — Verification Report

**Phase Goal:** A freshly scaffolded app (`ferro new → make:auth → make:scaffold ×3 → make:job → cargo build`) compiles clean against the published `ferro` crate, and a CI smoke test enforces this on every release so a non-compiling scaffold can never ship silently again.
**Verified:** 2026-06-13T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                    | Status     | Evidence                                                                                                                           |
|----|------------------------------------------------------------------------------------------|------------|------------------------------------------------------------------------------------------------------------------------------------|
| 1  | `framework/src/lib.rs` exports `error_response!` macro at crate root                    | ✓ VERIFIED | `macro_rules! error_response` at line 404; `#[macro_export]` at line 403; body is bare `HttpResponse::json().status()` chain     |
| 2  | `framework/src/lib.rs` exports `ActiveValue` in the `pub use sea_orm` facade block      | ✓ VERIFIED | `ActiveValue,` present at line 123 in the `pub use sea_orm::{ActiveModelTrait, ActiveValue, ...}` block                           |
| 3  | Full-stack scaffold templates emit no non-existent `HttpResponse` methods (CR-01 fixed) | ✓ VERIFIED | No `HttpResponse::(internal_server_error\|not_found\|bad_request\|redirect)` in scaffold.rs; all error arms use `ferro::error_response!`; redirects use `Inertia::redirect_ctx` / `Inertia::redirect` |
| 4  | API scaffold templates import `ActiveValue` and `ValidateRules` from the ferro facade   | ✓ VERIFIED | Lines 963-965, 1270 of scaffold.rs: `use ferro::{{..., ActiveValue, ..., ValidateRules}}` and `#[derive(..., ValidateRules)]` on all four form structs |
| 5  | `make:job` template routes through `ferro::queue::*` with no `ferro-queue` direct import | ✓ VERIFIED | `make.rs:339`: `use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}}` — no `use ferro_queue` present                       |
| 6  | `make:auth` template references `crate::models::user` (singular) and correct DB pattern  | ✓ VERIFIED | `auth.rs:105`: `use crate::models::user;`; no `models::users` plural; DB insert uses `Entity::insert_one()` via `ModelMut` (auto-fixed deviation from plan, test passes) |
| 7  | Non-ignored test `scaffold_builds_against_workspace_ferro` scaffolds full sequence including non-`--api` step and compiles with `[patch.crates-io]` | ✓ VERIFIED | `benchmark_new_project.rs:15-16`: `#[test]` only (no `#[ignore]`); Steps 3a-3c use `--api`, Step 3d scaffolds `Post` without `--api` (full-stack templates); `[patch.crates-io]` appended at line 147; test passed per 214-02-SUMMARY |
| 8  | `ci.yml` has a `scaffold-smoke` job running the test on every push                      | ✓ VERIFIED | Lines 93-103 of ci.yml: `scaffold-smoke` job, `needs: check`, toolchain `"1.88.0"`, runs `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture` |
| 9  | `Dockerfile` parameterized with `ARG FERRO_VERSION`                                     | ✓ VERIFIED | `Dockerfile:34`: `ARG FERRO_VERSION=0.2.55`; line 42: `cargo install ferro-cli --version ${FERRO_VERSION} --locked`; retry loop present (lines 42-46) |
| 10 | `publish.yml` has `post-publish-scaffold-smoke` job gating every release                | ✓ VERIFIED | Lines 330-374 of publish.yml: job `needs: [check-version, publish]`, `if: always() && needs.publish.result == 'success'`; extracts version, waits for crates.io propagation, builds Dockerfile with `--build-arg FERRO_VERSION="$VERSION"`, runs container |

**Score:** 10/10 truths verified

---

### Required Artifacts

| Artifact                                              | Expected                                                    | Status     | Details                                                                                          |
|-------------------------------------------------------|-------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------|
| `framework/src/lib.rs`                                | `error_response!` macro + `ActiveValue` in sea_orm block    | ✓ VERIFIED | Both present; commits 11acc90d                                                                   |
| `ferro-cli/src/templates/scaffold.rs`                 | `ferro::ActiveValue` + `ValidateRules` derive on all forms  | ✓ VERIFIED | All four controller templates (2 Inertia + 2 API) use ferro-facade imports; CR-01 fixed; commit daf72308 |
| `ferro-cli/src/templates/make.rs`                     | job template imports `ferro::queue::*`                      | ✓ VERIFIED | `queue::{{Error, Job, Queueable}}` via ferro facade; commit e4c067c7                            |
| `ferro-cli/src/templates/auth.rs`                     | `crate::models::user` (singular) + correct DB pattern       | ✓ VERIFIED | Singular path; `Entity::insert_one()` via ModelMut (auto-fixed pattern); commits f7f5c58f + daf72308 |
| `ferro-mcp/src/tools/code_templates.rs`               | `error_response_arm` CodeTemplate surfacing `ferro::error_response!` | ✓ VERIFIED | `error_response_arm` at line 291; 3 occurrences of `ferro::error_response!` in the template; commit f7f5c58f |
| `ferro-cli/tests/benchmark_new_project.rs`            | Non-ignored `scaffold_builds_against_workspace_ferro` test  | ✓ VERIFIED | `#[test]` with no `#[ignore]`; includes Step 3d (full-stack Post); `[patch.crates-io]` block; commit daf72308 |
| `ferro-cli/tests/fixtures/benchmark/Dockerfile`       | `ARG FERRO_VERSION` parameterized                           | ✓ VERIFIED | `ARG FERRO_VERSION=0.2.55` + retry loop; commit ea52eb2f                                        |
| `.github/workflows/ci.yml`                            | `scaffold-smoke` per-PR job                                 | ✓ VERIFIED | Job at line 93; `needs: check`; correct toolchain; commit 1371bbb7                             |
| `.github/workflows/publish.yml`                       | `post-publish-scaffold-smoke` release gate                  | ✓ VERIFIED | Job at line 330; `needs: [check-version, publish]`; version extraction + crates.io wait + Docker build/run; commit ea52eb2f |
| `docs/src/the-basics/action-handlers.md`              | `error_response!` macro documented                          | ✓ VERIFIED | `## error_response! macro` subsection present; commit f7f5c58f                                  |
| `docs/src/features/database.md`                       | `ferro::ActiveValue` facade re-export noted                 | ✓ VERIFIED | Line 148: "ActiveValue is re-exported from the ferro facade as `ferro::ActiveValue`"; commit f7f5c58f |

### Key Link Verification

| From                                              | To                                                | Via                                                    | Status     | Details                                                              |
|---------------------------------------------------|---------------------------------------------------|--------------------------------------------------------|------------|----------------------------------------------------------------------|
| `scaffold.rs` API controller template             | `framework/src/lib.rs` (`error_response!`, `ActiveValue`) | `ferro::error_response!` / `ferro::ActiveValue` at crate root | ✓ WIRED    | API import blocks: `use ferro::{{..., ActiveValue, ..., ValidateRules}}` |
| `scaffold.rs` full-stack controller template      | `framework/src/lib.rs` (`error_response!`)        | `ferro::error_response!` in all error arms             | ✓ WIRED    | All error arms replaced; verified with grep across lines 530-931      |
| `make.rs` job template                            | `framework/src/lib.rs` queue module               | `use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}}` | ✓ WIRED    | Direct ferro facade import; no ferro_queue dep                       |
| `auth.rs` auth controller template                | `framework/src/lib.rs` DB + ModelMut              | `Entity::insert_one()` via `ferro::database::ModelMut` | ✓ WIRED    | `use ferro::database::ModelMut;` at line 98; insert_one at line 184  |
| `benchmark_new_project.rs` test                   | workspace `framework/` via `[patch.crates-io]`    | Appended `ferro-rs = { path = <workspace>/framework }` | ✓ WIRED    | Patch block at lines 141-155; test passed per SUMMARY                |
| `ci.yml` `scaffold-smoke` job                     | `benchmark_new_project.rs` test                   | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` | ✓ WIRED    | Line 103 of ci.yml                                                   |
| `publish.yml` `post-publish-scaffold-smoke`       | `Dockerfile`                                      | `docker build --build-arg FERRO_VERSION="$VERSION" ferro-cli/tests/fixtures/benchmark/` | ✓ WIRED    | Lines 365-368 of publish.yml                                         |

### Behavioral Spot-Checks

Step 7b not run as a live test per instructions (disk at 97% capacity; test pass already recorded in 214-02-SUMMARY). Evidence from SUMMARY and git log substitutes.

| Behavior                                                            | Evidence                                                     | Status  |
|---------------------------------------------------------------------|--------------------------------------------------------------|---------|
| `scaffold_builds_against_workspace_ferro` test passes               | 214-02-SUMMARY: "Test result: scaffold_builds_against_workspace_ferro passed (1 passed, 0 failed)" | ✓ PASS  |
| All six commits from summaries exist in git log                     | Confirmed: 11acc90d, e4c067c7, f7f5c58f, daf72308, 1371bbb7, ea52eb2f | ✓ PASS  |
| No `HttpResponse::(internal_server_error\|not_found\|bad_request\|redirect)` in scaffold.rs | grep returned no output | ✓ PASS  |

### Requirements Coverage

Requirements SCAF-01 through SCAF-05 are defined in 214-CONTEXT.md (CONTEXT D-10) and are not yet rows in REQUIREMENTS.md. This is noted as a minor traceability follow-up by the phase instructions — not a goal failure.

| Requirement | Source Plan | Description                                                                 | Status      | Evidence                                                             |
|-------------|-------------|-----------------------------------------------------------------------------|-------------|----------------------------------------------------------------------|
| SCAF-01     | 214-01-PLAN | Templates reference only published ferro surface                            | ✓ SATISFIED | `error_response!` + `ActiveValue` exported; all templates use ferro facade |
| SCAF-02     | 214-01-PLAN | `make:job` routes through `ferro::queue::*`, no missing dep                 | ✓ SATISFIED | `make.rs:339`: `use ferro::{{async_trait, queue::{{Error, Job, Queueable}}}}` |
| SCAF-03     | 214-02-PLAN | Clean scaffold `cargo build`s exit 0 against published ferro-rs             | ✓ SATISFIED | `scaffold_builds_against_workspace_ferro` passed; Step 3d covers full-stack path |
| SCAF-04     | 214-02-PLAN | Release-time CI gate against published artifact                             | ✓ SATISFIED | `post-publish-scaffold-smoke` in publish.yml; Dockerfile with ARG FERRO_VERSION |
| SCAF-05     | 214-02-PLAN | Per-PR scaffold-build guard against workspace path dep                      | ✓ SATISFIED | `scaffold-smoke` job in ci.yml, `needs: check`                       |

**Traceability note:** SCAF-01–SCAF-05 are not yet entered as rows in `.planning/REQUIREMENTS.md`. The file currently covers v13.0 milestone requirements only. Adding SCAF-* rows is a minor paperwork follow-up that does not affect goal achievement.

### Anti-Patterns Found

| File                               | Pattern                                        | Severity | Impact                                                                                     |
|------------------------------------|------------------------------------------------|----------|--------------------------------------------------------------------------------------------|
| `ferro-cli/src/templates/scaffold.rs` lines 630, 805, 967, 1272 | `use sea_orm::Set;` in generated controller code | ℹ Info | Not drift — `sea-orm` is a declared direct dependency in the generated `Cargo.toml.tpl` (lines 15-16); `Set` is the active-value setter variant, distinct from `ActiveValue`. No compile error. |

No blockers. No warnings.

### Human Verification Required

None. All must-haves are verifiable programmatically or are backed by test-pass evidence in the phase summaries. The CI workflow changes require a manual `git push` (token lacks `workflow` scope, per project memory and documented in 214-02-SUMMARY), but this is an operational handoff — the YAML is committed and correct; pushing is mechanical.

### Operational Handoff Note (not a gap)

`.github/workflows/ci.yml` and `.github/workflows/publish.yml` changes are committed (commits 1371bbb7, ea52eb2f) but must be pushed manually because the CI token lacks `workflow` scope. This is a known constraint (documented in `project_ferro_ci_disk_and_push.md`). The CI artifacts are complete and correct; the developer pushes them with a standard `git push`.

---

## Gaps Summary

No gaps. All 10 observable truths are verified. All plan must-haves are satisfied. The CR-01 critical issue (full-stack templates emitting non-existent `HttpResponse` methods) and the WR-01 warning (test missing non-`--api` scaffold step) were both addressed before this verification: the full-stack templates now use `ferro::error_response!` throughout, and Step 3d in `scaffold_builds_against_workspace_ferro` exercises the full-stack (non-`--api`) template path.

The one deviation from plan spec (D-06: `ferro::DB::connection()?.inner()` replaced by `Entity::insert_one()` via `ModelMut`) is an auto-fixed improvement that makes the generated auth controller compile against the actual published surface. The test passing is the authoritative proof.

---

_Verified: 2026-06-13T00:00:00Z_
_Verifier: Claude (gsd-verifier)_

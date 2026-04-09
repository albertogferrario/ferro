---
phase: 128-deploy-preflight
verified: 2026-04-09T06:30:00Z
status: passed
score: 14/14 must-haves verified
re_verification: false
---

# Phase 128: Deploy Preflight Verification Report

**Phase Goal:** Extend `ferro doctor` with deploy-specific preflight checks (copy_dirs vs .dockerignore collision, ferro version skew, Cargo.docker.toml staleness). Ship interactive `ferro deploy:init` scaffolder for `[package.metadata.ferro.deploy]`. Expose same check registry via MCP `deploy_check`. Absorbs REPORT items 3, 4, 13, 15, 17.
**Verified:** 2026-04-09T06:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `CheckCategory` enum with `General`/`Deploy` variants exported from `doctor::check` | VERIFIED | `ferro-cli/src/doctor/check.rs` lines 10-15: `pub enum CheckCategory { General, Deploy }` with `Serialize` + `PartialEq` derives |
| 2 | `DoctorCheck::category()` defaults to `General` | VERIFIED | `check.rs` lines 76-78: default body `{ CheckCategory::General }` |
| 3 | `copy_dirs_dockerignore_collision` check errors when a `copy_dirs` entry is excluded by `.dockerignore` | VERIFIED | `ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs` — full `check_impl` + 4 passing unit tests |
| 4 | `copy_dirs_dockerignore_collision` skips when `.dockerignore` absent | VERIFIED | `check_impl` line 27-29: returns `Ok` with "skipped (.dockerignore absent)" |
| 5 | `ferro_version_skew` returns `Error` on major/minor drift, `Warn` on patch-only drift, `Ok` when aligned | VERIFIED | `ferro-cli/src/doctor/checks/ferro_version_skew.rs` — `classify()` + `DriftKind` enum + 4 passing unit tests |
| 6 | `ferro_version_skew` skips when `Cargo.docker.toml` absent | VERIFIED | `check_impl` line 57-59: returns `Ok` with "skipped (Cargo.docker.toml absent)" |
| 7 | `cargo_docker_toml_staleness` categorized as `CheckCategory::Deploy` | VERIFIED | `cargo_docker_toml_staleness.rs` has `fn category()` override returning `CheckCategory::Deploy` |
| 8 | `default_checks()` returns exactly 11 checks in canonical order | VERIFIED | `registry.rs` lines 37-57: test `default_checks_returns_eleven_in_declared_order` asserts length 11 and exact names |
| 9 | `ferro doctor --deploy` runs only the three `Deploy`-category checks | VERIFIED | `commands/doctor.rs` lines 23-30: `deploy_only` flag filters by `CheckCategory::Deploy`; registry test `deploy_category_filter_returns_three` confirms exactly 3 |
| 10 | `ferro deploy:init --yes` writes `[package.metadata.ferro.deploy]` into root `Cargo.toml` | VERIFIED | `commands/deploy_init.rs` — `execute()` + `persist_deploy_block()` with `toml_edit::DocumentMut`; test `persist_inserts_block_when_absent` passes |
| 11 | `ferro deploy:init --dry-run` writes zero files | VERIFIED | `commands/deploy_init.rs` — `execute()` early-returns after `print_dry_run()`; test `dry_run_writes_zero_files` asserts file unchanged |
| 12 | Existing-table collision aborts with non-zero exit when `--yes` passed without override | VERIFIED | `execute()` defaults `on_exists` to `OnExists::Abort` when `opts.yes` is true and table exists; `persist_deploy_block` returns `Err` on `Abort` policy; test `persist_aborts_when_table_exists_and_policy_abort` passes |
| 13 | MCP `deploy_check` tool registered on `FerroMcpService` and shells out to `ferro doctor --deploy --json` | VERIFIED | `ferro-mcp/src/service.rs` lines 375-392: `#[tool(name = "deploy_check", ...)]` on `FerroMcpService`; `ferro-mcp/src/tools/deploy_check.rs`: `DEPLOY_CHECK_ARGS = &["doctor", "--deploy", "--json"]` |
| 14 | Docs page updated with all Phase 128 surfaces | VERIFIED | `docs/src/cli/doctor.md` contains `ferro deploy:init`, `ferry doctor --deploy`, `copy_dirs_dockerignore_collision`, `ferro_version_skew`, `deploy_check` MCP tool section |

**Score:** 14/14 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-cli/src/doctor/check.rs` | `CheckCategory` enum + `category()` default | VERIFIED | Enum at lines 10-15; default method at lines 76-78; test `non_deploy_checks_return_general_category` at line 222 |
| `ferro-cli/src/deploy/mod.rs` | `pub(crate) fn read_path_dep_version` | VERIFIED | Lines 19-28; no duplicate — grep confirms single definition in crate |
| `ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs` | `CopyDirsDockerignoreCollisionCheck` with `DoctorCheck` impl | VERIFIED | Full implementation + 4 unit tests; `category()` returns `Deploy` |
| `ferro-cli/src/doctor/checks/ferro_version_skew.rs` | `FerroVersionSkewCheck` with `DoctorCheck` impl | VERIFIED | Full implementation + 4 unit tests; `classify()` covers None/Patch/MajorMinor |
| `ferro-cli/src/doctor/registry.rs` | 11-entry `default_checks()` + two registry tests | VERIFIED | Lines 16-30: 11 entries; tests at lines 37-76 |
| `ferro-cli/src/commands/doctor.rs` | `deploy_only` param + `CheckCategory::Deploy` filter | VERIFIED | Lines 10, 23-30: `deploy_only: bool` param + filter |
| `ferro-cli/src/commands/deploy_init.rs` | `execute()`, `compute_deploy_toml_block()`, `persist_deploy_block()`, `OnExists` enum | VERIFIED | 391 lines; all 7 unit tests pass including `dry_run_writes_zero_files` |
| `ferro-mcp/src/tools/deploy_check.rs` | `execute()` shelling out to `ferro doctor --deploy --json` | VERIFIED | Lines 20-53; `DEPLOY_CHECK_ARGS` const + 2 unit tests |
| `ferro-mcp/src/service.rs` | `deploy_check` tool method | VERIFIED | Lines 375-392 |
| `docs/src/cli/doctor.md` | All 4 Phase 128 surfaces documented | VERIFIED | 11-check table, `--deploy` section, preflight check descriptions, `deploy:init` section, `deploy_check` MCP note |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `cargo_docker_toml_staleness.rs` | `deploy/mod.rs::read_path_dep_version` | `use crate::deploy::read_path_dep_version` | VERIFIED | grep confirms `crate::deploy::read_path_dep_version` at callsite; private duplicate removed |
| `ferro_version_skew.rs` | `deploy/mod.rs::read_path_dep_version` | `use crate::deploy::read_path_dep_version` | VERIFIED | `ferro_version_skew.rs` line 5: `use crate::deploy::read_path_dep_version` |
| `commands/doctor.rs` | `CheckCategory::Deploy` filter | `deploy_only` flag on `default_checks()` | VERIFIED | Lines 23-30 |
| `registry.rs` | `CopyDirsDockerignoreCollisionCheck, FerroVersionSkewCheck` | `default_checks()` Vec | VERIFIED | Both in imports and vec at positions 7-8 |
| `deploy_check.rs` (ferro-mcp) | `ferro doctor --deploy --json` | `std::process::Command` | VERIFIED | Line 21-28: `Command::new("ferro").args(DEPLOY_CHECK_ARGS)` |
| `main.rs` | `commands::deploy_init::run` | `Commands::DeployInit` dispatch arm | VERIFIED | Line 657-659 |

---

### Data-Flow Trace (Level 4)

These artifacts perform file I/O against real filesystem paths (no static returns or hardcoded data):

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `copy_dirs_dockerignore_collision.rs` | `metadata.copy_dirs`, `ignore_lines` | `read_deploy_metadata(root)` + `fs::read_to_string(".dockerignore")` | Yes — real file reads | FLOWING |
| `ferro_version_skew.rs` | `local_version`, `docker_version` | `read_path_dep_version(root, rel_path)` + `Cargo.docker.toml` parse | Yes — real file reads | FLOWING |
| `deploy_init.rs` | `web_bin`, `copy_dirs` | `detect_web_bin(&root)` + filesystem dir existence check | Yes — real project introspection | FLOWING |
| `deploy_check.rs` (ferro-mcp) | `stdout` | `ferro doctor --deploy --json` subprocess output | Yes — shells out to real CLI | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `default_checks()` returns 11 entries | `cargo test -p ferro-cli registry` (runs `default_checks_returns_eleven_in_declared_order`) | All tests pass | PASS |
| Deploy filter returns 3 checks | `cargo test -p ferro-cli registry` (runs `deploy_category_filter_returns_three`) | All tests pass | PASS |
| `ferro_version_skew` errors on major/minor drift | `cargo test -p ferro-cli ferro_version_skew` | All tests pass | PASS |
| `deploy_init` dry-run writes zero files | `cargo test -p ferro-cli deploy_init` (runs `dry_run_writes_zero_files`) | Test passes | PASS |
| Full workspace build | `cargo clippy --all --all-targets -- -D warnings` | No warnings, no errors | PASS |
| Full test suite | `cargo test --all-features` | All test suites: `ok`. Zero failures across workspace | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| REPORT-03 | 128-02 | `copy_dirs` vs `.dockerignore` collision | SATISFIED | `copy_dirs_dockerignore_collision` check with Error on collision, skip when `.dockerignore` absent |
| REPORT-04 | 128-01, 128-02 | No version-skew detection between local ferro path deps and registry version | SATISFIED | `ferro_version_skew` check: Error on major/minor drift, Warn on patch drift. Note: REPORT-04 also mentions `cargo check --offline`; the plan deliberately chose version-string comparison via `read_path_dep_version` as a faster, dependency-free alternative. This is an accepted scope reduction. |
| REPORT-13 | 128-01, 128-02 | Better feedback when ferro is being pulled from crates.io vs path | SATISFIED | `ferro_version_skew` checks version alignment between local `ferro*` path deps and `Cargo.docker.toml` docker versions |
| REPORT-15 | 128-03 | `[package.metadata.ferro.deploy]` easy to typo, no interactive scaffolder | SATISFIED | `ferro deploy:init` with `--yes`/`--dry-run`, `OnExists` policies, `compute_deploy_toml_block`, `persist_deploy_block` |
| REPORT-17 | 128-01, 128-02 | `Cargo.docker.toml` can drift from `Cargo.toml` | SATISFIED | `cargo_docker_toml_staleness` check (pre-existing) now categorized as `Deploy`; `ferro_version_skew` provides additional staleness detection for version fields specifically |

---

### Anti-Patterns Found

No blockers or warnings found. Specific notes:

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-cli/src/deploy/mod.rs` line 5 | `#![allow(dead_code)]` at module level | Info | Pre-existing suppression from earlier phases; does not affect Phase 128 correctness |

---

### Human Verification Required

The following behaviors require manual testing (cannot be verified programmatically without a running project):

#### 1. Interactive `ferro deploy:init` prompt flow

**Test:** Run `ferro deploy:init` (without `--yes`) inside a ferro project with `migrations/` present and a `[[bin]]` entry in `Cargo.toml`.
**Expected:** Prompts for web binary name (pre-filled), copy_dirs (pre-filled with "migrations"), runtime_apt (blank). On confirm, writes `[package.metadata.ferro.deploy]` to `Cargo.toml` and prints "Next steps" footer mentioning `ferro docker:init` and `ferro doctor --deploy`.
**Why human:** Requires an interactive TTY and a real ferro project with a detected binary.

#### 2. MCP `deploy_check` tool end-to-end

**Test:** Launch `ferro mcp` in a ferro project, call `deploy_check` via an MCP client.
**Expected:** Returns a JSON `Report` with `summary` and `checks` keys; `checks` array contains exactly 3 entries named `cargo_docker_toml_staleness`, `copy_dirs_dockerignore_collision`, `ferro_version_skew`.
**Why human:** Requires `ferro` binary on PATH + running MCP session + real project root.

#### 3. `ferro doctor --deploy` output in a project with drift

**Test:** In a project where `Cargo.docker.toml` has `ferro = { version = "0.1.0" }` but local `framework/Cargo.toml` has `version = "0.2.0"`, run `ferro doctor --deploy`.
**Expected:** Output shows `ferro_version_skew` as error with details containing "major/minor drift"; exit code 1.
**Why human:** Requires a real ferro project filesystem state.

---

### Gaps Summary

None. All 14 must-haves verified, all REPORT items addressed, all CI checks pass.

One scope clarification: REPORT-04 requests `cargo check --offline` against the rewritten `Cargo.docker.toml`. The phase delivered version-string comparison via `ferro_version_skew` instead, which catches the described failure mode (version numbers diverging) without the cost of a full cargo resolve. This is a deliberate scope decision recorded in plan notes, not a gap.

---

_Verified: 2026-04-09T06:30:00Z_
_Verifier: Claude (gsd-verifier)_

---
phase: 128-deploy-preflight
plan: 04
subsystem: ferro-mcp
tags: [rust, ferro-mcp, deploy, mcp-tool, docs, doctor]

# Dependency graph
requires:
  - phase: 128-deploy-preflight
    plan: 02
    provides: ferro doctor --deploy --json CLI surface
  - phase: 128-deploy-preflight
    plan: 03
    provides: ferro deploy:init command for docs coverage
provides:
  - deploy_check MCP tool on FerroMcpService
  - docs/src/cli/doctor.md updated with all Phase 128 surfaces
affects: [ferro-mcp service, agent deploy workflow]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "shell-out pattern: ferro-mcp tools call CLI binaries to avoid dependency cycles"
    - "DEPLOY_CHECK_ARGS const: arg list extracted for unit-testable arg construction without shelling out"

key-files:
  created:
    - ferro-mcp/src/tools/deploy_check.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
    - docs/src/cli/doctor.md

key-decisions:
  - "Shell-out pattern avoids ferro-mcp -> ferro-cli cycle (ferro-cli already depends on ferro-mcp)"
  - "Non-zero exit from ferro doctor treated as valid JSON Report, not an error; only empty stdout is a hard error"
  - "DEPLOY_CHECK_ARGS const enables unit testing arg construction without spawning a process"

requirements-completed: [REPORT-03, REPORT-04, REPORT-13, REPORT-15, REPORT-17]

# Metrics
duration: ~4min
completed: 2026-04-09
---

# Phase 128 Plan 04: deploy_check MCP Tool + Deploy Docs Summary

**`deploy_check` MCP tool registered on FerroMcpService (shells out to `ferro doctor --deploy --json`); doctor.md updated with all Phase 128 surfaces: 11-check table, `--deploy` filter, preflight check descriptions, `ferro deploy:init` section.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-04-09T04:01:30Z
- **Completed:** 2026-04-09T04:05:51Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Created `ferro-mcp/src/tools/deploy_check.rs`:
  - `DEPLOY_CHECK_ARGS: &[&str]` const for testable arg construction
  - `execute(project_root: &Path) -> Result<String>` shells out to `ferro doctor --deploy --json`
  - Non-zero exit with valid JSON treated as success (doctor exits 1 on check errors)
  - Empty stdout or unparseable JSON returns `McpError::ExecutionError`
  - 2 unit tests asserting arg equality and length
- Re-exported via `tools/mod.rs` (alphabetical: after `dependency_graph`)
- Registered `#[tool(name = "deploy_check", ...)]` on `FerroMcpService` in `service.rs`
- No new entry in `ferro-mcp/Cargo.toml` (no ferro-cli dep introduced)
- Updated `docs/src/cli/doctor.md`:
  - Check table updated from 9 to 11 entries with Category column
  - New `## Deploy filter (--deploy)` section
  - New `## Preflight checks` section with descriptions for all 3 Deploy checks
  - New `## ferro deploy:init` section with synopsis, example TOML block, collision policy
  - `## deploy_check MCP tool` one-sentence reference

## Task Commits

1. **Task 1: deploy_check MCP tool** — `774c5fa7` (feat)
2. **Task 2: update deploy docs** — `e3daf183` (docs)

## Files Created/Modified

- `ferro-mcp/src/tools/deploy_check.rs` — NEW: shell-out tool, DEPLOY_CHECK_ARGS const, execute(), 2 unit tests
- `ferro-mcp/src/tools/mod.rs` — added `pub mod deploy_check;` after dependency_graph
- `ferro-mcp/src/service.rs` — added deploy_check tool method with full description
- `docs/src/cli/doctor.md` — 11-check table, Deploy filter, Preflight checks, deploy:init, MCP note

## Decisions Made

- Shell-out pattern (same as `create_project.rs`) avoids the ferro-mcp → ferro-cli dependency cycle.
- Non-zero exit from `ferro doctor` treated as informational (the JSON Report is still valid); only empty stdout is a hard error, consistent with the exit-code contract documented in doctor.md.
- Docs update placed entirely in `doctor.md` — the single page already covering `ferro doctor`; no new page needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Module ordering in tools/mod.rs**
- **Found during:** Task 1 overall verification (`cargo fmt --all -- --check`)
- **Issue:** `deploy_check` was placed before `dependency_graph` in tools/mod.rs; fmt wanted `dependency_graph` first (`depen` < `deplo`).
- **Fix:** Moved `pub mod deploy_check;` to after `dependency_graph`.
- **Files modified:** `ferro-mcp/src/tools/mod.rs`
- **Commit:** `774c5fa7` (corrected before commit)

## Known Stubs

None. The MCP tool shells out to the real CLI binary; no mock data or hardcoded responses.

---

## Self-Check: PASSED

- `ferro-mcp/src/tools/deploy_check.rs`: FOUND
- `ferro-mcp/src/tools/mod.rs` contains `pub mod deploy_check`: FOUND
- `ferro-mcp/src/service.rs` contains `deploy_check`: FOUND
- `docs/src/cli/doctor.md` contains `ferro deploy:init`, `ferro doctor --deploy`, `copy_dirs_dockerignore_collision`, `ferro_version_skew`: FOUND (12 matches)
- Commit `774c5fa7`: FOUND
- Commit `e3daf183`: FOUND
- `cargo fmt --all -- --check`: PASSED
- `cargo clippy --all --all-targets -- -D warnings`: PASSED
- `cargo test --all-features`: PASSED

---

*Phase: 128-deploy-preflight*
*Completed: 2026-04-09*

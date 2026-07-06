---
phase: 123-deploy-mcp-tools
plan: 03
subsystem: ferro-mcp
tags: [deploy, mcp, pre-flight, severity-report]
requires:
  - ferro_mcp::tools::deploy_common (from 123-02)
  - ferro_cli::deploy::find_ferro_path_deps
  - ferro_cli::deploy::parse_env_example
provides:
  - ferro_mcp::tools::deploy_check::execute
  - ferro_mcp::tools::deploy_check::{DeployCheckReport, Finding, Severity, CheckedFiles}
  - MCP tool `deploy_check`
affects:
  - ferro-mcp tool_router surface (+1 tool)
tech_added: []
patterns: [structured severity report, best-effort git introspection, read-only checks]
files_created:
  - ferro-mcp/src/tools/deploy_check.rs
files_modified:
  - ferro-mcp/src/tools/mod.rs
  - ferro-mcp/src/service.rs
decisions:
  - Handler returns `String` (pretty JSON) mirroring `application_info`, not `CallToolResult` — consistent with existing ferro-mcp tool idiom
  - Used a minimal manual `- key:` line scanner for .do/app.yaml env parsing instead of pulling in serde_yaml (avoid new dep)
  - Git checks are best-effort: non-git trees and missing upstream downgrade to info, never Err
  - `no_upstream` is surfaced as Info (not Warning) so clean local-only branches don't trigger "warnings" status
metrics:
  duration: ~10min
  tasks: 2
  tests_added: 5
  completed: 2026-04-07
requirements: [D-01, D-02, D-03, D-11]
---

# Phase 123 Plan 03: deploy_check MCP Tool Summary

Added the `deploy_check` read-only MCP tool — pre-flight deploy validation returning a structured, severity-tagged report covering the six detection categories from SCOPE D-02 plus git-state hygiene.

## What Was Built

**New tool module `ferro-mcp/src/tools/deploy_check.rs`:**
- `Severity` enum (Blocker | Warning | Info), `Finding`, `CheckedFiles`, `DeployCheckReport` Serialize structs
- `pub fn execute(project_root: &Path) -> Result<DeployCheckReport>` — orchestrates six check stages
- Checks:
  1. `missing_dockerfile` (Blocker) — Dockerfile missing at root
  2. `missing_app_yaml` (Blocker) — .do/app.yaml missing
  3. `ferro_path_deps` (Blocker) — delegates to `deploy_common::find_ferro_path_deps`, detail = `{ crates: [...] }`
  4. `sqlite_database_url` (Blocker) — parses .env.example via `deploy_common::parse_env_example`, flags `sqlite:` scheme
  5. `missing_env_var` (Warning) / `extra_env_var` (Info) — set-diff between .env.example keys and `.do/app.yaml` envs block (minimal `- key:` line scanner, no serde_yaml dep)
  6. `dirty_git_tree` (Warning), `unpushed_commits` (Warning), `no_upstream` (Info) — best-effort `git status --porcelain` + `git rev-list --count @{u}..HEAD`
- Derived `status` field: `blocked` | `warnings` | `ok`
- 5 unit tests against `TempDir` fixtures covering each category plus a clean-project happy path

**Registration `ferro-mcp/src/service.rs`:**
- New `#[tool(name = "deploy_check", description = ...)]` handler inserted right after `application_info`
- Handler body follows the `application_info` idiom: pretty-JSON on Ok, `{"error": ...}` on Err
- No params struct needed — reads `self.project_root`

**Module registry `ferro-mcp/src/tools/mod.rs`:**
- Added `pub mod deploy_check;` in alphabetical position before `deploy_common`

## Deviations from Plan

### Minor — Handler return type

The plan sketched `async fn deploy_check(&self) -> Result<CallToolResult, rmcp::Error>`, but the existing `application_info` handler (which the plan instructed to mirror) returns `String`. Followed the existing idiom: `String` pretty-JSON. This keeps ferro-mcp tool shapes uniform and avoids inventing a new result wrapper. Tracked as Rule 1 (matching established pattern).

### Minor — test_env_drift_warning fixture adjustment

Initial fixture included `DATABASE_URL=postgres://x` in .env.example, which then surfaced as an additional `missing_env_var` and broke the test's positional `find(code == "missing_env_var")` assertion. Dropped DATABASE_URL from the drift-test fixture so the test asserts exclusively on the FOO/BAR drift pair. No behavior change in the tool itself.

## Read-Only Enforcement (D-11)

```
$ grep -n "fs::write\|OpenOptions\|File::create\|create(" ferro-mcp/src/tools/deploy_check.rs
# (no matches)
```

The tool performs only `fs::read_to_string`, `path.exists()`, and `git` read commands (`status --porcelain`, `rev-list --count`). No mutating syscalls.

## Verification

```
cargo test -p ferro-mcp --lib tools::deploy_check  # 5/5 passed
cargo clippy -p ferro-mcp --all-targets --no-deps -- -D warnings  # clean
cargo fmt -p ferro-mcp -- --check  # clean
```

Acceptance greps:
- `pub fn execute` in ferro-mcp/src/tools/deploy_check.rs — yes
- `pub enum Severity` — yes
- `Blocker` — yes
- All 5 finding codes (`missing_dockerfile`, `missing_app_yaml`, `ferro_path_deps`, `sqlite_database_url`, `missing_env_var`) — yes
- `pub mod deploy_check` in ferro-mcp/src/tools/mod.rs — yes
- `name = "deploy_check"` in ferro-mcp/src/service.rs — yes
- `tools::deploy_check::execute` in ferro-mcp/src/service.rs — yes
- No write-side syscalls in tool module — confirmed

## Deferred Issues (Out of Scope)

- `ferro-json-ui/src/render.rs:391` pre-existing `clippy::uninlined_format_args` warning — same pre-existing warning flagged in 123-01 and 123-02. Not caused by this plan. Scoped clippy to `-p ferro-mcp --no-deps` for enforcement.
- Integration test exercising the tool via the rmcp service transport — current tool coverage is unit-level against the `execute()` function, which is consistent with the rest of ferro-mcp's tool tests. Service-level integration covered by the broader ferro-mcp test suite (201 pre-existing tests still green).

## Commits

- `3fbf17d2` feat(123-03): add deploy_check MCP tool with severity report
- `7bbfad30` feat(123-03): register deploy_check in ferro-mcp tool_router

## Self-Check: PASSED

- ferro-mcp/src/tools/deploy_check.rs: FOUND (new, 364 lines)
- ferro-mcp/src/tools/mod.rs: FOUND (modified — deploy_check added)
- ferro-mcp/src/service.rs: FOUND (modified — handler added)
- Commit 3fbf17d2: FOUND
- Commit 7bbfad30: FOUND
- 5 new deploy_check tests: all passing
- clippy -p ferro-mcp --all-targets --no-deps -- -D warnings: clean
- No stubs; no placeholder data paths

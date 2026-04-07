---
phase: 123-deploy-mcp-tools
plan: 04
subsystem: ferro-mcp
tags: [deploy, mcp, env-drift, classification]
requires:
  - ferro_mcp::tools::deploy_common (from 123-02)
  - ferro_cli::deploy::parse_env_example
  - ferro_cli::deploy::is_secret
provides:
  - ferro_mcp::tools::deploy_diff_env::execute
  - ferro_mcp::tools::deploy_diff_env::{DiffRow, DiffEnvReport, Classification}
  - MCP tool `deploy_diff_env`
affects:
  - ferro-mcp tool_router surface (+1 tool)
tech_added: []
patterns: [block-based yaml line scanner, union-map diff, classification enum]
files_created:
  - ferro-mcp/src/tools/deploy_diff_env.rs
files_modified:
  - ferro-mcp/src/tools/mod.rs
  - ferro-mcp/src/service.rs
decisions:
  - Used a stateful `- key:` block scanner + per-line regex rather than serde_yaml (keeps ferro-mcp dep surface flat; regex is already a dep)
  - Handler returns pretty-JSON String mirroring deploy_check/application_info idiom, not CallToolResult
  - Errors surface as McpError::ToolError (no dedicated constructor helper exists; matches existing crate error conventions)
  - `.env.example` fallback is reported via `source: "env_example"` field on DiffEnvReport rather than as a separate finding
  - Scope mismatch is both a classification (`ScopeMismatch`) on the row AND a flat list in `secrets_marked_plain` so agents can act without scanning rows
metrics:
  duration: ~12min
  tasks: 2
  tests_added: 7
  completed: 2026-04-07
requirements: [D-04, D-05, D-06, D-11]
---

# Phase 123 Plan 04: deploy_diff_env MCP Tool Summary

Added the `deploy_diff_env` read-only MCP tool — compares the project's local env source (`.env`, or `.env.example` fallback) against the `envs` block of `.do/app.yaml` and surfaces drift classified per row, with a dedicated flat list of secret/scope mismatches.

## What Was Built

**New tool module `ferro-mcp/src/tools/deploy_diff_env.rs`:**
- `Classification` enum (`Aligned` | `MissingLocal` | `MissingRemote` | `ScopeMismatch`) — serde `snake_case`
- `DiffRow { key, local: Option<String>, remote: Option<String>, classification }`
- `DiffEnvReport { source, rows, drift_count, secrets_marked_plain }` where `source` is `"env"` or `"env_example"`
- `pub fn execute(project_root: &Path) -> Result<DiffEnvReport>`
- `load_local_env` — tries `.env` first, falls back to `.env.example`; returns None if neither exists
- `parse_app_yaml_envs` — stateful `- key:` block scanner using a single-line regex `^\s*(key|value|type):\s*(.+)$`, tolerates arbitrary indentation and optional `type: SECRET`
- Union-map diff: seed with local entries as `MissingRemote`, walk remote entries to fill `remote`, reclassify (`Aligned` | `ScopeMismatch` | `MissingLocal`)
- `secrets_marked_plain` accumulated from keys where `is_secret(key)` but remote `type` is not `SECRET`; sorted + deduped
- Rows emitted sorted alphabetically via `BTreeMap`
- 7 unit tests: drift both sides, secret marked plain, aligned secret with `type: SECRET`, `.env.example` fallback, no-local error, missing app.yaml error, row sort order

**Registration `ferro-mcp/src/service.rs`:**
- New `#[tool(name = "deploy_diff_env", description = ...)]` handler inserted right after `deploy_check`
- Handler body mirrors `deploy_check`: pretty-JSON on Ok, `{"error": "..."}` on Err
- No params struct; reads `self.project_root`

**Module registry `ferro-mcp/src/tools/mod.rs`:**
- Added `pub mod deploy_diff_env;` right after `deploy_common`

## Deviations from Plan

### Minor — McpError::tool helper does not exist

The plan sketched `McpError::tool("deploy_diff_env", "...")`. The crate's `error.rs` only exposes the `ToolError(String)` variant with no convenience constructor. Used `McpError::ToolError("deploy_diff_env: …".to_string())` to stay aligned with the existing error surface. Tracked as Rule 1 (matching established pattern).

### Minor — Block scanner instead of split-then-line-regex

The plan suggested splitting on `- key:` then running line-regex per segment. Implemented as a single-pass stateful walk: when a line starts with `- key:`, flush the previous block and start a new one; subsequent indented `key`/`value`/`type` lines populate the current block. Equivalent output, simpler allocation pattern. No behavior change from the plan's intent.

## Read-Only Enforcement (D-11)

```
$ grep -n "fs::write\|OpenOptions\|File::create\|create(" ferro-mcp/src/tools/deploy_diff_env.rs
# (no matches)
```

The tool performs only `fs::read_to_string` and `Path::exists`. No mutating syscalls, no process spawning.

## Verification

```
cargo fmt -p ferro-mcp -- --check                                # clean
cargo clippy -p ferro-mcp --all-targets --no-deps -- -D warnings # clean
cargo test -p ferro-mcp                                          # 213/213 passed
cargo test -p ferro-mcp --lib tools::deploy_diff_env             # 7/7 passed
```

Acceptance greps:
- `pub fn execute` in `ferro-mcp/src/tools/deploy_diff_env.rs` — yes
- `DiffRow`, `DiffEnvReport`, `Classification` — all present
- `is_secret` reference — yes (imported from `deploy_common`)
- `secrets_marked_plain` — yes
- `pub mod deploy_diff_env` in `ferro-mcp/src/tools/mod.rs` — yes
- `name = "deploy_diff_env"` in `ferro-mcp/src/service.rs` — yes
- `tools::deploy_diff_env::execute` in `ferro-mcp/src/service.rs` — yes
- No write-side syscalls in tool module — confirmed

## Deferred Issues (Out of Scope)

- `ferro-json-ui/src/render.rs:391` pre-existing `clippy::uninlined_format_args` warning — same pre-existing warning flagged in 123-01/02/03. Not caused by this plan. Scoped clippy to `-p ferro-mcp --no-deps` for enforcement.
- Service-level integration test exercising the tool via the rmcp transport — unit-level coverage on `execute()` is consistent with the rest of ferro-mcp's tool tests.

## Commits

- `b65069d4` feat(123-04): add deploy_diff_env MCP tool
- `66020d4c` feat(123-04): register deploy_diff_env in ferro-mcp tool_router

## Self-Check: PASSED

- ferro-mcp/src/tools/deploy_diff_env.rs: FOUND (new)
- ferro-mcp/src/tools/mod.rs: FOUND (modified)
- ferro-mcp/src/service.rs: FOUND (modified)
- Commit b65069d4: FOUND
- Commit 66020d4c: FOUND
- 7 new deploy_diff_env tests: all passing
- full ferro-mcp suite: 213/213 passing
- clippy -p ferro-mcp --all-targets --no-deps -- -D warnings: clean
- fmt -p ferro-mcp -- --check: clean
- No stubs; no placeholder data paths

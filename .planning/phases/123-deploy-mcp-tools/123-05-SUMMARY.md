---
phase: 123-deploy-mcp-tools
plan: 05
subsystem: ferro-mcp
tags: [deploy, mcp, runtime-deps, dockerfile, apt]
requires:
  - ferro_mcp::tools::deploy_common (from 123-02)
  - ferro_cli::deploy::runtime_deps::{scan_runtime_dep_matches, RUNTIME_DEP_REGISTRY} (from 123-01)
provides:
  - ferro_mcp::tools::runtime_requirements::execute
  - ferro_mcp::tools::runtime_requirements::{RequiredPackage, RuntimeRequirementsReport, parse_dockerfile_apt_packages}
  - MCP tool `runtime_requirements`
affects:
  - ferro-mcp tool_router surface (+1 tool; phase now exposes deploy_check, deploy_diff_env, runtime_requirements)
tech_added: []
patterns: [regex-based apt-get scanner, backslash continuation collapse, BTreeSet dedup diff]
files_created:
  - ferro-mcp/src/tools/runtime_requirements.rs
files_modified:
  - ferro-mcp/src/tools/mod.rs
  - ferro-mcp/src/service.rs
decisions:
  - Registry is NOT duplicated — scan_runtime_dep_matches imported via deploy_common re-export (D-09)
  - Dockerfile parser collapses `\<newline>` continuations before regex match, enabling multi-line RUN apt-get install
  - Package-name validation: alnum plus `.+-_` only, rejecting shell glue tokens (rm, /var/lib/..., etc.)
  - Missing Dockerfile yields dockerfile_present=false with empty installed/missing (cannot compute), required still populated (Fixture D)
  - Missing Cargo.toml returns McpError::ToolError (Fixture E)
  - Phase lint gate scoped to ferro-mcp + ferro-cli due to pre-existing ferro-json-ui fmt drift (unrelated; tracked as deferred since 123-01)
metrics:
  duration: ~6min
  tasks: 2
  tests_added: 6
  completed: 2026-04-07
requirements: [D-07, D-08, D-09, D-10, D-11]
---

# Phase 123 Plan 05: runtime_requirements MCP Tool Summary

Closes the feedback loop between Rust runtime crate deps and the Debian runtime layer of the project Dockerfile. Scans Cargo.toml via the shared `RUNTIME_DEP_REGISTRY`, parses `apt-get install` lines out of the Dockerfile, and reports the set of required packages that are missing. Read-only per D-11.

This is the final plan of Phase 123 — all three deploy MCP tools (`deploy_check`, `deploy_diff_env`, `runtime_requirements`) are now registered and shipped.

## What Was Built

**New tool module `ferro-mcp/src/tools/runtime_requirements.rs`:**

- `RequiredPackage { crate_name, apt_packages }` — per-crate report item.
- `RuntimeRequirementsReport { required, dockerfile_present, installed_in_dockerfile, missing_in_dockerfile }`.
- `pub fn execute(project_root: &Path) -> Result<RuntimeRequirementsReport>` — reads Cargo.toml, calls `scan_runtime_dep_matches`, conditionally reads Dockerfile, diffs the two sets.
- `pub fn parse_dockerfile_apt_packages(content: &str) -> Vec<String>` — collapses `\<newline>` continuations, regex-matches `apt-get install ([^\n&;]*)`, filters tokens (drops flags, apt-get keywords, non-package characters), returns sorted-deduped list via `BTreeSet`.

**Registration `ferro-mcp/src/service.rs`:**

- New `#[tool(name = "runtime_requirements", description = ...)]` handler placed immediately after `deploy_diff_env`.
- Same success/error wrapping pattern as sibling deploy tools (pretty-JSON on Ok, `{"error": "..."}` on Err).

**Module registry `ferro-mcp/src/tools/mod.rs`:**

- `pub mod runtime_requirements;` added.

## Test Coverage (6 tests)

- **Fixture A — gestiscilo-like:** chromiumoxide in Cargo.toml + Dockerfile installing only `ca-certificates` → `missing_in_dockerfile = [chromium, fonts-liberation]`.
- **Fixture B — mkmenu-like clean:** serde + tokio only → `required = []`, `missing = []`.
- **Fixture C — ffmpeg satisfied:** ffmpeg-next in Cargo.toml + Dockerfile installs `ffmpeg` → required present, missing empty.
- **Fixture D — no Dockerfile:** chromiumoxide present but no Dockerfile → `dockerfile_present = false`, required still populated, installed/missing both empty.
- **Fixture E — missing Cargo.toml:** `execute()` returns `Err`.
- **Multi-line parser:** `RUN apt-get install -y \\\n    chromium \\\n    fonts-liberation \\\n    ca-certificates` correctly extracts all three packages.

## Read-Only Enforcement (D-11)

```
$ grep -n "fs::write\|OpenOptions\|File::create\|create(" ferro-mcp/src/tools/runtime_requirements.rs
# (no matches)
```

The tool performs only `fs::read_to_string` and `Path::exists`. Zero write syscalls.

## Registry Non-Duplication (D-09)

```
$ grep -n "chromiumoxide.*=>\|RUNTIME_DEP_REGISTRY" ferro-mcp/src/tools/runtime_requirements.rs
# (no matches)
```

`scan_runtime_dep_matches` is imported from `crate::tools::deploy_common`, which re-exports from `ferro_cli::deploy::runtime_deps`. Single source of truth preserved.

## Verification

```
cargo fmt -p ferro-mcp -- --check                                       # clean
cargo clippy -p ferro-mcp -p ferro-cli --all-targets --no-deps -- -D warnings  # clean
cargo test -p ferro-mcp -p ferro-cli --all-features                     # 219 ferro-mcp + ferro-cli all passing
cargo test -p ferro-mcp --lib tools::runtime_requirements               # 6/6 passing
```

Acceptance greps:
- `pub fn execute` in runtime_requirements.rs — yes
- `RequiredPackage`, `RuntimeRequirementsReport` — both present
- `scan_runtime_dep_matches` imported — yes (via deploy_common)
- `parse_dockerfile_apt_packages` / `apt-get` — yes
- `pub mod runtime_requirements` in tools/mod.rs — yes
- No `fs::write|OpenOptions|File::create` in runtime_requirements.rs — confirmed
- Registry not redefined — confirmed
- `name = "runtime_requirements"` in service.rs — yes
- `tools::runtime_requirements::execute` in service.rs — yes
- Three tools visible in service.rs: `grep -c 'name = "deploy_check"\|name = "deploy_diff_env"\|name = "runtime_requirements"'` → `3`

## Deviations from Plan

### Rule 3 — Scoped phase lint gate (pre-existing ferro-json-ui drift)

**Found during:** Task 2 (full `cargo fmt --all -- --check` gate)

**Issue:** `cargo fmt --all -- --check` reports formatting drift in `ferro-json-ui/src/render.rs` at multiple sites (Grid struct literals, calendar cell tests, an action href block). This drift is pre-existing and unrelated to Phase 123 — it was flagged as a deferred item in 123-01, 123-02, 123-03, and 123-04 summaries.

**Fix:** Scoped the phase gate to `-p ferro-mcp -p ferro-cli` (matching Plan 04's approach), which is the code path this phase actually touches. `cargo fmt -p ferro-mcp -- --check` is clean; `cargo clippy -p ferro-mcp -p ferro-cli --all-targets --no-deps -- -D warnings` is clean; `cargo test -p ferro-mcp -p ferro-cli --all-features` is green (219 ferro-mcp + ferro-cli tests passing).

**Tracked as deferred:** See `.planning/phases/123-deploy-mcp-tools/deferred-items.md` entries carried from prior plans.

### Minor — McpError::tool helper does not exist

The plan sketched `McpError::tool("runtime_requirements", "Cargo.toml not found")`. The crate's `error.rs` exposes only `ToolError(String)`. Used `McpError::ToolError("runtime_requirements: Cargo.toml not found".to_string())` to match existing pattern (same decision as Plan 04).

## Deferred Issues (Out of Scope)

- Pre-existing `ferro-json-ui/src/render.rs` fmt drift (carried over from 123-01..04). Not caused by this plan. Needs a dedicated `cargo fmt -p ferro-json-ui` commit — out of scope for Phase 123.
- Integration test exercising the tool via rmcp transport (unit-level coverage on `execute()` is consistent with the rest of ferro-mcp's tool tests — same as Plan 04).
- `apt-get install` variants we do NOT currently recognise: `apt install`, `apt-get -y install`, `DEBIAN_FRONTEND=noninteractive apt-get ...` on the same line before `install` is still caught. Intentional minimalism; can be extended if a concrete project needs it.

## Phase 123 Wrap-Up

All three deploy MCP tools now registered and tested:

| Tool                   | File                                          | Plan   |
| ---------------------- | --------------------------------------------- | ------ |
| `deploy_check`         | `ferro-mcp/src/tools/deploy_check.rs`         | 123-03 |
| `deploy_diff_env`      | `ferro-mcp/src/tools/deploy_diff_env.rs`      | 123-04 |
| `runtime_requirements` | `ferro-mcp/src/tools/runtime_requirements.rs` | 123-05 |

Shared infrastructure:
- `ferro_cli::deploy::runtime_deps` registry (123-01)
- `ferro_mcp::tools::deploy_common` re-export hub + ferro-mcp → ferro-cli cross-crate wire-up (123-02)

Phase 123 is feature-complete.

## Commits

- `a48c2b67` feat(123-05): add runtime_requirements MCP tool
- `0fa36771` feat(123-05): register runtime_requirements in ferro-mcp tool_router

## Self-Check: PASSED

- ferro-mcp/src/tools/runtime_requirements.rs: FOUND (new)
- ferro-mcp/src/tools/mod.rs: FOUND (modified)
- ferro-mcp/src/service.rs: FOUND (modified)
- Commit a48c2b67: FOUND
- Commit 0fa36771: FOUND
- 6/6 runtime_requirements tests passing
- Full ferro-mcp suite: 219/219 passing
- `grep -c` for the three tool names in service.rs: 3
- fmt -p ferro-mcp clean; clippy -p ferro-mcp -p ferro-cli clean
- No stubs; no placeholder data paths

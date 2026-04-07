---
phase: 123-deploy-mcp-tools
plan: 02
subsystem: ferro-mcp/ferro-cli
tags: [deploy, cross-crate, mcp, dependency-graph]
requires:
  - ferro_cli::deploy::runtime_deps (from 123-01)
provides:
  - ferro_cli::deploy::find_ferro_path_deps
  - ferro_mcp::tools::deploy_common
  - ferro-mcp binary entry point
affects:
  - ferro-cli `mcp` subcommand now spawns ferro-mcp as subprocess
tech_added: []
patterns: [subprocess spawn to break dep cycle, re-export hub module]
files_created:
  - ferro-mcp/src/tools/deploy_common.rs
  - ferro-mcp/src/bin/ferro-mcp.rs
files_modified:
  - ferro-cli/src/deploy/ferro_deps.rs
  - ferro-cli/src/deploy/mod.rs
  - ferro-cli/src/commands/mcp.rs
  - ferro-cli/Cargo.toml
  - ferro-mcp/Cargo.toml
  - ferro-mcp/src/tools/mod.rs
decisions:
  - Broke ferro-cli <-> ferro-mcp cyclic dep by removing ferro-mcp from ferro-cli's Cargo.toml and spawning ferro-mcp as a standalone binary from the `ferro mcp` subcommand
  - Added `deploy_common` as the single re-export hub for all deploy_* MCP tools (Plans 03/04/05 will import from `crate::tools::deploy_common::*`)
  - Renamed private `discover_ferro_path_deps` to public `find_ferro_path_deps` rather than adding a wrapper (cleaner surface)
metrics:
  duration: ~8min
  tasks: 2
  tests_added: 1
  completed: 2026-04-07
---

# Phase 123 Plan 02: ferro-mcp -> ferro-cli Cross-Crate Wire-up Summary

Established the cross-crate dependency path from ferro-mcp to ferro-cli so the upcoming deploy_* MCP tools (Plans 03/04/05) can call Phase 122 deploy primitives directly, per D-12 (single source of truth — reuse, don't duplicate).

## What Was Built

**ferro-cli side (public surface promotion):**
- `find_ferro_path_deps(content: &str) -> Vec<String>` promoted from private to `pub` in `ferro-cli/src/deploy/ferro_deps.rs`
- Re-exported from `ferro-cli/src/deploy/mod.rs` alongside the existing `render_rewrite_script`
- Public wrapper test added (`find_ferro_path_deps_public_wrapper`)

**ferro-mcp side (dependency wire-up):**
- `ferro-cli = { path = "../ferro-cli", version = "0.1" }` added to `ferro-mcp/Cargo.toml`
- New `ferro-mcp/src/tools/deploy_common.rs` module re-exports every deploy helper the three upcoming MCP tools need:
  - `check_ref`, `parse_env_example`, `is_secret`, `find_ferro_path_deps`, `scan_runtime_deps_str`, `scan_runtime_dep_matches`, `EnvEntry`, `RuntimeDep`, `RUNTIME_DEP_REGISTRY`, `find_project_root`
- Registered as `pub mod deploy_common;` in `ferro-mcp/src/tools/mod.rs`

**Cycle-breaking (deviation — see below):**
- New `ferro-mcp/src/bin/ferro-mcp.rs` binary crate
- `ferro-cli/src/commands/mcp.rs` now spawns `ferro-mcp` as a subprocess instead of calling `ferro_mcp::McpServer` in-process
- `ferro-mcp` dependency removed from `ferro-cli/Cargo.toml`

## Deviations from Plan

### Rule 3 — Blocking Issue: Cyclic package dependency

**Found during:** Task 2 (`cargo build -p ferro-mcp`)

**Issue:** Adding `ferro-cli` as a dependency of `ferro-mcp` triggered a cargo cycle error — `ferro-cli` previously declared `ferro-mcp` as a library dependency (used only by `ferro-cli/src/commands/mcp.rs` to call `ferro_mcp::McpServer::with_project_root`). Cargo forbids cyclic path dependencies even behind features.

**Fix:** Broke the cycle structurally by inverting the direction — ferro-mcp no longer needs to be linked into ferro-cli's library:
1. Added a `ferro-mcp` binary at `ferro-mcp/src/bin/ferro-mcp.rs` that takes an optional project-root argument, constructs `McpServer::with_project_root`, and runs it inside a Tokio runtime.
2. Rewrote `ferro-cli/src/commands/mcp.rs` to spawn `ferro-mcp` via `std::process::Command`, forwarding the project root, exit code propagation included.
3. Removed `ferro-mcp = { path = "../ferro-mcp", version = "0.1" }` from `ferro-cli/Cargo.toml`.

This preserves the `ferro mcp` subcommand's user experience (a diagnostic message then server start) while honoring D-12: ferro-mcp pulls deploy helpers from ferro-cli, not the reverse.

**Files modified:** `ferro-mcp/src/bin/ferro-mcp.rs` (new), `ferro-cli/src/commands/mcp.rs`, `ferro-cli/Cargo.toml`

**Commit:** `0c8b9074`

## Verification

```
cargo build -p ferro-mcp                                  # ok
cargo build -p ferro-cli                                  # ok
cargo clippy -p ferro-mcp -p ferro-cli --all-targets --no-deps -- -D warnings  # clean
cargo test -p ferro-mcp                                   # 201 passed, 0 failed
cargo test -p ferro-cli --lib deploy::ferro_deps::tests::find_ferro_path_deps_public_wrapper  # 1 passed
```

Acceptance greps:
- `pub fn find_ferro_path_deps` in `ferro-cli/src/deploy/ferro_deps.rs` — yes
- `find_ferro_path_deps` re-export in `ferro-cli/src/deploy/mod.rs` — yes
- `parse_env_example` / `is_secret` re-exports in `ferro-cli/src/deploy/mod.rs` — already present (pre-existing)
- `ferro-cli` dep in `ferro-mcp/Cargo.toml` — yes
- `pub mod deploy_common` in `ferro-mcp/src/tools/mod.rs` — yes
- `pub use ferro_cli::deploy::` in `ferro-mcp/src/tools/deploy_common.rs` — yes
- `scan_runtime_deps_str` in `ferro-mcp/src/tools/deploy_common.rs` — yes

## Deferred Issues (Out of Scope)

- `ferro-json-ui/src/render.rs:391` pre-existing `clippy::uninlined_format_args` warning (flagged in 123-01 too). Unrelated to this plan.
- `.github/workflows/publish.yml`: ferro-mcp already published; no new crate added so wave ordering unaffected. ferro-cli no longer depends on ferro-mcp, which may simplify future wave resolution — not addressed here.

## Commits

- `c0b7c844` feat(123-02): expose find_ferro_path_deps as public cross-crate helper
- `0c8b9074` feat(123-02): wire ferro-mcp -> ferro-cli for shared deploy helpers

## Self-Check: PASSED

- ferro-cli/src/deploy/ferro_deps.rs: FOUND (modified)
- ferro-cli/src/deploy/mod.rs: FOUND (modified)
- ferro-cli/src/commands/mcp.rs: FOUND (modified)
- ferro-cli/Cargo.toml: FOUND (modified)
- ferro-mcp/Cargo.toml: FOUND (modified)
- ferro-mcp/src/tools/mod.rs: FOUND (modified)
- ferro-mcp/src/tools/deploy_common.rs: FOUND (new)
- ferro-mcp/src/bin/ferro-mcp.rs: FOUND (new)
- Commit c0b7c844: FOUND
- Commit 0c8b9074: FOUND
- ferro-mcp: 201/201 tests pass
- ferro-cli find_ferro_path_deps_public_wrapper test: passes
- clippy -D warnings clean on both crates

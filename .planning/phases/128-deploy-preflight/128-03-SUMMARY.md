---
phase: 128-deploy-preflight
plan: 03
subsystem: ferro-cli
tags: [rust, ferro-cli, deploy, scaffolder, interactive, toml_edit]

# Dependency graph
requires:
  - phase: 128-01
    provides: CheckCategory + read_path_dep_version foundation
  - phase: 127
    provides: --dry-run convention (docker:init / do:init pattern to mirror)
provides:
  - ferro deploy:init subcommand (interactive scaffolder for [package.metadata.ferro.deploy])
  - compute_deploy_toml_block: pure TOML fragment formatter
  - persist_deploy_block: toml_edit in-place Cargo.toml mutator with Abort/Overwrite/Merge
affects: [ferro-mcp deploy_check tool docs, user-facing deploy workflow]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Three-layer shape: run -> run_with -> execute (mirrors do_init.rs)"
    - "compute/persist split enables --dry-run without touching the filesystem"
    - "toml_edit DocumentMut for in-place Cargo.toml mutation preserving comments and key order"
    - "IsTerminal guard: non-TTY without --yes errors immediately"
    - "OnExists enum (Abort/Overwrite/Merge) policy applied at persist time"

key-files:
  created:
    - ferro-cli/src/commands/deploy_init.rs
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs

key-decisions:
  - "RenderedFile / print_dry_run reused from docker_init (pub(crate) — same crate, no visibility change needed)"
  - "OnExists::Abort is the default when --yes is passed and table exists — fail loudly, require explicit policy"
  - "copy_dirs defaults to existing-only subset of [migrations, static] — no phantom dirs in generated config"

requirements-completed: [REPORT-15]

# Metrics
duration: 2min
completed: 2026-04-09
---

# Phase 128 Plan 03: ferro deploy:init Scaffolder Summary

**Interactive `ferro deploy:init` command that writes `[package.metadata.ferro.deploy]` into root Cargo.toml via toml_edit, with --dry-run preview, --yes bypass, and Abort/Overwrite/Merge collision policy**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-04-09T05:35:10Z
- **Completed:** 2026-04-09T05:37:18Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created `ferro-cli/src/commands/deploy_init.rs` with full compute/persist split:
  - `compute_deploy_toml_block` — pure formatter, no side effects
  - `persist_deploy_block` — toml_edit DocumentMut in-place mutation, preserves comments and key order
  - `OnExists` enum with Abort, Overwrite, Merge policies
  - `execute` — orchestrates defaults detection, optional interactive prompts, dry-run or persist
  - Non-TTY + no `--yes` guard returns `Err` immediately
- Wired `pub mod deploy_init` into `commands/mod.rs` (alphabetical, before `do_init`)
- Added `DeployInit` variant with `--yes` / `--dry-run` to the clap `Commands` enum in `main.rs`
- Added dispatch arm adjacent to `DoInit` in the match block
- 7 unit tests pass, including `dry_run_writes_zero_files` verifying zero filesystem mutations

## Task Commits

1. **Task 1: implement deploy_init module** — `6bc977e0` (feat)
2. **Task 2: wire deploy:init subcommand** — `4668890e` (feat)

## Files Created/Modified

- `ferro-cli/src/commands/deploy_init.rs` — new module, 387 lines, full implementation + tests
- `ferro-cli/src/commands/mod.rs` — added `pub mod deploy_init;`
- `ferro-cli/src/main.rs` — added `DeployInit` variant and dispatch arm

## Decisions Made

- Reused `RenderedFile` / `print_dry_run` from `docker_init` — same crate, `pub(crate)` visibility is sufficient, no API change needed.
- `OnExists::Abort` as default when `--yes` is set and table already exists — fail loudly; users who want overwrite or merge must pass `on_exists_override` or run interactively.
- `copy_dirs` defaults to existing-only subset of `["migrations", "static"]` — avoids generating phantom paths that would silently do nothing in Docker builds.

## Deviations from Plan

None — plan executed exactly as written. The plan's full implementation sketch was used verbatim with only minor rustfmt reformatting.

## Known Stubs

None. The scaffolder writes all fields (`runtime_apt`, `copy_dirs`, `web_bin`) with real detected or prompted values. No placeholder text.

## Self-Check: PASSED

- `ferro-cli/src/commands/deploy_init.rs` — FOUND
- Commit `6bc977e0` — FOUND
- Commit `4668890e` — FOUND
- All 7 unit tests pass (including `dry_run_writes_zero_files`)
- `cargo build -p ferro-cli` — PASSED
- `ferro deploy:init --help` — prints synopsis

---
*Phase: 128-deploy-preflight*
*Completed: 2026-04-09*

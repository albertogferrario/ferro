---
phase: 252
plan: 05
subsystem: ferro-cli
tags: [design-lint, cli, walkdir, json-output, tdd]
requirements: [DS-06]

dependency_graph:
  requires:
    - ferro_json_ui::design::lint (252-01)
    - ferro_json_ui::design::Finding (252-01)
    - ferro_json_ui::design::Severity (252-01)
    - ferro_json_ui::spec::SCHEMA_VERSION (252-01)
  provides:
    - ferro design:lint [path] [--json] [--deny]
    - commands::design_lint::lint_content
    - commands::design_lint::has_warning
    - commands::design_lint::run
  affects:
    - ferro-cli/src/commands/design_lint.rs (new)
    - ferro-cli/src/main.rs (Commands::DesignLint variant + dispatch)
    - ferro-cli/src/commands/mod.rs (module registration)

tech_stack:
  added: []
  patterns:
    - walkdir recursive *.json discovery (follow_links=false — T-252-01)
    - FileFinding flat serde envelope (file field + #[serde(flatten)] finding)
    - console::style human output grouped by file
    - serde_json::to_string_pretty --json branch (stable gestiscilo Phase 232 contract)
    - std::process::exit(1) --deny gate (warning-level only; info never fails)

key_files:
  created:
    - ferro-cli/src/commands/design_lint.rs
  modified:
    - ferro-cli/src/main.rs
    - ferro-cli/src/commands/mod.rs

decisions:
  - "FileFinding uses #[serde(flatten)] on Finding — keeps the flat wire shape without duplicating fields"
  - "spec-parse warning for marker-bearing unparseable files — parse errors are caught, never panicked (T-252-02)"
  - "design_lint registered alphabetically after deploy_init (dep < des) — rustfmt enforced this"

metrics:
  duration: 288s (~5m)
  completed: 2026-07-03T18:29:00Z
  tasks: 2
  files: 3
---

# Phase 252 Plan 05: design:lint CLI Command Summary

`ferro design:lint [path] [--json] [--deny]` with recursive `*.json` discovery, per-file `ferro_json_ui::design::lint`, human output grouped by file, flat `--json` array (stable DS-06 contract for gestiscilo Phase 232 CI), and a `--deny` gate that exits non-zero only on warning-level findings.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | design_lint command module (lint_content, has_warning, run) + tests | 78146eb2 | design_lint.rs (new), commands/mod.rs |
| 2 | Register CLI command + smoke test | 1b1a864e | main.rs, commands/mod.rs |

## Verification

- `cargo test -p ferro-cli design_lint` — 7 tests pass (clean/warn/skip/bad-parse cases, has_warning true/false/skip)
- `cargo build -p ferro-cli` exits 0
- `cargo run -p ferro-cli -- design:lint --help` shows the command (contains "design")
- `cargo run -p ferro-cli -- design:lint app/src/views` exits 0; emits human-readable findings (1 warning, 3 info across 3 files — Plan 06 will make them lint-clean)
- `cargo run -p ferro-cli -- design:lint app/src/views --json` emits a valid flat JSON array with `file`, `rule`, `element_id`, `severity`, `message`, `suggestion` fields
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean
- `cargo doc -p ferro-cli --no-deps` clean (zero warnings)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Alphabetical position: design_lint after deploy_init**
- **Found during:** Post-Task 2 fmt gate
- **Issue:** PATTERNS.md said to place `design_lint` between `db_sync` and `deploy_init`, but alphabetically `dep` < `des` so `design_lint` must follow `deploy_init`. rustfmt enforced the correct order.
- **Fix:** Moved `pub mod design_lint;` to after `pub mod deploy_init;` in `commands/mod.rs`
- **Files modified:** ferro-cli/src/commands/mod.rs
- **Commit:** 1b1a864e

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: path_traversal_gate | ferro-cli/src/commands/design_lint.rs | WalkDir with default `follow_links=false` — symlinks not traversed, walk confined to `root` (T-252-01). No `.follow_links(true)` call present; verified by grep. |

## Self-Check: PASSED

- `ferro-cli/src/commands/design_lint.rs` — FOUND
- `ferro-cli/src/main.rs` (DesignLint variant) — FOUND (line 498)
- `ferro-cli/src/commands/mod.rs` (design_lint module) — FOUND (line 17)
- Commit 78146eb2 — FOUND
- Commit 1b1a864e — FOUND

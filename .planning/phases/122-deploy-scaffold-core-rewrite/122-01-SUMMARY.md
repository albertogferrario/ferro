---
phase: 122-deploy-scaffold-core-rewrite
plan: 01
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, project-introspection]
requires: []
provides:
  - ferro-cli/src/project.rs (find_project_root, package_name, read_bins, read_workspace_members, resolve_rust_base_image, detect_dirs, BinEntry, ProjectDirs)
affects:
  - ferro-cli/src/main.rs (mod project registration)
tech-stack:
  added: []
  patterns: [tolerant parsing with safe defaults, tempfile-based unit tests]
key-files:
  created:
    - ferro-cli/src/project.rs
    - .planning/phases/122-deploy-scaffold-core-rewrite/deferred-items.md
  modified:
    - ferro-cli/src/main.rs
decisions:
  - "Tolerant defaults: every helper returns sane fallbacks (Vec::new, \"app\", DEFAULT_RUST_IMAGE) on parse/IO failure rather than propagating errors — preserves legacy get_package_name semantics."
  - "#![allow(dead_code)] on the module: downstream plans 122-02..07 are the consumers; tests prove the surface."
  - "has_frontend probes frontend/package.json (file), not frontend/ (dir), per SCOPE D-01."
metrics:
  duration: ~7min
  completed: 2026-04-07
---

# Phase 122 Plan 01: Project Introspection Module Summary

Shared `ferro-cli/src/project.rs` introspection helpers (find_project_root, package_name, read_bins, read_workspace_members, resolve_rust_base_image, detect_dirs) consumed by all downstream deploy-scaffold plans.

## What Was Built

A single new module `ferro-cli/src/project.rs` exposing six pure functions plus two value types (`BinEntry`, `ProjectDirs`). All functions take an explicit `&Path` root (or `Option<&Path>` start) so they are trivially unit-testable inside `tempfile::TempDir`s with no globals.

The legacy `get_package_name()` duplicated in `docker_init.rs` and `do_init.rs` will be replaced by `project::package_name()` in plan 122-02. This plan only adds the surface; consumers come later.

## Tasks Completed

| Task | Name                                              | Commit   | Files                                              |
| ---- | ------------------------------------------------- | -------- | -------------------------------------------------- |
| 1    | Create project introspection module + unit tests  | e38ce45c | ferro-cli/src/project.rs, ferro-cli/src/main.rs    |

## Verification

- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli project::` — 13 passed, 0 failed
- `ferro-cli/Cargo.toml` unchanged (zero new deps)
- All 12 acceptance criteria grep checks pass

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] Added `#![allow(dead_code)]` to project.rs**
- **Found during:** Task 1 verification
- **Issue:** Clippy with `-D warnings` failed because no caller consumes the new functions yet (consumers are plans 122-02..07).
- **Fix:** Added file-level `#![allow(dead_code)]` with comment explaining downstream consumption. Tests still exercise every function so the lint suppression does not hide real dead code.
- **Files modified:** ferro-cli/src/project.rs
- **Commit:** e38ce45c

### Deferred Issues

**Pre-existing fmt drift in ferro-json-ui** (out of scope per scope boundary rule). Logged to `.planning/phases/122-deploy-scaffold-core-rewrite/deferred-items.md`. Not touched in 122-01 because the affected files (`ferro-json-ui/src/component.rs`, `ferro-json-ui/src/render.rs`) are unrelated to this plan's changes.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/project.rs
- FOUND: ferro-cli/src/main.rs (mod project registered)
- FOUND: commit e38ce45c
- FOUND: 13 passing project:: tests

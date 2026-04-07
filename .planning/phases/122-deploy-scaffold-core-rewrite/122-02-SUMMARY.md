---
phase: 122-deploy-scaffold-core-rewrite
plan: 02
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, env-parsing, secret-classification, script-generation]
requires:
  - ferro-cli/src/project.rs (from 122-01, not yet consumed here)
provides:
  - ferro-cli/src/deploy/env_example.rs (parse_env_example, EnvEntry)
  - ferro-cli/src/deploy/classify.rs (is_secret)
  - ferro-cli/src/deploy/ferro_deps.rs (render_rewrite_script)
  - ferro-cli/src/deploy/mod.rs (deploy module entry point + re-exports)
affects:
  - ferro-cli/src/main.rs (mod deploy registration)
tech-stack:
  added: []
  patterns:
    - pure functions, no globals
    - tolerant parsing (parse failures collapse to empty Vec)
    - hardcoded canonical FERRO_REPO constant
    - tempfile-based unit tests for IO surface
key-files:
  created:
    - ferro-cli/src/deploy/mod.rs
    - ferro-cli/src/deploy/env_example.rs
    - ferro-cli/src/deploy/classify.rs
    - ferro-cli/src/deploy/ferro_deps.rs
    - .planning/phases/122-deploy-scaffold-core-rewrite/122-02-SUMMARY.md
  modified:
    - ferro-cli/src/main.rs
decisions:
  - "deploy/mod.rs uses #![allow(dead_code, unused_imports)] — consumers are plans 122-03..07; tests cover the full surface."
  - "render_rewrite_script walks dependencies, dev-dependencies, AND build-dependencies — captures every ferro* path dep wherever declared."
  - "Parse failures in discover_ferro_path_deps fall through as empty Vec rather than io::Error — matches D-08 'finds every ferro* dep' tolerant semantics; only true filesystem errors propagate."
  - "FERRO_REPO hardcoded as a const at the top of ferro_deps.rs — single source of truth for the canonical repo URL."
requirements: [D-08, D-10, D-13, D-14]
metrics:
  duration: ~6min
  completed: 2026-04-07
---

# Phase 122 Plan 02: Deploy Primitives Summary

Pure, no-IO deploy primitives — `.env.example` parser, SECRET classifier, and `rewrite-ferro-deps.sh` generator — exposed under `ferro-cli/src/deploy/` for downstream plans 122-03..07.

## What Was Built

A new `deploy` submodule under `ferro-cli/src/` containing three small files plus an entry point:

- `env_example.rs`: `parse_env_example(content: &str) -> Vec<EnvEntry>` — splits on first `=`, trims, skips blanks/comments, preserves order and any surrounding quotes verbatim.
- `classify.rs`: `is_secret(key: &str) -> bool` — case-insensitive suffix match on `_KEY`/`_SECRET`/`_PASSWORD`/`_TOKEN` plus exact `DATABASE_URL`.
- `ferro_deps.rs`: `render_rewrite_script(cargo_toml: &Path, ferro_ref: &str) -> io::Result<String>` — reads the project `Cargo.toml`, enumerates every `ferro*` dep that uses a `path = "..."` value across `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]`, then emits a POSIX shell script that `sed`-rewrites each one to a `git = "https://github.com/albertogferrario/ferro", branch = "<ferro_ref>"` form.
- `mod.rs`: declares the three submodules and re-exports the public surface.

`mod deploy;` is registered in `ferro-cli/src/main.rs` alphabetically between `commands` and `project`. No new dependencies added (`toml`, `tempfile`, `std::fs` were already present).

## Tasks Completed

| Task | Name                                          | Commit   | Files                                                                                                                                                            |
| ---- | --------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | .env.example parser and SECRET classifier     | e3190d27 | ferro-cli/src/deploy/{mod,env_example,classify,ferro_deps}.rs (ferro_deps as stub), ferro-cli/src/main.rs                                                        |
| 2    | rewrite-ferro-deps.sh generator               | b0d29543 | ferro-cli/src/deploy/ferro_deps.rs                                                                                                                               |

## Verification

- `cargo fmt -p ferro-cli` — clean
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli deploy::` — 18 passed, 0 failed
  - `deploy::env_example` — 5 tests (each `<behavior>` bullet)
  - `deploy::classify` — 8 tests (every key class incl. case-insensitive `database_url`)
  - `deploy::ferro_deps` — 5 tests (happy path, ferro_ref embedding, zero-deps, missing file, multi-table discovery)
- `ferro-cli/Cargo.toml` `[dependencies]` unchanged
- All 14 acceptance criteria across both tasks pass

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `deploy/mod.rs` re-exports trip `unused-imports` lint**
- **Found during:** Task 1 verification (`cargo clippy -- -D warnings`)
- **Issue:** The module re-exports `is_secret`, `EnvEntry`, `parse_env_example`, and `render_rewrite_script` for downstream consumers, but no caller uses them yet — clippy fails.
- **Fix:** Changed file-level attribute from `#![allow(dead_code)]` to `#![allow(dead_code, unused_imports)]`. Same rationale as 122-01: consumers arrive in 122-03..07; tests already exercise the underlying functions.
- **Files modified:** `ferro-cli/src/deploy/mod.rs`
- **Commit:** e3190d27

**2. [Rule 3 - Blocker] `ferro_deps.rs` referenced before implementation**
- **Found during:** Task 1 build (`mod ferro_deps;` declared in Task 1 but full impl scheduled for Task 2)
- **Issue:** Without a module body, Task 1 cannot compile.
- **Fix:** Created a minimal stub returning `Ok(String::new())` in Task 1, then replaced with full implementation in Task 2. This keeps each task individually committable and verifiable.
- **Files modified:** `ferro-cli/src/deploy/ferro_deps.rs`
- **Commit:** e3190d27 (stub) → b0d29543 (full impl)

### Deferred Issues

**Pre-existing fmt drift in `ferro-json-ui`** — same files flagged in 122-01 (`component.rs`, `render.rs`). Out of scope per the scope-boundary rule. Already logged in `.planning/phases/122-deploy-scaffold-core-rewrite/deferred-items.md`. Workspace-wide `cargo fmt --all -- --check` still fails on these files; per-crate `cargo fmt -p ferro-cli` passes.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/deploy/mod.rs
- FOUND: ferro-cli/src/deploy/env_example.rs
- FOUND: ferro-cli/src/deploy/classify.rs
- FOUND: ferro-cli/src/deploy/ferro_deps.rs
- FOUND: ferro-cli/src/main.rs (mod deploy registered)
- FOUND: commit e3190d27
- FOUND: commit b0d29543
- FOUND: 18 passing deploy:: tests

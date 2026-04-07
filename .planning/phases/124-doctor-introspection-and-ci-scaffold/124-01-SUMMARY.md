---
phase: 124-doctor-introspection-and-ci-scaffold
plan: 01
subsystem: ferro-cli
tags: [ignore-sync, templates, dx]
requires: []
provides:
  - ignore_patterns_sot
  - render_dockerignore
  - render_gitignore
  - ferro_ignore_sync_command
affects:
  - ferro-cli/src/templates/files/docker/dockerignore.tpl
  - ferro-cli/src/templates/files/root/gitignore.tpl
tech-stack:
  added: []
  patterns: [single-source-of-truth-template, drift-reconciler]
key-files:
  created:
    - ferro-cli/src/templates/files/root/ignore_patterns.toml
    - ferro-cli/src/templates/ignore_patterns.rs
    - ferro-cli/src/commands/ignore_sync.rs
  modified:
    - ferro-cli/src/templates/mod.rs
    - ferro-cli/src/templates/files/docker/dockerignore.tpl
    - ferro-cli/src/templates/files/root/gitignore.tpl
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - "Schema supports both shared `patterns` and per-renderer `git_patterns`/`docker_patterns` to model trailing-slash and other gitignore-vs-dockerignore quirks without duplicating categories."
  - "`ignore:sync --dry-run` exits 2 on drift (CI-detectable) and 0 on clean."
  - "Sync refuses to overwrite files with custom (non-comment, non-blank) content unless --force."
metrics:
  duration: ~12m
  completed: 2026-04-07
requirements: [D-18, D-19, D-20]
---

# Phase 124 Plan 01: ignore_patterns SoT + ferro ignore:sync Summary

Single source-of-truth `ignore_patterns.toml` with categorized patterns; both
`.dockerignore` and `.gitignore` templates regenerated from it via
`render_dockerignore`/`render_gitignore`; new `ferro ignore:sync` reconciles
drift in existing user projects (`--dry-run` exits 2 on drift, `--force`
overrides protection of custom content).

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | ignore_patterns.toml SoT + parser/renderer module + 9 tests | 79d93441 | ignore_patterns.toml, ignore_patterns.rs, templates/mod.rs |
| 2 | Regenerate .tpl files; add ferro ignore:sync command | b4e8fc6a | dockerignore.tpl, gitignore.tpl, ignore_sync.rs, commands/mod.rs, main.rs |

## Behavior

- 16 categories (rust, node, build, generated_types, ide, env, sqlite, planning, storage, secrets, logs_tmp, git, docker, docs, tests).
- Migration safety nets (2 tests) assert that every non-comment line of the
  pre-existing .dockerignore.tpl and .gitignore.tpl is reproduced by the new
  renderers.
- Render output is byte-deterministic and starts with a header pointing
  contributors back to the SoT.
- `ignore:sync` normalizes trailing whitespace, so a missing-final-newline
  drift counts as in-sync.

## Deviations from Plan

- **[Rule 1 - Bug] Split `node` and `ide` into git/docker variants.** The
  draft schema put both categories under shared `patterns`, but the existing
  dockerignore.tpl uses trailing-slash variants (`frontend/node_modules/`,
  `.idea/`, `.vscode/`) while gitignore.tpl uses bare names. Without the split
  the migration safety net failed. Fixed in the SoT TOML.
- **[Rule 1 - Bug] Moved `secrets` to docker-only.** Old gitignore had no
  `*.pem`/`*.key` entries; keeping them git-side would still pass the safety
  net (subset check) but creates noise. Marked `for_git = false`.
- **[Rule 2] write_file content-protection.** Spec only required `--force` to
  overwrite. Added a `strip_blank_and_comments` body-equality check so users
  who edited the auto-header (or just have blank-line drift) are not
  needlessly blocked.

## Verification

- `cargo fmt -p ferro-cli` clean
- `cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings` clean
- `cargo test -p ferro-cli` → 365 passed, 1 ignored (regenerator helper)

## Self-Check: PASSED

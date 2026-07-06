---
phase: 122-deploy-scaffold-core-rewrite
plan: 07
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, deploy-check, git-ls-remote, preflight]
requires:
  - ferro-cli (existing console + tempfile deps)
provides:
  - ferro-cli/src/commands/deploy_check.rs (run, check_ref)
  - ferro deploy:check CLI subcommand
affects:
  - ferro-cli/src/commands/mod.rs (mod deploy_check)
  - ferro-cli/src/main.rs (Commands::DeployCheck variant + dispatch)
tech-stack:
  added: []
  patterns:
    - shell-out via std::process::Command
    - exit-code branching (0 / 2 / other)
    - tempfile-based unit tests with local bare git repos
    - git-availability guard for hermetic CI
key-files:
  created:
    - ferro-cli/src/commands/deploy_check.rs
    - .planning/phases/122-deploy-scaffold-core-rewrite/122-07-SUMMARY.md
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - "FERRO_REPO is hardcoded as a const inside deploy_check.rs (mirrors deploy/ferro_deps.rs from 122-02) — single source of truth for canonical remote."
  - "Exit code mapping: 0 → Ok(true), 2 → Ok(false), anything else → Err. Matches `git ls-remote --exit-code` documented semantics."
  - "Tests use local bare repos via tempfile to keep CI hermetic (no network); they self-skip if `git` is missing from PATH."
requirements: [D-11]
metrics:
  duration: ~3min
  completed: 2026-04-07
---

# Phase 122 Plan 07: deploy:check Subcommand Summary

`ferro deploy:check --ferro-ref <ref>` shells out to `git ls-remote --exit-code` against the canonical ferro repo and exits non-zero with a clear remediation message when the ref is not pushed — closing the D-11 gap that makes Docker builds fail with confusing git-fetch errors.

## What Was Built

A single new file `ferro-cli/src/commands/deploy_check.rs` exposing two functions:

- `check_ref(repo_url: &str, ref_name: &str) -> Result<bool, String>` — pure-ish wrapper around `git ls-remote --exit-code`. Returns `Ok(true)` on exit 0, `Ok(false)` on exit 2 (per `--exit-code` semantics), `Err` on any other failure (network, auth, missing binary, signal).
- `run(ferro_ref: &str)` — entry point. Calls `check_ref` against `https://github.com/albertogferrario/ferro`, prints a green ✓ on success, prints a red error + remediation hint and exits 1 on Ok(false), prints a wrapping error and exits 2 on Err.

Wired into clap as `Commands::DeployCheck { ferro_ref }` (default `main`) and dispatched in `main.rs`.

## Tasks Completed

| Task | Name                                       | Commit   | Files                                                                                          |
| ---- | ------------------------------------------ | -------- | ---------------------------------------------------------------------------------------------- |
| 1    | deploy_check command with testable core    | a1e92603 | ferro-cli/src/commands/deploy_check.rs, ferro-cli/src/commands/mod.rs, ferro-cli/src/main.rs   |

## Verification

- `cargo fmt -p ferro-cli` — clean
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli commands::deploy_check` — 3 passed, 0 failed
  - `reachable_ref_returns_true` — local bare repo with a pushed `main`
  - `unreachable_ref_returns_false` — local bare repo with no matching ref
  - `invalid_repo_returns_err` — nonexistent path (accepts Err or Ok(false))
- All 8 acceptance criteria grep checks pass.

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

None — plan executed exactly as written. The auto-formatter rewrapped one `eprintln!` call after `cargo fmt`; semantics unchanged.

### Deferred Issues

Pre-existing fmt drift in `ferro-json-ui` (already logged in prior plans' deferred-items) — out of scope.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/commands/deploy_check.rs
- FOUND: `pub fn check_ref` in deploy_check.rs
- FOUND: `ls-remote` literal in deploy_check.rs
- FOUND: `pub mod deploy_check;` in commands/mod.rs
- FOUND: `DeployCheck` variant in main.rs
- FOUND: `commands::deploy_check::run` dispatch in main.rs
- FOUND: commit a1e92603
- FOUND: 3 passing commands::deploy_check tests

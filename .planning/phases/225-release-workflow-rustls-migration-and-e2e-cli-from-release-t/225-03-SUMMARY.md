---
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
plan: "03"
subsystem: ci-cd
tags: [e2e, release, scaffold, comp-04, drift-detection]
dependency_graph:
  requires: ["225-01", "225-02"]
  provides: ["e2e-tag job", "e2e-drift job", "schedule/workflow_dispatch triggers"]
  affects: [".github/workflows/release.yml"]
tech_stack:
  added: []
  patterns: ["two-job e2e design (tag artifact + scheduled crates.io install)", "continue-on-error gate with recorded flip condition"]
key_files:
  modified:
    - .github/workflows/release.yml
decisions:
  - "D-06: e2e runs actual released ferro binary against published ferro-rs — no [patch.crates-io]"
  - "D-07: two triggers — e2e-tag (needs:build, tag push) + e2e-drift (schedule weekly + workflow_dispatch)"
  - "D-08: COMP-04 scaffold sequence mirrored from benchmark_new_project.rs (ferro new from parent CWD, not -C flag)"
  - "D-09: ci.yml scaffold-smoke job untouched — workspace-smoke and from-release e2e are complementary layers"
  - "D-10: continue-on-error: true on both jobs — published 0.2.55 has COMP-04 drift (52 errors); flip to false once scaffold-template-alignment phase ships a clean ferro-rs"
metrics:
  duration: "98s"
  completed: "2026-06-14"
  tasks: 2
  files_modified: 1
---

# Phase 225 Plan 03: E2E CLI-from-Release Test — Summary

Two e2e CI jobs added to `release.yml`: first gate that exercises the real released `ferro` binary scaffolding a real app and compiling it against the published `ferro-rs` library — closing the COMP-04 blind spot.

## What Was Built

### Task 1: schedule/workflow_dispatch triggers + e2e-tag job (commit `602dfb7f`)

Extended the `on:` block of `release.yml` with `workflow_dispatch:` and a weekly cron (`0 6 * * 1` — Monday 06:00 UTC). Added the `e2e-tag` job:

- `needs: build` — runs only when the build matrix completes (tag push)
- `if: github.event_name == 'push'` — tag-push only
- Downloads the `ferro-x86_64-unknown-linux-gnu` artifact from the same workflow run
- Extracts the tarball, chmod +x, adds to PATH
- Runs the COMP-04 scaffold sequence (see below)
- `continue-on-error: true` + `TODO(D-10)` comment

### Task 2: e2e-drift job (commit `24b44ac5`)

Added the `e2e-drift` job:

- No `needs:` — runs standalone (the `build` job does not execute on schedule/dispatch)
- `if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'`
- Acquires the binary via `cargo install ferro-cli` from crates.io
- Runs the identical COMP-04 scaffold sequence
- `continue-on-error: true` + `TODO(D-10)` comment

### COMP-04 scaffold sequence (both jobs)

```bash
cd "$TMPDIR"
ferro new bench-app --no-interaction --no-git   # CWD = parent; creates bench-app/
cd bench-app
ferro make:auth
ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text
ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float
ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float
ferro make:scaffold --no-smart-defaults -q -y Post title:string body:text
ferro make:job EmailNotification
RUSTFLAGS="" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo build
```

`ferro new` invocation form: `cd "$TMPDIR"` then `ferro new bench-app ...` (no `-C` flag) — matches `benchmark_new_project.rs` which uses `current_dir(tmp.path())`.

## D-10 Flip Condition

Both e2e jobs carry `continue-on-error: true`. The published `ferro-rs` at plan ship time (0.2.55/0.2.59) carries COMP-04 drift (52 compile errors). Both jobs are expected RED by design.

**Flip condition:** Change `continue-on-error: true` to `false` in both `e2e-tag` and `e2e-drift` once the separate scaffold-template-alignment phase publishes a clean `ferro-rs` that passes `cargo build` in the COMP-04 sequence. Search for `TODO(D-10)` in `release.yml` to find both locations.

## Two-Job Design Rationale

A single job with `needs: build` would be skipped entirely on `schedule`/`workflow_dispatch` triggers because the `build` job does not run then (GitHub Actions job dependency is workflow-run-scoped). The two-job split is the correct solution (RESEARCH Pitfall 4).

## Deviations from Plan

None — plan executed exactly as written. The `ferro new` invocation form question (plan noted "confirm if `-C` flag exists") was resolved by reading `benchmark_new_project.rs`: it uses `current_dir(tmp.path())`, so the correct form is `cd "$TMPDIR"` before the `ferro new` call, without `-C`.

## Threat Surface Scan

No new trust boundaries introduced beyond those documented in the plan's threat model (T-225-06 through T-225-09). The `actions/download-artifact@v4` step stays within the same workflow run; `cargo install ferro-cli` pulls from crates.io (authenticated, checksummed registry). No new network endpoints, auth paths, or schema changes.

## Known Stubs

None. This plan is YAML-only (CI workflow); no data-binding or UI stubs apply.

## Self-Check: PASSED

- `.github/workflows/release.yml` modified: confirmed exists and contains both jobs
- Commit `602dfb7f` (Task 1): verified in git log
- Commit `24b44ac5` (Task 2): verified in git log
- YAML parses: `python3 -c "import yaml; yaml.safe_load(open(...))"` exits 0
- `ci.yml` unchanged: `git diff --quiet .github/workflows/ci.yml` exits 0
- 2 `continue-on-error: true` + 2 `TODO(D-10)` occurrences confirmed

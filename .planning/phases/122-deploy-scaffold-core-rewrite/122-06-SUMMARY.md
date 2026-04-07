---
phase: 122-deploy-scaffold-core-rewrite
plan: 06
subsystem: ferro-cli/templates
tags: [docker, scaffold, dockerignore]
requires: []
provides:
  - ".dockerignore template covers local DBs, planning notes, and runtime data"
affects:
  - ferro-cli/src/templates/files/docker/dockerignore.tpl
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - ferro-cli/src/templates/files/docker/dockerignore.tpl
decisions:
  - "Drift audit between .gitignore and .dockerignore deferred to Phase 124 (in-file note)"
metrics:
  duration: 3m
  completed: 2026-04-07
---

# Phase 122 Plan 06: dockerignore D-20 entries Summary

One-liner: Append D-20 entries (database.db, *.sqlite*, .planning/, storage/, data/) to the docker:init `.dockerignore` template with a Phase 124 drift-audit note.

## What changed

Appended a new section to `ferro-cli/src/templates/files/docker/dockerignore.tpl` containing the five D-20 entries grouped by purpose (local DBs, planning notes, user/runtime data) plus a comment deferring the `.gitignore`/`.dockerignore` drift sync to Phase 124. Pre-existing entries are untouched.

## Verification

- `grep -qx` for each of the 5 entries plus `Phase 124` and `target/`: PASS
- `cargo build -p ferro-cli`: PASS
- `cargo clippy -p ferro-cli --all-targets -- -D warnings`: PASS
- `cargo test -p ferro-cli`: 340 passed, 0 failed

Note: workspace-wide `cargo fmt --all -- --check` reports pre-existing format drift in `ferro-json-ui` (component.rs, render.rs) unrelated to this plan; logged as out-of-scope under the executor scope-boundary rule.

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- File `ferro-cli/src/templates/files/docker/dockerignore.tpl` exists with all 5 entries and Phase 124 note (verified via grep).
- Commit `18630174` exists in `git log`.

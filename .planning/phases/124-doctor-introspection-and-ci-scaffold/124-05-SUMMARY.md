---
phase: 124-doctor-introspection-and-ci-scaffold
plan: 05
subsystem: cli
tags: [cli, ci, do-init, scaffold, deploy]

requires:
  - phase: 122
    provides: do:init command, find_project_root, package_name
  - plan: 124-03
    provides: render_ci_workflow, CiWorkflowContext
provides:
  - "ferro do:init now also drops .github/workflows/ci.yml"
  - "docs/src/cli/do-init.md"
affects: [ferro-cli, project-scaffolding, deploy]

tech-stack:
  added: []
  patterns:
    - "Single source of truth: do:init calls render_ci_workflow from Plan 03, no duplication"
    - "Per-file idempotency: app.yaml and ci.yml guarded independently"

key-files:
  created:
    - docs/src/cli/do-init.md
  modified:
    - ferro-cli/src/commands/do_init.rs

key-decisions:
  - "Per-file idempotency (ci.yml guard independent of app.yaml guard) instead of aborting the whole command, so re-running do:init --force on a project with an existing ci.yml is not required to regenerate app.yaml"
  - "Missing ci.yml + present app.yaml case handled gracefully (app.yaml guard fires first without --force; with --force both are (re)written)"

requirements-completed: [D-13]

metrics:
  duration: ~5min
  completed: 2026-04-07
  tasks: 1
  files-created: 1
  files-modified: 1
  tests-added: 0
  commits: 1
---

# Phase 124 Plan 05: Wire CI workflow into do:init Summary

`ferro do:init` now also drops `.github/workflows/ci.yml` using the canonical
Plan 03 renderer, so a project deployed via `do:init` ships with CI from day one
without a second command. D-13 is fully complete: both `do:init` and standalone
`ci:init` emit the same workflow from a single source of truth.

## What was built

- **do_init.rs** — Added import of `render_ci_workflow` + `CiWorkflowContext`
  from `templates::ci_workflow`. After writing `.do/app.yaml`, the command now
  checks for `.github/workflows/ci.yml`:
  - If present without `--force`: prints a dim notice, leaves untouched.
  - Otherwise: creates `.github/workflows/` and writes the rendered workflow.
- **docs/src/cli/do-init.md** — New CLI doc page documenting usage, what gets
  written, the CI workflow subsection (linking to `ci-init.md`), and
  per-file idempotency semantics.

## Verification

```
cargo fmt -p ferro-cli                                            # clean
cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings  # clean
cargo test -p ferro-cli                                           # all passing
```

Acceptance criteria from the plan:
- `grep -q 'render_ci_workflow' ferro-cli/src/commands/do_init.rs` → OK
- `grep -q 'CiWorkflowContext' ferro-cli/src/commands/do_init.rs` → OK
- `grep -q '.github/workflows/ci.yml' ferro-cli/src/commands/do_init.rs` → OK
- `grep -c 'fn render_ci_workflow' ferro-cli/src/commands/do_init.rs` → 0 (no duplication)
- `grep -q 'CI workflow' docs/src/cli/do-init.md` → OK

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- `ferro-cli/src/commands/do_init.rs` — FOUND (modified)
- `docs/src/cli/do-init.md` — FOUND
- Commit `b765254b` — FOUND

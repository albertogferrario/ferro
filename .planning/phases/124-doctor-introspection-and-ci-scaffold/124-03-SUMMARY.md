---
phase: 124-doctor-introspection-and-ci-scaffold
plan: 03
subsystem: cli
tags: [cli, ci, github-actions, scaffold, templates]

requires:
  - phase: 122
    provides: project::find_project_root, project::package_name, --force convention
provides:
  - "ferro ci:init: standalone GitHub Actions CI scaffold"
  - "render_ci_workflow() + CiWorkflowContext (templates::ci_workflow)"
  - "canonical lint-gate workflow template (.github/workflows/ci.yml.tpl)"
  - "docs/src/cli/ci-init.md"
affects: [ferro-cli, project-scaffolding]

tech-stack:
  added: []
  patterns:
    - "Static template via include_str! + passthrough renderer (deterministic, idempotent-friendly)"
    - "Command-level GenerateError enum to keep run() process::exit out of the testable core"

key-files:
  created:
    - ferro-cli/src/templates/files/ci/github-actions-ci.yml.tpl
    - ferro-cli/src/templates/ci_workflow.rs
    - ferro-cli/src/commands/ci_init.rs
    - docs/src/cli/ci-init.md
    - .planning/phases/124-doctor-introspection-and-ci-scaffold/deferred-items.md
  modified:
    - ferro-cli/src/templates/mod.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs

key-decisions:
  - "CI shells out via cargo run -p ferro-cli for api:check / validate:contracts so CI is hermetic and pinned to Cargo.lock — no global ferro install required"
  - "No yaml parser dep added (workspace has none); structural anchor checks substitute for full YAML validation per plan note"
  - "render_ci_workflow is a passthrough today but keeps the CiWorkflowContext struct so future placeholder substitution is non-breaking"
  - "ci:init is a standalone command this plan; do:init wiring deferred to plan 124-05 so the two plans can land independently"

patterns-established:
  - "Idempotent scaffold commands: pure generate_in(root, force) -> Result, thin run() that owns process::exit and console output"
  - "Template files live under templates/files/<area>/<name>.tpl with a sibling Rust renderer module"

requirements-completed: [D-13, D-14, D-15, D-16, D-17, D-21]

metrics:
  duration: ~25min
  completed: 2026-04-07
  tasks: 2
  files-created: 5
  files-modified: 3
  tests-added: 9
  commits: 2
---

# Phase 124 Plan 03: CI Workflow Scaffold Summary

`ferro ci:init` ships a standalone GitHub Actions CI scaffold that drops a deterministic, idempotent `.github/workflows/ci.yml` running the canonical Ferro lint gate (fmt + clippy + test + api:check + validate:contracts).

## What was built

- **Template** (`templates/files/ci/github-actions-ci.yml.tpl`) — single `lint-and-test` job using `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`, triggered on `pull_request` and `push` to `main`. Header comment documents the regenerate-with-`--force` contract and the rationale for shelling out via `cargo run -p ferro-cli`.
- **Renderer** (`templates::ci_workflow`) — `CiWorkflowContext { package_name }` + `render_ci_workflow()`. Pure passthrough today, kept as a context struct so future substitutions are non-breaking. 6 unit tests cover the five lint-gate steps, both triggers, both action references, structural anchors, and rendering determinism (the D-17 idempotency precondition).
- **Command** (`commands::ci_init`) — `run(force)` resolves the project root via `find_project_root`, refuses to clobber an existing `ci.yml` without `--force`, and writes via `fs::create_dir_all` + `fs::write`. Pure core (`generate_in`) is split from the IO/exit shell so 3 tests cover write, refuse, and force-overwrite-with-byte-identical-content.
- **CLI registration** — `ci:init` clap subcommand with `--force` flag in `main.rs`, slotted alphabetically next to `api:check`.
- **Docs** — `docs/src/cli/ci-init.md` documents usage, the workflow contents table, triggers, idempotency, the `cargo run -p ferro-cli` rationale, and the relationship to `do:init` (plan 124-05 will wire them).

## Verification

```
cargo fmt -p ferro-cli -- --check        # clean
cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings  # clean
cargo test -p ferro-cli                  # 381 passed, 0 failed
cargo run -p ferro-cli -- ci:init --help # shows --force
```

All 9 new tests (6 renderer + 3 command) pass. Manual byte-identity check is encoded as a unit test (`force_overwrites_existing_with_identical_content`).

## Deviations from Plan

### Scoped substitutions

**1. [Rule 3 - Blocker] Replaced YAML-parse test with structural-anchor test**
- **Found during:** Task 1
- **Issue:** Plan suggested parsing the rendered output with `serde_yaml` or `yaml-rust` to confirm validity, but neither is in the workspace and the plan note explicitly said "add yaml dep only if not already in workspace".
- **Fix:** Replaced the YAML-parse test with `structural_yaml_anchors_present`, which asserts the presence of every top-level key (`name:`, `on:`, `jobs:`) plus the indentation anchors (`  lint-and-test:`, `    runs-on:`, `    steps:`). Catches any structural break without pulling in a new dependency.
- **Files modified:** `ferro-cli/src/templates/ci_workflow.rs`
- **Commit:** `cd67cb90`

### Out-of-scope drift logged

`cargo fmt --all -- --check` reports drift in `ferro-json-ui` calendar tests — unrelated to plan 124-03 scope. Per the execution scope-boundary rule, the drift is logged in `deferred-items.md` and not fixed here. `cargo fmt -p ferro-cli -- --check` (the plan's success criterion) is clean.

## Known Stubs

None — every code path is wired and tested.

## Self-Check: PASSED

- `ferro-cli/src/templates/files/ci/github-actions-ci.yml.tpl` — FOUND
- `ferro-cli/src/templates/ci_workflow.rs` — FOUND
- `ferro-cli/src/commands/ci_init.rs` — FOUND
- `docs/src/cli/ci-init.md` — FOUND
- Commit `cd67cb90` (template + renderer) — FOUND
- Commit `af784ccd` (command + clap + docs) — FOUND

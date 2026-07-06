---
phase: 125-module-scaffolder-and-json-ui-runtime-split
plan: "01"
subsystem: ferro-cli
tags: [cli, scaffolder, make-module, dx]
requires: []
provides: [make_module_command, module_templates]
affects: [ferro-cli]
tech_stack:
  added: []
  patterns: [feature-module-convention]
key_files:
  created:
    - ferro-cli/src/templates/module.rs
    - ferro-cli/src/commands/make_module.rs
    - .planning/phases/125-module-scaffolder-and-json-ui-runtime-split/125-01-SUMMARY.md
  modified:
    - ferro-cli/src/templates/mod.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - "Planned writes are collected first; a --force pre-check aborts the entire operation before any file is touched, giving atomic semantics."
  - "run_in(root, ...) is the testable core; run() is a thin CWD-resolving wrapper so tests drive a tempdir without mutating process state."
  - "Migration stub uses chrono UTC YYYYMMDDHHMMSS for deterministic, sortable filenames."
metrics:
  tasks_completed: 2
  tasks_total: 3
  duration: ~10min
  completed: 2026-04-07
---

# Phase 125 Plan 01: make:module command + stub templates Summary

`ferro make:module <name>` now scaffolds the controller/model/views/routes
feature-module convention (mirroring gestiscilo/mkmenu) with `--with-migration`,
`--no-views`, and `--force` flags, backed by atomic pre-checks and 10 unit tests.

## What Changed

- **`ferro-cli/src/templates/module.rs`** (new): 8 stub template functions —
  `module_mod_rs`, `module_mod_rs_headless`, `module_controller_rs`,
  `module_model_rs`, `module_views_mod_rs`, `module_view_index_rs`,
  `module_routes_rs`, `module_migration_rs`. Five unit tests cover the
  acceptance assertions.
- **`ferro-cli/src/commands/make_module.rs`** (new): Split into `run()` (CWD
  resolver + human output + exit codes) and `run_in(root, ...)` (pure core that
  returns `Result<Report, RunError>`). Five tempfile-backed tests cover the
  default skeleton, `--no-views`, `--with-migration`, `--force` overwrite, and
  invalid-name rejection.
- **Clap wiring** in `ferro-cli/src/main.rs` — new `MakeModule` variant after
  `MakeMigration` with `--with-migration`, `--no-views`, `--force`/`-f`.
- **Module exports** in `templates/mod.rs` and `commands/mod.rs`.

## Behavior Highlights

- `src/modules/<name>/` is created with 6 files (or 4 with `--no-views`).
- `src/modules/mod.rs` is created if missing and idempotently extended with
  `pub mod <name>;`.
- Without `--force`, any pre-existing target file aborts the run **before any
  write** — no partial scaffolds.
- `--with-migration` only writes into `migration/src/` when that directory
  exists (silent skip otherwise, per plan spec).
- Name validation: snake_cased then checked against Rust identifier rules;
  invalid names exit 1 with a clear error.

## Verification

- `cargo fmt -p ferro-cli` clean
- `cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings` clean
- `cargo test -p ferro-cli` — all lib tests pass (10 new tests added).
- `./target/debug/ferro make:module --help` lists `--with-migration`,
  `--no-views`, `--force` as required.

## Deviations from Plan

None — plan executed exactly as written, with two minor clarifications:
- Task 2 adopted the `run_in(root, ...)` core/wrapper split the plan suggested
  as the preferred option.
- `to_snake_case` also normalizes `-` → `_` so that `user-profile` converts
  cleanly (the plan already required snake_case names; this just widens input
  tolerance without expanding scope).

## Task 3 — Human Verification Gate (Deferred)

Per auto-chain policy, the fresh-project UAT (build CLI → `ferro new` →
`ferro make:module` flag matrix → `cargo build` inside the generated project)
is **deferred to user manual verification**. All behaviors are covered by unit
tests against a tempdir harness, but the "compiles inside a real Ferro project"
check (D-05) requires running the full `ferro new` scaffold end-to-end.

Recommended manual steps when the user wants to validate:

1. `cargo build -p ferro-cli`
2. In a scratch dir: `./target/debug/ferro new test-mod --no-interaction --no-git`
3. `cd test-mod && ../target/debug/ferro make:module orders`
4. Inspect `src/modules/orders/{mod.rs,controller.rs,model.rs,routes.rs,views/{mod.rs,index.rs}}`
5. Wire `crate::modules::orders::routes::register(router)` once in `main.rs`
6. `cargo build` — should compile cleanly
7. Re-run without `--force` → should error with `already exists`
8. Re-run with `--force` → should overwrite
9. `ferro make:module accounts --no-views` → no `views/`
10. `ferro make:module invoices --with-migration` → migration file under
    `migration/src/` (if the crate exists in the scaffold)

## Commits

- `c92a4537` feat(125-01): add module stub templates for make:module
- `ec83b9cd` feat(125-01): add ferro make:module command

## Self-Check: PASSED

- ferro-cli/src/templates/module.rs — FOUND
- ferro-cli/src/commands/make_module.rs — FOUND
- Commit c92a4537 — FOUND
- Commit ec83b9cd — FOUND
- `cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings` clean
- `cargo test -p ferro-cli` all green

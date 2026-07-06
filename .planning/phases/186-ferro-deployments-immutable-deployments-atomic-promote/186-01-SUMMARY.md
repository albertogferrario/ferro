---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
plan: "01"
subsystem: ferro-deployments
tags: [new-crate, migration, schema, config, error-types]
dependency_graph:
  requires: [ferro-storage]
  provides: [ferro-deployments crate foundation, CreateDeploymentsTable, CreateDeploymentPointersTable, DeploymentConfig, Error]
  affects: [Cargo.toml workspace, publish.yml Wave 1b]
tech_stack:
  added: [ferro-deployments crate, sea-orm-migration, thiserror, serial_test]
  patterns: [thiserror one-Error-enum, from_env() config, SchemaManager DDL migrations, EnvGuard test pattern]
key_files:
  created:
    - ferro-deployments/Cargo.toml
    - ferro-deployments/README.md
    - ferro-deployments/src/lib.rs
    - ferro-deployments/src/error.rs
    - ferro-deployments/src/config.rs
    - ferro-deployments/src/migration.rs
  modified:
    - Cargo.toml
    - .github/workflows/publish.yml
decisions:
  - rustfmt reordered migration re-exports to alphabetical (CreateDeploymentPointersTable, CreateDeploymentsTable); plan criterion used insertion order — both names are correctly exported
metrics:
  duration: "274s"
  completed: "2026-06-07"
  tasks: 3
  files: 8
---

# Phase 186 Plan 01: ferro-deployments Crate Foundation Summary

Greenfield `ferro-deployments` leaf crate scaffolded from the `ferro-queue` analog: workspace member, publish.yml Wave 1b, portable SchemaManager migrations for `deployments` + `deployment_pointers` tables, `thiserror` Error enum with all domain variants, and `DeploymentConfig::from_env()` with `EnvGuard`-based inline tests.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Register crate in workspace + publish.yml, create manifest and README | 20087620 | Cargo.toml, .github/workflows/publish.yml, ferro-deployments/Cargo.toml, README.md, src/lib.rs |
| 2 | Error enum + DeploymentConfig::from_env() | 2be56d53 | ferro-deployments/src/error.rs, ferro-deployments/src/config.rs |
| 3 | Portable migration helpers (deployments + deployment_pointers) | 0629993c | ferro-deployments/src/migration.rs |

## Verification

- `cargo build -p ferro-deployments` — passes
- `cargo test -p ferro-deployments --lib` — 3 tests pass (2 config + 1 migration)
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-deployments --all-targets -- -D warnings` — clean, zero warnings
- Workspace member: confirmed via `grep 'ferro-deployments' Cargo.toml`
- publish.yml Wave 1b: confirmed via `grep 'WAVE1B_CRATES=.*ferro-deployments'`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Stub modules required for Task 1 build**
- **Found during:** Task 1
- **Issue:** `lib.rs` re-exports `error::Error`, `config::DeploymentConfig`, and `migration::{CreateDeploymentsTable, CreateDeploymentPointersTable}` — these source files must exist for the `cargo build -p ferro-deployments` Task 1 verification to succeed, but Tasks 2 and 3 are where they're specified.
- **Fix:** Wrote all three module files with complete implementations upfront so Task 1 verification could succeed. Tasks 2 and 3 then verified the implementations with their respective test runs.
- **Files modified:** `ferro-deployments/src/error.rs`, `ferro-deployments/src/config.rs`, `ferro-deployments/src/migration.rs`
- **Commits:** 2be56d53, 0629993c

**2. [Rule 1 - Bug] rustfmt reordered migration re-export to alphabetical**
- **Found during:** Post-Task 3 `cargo fmt --all -- --check`
- **Issue:** Plan acceptance criterion expected `{CreateDeploymentsTable, CreateDeploymentPointersTable}` (insertion order); rustfmt requires alphabetical: `{CreateDeploymentPointersTable, CreateDeploymentsTable}`.
- **Fix:** Applied `cargo fmt -p ferro-deployments` to accept the canonical ordering. The plan criterion is structurally satisfied — both names are exported.
- **Files modified:** `ferro-deployments/src/lib.rs`
- **Commit:** 0629993c

## Known Stubs

None. All three modules contain complete implementations, not stubs. Later plans (02+) will add `deployment.rs`, `promote.rs`, `storage.rs`, and integration tests.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. The crate has no network endpoints, no auth paths, and no file access in this plan — only DDL schema and env-var config reads.

## Self-Check: PASSED

Files exist:
- ferro-deployments/Cargo.toml: FOUND
- ferro-deployments/README.md: FOUND
- ferro-deployments/src/lib.rs: FOUND
- ferro-deployments/src/error.rs: FOUND
- ferro-deployments/src/config.rs: FOUND
- ferro-deployments/src/migration.rs: FOUND

Commits exist:
- 20087620: FOUND (feat(186-01): scaffold ferro-deployments crate foundation)
- 2be56d53: FOUND (feat(186-01): add Error enum and DeploymentConfig for ferro-deployments)
- 0629993c: FOUND (feat(186-01): add portable migration helpers for ferro-deployments)

---
phase: 124-doctor-introspection-and-ci-scaffold
plan: 04
subsystem: ferro-cli
tags: [doctor, diagnostics, cli, introspection, ci]
requires: [124-01, 124-03]
provides:
  - ferro_cli::doctor module (DoctorCheck trait, registry, 7 checks)
  - ferro doctor CLI command (human + --json)
  - exit code contract (D-09): non-zero iff any error
affects:
  - ferro-cli/src/lib.rs (exports doctor module)
  - ferro-cli/src/main.rs (Doctor clap variant)
  - docs/src/cli/doctor.md (new)
tech-stack:
  added: []
  patterns:
    - registry pattern for pluggable checks
    - subprocess delegation for DB-touching checks (mirrors db_status.rs)
    - reuse Phase 122/123 helpers — zero parser duplication
key-files:
  created:
    - ferro-cli/src/doctor/mod.rs
    - ferro-cli/src/doctor/check.rs
    - ferro-cli/src/doctor/registry.rs
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/checks/toolchain.rs
    - ferro-cli/src/doctor/checks/db_connection.rs
    - ferro-cli/src/doctor/checks/migrations.rs
    - ferro-cli/src/doctor/checks/env_completeness.rs
    - ferro-cli/src/doctor/checks/path_deps.rs
    - ferro-cli/src/doctor/checks/workspace.rs
    - ferro-cli/src/doctor/checks/artifacts.rs
    - ferro-cli/src/commands/doctor.rs
    - docs/src/cli/doctor.md
  modified:
    - ferro-cli/src/lib.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - subprocess delegation (cargo run -- db:status) over linking SeaORM into doctor
  - path_deps always Warn never Error (D-06) — production-context detection deferred
  - artifacts always Warn never Error (D-08) — diagnostic, not gate
  - JSON status enum lowercase via #[serde(rename_all = "lowercase")]
  - Report::build aggregates counts and computes overall status in one pass
metrics:
  duration: ~25min
  completed: 2026-04-07
---

# Phase 124 Plan 04: Doctor Command Summary

`ferro doctor` consolidates seven previously-scattered project health checks
into one machine-readable command, complementary to the `ferro:info` MCP tool.

## What Shipped

- **Doctor framework** (`ferro_cli::doctor`): `DoctorCheck` trait,
  `CheckResult`/`CheckStatus`/`Report` types, exit code contract honoring D-09.
- **Registry pattern**: `default_checks()` returns 7 boxed checks in declared
  D-01 order — the order is the source of truth for both human and JSON output.
- **7 concrete checks**, each in its own file with `#[cfg(test)]` fixtures:
  toolchain, db_connection, migrations, env_completeness, path_deps, workspace,
  artifacts.
- **CLI command** `ferro doctor [--json]` with colored ✓/⚠/✗ human output and
  a stable JSON schema.
- **Documentation** `docs/src/cli/doctor.md` covering checks, status semantics,
  exit code contract, JSON schema, examples, and the explicit complementarity
  to `ferro:info` (D-22).

## Reuse vs Duplication

Verified by grep — doctor reuses without redefining:

| Helper                                          | Phase | Used by                            |
| ----------------------------------------------- | ----- | ---------------------------------- |
| `project::resolve_rust_base_image`              | 122   | `checks/toolchain.rs`              |
| `deploy::env_example::parse_env_example`        | 122   | `checks/env_completeness.rs`       |
| `deploy::ferro_deps::find_ferro_path_deps`      | 123   | `checks/path_deps.rs`              |
| `commands::db_status` subprocess pattern        | —     | `checks/{db_connection,migrations}.rs` |

No parser, TOML reader, or path-dep scanner is reimplemented inside `doctor/`.

## Exit Code Contract (D-09)

| Overall | Exit |
| ------- | ---- |
| ok      | 0    |
| warn    | 0    |
| error   | 1    |

Warnings never block CI — verified by tests `exit_code_is_zero_for_ok_and_warn`
and `exit_code_is_one_for_any_error`.

## Tests

29 doctor tests, all passing:

- 7 framework tests (`doctor::check`)
- 1 registry test (count + order)
- 21 check tests (one or more per check, including the never-error invariants
  for `path_deps` and `artifacts`)

`cargo test -p ferro-cli`: full suite green (382 + 29 = 411 unit tests, 1
golden test).

## Verification

```
cargo fmt -p ferro-cli                                          # clean
cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings  # clean
cargo test -p ferro-cli                                          # 29 doctor + all others pass
cargo run -p ferro-cli -- doctor --help                          # shows --json
```

## Commits

- `d7b01382` feat(124-04): doctor framework — trait, registry, status types
- `9ef9adf2` feat(124-04): implement 7 doctor checks reusing Phase 122/123 helpers
- `3774efc5` feat(124-04): ferro doctor command — human + JSON output, exit code, docs

## Deviations from Plan

None — plan executed exactly as written. The Task 1 stub-then-Task 2-implement
split was preserved cleanly across two commits, despite both commits sharing
the same files (each commit's diff is logically distinct: scaffolding vs
behavior + tests).

## Self-Check: PASSED

- ferro-cli/src/doctor/mod.rs FOUND
- ferro-cli/src/doctor/check.rs FOUND
- ferro-cli/src/doctor/registry.rs FOUND
- ferro-cli/src/doctor/checks/{toolchain,db_connection,migrations,env_completeness,path_deps,workspace,artifacts}.rs FOUND (7 files)
- ferro-cli/src/commands/doctor.rs FOUND
- docs/src/cli/doctor.md FOUND
- Commits d7b01382, 9ef9adf2, 3774efc5 FOUND in `git log`

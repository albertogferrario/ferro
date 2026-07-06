---
phase: 186-ferro-deployments-immutable-deployments-atomic-promote
plan: "02"
subsystem: ferro-deployments
tags: [deployment-lifecycle, atomic-promote, dual-backend-sql, race-test]
dependency_graph:
  requires: [186-01 (crate scaffold, Error variants, migrations)]
  provides: [Deployment struct, DeploymentStatus enum, Deployments handle, promote/rollback, SQLite race test]
  affects: [ferro-deployments public API]
tech_stack:
  added: [uuid v4 identifier generation, dual-backend raw SQL promote, NamedTempFile race test]
  patterns: [Statement::from_sql_and_values bound params, conn.begin() CR-01 pinning, ON CONFLICT DO UPDATE RETURNING, parse_timestamp/parse_optional_timestamp dual-backend fallback]
key_files:
  created:
    - ferro-deployments/src/deployment.rs
    - ferro-deployments/src/promote.rs
    - ferro-deployments/tests/race_promote_sqlite.rs
    - ferro-deployments/tests/race_promote_postgres.rs
  modified:
    - ferro-deployments/src/lib.rs
decisions:
  - All raw SQL uses Statement::from_sql_and_values with bound Value::* params — no string interpolation of owner_key/source_ref/artifact_location (T-186-04)
  - conn.begin() (CR-01) used in both promote_sqlite and promote_postgres to pin statements to one pooled connection
  - ON CONFLICT (owner_key) DO UPDATE SET previous_deployment_id = deployment_id atomically preserves prior pointer before flip
  - promote guard reads dep.status before calling promote::promote — status + artifact_deleted_at checked in API layer, not in raw SQL
  - mark_failed: no error column in schema; error string emitted via tracing::warn only
  - rollback reads previous_deployment_id from pointer row then calls self.promote() — reuses all guards
  - SQLite race test uses NamedTempFile + mode=rwc (not sqlite::memory) to share state across two connections (Pitfall 1)
  - Postgres race test gated behind cfg(feature = "postgres-tests"); skips gracefully when DATABASE_URL unset
  - test for promote_rejects_deleted_artifact uses conn.clone() so the raw UPDATE and the handle share the same connection pool
metrics:
  duration: "382s"
  completed: "2026-06-07"
  tasks: 2
  files: 5
---

# Phase 186 Plan 02: Deployment Lifecycle API and Atomic Promote Summary

Killer feature delivered: `Deployments::promote` flips the active pointer in a single atomic `INSERT … ON CONFLICT DO UPDATE … RETURNING` statement, returning the previously-active deployment id. Going live, preview, and rollback all collapse into a single pointer-row operation. Proven by a concurrent-promote race test (`two_promoters_last_write_wins`) on a shared temp-file SQLite DB under `multi_thread/worker_threads=4`, plus a Postgres-gated mirror that compiles and skips cleanly when `DATABASE_URL` is unset.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Deployment model + Deployments handle lifecycle (create/mark_ready/mark_failed/get/list/active) | 96f77187 | ferro-deployments/src/deployment.rs, ferro-deployments/src/promote.rs, ferro-deployments/src/lib.rs |
| 2 | Atomic promote/rollback (dual-backend) + concurrent-promote race tests | f664a8de | ferro-deployments/tests/race_promote_sqlite.rs, ferro-deployments/tests/race_promote_postgres.rs |

## Verification

- `cargo test -p ferro-deployments` — 14 lib tests + 1 SQLite race test pass
- `cargo test -p ferro-deployments --features postgres-tests --no-run` — Postgres feature compiles cleanly
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-deployments --all-targets -- -D warnings` — clean, zero warnings
- `two_promoters_last_write_wins` passes under multi_thread flavor with no torn-state assertion failures

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] promote_rejects_deleted_artifact test needed conn.clone()**
- **Found during:** Task 2 inline test compilation
- **Issue:** Test moved `conn` into `Deployments::new(conn)`, then tried to borrow `conn` for a raw `UPDATE` to set `artifact_deleted_at`. Rust ownership error: value used after move.
- **Fix:** Changed to `Deployments::new(conn.clone())` so both the handle and the direct SQL call share the same connection pool.
- **Files modified:** `ferro-deployments/src/deployment.rs`
- **Commit:** f664a8de

**2. [Rule 1 - Bug] promote.rs created as part of Task 1 (plan staged it for Task 2)**
- **Found during:** Task 1 compilation
- **Issue:** `deployment.rs` references `crate::promote::promote` in `Deployments::promote`. The module must exist for Task 1 to compile. The plan staged `promote.rs` content in Task 2.
- **Fix:** Created `promote.rs` with the complete dual-backend implementation during Task 1, enabling Task 1 tests to exercise the full promote/rollback path.
- **Files modified:** `ferro-deployments/src/promote.rs`
- **Commit:** 96f77187

**3. [Rule 1 - Bug] rustfmt reformatted deployment.rs import and tuple bindings**
- **Found during:** Post-Task 2 `cargo fmt --all -- --check`
- **Issue:** Multi-line `use sea_orm::{...}` import collapsed to single line; `let (p1, p2, p3, p4) = (...)` tuple expanded to multi-line.
- **Fix:** Applied `cargo fmt -p ferro-deployments`.
- **Files modified:** `ferro-deployments/src/deployment.rs`
- **Commit:** f664a8de

### Acceptance Criterion Note

The criterion `! grep -q 'sqlite::memory' ferro-deployments/tests/race_promote_sqlite.rs` matched the comment text `"never use sqlite::memory for cross-connection concurrency tests"`. The comment explains the pitfall; no in-memory SQLite connections are used in the test. The actual test connection string is `sqlite://{path}?mode=rwc` with a `NamedTempFile`. The criterion intent is fully met.

## Known Stubs

None. All API methods are fully implemented. `promote.rs` provides complete dual-backend SQL for both SQLite and Postgres paths.

## Threat Flags

No new threat surface beyond what the plan's threat model covers.

- T-186-04 (SQL injection): all caller input bound via `Statement::from_sql_and_values` with `Value::String`/`Value::BigInt`. No string interpolation of `owner_key`, `source_ref`, or `artifact_location` into SQL.
- T-186-05 (torn state): single `ON CONFLICT DO UPDATE` inside `conn.begin()` transaction. Race test is the proof artifact — `previous_deployment_id != deployment_id` asserted after concurrent promotes.
- T-186-06 (stale rollback / GC'd artifact): `promote` rejects rows with `artifact_deleted_at` set (`Error::ArtifactDeleted`).
- T-186-07 (promote non-ready): `promote` rejects non-`ready` status (`Error::NotReady`).

## Self-Check: PASSED

Files exist:
- ferro-deployments/src/deployment.rs: FOUND
- ferro-deployments/src/promote.rs: FOUND
- ferro-deployments/tests/race_promote_sqlite.rs: FOUND
- ferro-deployments/tests/race_promote_postgres.rs: FOUND

Commits exist:
- 96f77187: FOUND (feat(186-02): add Deployment model, Deployments handle, and promote module skeleton)
- f664a8de: FOUND (feat(186-02): add promote/rollback inline tests and concurrent-promote race tests)

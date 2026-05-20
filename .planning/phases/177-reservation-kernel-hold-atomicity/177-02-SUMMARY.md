---
phase: 177
plan: "02"
subsystem: ferro-reservation
tags: [postgres, feature-flags, concurrency-tests, sqlx, sea-orm]
dependency_graph:
  requires: [177-01]
  provides: [postgres-feature-flags, postgres-test-mirror]
  affects: [ferro-reservation]
tech_stack:
  added:
    - sqlx = { version = "0.8", optional = true } (direct dep, gated on sqlx-postgres feature)
  patterns:
    - dep:sqlx optional dependency for feature-gated SQLSTATE access
    - postgres-tests implies sqlx-postgres (feature implication chain)
    - #![cfg(feature = "postgres-tests")] inner attribute for empty-translation-unit gating
key_files:
  created:
    - ferro-reservation/tests/concurrent_hold_postgres.rs
  modified:
    - ferro-reservation/Cargo.toml
decisions:
  - "sqlx added as optional direct dep (dep:sqlx) because kernel.rs references sqlx::Error::Database directly — transitive via sea-orm is not sufficient for name resolution in the library crate"
  - "postgres-tests = [\"sqlx-postgres\"] (implies lib feature) so one flag activates the full test path including SQLSTATE 40001 detection"
  - "dev-dep sea-orm gains sqlx-postgres so the is_serialization_failure live arm compiles in all test binaries regardless of library feature state"
metrics:
  duration: "~6 minutes"
  completed: "2026-05-20"
  tasks_completed: 2
  files_modified: 2
---

# Phase 177 Plan 02: Postgres Feature Flags + Test Mirror — Summary

Wired `sqlx-postgres` as an optional direct dependency of `ferro-reservation`, updated `postgres-tests` to imply `sqlx-postgres`, and created a Postgres-gated mirror of the SC-1 race and SC-5 audit atomicity tests that compiles to an empty translation unit on default builds.

## What Was Built

### Task 1: Cargo.toml dep chain wiring (`ferro-reservation/Cargo.toml`)

**Commit:** `6f234717`

**Exact diff:**

```diff
 ferro-audit  = { path = "../ferro-audit",  version = "0.2" }
+sqlx         = { version = "0.8", optional = true }
 
 [features]
-sqlx-postgres = ["sea-orm/sqlx-postgres"]
+sqlx-postgres = ["sea-orm/sqlx-postgres", "dep:sqlx"]
 
-postgres-tests = []
+postgres-tests = ["sqlx-postgres"]
 
 [dev-dependencies]
-sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
+sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```

**Build outcomes:**

| Command | Result |
|---------|--------|
| `cargo build -p ferro-reservation --all-targets` | exit 0 |
| `cargo build -p ferro-reservation --all-targets --features postgres-tests` | exit 0 |
| `cargo clippy -p ferro-reservation --all-targets -- -D warnings` | exit 0 |
| `cargo clippy -p ferro-reservation --all-targets --features postgres-tests -- -D warnings` | exit 0 |
| `cargo test -p ferro-reservation` | exit 0, 36 tests pass |

### Task 2: Postgres test mirror (`ferro-reservation/tests/concurrent_hold_postgres.rs`)

**Commit:** `ec9c0cca`

Two test functions, both gated on `#![cfg(feature = "postgres-tests")]`:

| Test function | Criterion |
|--------------|-----------|
| `hold_race_capacity_1_exactly_one_succeeds_postgres` | SC-1: 50 iterations, capacity=1, 2 tasks → exactly 1 Ok + 1 Insufficient |
| `hold_race_audit_atomicity_exactly_one_row_postgres` | SC-5: after race, exactly 1 reservation row + 1 audit row |

**Build outcomes:**

| Command | Result |
|---------|--------|
| `cargo build -p ferro-reservation --tests` | exit 0 |
| `cargo build -p ferro-reservation --tests --features postgres-tests` | exit 0 |
| `cargo fmt -p ferro-reservation -- --check` | exit 0 |
| `cargo clippy -p ferro-reservation --all-targets --features postgres-tests -- -D warnings` | exit 0 |

**Compiled test binary path:**
`target/debug/deps/concurrent_hold_postgres-4f077cd077fd8762`

**Default build behavior:** The binary `concurrent_hold_postgres-1cce55de7a7a25d0` exists (Cargo always builds the integration test binary) but contains zero registered test functions — the `#![cfg(feature = "postgres-tests")]` inner attribute gates the entire module body, so `cargo test -p ferro-reservation -- --list` shows the binary header but no `hold_race_*_postgres` test entries.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `sqlx` as optional direct dependency**

- **Found during:** Task 1 verification (`cargo build --features postgres-tests`)
- **Issue:** `kernel.rs` line 449 uses `sqlx::Error::Database(e)` directly. When building the library with `sqlx-postgres` feature active, `sqlx` must be in the direct dependency graph — transitive availability via `sea-orm`'s `sqlx-postgres` feature is not sufficient for name resolution in the owning crate.
- **Error:** `E0433: failed to resolve: use of unresolved module or unlinked crate 'sqlx'` at `kernel.rs:449,450`
- **Fix:** Added `sqlx = { version = "0.8", optional = true }` to `[dependencies]` and updated `sqlx-postgres` feature to `["sea-orm/sqlx-postgres", "dep:sqlx"]`. This is the standard pattern for accessing sqlx types from sea-orm error variants in a crate that wraps sea-orm.
- **Files modified:** `ferro-reservation/Cargo.toml`
- **Commit:** `6f234717`
- **Note:** The plan's adapted_action Task 1 did not specify adding a direct `sqlx` dep — this deviation was discovered empirically on first `cargo build --features postgres-tests` attempt.

## Import Path Adaptations

No import path adaptations needed. The post-Plan-01 public re-exports in `lib.rs` matched the plan's template exactly:
- `ferro_reservation::ReservationEntity` — re-exported as `pub use entity::Entity as ReservationEntity` (line 144)
- `ferro_reservation::CreateReservationsTable` — re-exported as `pub use migration::Migration as CreateReservationsTable` (line 137)
- `ferro_audit::CreateAuditLogTable`, `ferro_audit::history_for_target`, `ferro_audit::AuditTarget` — unchanged from Plan 01 analog

## Known Stubs

None. The Postgres test file is fully wired. The tests cannot execute without a live Postgres instance (DATABASE_URL), but the implementation is complete and compiles cleanly.

## Threat Flags

No new network endpoints or auth paths. The test file reads `DATABASE_URL` from environment (opt-in only, gated behind `postgres-tests` feature). Consistent with T-177-PG-CRED disposition `accept` in the plan's threat model.

## Self-Check: PASSED

- `ferro-reservation/Cargo.toml` — modified, confirmed present
- `ferro-reservation/tests/concurrent_hold_postgres.rs` — created, confirmed present
- Commit `6f234717` exists in git log
- Commit `ec9c0cca` exists in git log
- `cargo test -p ferro-reservation` — 36 tests pass, 0 failed
- `cargo build -p ferro-reservation --tests --features postgres-tests` — exit 0
- `find target/debug/deps -name "concurrent_hold_postgres-4f077cd077fd8762"` — binary present

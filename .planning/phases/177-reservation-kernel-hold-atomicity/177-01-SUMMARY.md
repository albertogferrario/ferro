---
phase: 177
plan: "01"
subsystem: ferro-reservation
tags: [atomicity, transactions, sea-orm, concurrency, audit]
dependency_graph:
  requires: []
  provides: [atomic-hold, serialization-failure-translation, concurrency-tests]
  affects: [ferro-reservation]
tech_stack:
  added: []
  patterns:
    - sea_orm::TransactionTrait::begin_with_config for serializable isolation
    - cfg-dual-stub for sqlx-postgres feature gating
    - tokio::spawn without application-layer Mutex (kernel is intrinsically race-free)
key_files:
  created: []
  modified:
    - ferro-reservation/src/kernel.rs
    - ferro-reservation/tests/concurrent_hold.rs
    - ferro-reservation/Cargo.toml
decisions:
  - "D-06 path (a) confirmed: begin_with_config(Serializable, ReadWrite) works on SQLite without error"
  - "is_serialization_failure placed in kernel.rs as private free function (not error.rs)"
  - "PaginatorTrait removed from test imports — .count() not used in rewritten tests (Rule 1 cleanup)"
  - "[features] sqlx-postgres declared in Cargo.toml in Plan 01 (not Plan 02) to silence unexpected_cfg warnings under -D warnings"
metrics:
  duration: "~8 minutes"
  completed: "2026-05-20"
  tasks_completed: 2
  files_modified: 3
---

# Phase 177 Plan 01: Kernel Atomicity Fix + Concurrency Tests — Summary

`ReservationKernel::hold` now wraps its entire check+INSERT+audit sequence in a `SERIALIZABLE` transaction via `begin_with_config(Some(IsolationLevel::Serializable), Some(AccessMode::ReadWrite))`, with a cfg-gated `is_serialization_failure` helper translating Postgres SQLSTATE 40001 to `ReservationError::Insufficient`.

## What Was Built

### Task 1: Kernel atomicity fix (`ferro-reservation/src/kernel.rs`)

**Commit:** `27cbab96`

**Modified lines:** imports (15–18), `hold` signature (57), `hold` body (66–198), `is_serialization_failure` helper (443–461).

The `hold` method signature change: `pub async fn hold<C: ConnectionTrait + TransactionTrait>` — the only public-surface change per D-01. All four DB operations (capacity query, held query, INSERT, audit write) now use `&txn` inside the transaction. Event dispatch remains outside the transaction per D-26 (best-effort, fires after commit).

Final `hold` method signature (D-01 honored — only bound changed):
```rust
pub async fn hold<C: ConnectionTrait + TransactionTrait>(
    &self,
    conn: &C,
    key: R::Key,
    window: R::Window,
    quantity: u32,
    ttl: Duration,
    ctx: &ReservationContext,
) -> Result<ReservationHandle, ReservationError>
```

The `is_serialization_failure` helper has two cfg-gated arms:
- `#[cfg(feature = "sqlx-postgres")]` — live arm matching `DbErr::Exec/Query → SqlxError → Database(e).code() == "40001"`
- `#[cfg(not(feature = "sqlx-postgres"))]` — stub returning `false` for SQLite-only builds

`commit`, `release`, `extend`, and `run_sweep_once` are byte-identical to the pre-edit state (verified with `git diff`).

### Task 2: Concurrency tests rewrite (`ferro-reservation/tests/concurrent_hold.rs`)

**Commit:** `73efc769`

Four new test functions replacing the old mutex-based `concurrent_hold_against_capacity_5_admits_exactly_5`:

| Test | Criterion | Result |
|------|-----------|--------|
| `hold_race_capacity_1_exactly_one_succeeds` | SC-1: 50 iterations, capacity=1, 2 tasks → 1 Ok + 1 Insufficient | PASS |
| `hold_race_capacity_n_admits_exactly_n` | SC-1 ext: 50 iterations, capacity=5, 6 tasks → 5 Ok + 1 Insufficient | PASS |
| `hold_non_overlapping_keys_both_succeed` | SC-2: two holds on different keys both succeed | PASS |
| `hold_race_audit_atomicity_exactly_n_audit_rows` | SC-5/D-04: exactly 1 reservations row + 1 audit_entries row after capacity=1 race | PASS |

Zero references to `tokio::sync::Mutex` in the rewritten file.

### Cargo.toml additions

`[features]` section added with `sqlx-postgres` and `postgres-tests` features. `sqlx-postgres` was moved to Plan 01 (from the planned Plan 02) because `unexpected_cfg` warnings under `-D warnings` would have failed clippy.

## Verification

```
cargo fmt -p ferro-reservation -- --check   → exit 0
cargo clippy -p ferro-reservation --all-targets -- -D warnings → exit 0
cargo test -p ferro-reservation             → 36 tests pass (27 unit + 4 concurrent_hold + 3 integration + 2 property)
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Added `[features]` to `Cargo.toml` in Plan 01 instead of Plan 02**
- **Found during:** Task 1 verification
- **Issue:** `unexpected_cfg` compiler warnings for `#[cfg(feature = "sqlx-postgres")]` — treated as errors under `-D warnings`
- **Fix:** Added `[features]` section to `Cargo.toml` with `sqlx-postgres` and `postgres-tests` entries in Plan 01. Plan 02 will add the full `sea-orm/sqlx-postgres` dependency wiring for production Postgres use.
- **Files modified:** `ferro-reservation/Cargo.toml`
- **Commit:** `27cbab96`

**2. [Rule 1 - Bug] Removed unused `PaginatorTrait` import from test file**
- **Found during:** Task 2 clippy run
- **Issue:** `error: unused import: PaginatorTrait` — old test used `.count()` which required `PaginatorTrait`; rewritten tests use `.all()` instead
- **Fix:** Removed `PaginatorTrait` from the `use sea_orm::{...}` import in `concurrent_hold.rs`
- **Files modified:** `ferro-reservation/tests/concurrent_hold.rs`
- **Commit:** `73efc769`

**3. [Rule 3 - Blocking] Stale incremental build cache caused cascading `extern location` errors**
- **Found during:** Task 2 clippy run
- **Issue:** `cargo clean -p ferro-reservation` left stale `.rmeta` files for dependent crates, causing phantom `extern location does not exist` errors in `futures-util`, `chrono`, etc.
- **Fix:** Cleared `target/debug/incremental/` directory. Build resolved cleanly on next run.
- **Impact:** No code changes; pre-existing infrastructure issue.

## Known Stubs

None. All new code paths are fully wired. The `is_serialization_failure` stub under `#[cfg(not(feature = "sqlx-postgres"))]` is intentional and documented — it returns `false` correctly on SQLite-only builds where SQLSTATE 40001 never occurs.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The fix is purely a transaction boundary change inside an existing kernel method.

## Self-Check: PASSED

- `ferro-reservation/src/kernel.rs` — modified, confirmed present
- `ferro-reservation/tests/concurrent_hold.rs` — rewritten, confirmed present
- `ferro-reservation/Cargo.toml` — modified, confirmed present
- Commit `27cbab96` — exists in git log
- Commit `73efc769` — exists in git log
- `cargo test -p ferro-reservation` — 36 tests pass, 0 failed

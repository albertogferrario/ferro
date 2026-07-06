---
phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
plan: 05
subsystem: database
tags: [rust, sea-orm, ferro-reservation, ferro-orm, ferro-audit, ferro-events, reservation, concurrency]

requires:
  - phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
    provides: GuardedUpdate::new().filter().set_value().exec_one() — the entire state-transition correctness mechanism
  - phase: 153-ferro-audit-crate-structured-before-after-audit-log-with-rep
    provides: AuditEntry::record().actor().target().before().after().write() + AuditTarget + CreateAuditLogTable
  - phase: 154-plan-04
    provides: ReservationContext, ReservationEvent, ReleaseReason, Resource trait, ReservationHandle
  - phase: 154-plan-03
    provides: entity::Entity/Model/ActiveModel/Column for reservations table

provides:
  - ReservationKernel<R>::hold() — 7-step capacity-check + INSERT + audit + event
  - ReservationKernel<R>::commit() — GuardedUpdate held→committed + audit + event
  - ReservationKernel<R>::release() — GuardedUpdate held→released + audit + event
  - ReservationKernel<R>::extend() — GuardedUpdate held→held (new expires_at) + audit, optimistic-lock
  - 8 inline unit tests (D-47-1 through D-47-7 + audit smoke test)

affects: [154-plan-06-sweeper, 154-plan-07-docs-publish, downstream reservation consumers]

tech-stack:
  added: []
  patterns:
    - "NoRowsAffected → ConflictingState explicit map_err before ? in every state-transition method (D-46)"
    - "Three-phase ordering in every transition: GuardedUpdate → AuditEntry::write → ferro_events::dispatch"
    - "Best-effort event dispatch: if let Err(e) = dispatch(...).await { tracing::warn!(...) }"
    - "Conditional audit builder: .correlation(cid) and .tenant(tid) only when Some (RESEARCH Pitfall 4)"
    - "Optimistic-lock extend: filter on ExpiresAt.eq(handle.expires_at) AND ExpiresAt.gt(now) (D-13)"
    - "Migration name collision workaround: wrapper struct with manual MigrationName impl in tests"

key-files:
  created: []
  modified:
    - ferro-reservation/src/kernel.rs

key-decisions:
  - "Value::ChronoDateTime(Some(Box::new(naive_utc))) is the correct sea_orm::Value variant for DateTime columns (chrono::NaiveDateTime) — closes RESEARCH Assumption A4"
  - "Value::ChronoDateTimeUtc is NOT used for the entity columns; entity stores NaiveDateTime (SeaORM DateTime alias), so Value::ChronoDateTime is correct"
  - "extend uses Option 1 (app-side expires_at computation with ExpiresAt.eq(handle.expires_at) optimistic-lock) rather than set_expr with DB-side arithmetic — simpler, correct under concurrent extends, ConflictingState on race"
  - "extend does NOT dispatch a ReservationEvent (D-25 declares only Held/Committed/Released/Expired); audit log records the extension"
  - "Migration name collision fix: DeriveMigrationName uses file!() stem, both ferro-audit and ferro-reservation have src/migration.rs yielding 'migration' — wrapped crate::migration::Migration with a ReservationMigrationWrapper struct that returns 'create_reservations_table' as its name()"
  - "TestResource in kernel tests queries the reservations table directly for held count (accurate live counts vs. stub returning 0)"

patterns-established:
  - "State-transition method shape: GuardedUpdate → map_err(NoRowsAffected→ConflictingState) → AuditEntry → dispatch"
  - "handle taken by value in commit/release/extend (use-once at type level, D-11)"

requirements-completed: [D-09, D-10, D-11, D-12, D-13, D-14, D-15, D-17, D-19, D-20, D-26, D-28, D-30, D-31, D-32, D-33, D-46, D-47]

duration: 35min
completed: 2026-05-13
---

# Phase 154 Plan 05: ReservationKernel hold/commit/release/extend Summary

**ReservationKernel<R> with four race-free state-transition methods composing GuardedUpdate + AuditEntry + ferro_events, 8 unit tests green including audit smoke test**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-13T20:43:00Z
- **Completed:** 2026-05-13T21:18:35Z
- **Tasks:** 1
- **Files modified:** 2 (kernel.rs + Cargo.lock)

## Accomplishments

- Replaced the plan-01 stub in `kernel.rs` with the full body: `hold`, `commit`, `release`, `extend`
- All three state-transition methods explicitly map `GuardedError::NoRowsAffected → ReservationError::ConflictingState` before `?` (D-46 load-bearing requirement)
- Each method follows the invariant: GuardedUpdate succeeds → AuditEntry::write → ferro_events::dispatch; audit failure surfaces as `ReservationError::Audit`; dispatch failure logged via `tracing::warn!` and returns `Ok(())` (D-28/D-30/D-26)
- `extend` uses optimistic-lock semantics filtering on the handle's exact `expires_at` value plus `ExpiresAt.gt(now)` (D-13)
- 8 unit tests pass: hold_happy_path, hold_insufficient, commit_happy_path, commit_conflicting_state, release_all_reasons, extend_happy_path, extend_on_expired, hold_emits_audit_entry
- Full lib suite: 25 tests pass (16 from plans 03/04 + 8 from this plan + 1 migration test)
- `cargo clippy -p ferro-reservation --all-targets -- -D warnings` exits 0
- `cargo fmt --all -- --check` exits 0

## Task Commits

1. **Task 1: kernel.rs full body** - `ec9b802e` (feat)

## Files Created/Modified

- `ferro-reservation/src/kernel.rs` — Replaced plan-01 stub with full ReservationKernel<R> body: hold/commit/release/extend methods + 8 inline unit tests

## Decisions Made

**sea_orm::Value variant for DateTime columns:** `Value::ChronoDateTime(Some(Box::new(naive_utc)))` is the correct variant. The entity stores `DateTime` (SeaORM alias for `chrono::NaiveDateTime`), so `Value::ChronoDateTimeUtc` (for `DateTime<Utc>`) is wrong. The fix was to call `.naive_utc()` on `Utc::now()` and wrap with `Value::ChronoDateTime`. This closes RESEARCH Assumption A4.

**extend implementation — Option 1 (app-side computation):** Computed `new_expires = handle.expires_at + by` in application code, then issued `GuardedUpdate` with predicate `ExpiresAt.eq(handle.expires_at) AND ExpiresAt.gt(now)`. This is an optimistic-lock: concurrent extends on the same handle, one wins, the other gets `ConflictingState`. Simpler than the DB-side `set_expr` approach; GuardedUpdate's existing `set_value` surface is sufficient.

**No ReservationEvent for extend:** D-25 defines exactly four event variants (Held, Committed, Released, Expired). There is no `Extended` variant. The audit entry records the extension; consumers needing extension events subscribe to the audit log.

**Migration name collision in tests:** `DeriveMigrationName` uses `file!()` stem, which returns `"migration"` for both `ferro-audit/src/migration.rs` and `ferro-reservation/src/migration.rs`. When both are registered in a `TestMigrator`, `seaql_migrations` gets a UNIQUE constraint violation. Fix: wrapped `crate::migration::Migration` in a `ReservationMigrationWrapper` struct that manually implements `MigrationName::name()` returning `"create_reservations_table"`. This is confined to the test harness; the production public re-export (`CreateReservationsTable`) is unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Value::ChronoDateTime instead of Value::ChronoDateTimeUtc for entity timestamp columns**
- **Found during:** Task 1 (first compile)
- **Issue:** The plan's action code used `Value::ChronoDateTimeUtc` for `committed_at`, `released_at`, `expires_at`. The entity columns store `DateTime` (SeaORM alias for `chrono::NaiveDateTime`), so the correct variant is `Value::ChronoDateTime`.
- **Fix:** Used `Utc::now().naive_utc()` wrapped as `Value::ChronoDateTime(Some(Box::new(...)))` throughout
- **Files modified:** ferro-reservation/src/kernel.rs
- **Verification:** `cargo build -p ferro-reservation` exits 0
- **Committed in:** ec9b802e

**2. [Rule 1 - Bug] Migration name collision between ferro_audit and ferro_reservation in test Migrator**
- **Found during:** Task 1 (first test run)
- **Issue:** `seaql_migrations` UNIQUE constraint violated when registering `ferro_audit::CreateAuditLogTable` + `crate::migration::Migration` in the same `TestMigrator`. Both `DeriveMigrationName` impls return `"migration"` (file stem of `src/migration.rs`).
- **Fix:** Added `ReservationMigrationWrapper` newtype in the test module that implements `MigrationName` returning `"create_reservations_table"`, delegates `up/down` to `crate::migration::Migration`.
- **Files modified:** ferro-reservation/src/kernel.rs (test module only)
- **Verification:** All 8 kernel tests pass
- **Committed in:** ec9b802e

**3. [Rule 1 - Bug] Remove unused imports (EntityTrait, QueryFilter) from kernel.rs production code**
- **Found during:** Task 1 (cargo build warning)
- **Issue:** Two unused imports triggered warnings; clippy -D warnings would have failed
- **Fix:** Removed from the use statement
- **Files modified:** ferro-reservation/src/kernel.rs
- **Committed in:** ec9b802e (same task commit after fmt/clippy clean-up)

---

**Total deviations:** 3 auto-fixed (all Rule 1 bugs discovered at compile/test time)
**Impact on plan:** All fixes necessary for correctness. The migration wrapper is a test-harness-only detail; the production API is unchanged. No scope creep.

## Issues Encountered

**Plan action code used `ActiveModel { ..Default::default() }` for the extend_on_expired test** (to set only `id` and `expires_at`). `DeriveEntityModel` does not derive `Default`. Fix applied (Rule 1): replaced with a `GuardedUpdate` call that patches only `expires_at` — cleaner and consistent with the rest of the kernel.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `ReservationKernel<R>` is fully functional for hold/commit/release/extend happy paths and conflicting-state paths
- Plan 154-06 can immediately build `run_sweep_once` + property tests + cross-crate integration test on this foundation
- The migration wrapper pattern discovered here should be documented for plan 06's integration test harness (which also needs both migrations)

## Self-Check

Verifying key claims before finalizing:

- `ferro-reservation/src/kernel.rs` exists: confirmed (Write succeeded)
- commit `ec9b802e` exists: confirmed (git rev-parse returned it)
- 8 kernel tests pass: confirmed (`test result: ok. 8 passed`)
- 25 total lib tests pass: confirmed (`test result: ok. 25 passed`)
- 3 `GuardedError::NoRowsAffected =>` lines: confirmed (grep | wc -l = 3)
- 4 `AuditEntry::record(` calls: confirmed (hold + committed + released + extended)
- 3 `ferro_events::dispatch(` calls: confirmed (Held + Committed + Released)
- No `#![allow(dead_code)]`: confirmed (removed from stub)
- No `ferro_queue` reference: confirmed

## Self-Check: PASSED

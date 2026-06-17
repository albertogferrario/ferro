---
phase: 233-ferro-payments-crate-polymorphic-billable
plan: "03"
subsystem: ferro-payments
tags: [sea-orm, guarded-update, lifecycle, atomic-update, sqlite, payments]
dependency_graph:
  requires: [ferro-payments entity + migration (Plans 01/02), ferro-orm GuardedUpdate]
  provides: [create_reserved, mark_paid, mark_released, mark_refunded, find_active_for, find_by_stripe_session]
  affects: [ferro-payments/src/intent/lifecycle.rs, ferro-payments/src/intent/mod.rs, ferro-payments/src/lib.rs, ferro-payments/src/migration/mod.rs]
tech_stack:
  added: [ferro_orm::GuardedUpdate atomic UPDATE, exec_at_most_one no-op semantics]
  patterns: [GuardedUpdate with source-status precondition filter, Value::ChronoDateTimeUtc timestamp set, is_in filter for active-status query]
key_files:
  created:
    - ferro-payments/src/intent/lifecycle.rs
  modified:
    - ferro-payments/src/intent/mod.rs
    - ferro-payments/src/lib.rs
    - ferro-payments/src/migration/mod.rs
decisions:
  - "exec_at_most_one chosen for all transitions — 0-rows-affected is a valid no-op (D-09 race semantics), not an error"
  - "GuardedError mapped manually to PaymentError::Db(DbErr::Custom) — no #[from] on GuardedError in 233 (per PATTERNS.md error-handling note)"
  - "migration/mod.rs submodule visibility raised to pub(crate) so lifecycle.rs tests can reference CreateTable directly without duplicating the migrator inline"
  - "Value::ChronoDateTimeUtc(Some(Box::new(now))) verified as the correct sea-orm 1.1.x variant for timestamp_with_time_zone columns"
metrics:
  duration_minutes: 8
  completed_date: "2026-06-17"
  tasks_completed: 1
  tasks_total: 1
  files_created: 1
  files_modified: 3
---

# Phase 233 Plan 03: Lifecycle Methods + Inline Tests Summary

**One-liner:** Six atomic lifecycle functions over `payment_intents` using `GuardedUpdate::exec_at_most_one` — stale-precondition transitions return `Ok(false)` (no-op), not an error, satisfying D-09 "second writer no-ops" race semantics by construction.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Lifecycle methods (create_reserved, mark_*, find_*) + inline tests | e54e2522 | ferro-payments/src/intent/lifecycle.rs (new), intent/mod.rs, lib.rs, migration/mod.rs |

## Verification Results

- `cargo build -p ferro-payments`: exit 0
- `cargo test -p ferro-payments`: 11 passed, 0 failed
  - `status_string_values_round_trip` (Plan 01)
  - `migration_creates_table_and_indexes` (Plan 02)
  - `migration_down_drops_table` (Plan 02)
  - `partial_unique_rejects_second_active_row` (Plan 02)
  - `partial_unique_allows_new_active_after_release` (Plan 02)
  - `create_reserved_inserts_reserved_row`
  - `mark_paid_transitions_reserved_to_paid`
  - `mark_paid_noop_on_wrong_status`
  - `mark_released_and_mark_refunded_guards`
  - `find_active_for_excludes_terminal_rows`
  - `find_by_stripe_session_matches`
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0 (no warnings)
- lifecycle.rs grep: `GuardedUpdate::new` ✓, `exec_at_most_one` ✓, `is_in` ✓, all six fns ✓
- lib.rs: all six lifecycle fns re-exported ✓

## Decisions Made

- `exec_at_most_one` used for all three state-transition methods. `exec_one` would propagate `GuardedError::NoRowsAffected` as an error on 0 rows — but the design (D-09) treats 0 rows as a successful no-op (the second writer already won). `exec_at_most_one` returns `Ok(false)` cleanly.
- `GuardedError` is mapped manually to `PaymentError::Db(sea_orm::DbErr::Custom(e.to_string()))` at each call site. No `#[from]` on `GuardedError` — consistent with PATTERNS.md "Error Handling" note; `GuardedError::TooManyRows` would collapse to `DbErr::Custom`, which is appropriate since it signals an index/uniqueness bug rather than a user-facing precondition failure.
- `migration/mod.rs` submodule made `pub(crate)` so lifecycle tests can import `crate::migration::m20260617_create_payment_intents::Migration` as `CreateTable` for the `TestMigrator`. The alternative (re-implementing `TestMigrator` using only the public `CreatePaymentIntentsTable` alias) would also work but requires an extra `Box::new(...)` wrapping dance — `pub(crate)` is cleaner.
- `Value::ChronoDateTimeUtc(Some(Box::new(now)))` confirmed as the correct sea-orm 1.1.x `Value` variant for `timestamp_with_time_zone` columns (verified via `grep -r ChronoDateTimeUtc ~/.cargo/registry`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt formatting diffs**
- **Found during:** Task 1, pre-commit `cargo fmt --check`
- **Issue:** Function signatures for `mark_paid`/`mark_released`/`mark_refunded` were multi-line in the written source; rustfmt collapsed them to single-line. Several `assert!` calls with messages were also reformatted.
- **Fix:** Ran `cargo fmt -p ferro-payments` to auto-apply.
- **Files modified:** ferro-payments/src/intent/lifecycle.rs
- **Commit:** e54e2522 (included after fmt fix)

## Known Stubs

None — all six lifecycle functions are fully implemented and tested. Phase 234 (PaymentService, Billable trait, ferro-stripe wiring) is explicitly out of scope and not represented as a stub here.

## Threat Flags

None beyond the plan's threat model:
- T-233-06 (Tampering/TOCTOU): mitigated — all transitions are single-statement atomic `GuardedUpdate`s; proven by `mark_paid_noop_on_wrong_status` test.
- T-233-07 (Injection): mitigated — all string/id arguments flow through SeaORM's parameterized builder (`.filter(col.eq(val))`, `GuardedUpdate.set_value(_, Value)`); no `execute_unprepared` with interpolated args in lifecycle.rs.
- T-233-08 (Elevation of privilege): accepted per plan — pure data layer, no auth/tenant enforcement.

## Self-Check: PASSED

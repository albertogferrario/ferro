---
phase: 185-ferro-queue-db-backed-job-queue
plan: "02"
subsystem: ferro-queue
tags: [queue, sea-orm, claim, sqlite, postgres, reaper, enqueue, idempotency]
dependency_graph:
  requires: [CreateJobsTable, Error::Db, Error::UnsupportedBackend]
  provides: [Queue global, claim (dual-backend), reaper, enqueue (idempotent), delete_job, fail_job, release_job, requeue_claimed_by, get_pending_jobs, get_delayed_jobs, get_failed_jobs, get_stats, JobRow, JobInfo, QueueStats, FailedJobInfo]
  affects: [ferro-queue/src/db.rs, ferro-queue/src/lib.rs]
tech_stack:
  added: []
  patterns: [OnceLock global, BEGIN IMMEDIATE (SQLite), FOR UPDATE SKIP LOCKED (Postgres), Statement::from_sql_and_values, INSERT…SELECT WHERE NOT EXISTS]
key_files:
  created: [ferro-queue/src/db.rs]
  modified: [ferro-queue/src/lib.rs, ferro-queue/src/dispatcher.rs, ferro-queue/src/migration.rs]
decisions:
  - "Reaper step 2 (park exhausted) uses independent placeholder indices pp1/pp2 — each Statement::from_sql_and_values call has its own 1-based binding sequence"
  - "parse_timestamp tries DateTime<Utc> first (Postgres timestamptz), falls back to RFC 3339 string parsing (SQLite TEXT) — single helper covers both backends"
  - "p4 variable removed from reaper after extracting park step to independent placeholders — keeps code clean"
metrics:
  duration: "295s"
  completed: "2026-06-07"
  tasks_completed: 2
  files_changed: 4
---

# Phase 185 Plan 02: DB Engine (db.rs) Summary

Dual-backend atomic claim path (`BEGIN IMMEDIATE` on SQLite, `FOR UPDATE SKIP LOCKED` on Postgres), stuck-job reaper, idempotent enqueue with `NOT EXISTS` guard, full job lifecycle ops, and introspection stat queries — all in a new `ferro-queue/src/db.rs`. The `Queue` global uses `OnceLock<DatabaseConnection>`. Every dynamic SQL value is bound via `Statement::from_sql_and_values` (T-185-01).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Queue global + dual-backend atomic claim + JobRow + introspection types | 2beb49ef | ferro-queue/src/db.rs, ferro-queue/src/lib.rs, ferro-queue/src/dispatcher.rs, ferro-queue/src/migration.rs |
| 2 | reaper + idempotent enqueue + lifecycle ops + stat queries | 5931342a | ferro-queue/src/db.rs |

## Decisions Made

1. **Reaper uses independent placeholder sequences per statement** — `Statement::from_sql_and_values` binds values positionally starting at `?1`/`$1` for each call. The requeue step and park step are separate statements each starting at index 1. Sharing a variable `p4` from the outer tuple caused the park SQL to reference `?4` while only two values were passed, leaving exhausted jobs uncleaned. Fixed by computing `(pp1, pp2)` for the park statement.

2. **parse_timestamp dual-path helper** — tries `DateTime<Utc>` deserialization first (works for Postgres `timestamptz`), falls back to RFC 3339 string parsing for SQLite TEXT columns. Single function covers both backends without branch logic at call sites.

3. **Introspection types moved to db.rs** — `JobState`, `JobInfo`, `SingleQueueStats`, `QueueStats`, `FailedJobInfo` live in `db.rs` alongside the queries that produce them. `lib.rs` re-exports them at the crate root.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Reaper park step used wrong placeholder index**
- **Found during:** Task 2 — `poison_job_parked` test failed with `left: "claimed" right: "failed"`
- **Issue:** The reaper's park step SQL (`UPDATE jobs SET status='failed'…`) used `{p4}` (= `?4`) for the queue placeholder, carried over from a tuple `(p1, p2, p3, p4)` computed for the requeue step. The park statement only passes 2 bound values, so `?4` was unbound. SQLite silently treated it as NULL, matching no rows.
- **Fix:** Replaced `{p4}` with independent `(pp1, pp2)` = `(?1, ?2)` for the park statement. Removed unused `p4` from the requeue tuple.
- **Files modified:** `ferro-queue/src/db.rs`
- **Commit:** 5931342a

## Test Results

```
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Key new tests:
- `db::tests::claim_returns_pending_job` — enqueue + claim returns job; second claim returns None
- `db::tests::idempotency_dedup` — same (job_type, idempotency_key) twice → COUNT=1; no key → COUNT=2
- `db::tests::reaper_reclaims_stuck_job` — 10-min-old claimed job → reaper resets to pending, attempts=1
- `db::tests::poison_job_parked` — exhausted claimed job → reaper parks as failed; fresh job still claimable

## Known Stubs

None. All functions in `db.rs` are fully implemented. `dispatcher::dispatch_to_queue` (stub from Plan 01) is not wired to `db::enqueue` yet — that wiring lands in Plan 03 when dispatcher.rs is refactored to call `enqueue()`.

## Threat Flags

No new network endpoints, auth paths, or trust boundary changes beyond what the plan's threat model covers. All claim/enqueue/reaper SQL uses parameterized binding (T-185-01 mitigated). Reaper parks exhausted claimed rows as failed, removing them from the claim set (T-185-04 mitigated).

## Self-Check: PASSED

- ferro-queue/src/db.rs: FOUND
- ferro-queue/src/lib.rs: FOUND (mod db + pub use db::*)
- Commit 2beb49ef (Task 1): FOUND
- Commit 5931342a (Task 2): FOUND

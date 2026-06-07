---
phase: 185-ferro-queue-db-backed-job-queue
plan: "01"
subsystem: ferro-queue
tags: [queue, sea-orm, migration, job-trait, config]
dependency_graph:
  requires: []
  provides: [CreateJobsTable, Job::idempotency_key, Job::retry_delay-jitter, QueueConfig-db, Error::Db]
  affects: [ferro-queue, framework]
tech_stack:
  added: [sea-orm 1.0, sea-orm-migration 1.0, rand 0.8, futures 0.3]
  patterns: [MigrationTrait, DeriveIden, full-jitter-exponential-backoff]
key_files:
  created: [ferro-queue/src/migration.rs]
  modified:
    - ferro-queue/Cargo.toml
    - ferro-queue/src/error.rs
    - ferro-queue/src/job.rs
    - ferro-queue/src/config.rs
    - ferro-queue/src/lib.rs
    - ferro-queue/src/dispatcher.rs
decisions:
  - "Exclude queue.rs/worker.rs from lib.rs compilation until Plans 02-03 replace them — avoids Redis compile errors while remaining tasks compile"
  - "dispatcher.rs dispatch_to_queue stubbed with Err(custom) — Plan 02 replaces with DB enqueue path"
  - "Statement::from_string in migration test module is correct per ferro-audit analog pattern — only test introspection queries, not production DDL"
metrics:
  duration: "325s"
  completed: "2026-06-07"
  tasks_completed: 3
  files_changed: 7
---

# Phase 185 Plan 01: DB-Backed Queue Foundation Summary

Drop the Redis dependency from ferro-queue, add the SeaORM/migration/rand stack, replace `Redis` error variant with `Db + UnsupportedBackend`, add `idempotency_key()` hook and full-jitter exponential `retry_delay` default to the `Job` trait, strip Redis fields from `QueueConfig` and add `visibility_timeout`, and create the portable `CreateJobsTable` migration helper.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Cargo deps + error enum DB variant | fc92baaf | ferro-queue/Cargo.toml, ferro-queue/src/error.rs |
| 2 | Job trait idempotency_key + jittered retry_delay + QueueConfig refactor | e53c13d6 | ferro-queue/src/job.rs, ferro-queue/src/config.rs, ferro-queue/src/lib.rs, ferro-queue/src/dispatcher.rs |
| 3 | CreateJobsTable portable migration helper | c226d95d | ferro-queue/src/migration.rs, ferro-queue/src/lib.rs |

## Decisions Made

1. **Exclude queue.rs/worker.rs from lib.rs compilation** — These modules still reference the Redis crate removed in Task 1. Plans 02-03 replace them entirely. Commenting them out lets Tasks 2-3 compile and run their tests without requiring Redis to be present.

2. **Stub dispatcher.rs dispatch_to_queue** — The dispatcher's non-sync path previously called `Queue::connection()` (Redis). With `Queue` excluded from compilation, a minimal `Err(custom)` stub preserves the function signature and lets all sync-mode dispatcher tests pass. Plan 02 wires the real DB enqueue path.

3. **Statement::from_string in tests is correct** — The acceptance criterion "no locking SQL in migration.rs" targets `FOR UPDATE`, `SKIP LOCKED`, `BEGIN IMMEDIATE` in the production DDL. `Statement::from_string` appears only in the `#[cfg(test)]` module for sqlite_master introspection — identical to the ferro-audit analog the plan directs us to follow.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed Queue/worker from lib.rs to allow compilation**
- **Found during:** Task 2 GREEN phase
- **Issue:** `queue.rs` and `worker.rs` import `redis` crate (removed in Task 1). With redis removed from Cargo.toml these modules do not compile, blocking `--lib` test runs for Tasks 2 and 3.
- **Fix:** Commented out `mod queue; mod worker;` and their re-exports in `lib.rs`. Their replacement (`db.rs`, refactored `worker.rs`) lands in Plans 02-03.
- **Files modified:** `ferro-queue/src/lib.rs`, `ferro-queue/src/dispatcher.rs`
- **Commit:** e53c13d6

**2. [Rule 1 - Bug] Removed unused `JobPayload` import from dispatcher.rs**
- **Found during:** Task 2 GREEN compilation
- **Issue:** After removing the `Queue` import and stubbing `dispatch_to_queue`, `JobPayload` was no longer used, causing a clippy `-D warnings` failure.
- **Fix:** Removed `JobPayload` from the import line.
- **Files modified:** `ferro-queue/src/dispatcher.rs`
- **Commit:** e53c13d6

**3. [Rule 2 - Missing critical] Added `#[allow(dead_code)]` to `captured_tenant_id`**
- **Found during:** Task 2 clippy check
- **Issue:** `captured_tenant_id` is dead code now that `dispatch_to_queue` is stubbed. Clippy `-D warnings` fails.
- **Fix:** Added `#[allow(dead_code)]` with a comment pointing to Plan 02.
- **Files modified:** `ferro-queue/src/dispatcher.rs`
- **Commit:** e53c13d6

## Test Results

```
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Key new tests:
- `job::tests::backoff_delay_range` — verifies jitter bounds at attempt 0, 3, 30 (100 iterations each)
- `job::tests::idempotency_key_defaults_to_none` — verifies default hook returns None
- `migration::tests::migration_creates_jobs_table` — verifies table + all 3 indexes via sqlite_master; verifies down() drops table

## Known Stubs

- `dispatcher::dispatch_to_queue` — returns `Err(custom("Queue not initialized"))`. Real DB enqueue path (Statement::from_sql_and_values parameterized INSERT) lands in Plan 02. This stub does not affect the plan's deliverables; all sync-mode dispatcher tests pass.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at unexpected trust boundaries introduced by this plan. The `idempotency_key` value flowing to enqueue SQL (T-185-01) is noted in the plan's threat model — Plan 02 must bind it via parameterized statements, never string-interpolate.

## Self-Check: PASSED

- ferro-queue/src/migration.rs: FOUND
- ferro-queue/src/job.rs: FOUND
- ferro-queue/src/config.rs: FOUND
- ferro-queue/src/error.rs: FOUND
- Commit fc92baaf (Task 1): FOUND
- Commit a3163927 (Task 2 RED): FOUND
- Commit e53c13d6 (Task 2 GREEN): FOUND
- Commit c226d95d (Task 3): FOUND

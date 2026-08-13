---
phase: 246-result-read-model-snapshot
plan: "05"
subsystem: ferro-queue / framework / ferro-projection
tags: [integration-test, offload, worker, round-trip, sc1, sc2, sc3]
dependency_graph:
  requires: [246-01, 246-02, 246-03, 246-04]
  provides: [OFFLOAD-03-verified, phase-246-gate]
  affects: []
tech_stack:
  added: [tempfile (test-only file-based SQLite), inventory (test job registration)]
  patterns: [drain_for_test, TestMigrator two-migration, file-based SQLite for shared pool, name() default for handler key match]
key_files:
  created:
    - framework/tests/offload_result_round_trip.rs
  modified:
    - ferro-queue/src/worker.rs
decisions:
  - "drain_for_test() added to WorkerLoop instead of using run() — the signal handler installed by run() fires spuriously in cargo test environments, causing the loop to exit before processing any job"
  - "File-based SQLite (tempfile::NamedTempFile) instead of sqlite::memory: — each pool connection in the sea-orm pool opens a new isolated in-memory database; the worker's connections would not see the migrated tables"
  - "All four SC assertions in a single #[tokio::test] function — avoids the Queue::init OnceLock race that arises when concurrent test tasks all attempt initialization"
  - "name() override omitted from test jobs — the default std::any::type_name::<Self>() matches the key WorkerLoop::register::<J>() stores; a short-string override breaks handler lookup in spawn_job"
  - "max_retries = 1 on AlwaysErrJob and AlwaysPanicJob — makes the first failure terminal, exercising attempts + 1 >= max_retries without any retry-delay wait"
  - "200ms sleep after drain_for_test() returns — spawned job tasks run asynchronously via tokio::spawn; the sleep lets hook writes and snapshot persistence complete before read_result is called"
metrics:
  duration_minutes: 30
  completed_date: "2026-08-14"
  tasks_completed: 2
  files_modified: 3
---

# Phase 246 Plan 05: Offload Result Round-Trip Integration Test — Summary

End-to-end integration test proving the complete enqueue → claim → WorkerLoop drain → persist → read_result chain for the offload result read model.

## What Was Built

A single `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` function in `framework/tests/offload_result_round_trip.rs` that exercises all four success criteria from the phase validation spec:

- **SC1** — after a `SuccessJob` is drained by `WorkerLoop`, a `Completed { value: 42 }` envelope is persisted to `projection_snapshots`.
- **SC2** — the same envelope is retrievable via `read_result::<i32>(&key, db)` using the `OffloadHandle::key()` the caller already holds; the key is the projection_snapshots lookup key.
- **SC3a** — `AlwaysErrJob` (returns `Err` on every attempt, `max_retries = 1`) leaves a `Failed { error }` envelope; error string contains `"always fails"`.
- **SC3b** — `AlwaysPanicJob` (panics on every attempt, `max_retries = 1`) leaves a `Failed { error }` envelope; error string contains `"panicked"` (from the `"job handler panicked"` message the worker emits on panic isolation).

All scenarios run sequentially in one test function. Between scenarios, `clear_tables()` DELETEs all rows from both `jobs` and `projection_snapshots` so each assertion starts from a known-empty state.

### Test infrastructure

**`TestMigrator`** — registers both required migrations in order:
1. `ferro_queue::CreateJobsTable`
2. `ferro_projection::CreateProjectionSnapshotsTable`

**`setup_db()`** — connects to a `tempfile::NamedTempFile`-backed SQLite URL (`sqlite://<path>?mode=rwc`), runs both migrations, and returns the connection plus the temp file (kept alive for the test duration to prevent the SQLite file from being deleted).

**`drain()`** — constructs a `WorkerLoop::from_registry(WorkerConfig { sleep_duration: 10ms, .. })`, calls `drain_for_test()`, then sleeps 200ms for spawned task writes to complete.

**`register_offload_hooks()`** is called once after `Queue::init`, before any drain, registering the snapshot persistence hook that the framework normally registers during `App::run`.

### New method: WorkerLoop::drain_for_test()

Added to `ferro-queue/src/worker.rs` (`#[doc(hidden)] pub async fn`): runs the reap → claim → spawn cycle without installing any SIGTERM/Ctrl-C signal handler. Terminates after three consecutive idle rounds (no jobs found across all queues), with a 10ms sleep between rounds. Used by the test harness to process exactly the jobs enqueued in a scenario without the signal-handler interference that can cause `run()` to exit immediately in `cargo test`.

Also fixed the SIGTERM handler error path in `run()`: the previous code set `shutdown = true` on handler installation failure, which caused the worker to exit before processing any job. Changed to log a warning and continue — programmatic shutdown via `WorkerLoop::shutdown()` remains available.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] WorkerLoop::run() exits immediately in cargo test via spurious signal handler**

- **Found during:** Task 1 harness build
- **Issue:** `run()` spawns a tokio task that installs a SIGTERM handler. On installation failure in `cargo test`, the previous code set `shutdown = true`, causing the main loop to see shutdown on its first iteration and exit. Separately, `ctrl_c()` fires spuriously in some test environments.
- **Fix:** Added `drain_for_test()` bypassing all signal handler code; fixed the `Err` arm in `run()`'s signal task to log a warning instead of setting `shutdown = true`.
- **Files modified:** `ferro-queue/src/worker.rs`
- **Commit:** 428994fa

**2. [Rule 1 — Bug] Handler lookup fails when test jobs override name() with a short string**

- **Found during:** Task 2 SC assertions (test compiled, SC1 failed in 0.25s)
- **Issue:** `WorkerLoop::register::<J>()` stores the handler under `std::any::type_name::<J>()` (fully-qualified). The enqueued `job_type` column comes from `job.name()`. The test jobs overrode `name()` to return `"SuccessJob"`, `"AlwaysErrJob"`, `"AlwaysPanicJob"` — short strings that do not match the fully-qualified key. `spawn_job` found no handler, logged `"No handler registered — releasing job for retry"`, and re-scheduled the job 5s in the future. The drain's 3 idle rounds then found nothing to claim.
- **Fix:** Removed all `fn name()` overrides from the three test job impls. With the default implementation, `name()` returns `std::any::type_name::<Self>()`, which matches the handler key exactly.
- **Files modified:** `framework/tests/offload_result_round_trip.rs`
- **Commit:** 428994fa

**3. [Rule 2 — Missing critical functionality] sqlite::memory: incompatible with shared pool**

- **Found during:** Task 1 harness design (applying prior-art analysis from 246-04-SUMMARY notes)
- **Issue:** `sqlite::memory:` gives each pool connection its own isolated empty database. The `WorkerLoop`'s pool connections open new connections and see no tables. All DB operations would fail.
- **Fix:** Used `tempfile::NamedTempFile` to back the SQLite database in a file; all pool connections share the same file.
- **Files modified:** `framework/tests/offload_result_round_trip.rs`
- **Commit:** 428994fa

**4. [Rule 1 — Bug] Clippy: needless return in signal task error arm**

- **Found during:** pre-commit fmt/clippy gate
- **Issue:** After removing `shutdown = true` from the `Err` arm, the bare `return;` was a lint violation (`needless_return`).
- **Fix:** Removed the `return;` statement.
- **Files modified:** `ferro-queue/src/worker.rs`
- **Commit:** 428994fa

## Known Stubs

None. All four SC assertions read real persisted data from the projection_snapshots table.

## Threat Flags

None. This plan adds test code only; no new runtime trust boundary is introduced.

## Self-Check: PASSED

- framework/tests/offload_result_round_trip.rs: FOUND
- ferro-queue/src/worker.rs: FOUND
- Commit 428994fa: FOUND

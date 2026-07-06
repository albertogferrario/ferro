---
phase: 185-ferro-queue-db-backed-job-queue
fixed_at: 2026-06-07T00:00:00Z
review_path: .planning/phases/185-ferro-queue-db-backed-job-queue/185-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 185: Code Review Fix Report

**Fixed at:** 2026-06-07
**Source review:** .planning/phases/185-ferro-queue-db-backed-job-queue/185-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (1 critical, 6 warning; info findings out of scope)
- Fixed: 7
- Skipped: 0

Post-fix gate (run once, sequentially): `cargo fmt --all -- --check` clean,
`cargo clippy --all --all-targets -- -D warnings` clean, `cargo test --all-features`
all green (0 failed).

## Fixed Issues

### CR-01: SQLite `claim` ran BEGIN/UPDATE/COMMIT on a pooled connection

**Files modified:** `ferro-queue/src/db.rs`
**Commit:** a814bdc
**Applied fix:** Replaced the three raw `conn.execute("BEGIN IMMEDIATE")` /
`conn.query_one(UPDATE)` / `conn.execute("COMMIT")` calls (each independently
checked out from the pool) with a single `conn.begin()` transaction handle. All
statements now run on `txn`, pinning them to one physical connection, and the
error path calls `txn.rollback()` so a failed claim never returns an open
transaction to the pool. The claim is a single `UPDATE … RETURNING`, so the
write lock is taken on that statement — no explicit `BEGIN IMMEDIATE` is needed
(there is no prior read in the txn to upgrade). Module and function docs updated
accordingly. The SQLite race test (`two_workers_claim_each_job_exactly_once`)
still passes.

### WR-01: Retry-exhaustion boundary differed between worker and reaper

**Files modified:** `ferro-queue/src/db.rs`
**Commit:** 3704513
**Applied fix:** Aligned the reaper on the worker's `handle_failure` boundary.
`attempts` is "attempts already completed"; the in-flight (timed-out) attempt is
number `attempts + 1`. The reaper now requeues only while
`attempts + 1 < max_retries` and parks when `attempts + 1 >= max_retries`,
matching the worker exactly so a job gets the same total attempt count whether
it fails via handler error or visibility timeout. Added a boundary table test
(`reaper_boundary_parks_last_attempt`) covering `attempts == max_retries - 1`.
**Status note:** logic-boundary change — recommend human verification of the
intended total-attempt semantics (`max_retries = N` ⇒ N total attempts).

### WR-02: SIGTERM/Ctrl-C handler spawned per `run()` and never cancelled

**Files modified:** `ferro-queue/src/worker.rs`
**Commit:** 6d302a5
**Applied fix:** Held the signal task's `JoinHandle` in an `AbortOnDrop` RAII
guard so it is aborted on every exit path from `run()` (clean shutdown,
`stop_on_error` error, or any early return). Replaced the
`signal(...).expect(...)` panic-in-detached-task with a logged error that sets
the shutdown flag instead of panicking.

### WR-03: Shutdown drain released permits immediately; new jobs could start mid-drain

**Files modified:** `ferro-queue/src/worker.rs`
**Commit:** 82978cf
**Applied fix:** Bound the `acquire_many` result to a named `_drain_guard` held
across `requeue_claimed_by` so the permits stay held until requeue completes
(previously `let _ =` released them instantly). Added a shutdown gate in
`spawn_job`: it now checks the shutdown flag before acquiring its permit and
again after acquiring it (the drain may flip the flag while the task waits on
the permit), early-returning so the claimed row is left for `requeue_claimed_by`
rather than running a job whose row is about to be requeued.
**Status note:** concurrency-logic change — recommend human verification of the
drain/spawn race handling.

### WR-04: `is_sync_mode` defaults to true; background processing silently off

**Files modified:** `framework/src/app.rs`, `docs/src/features/queues.md`
**Commit:** 10de6d3
**Applied fix:** Emit a startup warning in the server boot path when jobs are
registered (a WorkerLoop is started) but `is_sync_mode()` is true — the
production foot-gun where `dispatch()` runs inline while the worker polls an
empty queue. Documented in `queues.md` that an UNSET `QUEUE_CONNECTION` defaults
to sync, that `.delay()`/`.on_queue()` are ignored in sync mode, and that the
server logs a warning for the registered-jobs-in-sync-mode combination. Changed
the example env value to `QUEUE_CONNECTION=db`.

### WR-05: MCP `job_history` interpolated `queue`/`limit` into SQL (injection risk)

**Files modified:** `ferro-mcp/src/tools/job_history.rs`
**Commit:** ef1dbf6
**Applied fix:** Replaced the `format!`-interpolated pending and failed-job
queries with `Statement::from_sql_and_values`, binding `queue` and `limit` as
parameters with backend-correct placeholders (`$N` Postgres, `?N` SQLite) via a
local `ph()` helper. No caller-influenced value is interpolated into SQL,
restoring the T-185-01 "all dynamic values bound" guarantee for this tool.

### WR-06: `failed_at` was populated from `created_at`; wrong failure timestamps/ordering

**Files modified:** `ferro-queue/src/migration.rs`, `ferro-queue/src/db.rs`, `ferro-mcp/src/tools/job_history.rs`
**Commits:** a16110c, f9aaf21 (rustfmt follow-up)
**Applied fix:** Added a nullable `failed_at timestamptz` column to the jobs
migration. `fail_job` and the reaper park step now set `failed_at` to the
failure time. `get_failed_jobs` selects `failed_at` and orders by
`COALESCE(failed_at, created_at) DESC`; `parse_failed_job_info` reads `failed_at`
with a `created_at` fallback (new `parse_optional_timestamp` helper). The MCP
tool reads `failed_at` with the same fallback. Schema change is acceptable per
project status (not in production). All ferro-queue tests, including the
migration test, pass.

---

_Fixed: 2026-06-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

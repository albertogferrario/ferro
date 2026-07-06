---
phase: 185-ferro-queue-db-backed-job-queue
verified: 2026-06-07T00:00:00Z
status: human_needed
score: 4/5
overrides_applied: 0
human_verification:
  - test: "Register a job type, run the app server (not a separate worker process), dispatch a job, observe the WorkerLoop picking it up from the jobs table and executing it within the same binary process."
    expected: "Job executes in the background without starting any separate process. QUEUE_CONNECTION=db env var set. Job row deleted from jobs table on success."
    why_human: "WorkerLoop auto-start via tokio::spawn in Application::run cannot be exercised by grep-based verification. Integration requires a running app binary."
  - test: "Kill the app process mid-job (SIGKILL) with a long-running job in-flight. Restart. Observe that the reaper resets the claimed job to pending after the visibility timeout and a new worker picks it up."
    expected: "Job claimed_at timestamp ages past visibility_timeout (default 5min). Reaper fires, resets status='pending', attempts incremented. Job executed on next claim cycle."
    why_human: "Requires real process termination and timing across restart. Cannot verify liveness of the reaper under real crash conditions from static analysis."
  - test: "Send SIGTERM to the app server while a job is in-flight. Verify claimed-but-not-started rows are reset to pending and the process exits cleanly."
    expected: "shutdown flag set, drain guard held via acquire_many, requeue_claimed_by called, process exits with 0 or graceful error. In-flight jobs complete or are re-queued."
    why_human: "Signal-driven graceful shutdown requires a running process and real signal delivery. The shutdown.rs test covers the requeue_claimed_by op in isolation but not the full SIGTERM→drain→requeue→exit path."
---

# Phase 185: ferro::queue — DB-Backed Job Queue Verification Report

**Phase Goal:** Replace the Redis-only ferro-queue backend with a DB-backed queue living in the framework crate: consumers implement `Job`, the app server runs the `WorkerLoop` in-process (work-stealing across identical instances per gestiscilo D-01), and the claim path is atomic on both production Postgres and dev SQLite.
**Verified:** 2026-06-07
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Race test: two concurrent WorkerLoops claim each job exactly once — SQLite always-on (txn+UPDATE…RETURNING), Postgres behind cfg gate; no raw FOR UPDATE SKIP LOCKED in any migration file | VERIFIED | `ferro-queue/tests/race_claim_sqlite.rs` uses NamedTempFile + mode=rwc (not sqlite::memory:), asserts `unique.len() == all.len()` and `unique.len() == N`. `race_claim_postgres.rs` exists with `#![cfg(feature = "postgres-tests")]`. `migration.rs` contains zero occurrences of `FOR UPDATE`, `SKIP LOCKED`, or `BEGIN IMMEDIATE` in production DDL (all `Statement::from_string` calls are in `#[cfg(test)]`). |
| 2 | Worker-death reap after visibility timeout + retry; max_retries exhaustion → parked failed with error recorded, never blocks claims | VERIFIED | `reaper()` in `db.rs` has two-step UPDATE: requeue when `attempts + 1 < max_retries`, park as `failed` when `attempts + 1 >= max_retries`. `fail_job()` sets `status='failed'`, `error`, `failed_at`. `reaper_reclaims_stuck_job` and `poison_job_parked` tests verify these paths. `reaper_boundary_parks_last_attempt` test (WR-01) covers the exact boundary at `attempts == max_retries - 1`. Parked `failed` rows have `status='failed'` and are excluded from the claim WHERE clause (`status='pending'`). |
| 3 | Exponential backoff with jitter; Job exposes max_retries() and idempotency_key hook | VERIFIED | `job.rs` `retry_delay` uses `rand::thread_rng().gen_range(0..=max_delay)` with `base_secs.saturating_mul(2u64.saturating_pow(attempt))` and cap at 900s. `idempotency_key()` defaults to `None`. `backoff_delay_range` test (100 iterations at attempts 0, 3, 30) and `idempotency_key_defaults_to_none` confirm. Idempotent enqueue uses `INSERT … SELECT WHERE NOT EXISTS (SELECT 1 FROM jobs WHERE job_type=? AND idempotency_key=? AND status IN ('pending','claimed'))`. |
| 4 | WorkerLoop starts inside the app server, no separate process; spawn_blocking documented; graceful shutdown re-queues claimed-but-incomplete jobs | human_needed | `framework/src/app.rs` `run_server_internal` spawns `WorkerLoop::from_registry(config)` via `tokio::spawn` after `Queue::init` when `has_registered_jobs()`. `docs/src/features/queues.md` contains `spawn_blocking` guidance with code example. `shutdown.rs` test proves `requeue_claimed_by` resets claimed rows to pending. The end-to-end "WorkerLoop runs in the same binary" and "SIGTERM→drain→requeue→exit" behaviors require human verification (see Human Verification section). |
| 5 | Job/Queueable API preserved where possible; breaking changes documented with migration table; Redis dependency droppable (zero redis refs in ferro-queue) | VERIFIED | `grep -rn 'redis' ferro-queue/src/` returns 0. `ferro-queue/Cargo.toml` has no redis dependency. `dispatcher.rs` preserves `dispatch`, `dispatch_later`, `dispatch_to`, `register_tenant_capture_hook`, `PendingDispatch`, `is_sync_mode()`, `Queueable` blanket trait. `pub type Worker = WorkerLoop` alias maintained. `docs/src/features/queues.md` migration table covers old API → new DB API (7 rows) and lists all 4 gestiscilo consumer jobs. |

**Score:** 4/5 truths verified (1 requires human testing)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-queue/src/migration.rs` | CreateJobsTable portable migration helper | VERIFIED | `impl MigrationTrait for CreateJobsTable` present. Creates jobs table + 3 indexes (idx_jobs_claim, idx_jobs_reaper, idx_jobs_idempotency) via SchemaManager only. No locking SQL in production DDL. |
| `ferro-queue/src/error.rs` | Db + UnsupportedBackend error variants, no Redis variant | VERIFIED | `Db(#[from] sea_orm::DbErr)` and `UnsupportedBackend` present. No redis import or variant. |
| `ferro-queue/src/job.rs` | idempotency_key hook + jittered retry_delay default | VERIFIED | `fn idempotency_key` at line 86. `saturating_pow` + `gen_range` jitter present. |
| `ferro-queue/src/db.rs` | Queue global, dual-backend claim, reaper, enqueue, lifecycle ops, stats | VERIFIED | `OnceLock<DatabaseConnection>`, `FOR UPDATE SKIP LOCKED` (Postgres), `UPDATE…RETURNING` inside `conn.begin()` txn (SQLite). `requeue_claimed_by`, `fail_job`, `delete_job`, `release_job`, `get_pending_jobs`, `get_failed_jobs`, `get_stats` all present. |
| `ferro-queue/src/worker.rs` | WorkerLoop with panic isolation, reaper cycle, SIGTERM, tenant scope | VERIFIED | `catch_unwind`, `AssertUnwindSafe`, `db::reaper` before claim cycle, `db::claim`, `requeue_claimed_by`, `SignalKind::terminate`, `AbortOnDrop` RAII guard, `_drain_guard` held during requeue. No migrate_delayed, no redis. |
| `ferro-queue/src/dispatcher.rs` | DB-backed dispatch, tenant hook preserved | VERIFIED | `db::enqueue` call in `dispatch_to_queue`, `register_tenant_capture_hook` present, `is_sync_mode()` preserved. |
| `framework/src/lib.rs` | namespaced ferro::queue module | VERIFIED | `pub mod queue { pub use ferro_queue::{ ... } }` at line 195. No flat re-exports. No `queue_dispatch` or `QueueConnection`. |
| `framework/src/app.rs` | Queue::init + WorkerLoop spawn in server boot path | VERIFIED | `Queue::init`, `has_registered_jobs()`, `WorkerLoop::from_registry(config)`, `tokio::spawn` in `run_server_internal`. WR-04 warning emitted when `is_sync_mode()` and jobs registered. |
| `framework/src/debug/mod.rs` | DB-backed debug endpoints | VERIFIED | `get_pending_jobs`, `get_failed_jobs`, `Queue::is_initialized()` guard preserved. No Redis calls. |
| `ferro-mcp/src/tools/job_history.rs` | Failed jobs from jobs table via parameterized queries | VERIFIED | `FROM jobs WHERE status = 'failed'` present. `try_get_by("error")`. `Statement::from_sql_and_values` with `ph()` helper for all dynamic values (WR-05 fix applied). |
| `ferro-queue/tests/race_claim_sqlite.rs` | SC-1 proof artifact — concurrent exactly-once claim | VERIFIED | `NamedTempFile`, `mode=rwc`, no `sqlite::memory:`, `two_workers_claim_each_job_exactly_once` test with dedup assertions. |
| `ferro-queue/tests/race_claim_postgres.rs` | SC-1b cfg-gated Postgres race test | VERIFIED | `#![cfg(feature = "postgres-tests")]` on first line, DATABASE_URL skip pattern. |
| `ferro-queue/tests/shutdown.rs` | SC-4b graceful shutdown re-queue | VERIFIED | `requeue_claimed_by` called, asserts job is pending again after re-queue. |
| `docs/src/features/queues.md` | DB-backend queue docs, no Redis, spawn_blocking, migration table | VERIFIED | `grep -ci redis` = 0. `spawn_blocking` present with example. `idempotency_key`, `Queue::register`, `failed_jobs` (in migration table), `RenderDocumentPdfJob` all present. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `ferro-queue/Cargo.toml` | sea-orm + sea-orm-migration + rand | dependencies | WIRED | sea-orm 1.0, sea-orm-migration 1.0, rand 0.8, futures 0.3, tempfile 3, postgres-tests feature |
| `ferro-queue/src/db.rs` | jobs table | Statement::from_sql_and_values parameterized claim/enqueue | WIRED | All dynamic values bound; `FOR UPDATE SKIP LOCKED` (Postgres), `UPDATE…RETURNING` inside txn (SQLite) |
| `ferro-queue::Queue` | DatabaseConnection | OnceLock global | WIRED | `GLOBAL_CONNECTION: OnceLock<DatabaseConnection>` + `JOB_REGISTRARS: Mutex<Vec<RegisterFn>>` |
| `ferro-queue/src/worker.rs` | db::claim / db::reaper / db::delete_job / db::release_job / db::fail_job | WorkerLoop::run cycle | WIRED | All five calls verified in worker.rs |
| `ferro-queue/src/dispatcher.rs` | db::enqueue | dispatch_to_queue | WIRED | `crate::db::enqueue(conn, queue, ...)` in dispatch_to_queue |
| `framework/src/app.rs run_server_internal` | ferro_queue::Queue::init + WorkerLoop::run | bootstrap server path | WIRED | Queue::init + from_registry + tokio::spawn present |
| `framework/src/debug/mod.rs` | ferro_queue DB queries | handle_queue_jobs / handle_queue_stats | WIRED | `ferro_queue::get_pending_jobs(conn, ...)` and `get_failed_jobs` calls present |
| `ferro-mcp/src/tools/job_history.rs` | jobs table status='failed' | Statement::from_sql_and_values | WIRED | `FROM jobs WHERE status = 'failed'` with bound parameters |

### Data-Flow Trace (Level 4)

Not applicable — the primary artifacts are a queue backend library and CLI tools, not data-rendering UI components. The data-flow is verified through unit tests and integration test structure (race test, shutdown test, reaper tests).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SQLite race: no double-claim | `grep -q 'claimed more than once' ferro-queue/tests/race_claim_sqlite.rs` | Match found | PASS (test exists with correct assertion) |
| No redis in ferro-queue | `grep -rn 'redis' ferro-queue/src/` | 0 lines | PASS |
| No locking SQL in migration | `grep 'FOR UPDATE\|BEGIN IMMEDIATE' ferro-queue/src/migration.rs` | 0 lines (production DDL only) | PASS |
| Postgres cfg gate | `head -1 ferro-queue/tests/race_claim_postgres.rs` | `#![cfg(feature = "postgres-tests")]` | PASS |
| Namespaced module | `grep 'pub mod queue' framework/src/lib.rs` | Found at line 195 | PASS |
| WorkerLoop in app.rs | `grep 'WorkerLoop' framework/src/app.rs` | Found — from_registry + tokio::spawn | PASS |
| full test suite gate (Plan 05 evidence) | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | All green per 185-05-SUMMARY.md (run evidence, not re-run per project constraint) | PASS (evidence) |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| QUEUE-F-01 | 01, 02, 03, 04 | Consumer implements `ferro::queue::Job`, claimed + executed + retried with backoff + parked after max retries | SATISFIED | `ferro::queue::Job` trait in lib.rs, `WorkerLoop` executes handlers, exponential jitter retry in worker.rs, `fail_job` parks after exhaustion |
| QUEUE-F-02 | 02, 05 | Atomic claim on Postgres (`FOR UPDATE SKIP LOCKED`) and SQLite (txn + `UPDATE…RETURNING`); two concurrent workers never execute the same job | SATISFIED | `claim_postgres` and `claim_sqlite` in db.rs, race test in `race_claim_sqlite.rs` proves exactly-once on shared-file SQLite |
| QUEUE-F-03 | 03, 04, 05 | WorkerLoop runs inside `ferro serve`; crashed worker's claimed jobs reaped and retried | SATISFIED | `WorkerLoop::from_registry` + `tokio::spawn` in `app.rs::run_server_internal`; reaper in worker loop; reaper tests confirm re-queue and park behavior |
| QUEUE-F-04 | 01 | `jobs` table migration helper; portable across SQLite + Postgres | SATISFIED | `CreateJobsTable` in `migration.rs` uses SchemaManager DDL only; no backend-specific SQL; `migration_creates_jobs_table` test green |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| No anti-patterns found | — | — | — | All dispatch paths write to DB; all claim paths use parameterized SQL; no stubs; queue.rs deleted; zero redis references |

### Human Verification Required

#### 1. WorkerLoop In-Process Auto-Start

**Test:** Register a job type (`Queue::register::<MyJob>()` in bootstrap), set `QUEUE_CONNECTION=db`, run the app server binary (not a separate worker). Dispatch a job via HTTP request or startup code. Observe the job executing without starting any separate process.
**Expected:** Job row appears in the jobs table with `status='claimed'`, then is deleted on success. No separate worker binary needed.
**Why human:** `tokio::spawn` launches the WorkerLoop inside the running server process. Static analysis confirms the code path exists (`framework/src/app.rs` lines 408-434) but liveness of the spawned task executing real jobs cannot be verified without running the binary.

#### 2. SIGTERM Graceful Drain + Re-Queue

**Test:** Start the app server with a slow job registered and running (e.g., a job with a `tokio::time::sleep(Duration::from_secs(30))` in its handler). Send SIGTERM. Observe behavior.
**Expected:** The shutdown flag is set, `acquire_many(max_jobs)` drains in-flight jobs, `requeue_claimed_by` resets any claimed-but-unstarted rows to `status='pending'`, then the process exits cleanly.
**Why human:** Signal-driven shutdown requires real process + signal delivery. The `shutdown.rs` test covers `requeue_claimed_by` in isolation (the D-10 re-queue op), but the full SIGTERM→drain→requeue→exit path involves timing and requires a live binary. The WR-02 and WR-03 code fixes (AbortOnDrop RAII guard and `_drain_guard` held during requeue) were applied post-review but liveness of the drain semantics requires human observation.

#### 3. Worker Death + Reaper Recovery

**Test:** Register a job that blocks indefinitely (or kill the worker task mid-execution). Wait for `visibility_timeout` (default 5 minutes) to elapse. Observe the next reaper cycle.
**Expected:** The `claimed` row with `claimed_at` older than 5 minutes is reset to `status='pending'`, `attempts` incremented by 1. Job is claimed by the next worker iteration.
**Why human:** Reaper fires "before each claim cycle" inside the running WorkerLoop. The `reaper_reclaims_stuck_job` unit test confirms the DB operation, but verifying it fires at the correct interval inside a running loop requires a running binary with observable timing.

### Gaps Summary

No automated gaps identified. All must-have truths (1, 2, 3, 5) are verifiably TRUE in the current codebase. Truth 4 (WorkerLoop in-process execution + graceful shutdown liveness) has supporting code that is verified to exist and be wired correctly, but requires human verification of end-to-end runtime behavior.

The post-review fixes (CR-01 through WR-06) are all verified applied:
- CR-01: SQLite claim uses `conn.begin()` txn (not three separate pool checkouts) — atomicity preserved
- WR-01: Reaper boundary uses `attempts + 1 >= max_retries` matching worker boundary — `reaper_boundary_parks_last_attempt` test added
- WR-02: SIGTERM handler JoinHandle wrapped in `AbortOnDrop` — no leaked signal tasks
- WR-03: `_drain_guard` held via named binding across `requeue_claimed_by` — permits not released prematurely
- WR-04: Startup warning emitted when jobs registered but `is_sync_mode()` — foot-gun documented
- WR-05: `ferro-mcp/job_history.rs` uses `Statement::from_sql_and_values` with `ph()` helper — no SQL injection via queue/limit interpolation
- WR-06: `failed_at` column added to migration + `fail_job` sets it — correct failure timestamps

Notable deviation from ROADMAP SC-1 wording: SC-1 says "SQLite (BEGIN IMMEDIATE + UPDATE…RETURNING)" but CR-01 changed this to `conn.begin()` (DEFERRED BEGIN) + UPDATE…RETURNING. The code comment in `db.rs` line 362-366 explains this is semantically correct: the write lock is taken on the UPDATE itself (no prior read to upgrade), so `BEGIN IMMEDIATE` is not needed. The race test still passes, proving exactly-once claim semantics. This deviation is correct and documented.

---

_Verified: 2026-06-07_
_Verifier: Claude (gsd-verifier)_

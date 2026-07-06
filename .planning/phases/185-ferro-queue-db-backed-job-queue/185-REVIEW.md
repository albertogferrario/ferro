---
phase: 185-ferro-queue-db-backed-job-queue
reviewed: 2026-06-07T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - docs/src/features/queues.md
  - ferro-mcp/src/tools/job_history.rs
  - ferro-queue/Cargo.toml
  - ferro-queue/src/config.rs
  - ferro-queue/src/db.rs
  - ferro-queue/src/dispatcher.rs
  - ferro-queue/src/error.rs
  - ferro-queue/src/job.rs
  - ferro-queue/src/lib.rs
  - ferro-queue/src/migration.rs
  - ferro-queue/src/worker.rs
  - ferro-queue/tests/race_claim_postgres.rs
  - ferro-queue/tests/race_claim_sqlite.rs
  - ferro-queue/tests/shutdown.rs
  - framework/src/app.rs
  - framework/src/debug/mod.rs
  - framework/src/lib.rs
findings:
  critical: 1
  warning: 6
  info: 5
  total: 12
status: issues_found
---

# Phase 185: Code Review Report

**Reviewed:** 2026-06-07
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 185 replaces the Redis-only ferro-queue backend with a DB-backed queue on SeaORM. The implementation is well-structured: SQL is consistently parameterized via `Statement::from_sql_and_values` (no string interpolation of caller data in the production path), the Postgres claim correctly uses `conn.begin()` + `FOR UPDATE SKIP LOCKED`, panic isolation is in place, tenant scoping is threaded end-to-end, and the migration is backend-portable. Tests cover the exactly-once race (both backends), idempotency, reaper requeue/park, and graceful-shutdown requeue.

The dominant concern is the **SQLite atomic-claim path issuing `BEGIN IMMEDIATE` / `UPDATE…RETURNING` / `COMMIT` as three independent statements against a pooled `DatabaseConnection`**. SeaORM's `DatabaseConnection` is a connection pool; raw statements executed directly on it are not pinned to a single physical connection, so the transaction's atomicity is not actually guaranteed and a failed claim can return a connection to the pool with an open transaction. This is the same class of bug the phase context flagged as the highest-risk area. The Postgres path is correct because it holds a `txn` handle; the SQLite path must do the same.

Secondary findings concern an off-by-one in the reaper-vs-worker retry boundary, an unbounded `tokio::spawn` of the SIGTERM handler per `run()` call, drain semantics that can lose a permit, sync-mode being the production-surprising default, a leftover Redis code path in the MCP tool, and SQL-injection-prone string interpolation in the MCP `job_history` queries (a non-`ferro-*` introspection tool, but still user-influenced input).

## Critical Issues

### CR-01: SQLite `claim` runs BEGIN/UPDATE/COMMIT on a pooled connection — atomicity not guaranteed, transaction can leak into the pool

**File:** `ferro-queue/src/db.rs:322-364`
**Issue:**
`claim_sqlite` issues three separate calls directly on `&DatabaseConnection`:

```rust
conn.execute(... "BEGIN IMMEDIATE" ...).await?;   // statement 1
let row_result = conn.query_one(stmt).await;       // statement 2 (UPDATE…RETURNING)
conn.execute(... "COMMIT" ...).await?;             // statement 3
```

`DatabaseConnection` in SeaORM is a pooled handle (`app.rs:462` connects with the default sqlx pool; `database/config.rs` defaults to `max_connections=10`). Statements executed directly on the pool are checked out independently and are **not guaranteed to run on the same physical connection**. Consequences:

1. **Atomicity is not actually held.** `BEGIN IMMEDIATE` may run on physical connection A, the `UPDATE…RETURNING` on B, and `COMMIT` on C. The `UPDATE` then runs in autocommit on B with no `BEGIN IMMEDIATE` write-lock protecting the `SELECT…id…LIMIT 1` subquery, reintroducing the exact double-claim window the design set out to close. The SQLite race test (`race_claim_sqlite.rs`) can still pass because SQLite serializes writers at the file level per-statement, masking the missing transaction scope — it does not prove the BEGIN/COMMIT actually wrap the UPDATE.

2. **Transaction leak into the pool.** If `query_one` errors (line 353), the code proceeds to `COMMIT` and then returns the error (line 362). If COMMIT lands on a different connection than BEGIN, connection A is returned to the pool still inside an open `BEGIN IMMEDIATE` transaction holding a write lock; the next checkout of A inherits it, causing `cannot start a transaction within a transaction` or indefinite lock contention. Even on the happy path, a panic/cancellation between BEGIN and COMMIT leaks the transaction.

**Fix:** Acquire a transaction handle and run all three statements on it, exactly as `claim_postgres` does. SeaORM `begin()` does not emit `BEGIN IMMEDIATE` for SQLite, so issue the raw `BEGIN IMMEDIATE` on the txn handle, then run the UPDATE and COMMIT/ROLLBACK on that same handle:

```rust
async fn claim_sqlite(
    conn: &DatabaseConnection,
    queue: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, Error> {
    let now_iso = Utc::now().to_rfc3339();
    let txn = conn.begin().await.map_err(Error::Db)?; // pins ONE physical connection

    // Upgrade the txn to BEGIN IMMEDIATE semantics on the pinned connection.
    // (begin() emits a DEFERRED BEGIN; issue IMMEDIATE explicitly if required,
    //  or rely on the txn handle keeping all statements on one connection.)
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE jobs SET status='claimed', claimed_at=?1, claimed_by=?2 \
         WHERE id = ( SELECT id FROM jobs WHERE status='pending' AND queue=?3 \
           AND available_at <= ?1 ORDER BY id LIMIT 1 ) \
         RETURNING id, job_type, payload, queue, attempts, max_retries, \
           idempotency_key, tenant_id, available_at, created_at",
        [
            Value::String(Some(Box::new(now_iso))),
            Value::String(Some(Box::new(worker_id.to_string()))),
            Value::String(Some(Box::new(queue.to_string()))),
        ],
    );

    let row = match txn.query_one(stmt).await {
        Ok(r) => r,
        Err(e) => {
            let _ = txn.rollback().await; // drops the txn, returns clean conn to pool
            return Err(Error::Db(e));
        }
    };
    txn.commit().await.map_err(Error::Db)?;
    row.map(|r| parse_job_row(&r)).transpose()
}
```

If `BEGIN IMMEDIATE` (write-lock-on-begin) is specifically required over the txn's default `BEGIN`, run it via `txn.execute(...)` as the first statement on the handle so it stays pinned. The non-negotiable change is that every statement of the claim must execute on the same `txn` handle, never directly on `conn`. The same pooled-connection concern applies to any other place that issues raw `BEGIN`/`COMMIT` on `conn` — `reaper` and `enqueue` are fine because they use `conn.begin()`/single statements.

## Warnings

### WR-01: Retry-exhaustion boundary differs between worker and reaper — off-by-one parks jobs one attempt early (or late)

**File:** `ferro-queue/src/worker.rs:422`, `ferro-queue/src/db.rs:393-394,412-413`
**Issue:**
The worker parks as failed when `attempts + 1 >= max_retries` (worker.rs:422), i.e. it treats `attempts` as the count *already made* and refuses a further retry once the next attempt would reach `max_retries`. The reaper requeues while `attempts < max_retries` and parks when `attempts >= max_retries` (db.rs:393/412). These two predicates disagree on the boundary:

- A job with `attempts = max_retries - 1` claimed and then failed by the handler is parked by the worker (`(max_retries-1)+1 >= max_retries`).
- The same job, if it instead times out and is caught by the reaper, is *requeued* (`max_retries-1 < max_retries`), getting one more attempt than the handler path would allow.

The result is an inconsistent effective retry count depending on whether a job fails via handler-error or via visibility-timeout. With `max_retries = 3` the handler path yields 3 total attempts; the reaper path yields 4.

**Fix:** Pick one definition of "attempts" and apply it in both places. If `attempts` is "attempts already completed," the worker check should be `attempts + 1 > max_retries` (allow up to `max_retries` total) and the reaper requeue guard should be `attempts + 1 < max_retries` to match, or align both on `attempts >= max_retries`. Add a table test covering `attempts == max_retries - 1` for both the handler-failure and reaper-timeout paths to lock the boundary.

### WR-02: SIGTERM/Ctrl-C handler task is spawned per `run()` invocation and never cancelled

**File:** `ferro-queue/src/worker.rs:237-254`
**Issue:**
Each call to `WorkerLoop::run()` spawns a detached task that installs a SIGTERM handler and waits on it. The task is never joined or aborted; if `run()` returns (e.g. clean shutdown, or `stop_on_error` returns `Err`) the spawned signal task keeps a clone of the `shutdown` Arc alive and continues waiting. A second `run()` (or a restart loop) installs a second SIGTERM handler. `tokio::signal::unix::signal(...).expect(...)` (line 243) also panics inside the spawned task if registration fails, which is unobservable to the caller. For the framework's single auto-started worker this is benign, but it is a latent leak for any code that runs more than one worker or restarts.

**Fix:** Spawn the signal handler once (e.g. guarded by a `OnceLock`/`Once`), or hold the `JoinHandle` and `abort()` it before `run()` returns. Replace the `.expect()` with a logged error that sets the shutdown flag rather than panicking in a detached task.

### WR-03: Shutdown drain ignores the acquire result and never re-adds permits — semaphore is permanently drained

**File:** `ferro-queue/src/worker.rs:262-265`
**Issue:**
```rust
let _ = self.semaphore.acquire_many(self.config.max_jobs as u32).await;
```
The returned `SemaphorePermit` guard is dropped immediately (`let _ =`), which actually *releases* the permits back. That defeats the intent of "hold all permits until drain completes": between the `acquire_many` returning and the function continuing, a still-pending `spawn_job` task (one that was spawned but had not yet reached `acquire_owned` at line 327) can grab a permit and start a brand-new job *after* the drain supposedly completed, then get its row requeued out from under it by `requeue_claimed_by`. Because `spawn_job` is fire-and-forget (`tokio::spawn`), there is no guarantee all spawned tasks have reached their `acquire_owned` call at the moment the drain runs.

**Fix:** Bind the guard to a named variable held across the `requeue_claimed_by` call so the permits stay held until requeue finishes, and gate new spawns on the shutdown flag. `spawn_job` should check `self.shutdown` (or be handed a permit acquired by the loop before spawning) so no new job starts once draining begins:

```rust
let _drain_guard = self.semaphore.acquire_many(self.config.max_jobs as u32).await;
crate::db::requeue_claimed_by(conn, &self.worker_id).await?;
// _drain_guard dropped here, after requeue
```
Also have `spawn_job` early-return if `self.shutdown.load(...)` is set.

### WR-04: `is_sync_mode` defaults to `true`, so background processing is off unless explicitly enabled — surprising in production

**File:** `ferro-queue/src/config.rs:68-72`, `framework/src/app.rs:415-423`
**Issue:**
`QueueConfig::is_sync_mode()` returns `true` when `QUEUE_CONNECTION` is unset. The worker is only auto-started when jobs are registered (`app.rs:415`), but `dispatch()` consults `is_sync_mode()` independently (`dispatcher.rs:86`). A production deployment that registers jobs (spawning a WorkerLoop) but forgets to set `QUEUE_CONNECTION` will run every `dispatch()` *inline* (sync) while the WorkerLoop polls an empty queue. Jobs silently execute in the request path with no delay/retry semantics, and `.delay()`/`.on_queue()` are silently ignored. The default-to-sync is reasonable for dev but is a foot-gun when the same binary ships to prod.

**Fix:** Either (a) make the auto-start path and `dispatch()` agree on a single source of truth — if a WorkerLoop is started, dispatch should not be sync; or (b) emit a startup warning when jobs are registered but `is_sync_mode()` is true, e.g. in `run_server_internal`. Document the default explicitly in the env-vars section of `docs/src/features/queues.md` (it currently shows `QUEUE_CONNECTION=sync` as an example but does not state that *unset* also means sync).

### WR-05: MCP `job_history` interpolates `queue_filter`, `limit`, and the redis queue name into SQL/queries — injection risk

**File:** `ferro-mcp/src/tools/job_history.rs:87-91,130-136`
**Issue:**
```rust
format!("SELECT * FROM jobs WHERE queue = '{queue}' ORDER BY created_at DESC LIMIT {limit}")
```
`queue_filter` is interpolated directly into the SQL string with naive single-quote wrapping; a queue name containing a quote breaks out of the literal (`'; DROP TABLE jobs; --`). `limit` is also interpolated (lower risk as it is typed `usize`, but `queue` is attacker-influenceable if the MCP caller controls the filter). The production `db.rs` path correctly uses bound parameters everywhere; this introspection tool regressed to interpolation. While ferro-mcp is a developer tool pointed at a local/dev DB, the inconsistency undermines the T-185-01 "all dynamic values bound" guarantee for anything reading the same table.

**Fix:** Use `Statement::from_sql_and_values` with backend placeholders and bound `Value`s, mirroring `db.rs::get_pending_jobs`/`get_failed_jobs`. Better still, have the MCP tool call the public `ferro_queue::get_pending_jobs` / `get_failed_jobs` / `get_stats` functions instead of issuing its own raw SQL, eliminating the duplicate query surface entirely.

### WR-06: `parse_failed_job_info` uses `created_at` as `failed_at`, so failed-job ordering and timestamps are wrong

**File:** `ferro-queue/src/db.rs:789-801`, `ferro-mcp/src/tools/job_history.rs:155-156`
**Issue:**
The `jobs` table has no `failed_at` column (migration.rs:141-158), so both `FailedJobInfo.failed_at` and the MCP `failed_at` are populated from `created_at`. `get_failed_jobs` also `ORDER BY created_at DESC` (db.rs:670). The result is that "most recently failed" actually means "most recently *created*," which can be very different for a long-lived job that fails late, or a reaper-parked job. Operators inspecting failures will see misleading timestamps and ordering. The code comments acknowledge this ("same as `created_at` for reaper-parked"), but the behavior is incorrect for handler-failed jobs whose creation and failure times diverge.

**Fix:** Add a nullable `failed_at` timestamp column to the migration, set it in `fail_job` and the reaper's park step, and order/report on it. If a schema change is out of scope for this phase, document the limitation in `queues.md` and rename the field to `created_at` in the introspection types to avoid implying it is the failure time.

## Info

### IN-01: Dead legacy `JobPayload` struct and Redis code path remain after the DB migration

**File:** `ferro-queue/src/job.rs:91-185`, `ferro-mcp/src/tools/job_history.rs:198-270`
**Issue:**
`JobPayload` (with `reserved_at`, `reserve()`, `increment_attempts()`, UUID id) is the old broker-era payload and is no longer used by the DB engine, which works off `JobRow`/columns. It is still exported (`lib.rs:66`, `framework/src/lib.rs:198`) and carries its own test suite, but the production claim/enqueue path never constructs or reads it. Similarly, `get_redis_job_history` and the `"redis"` driver branch (job_history.rs:57,198-270) keep a full Redis introspection path with `REDIS_URL`/`QUEUE_REDIS_URL` env vars. Per the project's "delete old code completely — no deprecation" principle and the Redis-removal goal of this phase, both are dead surface.

**Fix:** Delete `JobPayload` and its exports if nothing external depends on it; delete the redis branch and `get_redis_job_history`. If a consumer still references `JobPayload`, that is itself a migration gap to resolve rather than preserve.

### IN-02: `truncate_payload` can panic on multi-byte UTF-8 boundary

**File:** `ferro-mcp/src/tools/job_history.rs:287-293`
**Issue:**
`&payload[..max_len]` slices by byte index; if `max_len` falls inside a multi-byte UTF-8 codepoint this panics. Job payloads are JSON (often ASCII) but may contain UTF-8 in string fields.

**Fix:** Use `payload.char_indices().nth(max_len)` to find a safe boundary, or `payload.chars().take(max_len).collect()`.

### IN-03: `default_jitter_delay` and `Job::retry_delay` duplicate the same backoff formula

**File:** `ferro-queue/src/worker.rs:444-451`, `ferro-queue/src/job.rs:62-70`
**Issue:**
The full-jitter formula (base 5 s, cap 15 min, saturating pow) is implemented identically in two places. The worker falls back to `default_jitter_delay` only on the panic path because the handler destructured the job before panicking. Two copies risk drifting apart.

**Fix:** Extract a single `fn full_jitter_delay(attempt: u32) -> Duration` (e.g. in a shared module) and call it from both the trait default and the panic fallback.

### IN-04: `claim_postgres` commits a no-op transaction on the empty-queue path

**File:** `ferro-queue/src/db.rs:301-305`
**Issue:**
When the `SELECT…FOR UPDATE SKIP LOCKED` returns no row, the code commits an empty transaction. Functionally correct but an extra round-trip per idle poll per queue; with the default 1 s idle sleep this is minor, but under many queues it adds up.

**Fix:** Optional — `rollback()` is equally cheap and clearer for a read-only no-op, or restructure to avoid opening the txn until a row is found (not possible here because the lock must be held; leave as-is if clarity is preferred). Low priority.

### IN-05: `debug/mod.rs` queue error messages still mention Redis

**File:** `framework/src/debug/mod.rs:191,246`
**Issue:**
The 503 message reads `"Queue not initialized (QUEUE_CONNECTION=sync or Redis not configured)"`. Redis is no longer a backend; the message is misleading post-migration.

**Fix:** Update to `"Queue not initialized (QUEUE_CONNECTION=sync, or Queue::init not called)"`.

---

_Reviewed: 2026-06-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

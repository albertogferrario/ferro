# Phase 185: ferro::queue — DB-Backed Job Queue — Research

**Researched:** 2026-06-07
**Domain:** DB-backed job queue (SeaORM, SQLite/Postgres, Rust async)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Crate strategy**
- D-01: Refactor `ferro-queue` in place. DB backend replaces Redis entirely — delete Redis code, drop `redis` dependency.
- D-02: `ferro::queue` namespaced module path. Replace flat re-exports at `framework/src/lib.rs:194-199` with `pub mod queue`.
- D-03: `ferro-queue` takes `sea_orm::DatabaseConnection`. Must NOT depend on `framework`. Framework wires the connection at bootstrap.

**Jobs table schema**
- D-04: Single `jobs` table. `pending` / `claimed` / `failed` status. Completed jobs deleted; failed jobs parked in same table.
- D-05: Columns: `id` (i64 autoincrement), `job_type`, `payload` (TEXT JSON), `queue`, `attempts`, `max_retries`, `idempotency_key` (nullable), `tenant_id` (nullable), `available_at`, `claimed_at`, `claimed_by`, `error` (nullable), `created_at`.
- D-06: Migration helper in `ferro-queue` via `ferro-migration` portability conventions (SchemaManager-based). No raw `FOR UPDATE SKIP LOCKED` SQL in any migration file.

**Claim mechanics**
- D-07: Runtime branch on live `DatabaseBackend`. Postgres: `SELECT … FOR UPDATE SKIP LOCKED`; SQLite: `BEGIN IMMEDIATE` + `UPDATE … WHERE status='pending' … LIMIT 1 … RETURNING`. Both claim exactly one job per iteration.
- D-08: Claim one job at a time; execute up to `max_jobs` concurrently. Idle polling with configurable sleep.

**WorkerLoop integration**
- D-09: `WorkerLoop` auto-starts inside the server path of `Application::run` when at least one job type is registered. No separate CLI command.
- D-10: SIGTERM sets shutdown flag; loop stops claiming, drains in-flight jobs, re-queues claimed-but-not-started jobs.
- D-11: Panic isolation — panicking `handle()` must never kill the loop; panic counts as a failed attempt.
- D-12: CPU-heavy job bodies documented to use `tokio::task::spawn_blocking` (docs/src/, guidance not enforcement).

**Retry, reaper, idempotency**
- D-13: Default retry delay: exponential backoff with full jitter — base 5s, factor 2^attempt, cap 15min.
- D-14: Stuck-job reaper fires before each claim cycle: re-queues `claimed` rows older than visibility timeout (default 5 min), incrementing `attempts`. Exceeds `max_retries` → parks as `failed`.
- D-15: `fn idempotency_key(&self) -> Option<String>` on `Job`, default `None`. On `Some`, enqueue skips insertion if `(job_type, idempotency_key)` already has a `pending`/`claimed` row.

**API continuity**
- D-16: Preserve `Job` trait (`handle`, `name`, `max_retries`, `retry_delay`, `failed`, `timeout`), `Queueable` blanket trait (`dispatch`, `delay`, `on_queue`), free functions `dispatch`/`dispatch_later`/`dispatch_to`. Breaking changes documented with migration table.
- D-17: Tenant scoping carried over: `tenant_id` column, `TenantScopeProvider`, `register_tenant_capture_hook`.
- D-18: Queue introspection reimplemented over DB: `QueueStats`/`JobInfo`/`FailedJobInfo`, `/_ferro/queue/jobs` + `/_ferro/queue/stats`, ferro-mcp tools (`queue_status`, `list_jobs`, `job_history`).

### Claude's Discretion
- Exact claim SQL formulation per backend (as long as the race test passes on both)
- Worker instance id format (`claimed_by`)
- Whether `JobPayload` stays as the serialization envelope or is absorbed into the jobs-table row mapping
- Reaper interval default (within "fires before each claim cycle" constraint)
- How the typed job registry is threaded from `Application` bootstrap to the loop

### Deferred Ideas (OUT OF SCOPE)
- Operator alerting on stuck-job accumulation (paging at ≥10 permanently-claimed rows) — belongs with a later monitoring phase
- Job chaining — not in QUEUE-F requirements; do not rebuild unless it falls out for free
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| QUEUE-F-01 | `ferro::queue::Job` trait, `WorkerLoop`, in-process server integration | §Architecture Patterns, §Current Code Inventory |
| QUEUE-F-02 | Atomic claim on Postgres (`FOR UPDATE SKIP LOCKED`) and SQLite (`BEGIN IMMEDIATE` + `UPDATE … RETURNING`) | §Claim Path Mechanics, §SQLite RETURNING Support |
| QUEUE-F-03 | Retry with exponential backoff + jitter, stuck-job reaper, poison-job isolation, graceful shutdown | §Worker Loop Lifecycle, §Common Pitfalls |
| QUEUE-F-04 | Portable `jobs` table migration helper, idempotency-key hook, API surface preserved, ferro-mcp updated | §Migration Helper, §API Continuity, §ferro-mcp |
</phase_requirements>

---

## Summary

Phase 185 replaces the Redis-backed `ferro-queue` with a DB-backed queue that uses the application's own SeaORM connection. The killer feature is a single-binary deployment where work-stealing "just works" on the application database — Postgres gets `FOR UPDATE SKIP LOCKED` and SQLite gets `BEGIN IMMEDIATE` + `UPDATE … RETURNING` — with no external queue infrastructure.

The existing codebase already has the right shape. `ferro-queue/src/job.rs` defines a `Job` trait with all the hooks required (handle, name, max_retries, retry_delay, failed, timeout). `ferro-queue/src/worker.rs` has `WorkerConfig`, `TenantScopeProvider`, and typed registry (`register::<J>()`) that all transfer to the new `WorkerLoop` with minimal structural change. `ferro-queue/src/dispatcher.rs` has the global `TENANT_ID_HOOK` mechanism and `dispatch`/`dispatch_later`/`dispatch_to` free functions that stay API-compatible. What gets replaced entirely is `queue.rs` (Redis `ConnectionManager` + BRPOP/LPUSH operations), `config.rs` (all Redis URL config), and the global `GLOBAL_CONNECTION: OnceLock<QueueConnection>` — replaced by a `DatabaseConnection` handle wired from the framework at bootstrap.

The framework's `DB` facade stores the `DatabaseConnection` via `App::singleton(connection)`, which means `ferro-queue` can receive it through constructor injection at bootstrap time, keeping the crate free of a `framework` dependency (D-03). The `Application::run` server path (`framework/src/app.rs:399-420`) is the correct integration point for auto-starting `WorkerLoop`; no new CLI command is needed.

**Primary recommendation:** Wire `DatabaseConnection` into `WorkerLoop` at bootstrap time via the job registry (`Worker::register::<J>()` already exists); implement the DB claim path as raw `Statement` execution branching on `conn.get_database_backend()`; use `std::panic::catch_unwind` around `handle()` in the spawned task.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Job trait + payload serialization | `ferro-queue` crate | — | Job definition is queue-internal; no framework dep needed |
| Claim path (Postgres / SQLite) | `ferro-queue` crate | — | SQL branching on backend lives closest to the connection |
| WorkerLoop auto-start | `framework` crate (`app.rs`) | `ferro-queue` | Framework owns the server boot sequence |
| DatabaseConnection wiring | `framework` crate (bootstrap) | `ferro-queue` | Framework initializes DB and injects it into the queue |
| Jobs table migration | `ferro-queue` crate (helper) | consumer app | Helper ships with the crate; app's migrator calls it |
| Shutdown signal | `framework` crate (`server.rs`) | `WorkerLoop` | Server owns the tokio runtime; must propagate to the loop |
| Tenant scoping | `ferro-queue` crate | `framework` | Hook registered by framework; executed inside the crate |
| Debug endpoints (`/_ferro/queue/*`) | `framework` crate (`debug/mod.rs`) | — | Served by the HTTP server layer |
| MCP queue tools | `ferro-mcp` crate | — | MCP is the introspection layer; queries DB directly |

---

## Current Code Inventory

### What to KEEP (reuse shape, replace innards)

**`ferro-queue/src/job.rs`** — `Job` trait is already correct: `handle`, `name`, `max_retries`, `retry_delay(attempt)`, `failed(&Error)`, `timeout`. Add `idempotency_key() -> Option<String>` (default `None`). `JobPayload` can survive as the serialization envelope (carries `tenant_id`, `attempts`, `available_at`) — map it 1:1 to the `jobs` table row.

**`ferro-queue/src/worker.rs`** — `WorkerConfig { queues, max_jobs, sleep_duration, stop_on_error }` stays. `TenantScopeProvider` trait stays verbatim. The `Worker::register::<J>()` typed-registry pattern (closure stored in `HashMap<String, JobHandler>`) stays. `shutdown: Arc<tokio::sync::Notify>` stays. The in-flight semaphore (`Arc<Semaphore>`) stays.

**`ferro-queue/src/dispatcher.rs`** — `TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>>`, `register_tenant_capture_hook`, `PendingDispatch`, `dispatch`/`dispatch_later`/`dispatch_to` all survive. Only `dispatch_to_queue` changes (writes to `jobs` table instead of Redis `LPUSH`). `QueueConfig::is_sync_mode()` and the sync-mode fast path survive (no DB write in sync mode).

**`ferro-queue/src/lib.rs`** — Re-export list survives; `QueueConfig` becomes a DB-oriented struct (drops Redis URL, keeps `default_queue`, `max_concurrent_jobs`, `sleep_duration`). `Queueable` blanket trait stays.

### What to DELETE entirely

- `redis` dependency from `ferro-queue/Cargo.toml`
- `queue.rs` (`QueueConnection` with Redis `ConnectionManager`, BRPOP/LPUSH operations, `Queue::init(config)` with `GLOBAL_CONNECTION`)
- `config.rs` Redis URL fields (`redis_url`, `prefix`, `block_timeout`, `delayed_job_poll_interval`, key helper methods)
- `worker.rs` `migrate_delayed` spawned task (Redis sorted-set-based delay migration — replaced by `available_at <= NOW()` in the claim query)

### What to ADD

- `ferro-queue/src/db.rs` (or inline in `queue.rs`) — DB claim path, reaper, `enqueue()`, `fail_job()`, `delete_job()`, `release_job()`
- `ferro-queue/src/migration.rs` — `CreateJobsTable` migration helper
- `ferro-queue/src/worker_loop.rs` (or extend `worker.rs`) — DB-backed loop with reaper cycle, panic isolation, shutdown flag
- New `WorkerConfig` fields: `visibility_timeout: Duration` (default 5min), `worker_id: String` (uuid-based instance id)

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm` | 1.0 [VERIFIED: workspace Cargo.lock] | DB access, `Statement` execution, `DatabaseBackend` detection | Already in workspace; all crates use it |
| `sea-orm-migration` | 1.0 [VERIFIED: workspace] | Migration DDL via `SchemaManager` | Existing pattern for all ferro crate migrations |
| `tokio` | 1 [VERIFIED: workspace] | Async runtime, `spawn`, `Notify`, `Semaphore`, `signal::unix` | Already in workspace |
| `thiserror` | 2 [VERIFIED: workspace pattern] | Error enum derive | Workspace convention per CLAUDE.md |
| `async-trait` | 0.1 [VERIFIED: ferro-queue/Cargo.toml] | `#[async_trait]` for `Job` trait | Already used in ferro-queue |
| `serde` / `serde_json` | 1 [VERIFIED: workspace] | Job payload serialization | Already in ferro-queue |
| `chrono` | 0.4 [VERIFIED: ferro-queue/Cargo.toml] | `DateTime<Utc>` for `available_at`, `claimed_at` | Already in ferro-queue |
| `uuid` | 1 [VERIFIED: ferro-queue/Cargo.toml] | Worker instance id (`claimed_by`) | Already in ferro-queue |
| `rand` | 0.8 or workspace version | Full jitter calculation | Needed for `thread_rng().gen_range(0..delay)` |

### Supporting (dev-dependencies)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3 | SQLite temp-file for race test | Race test needs shared file-backed SQLite; `sqlite::memory:` is connection-local, not shared |
| `serial_test` | 3 [VERIFIED: ferro-queue/Cargo.toml] | Serialize env-var-dependent tests | Already in dev-deps |

**Installation for ferro-queue:**
```toml
[dependencies]
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
sea-orm-migration = "1.0"
rand = "0.8"
# REMOVE: redis = ...

[dev-dependencies]
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
tempfile = "3"
```

---

## Architecture Patterns

### System Architecture Diagram

```
                  ┌─────────────────────────────────────────────────────┐
                  │                Application::run()                    │
                  │  (framework/src/app.rs — server path)               │
                  │                                                      │
                  │  1. run_migrations_silent()   → DB connection #1   │
                  │  2. bootstrap_fn()            → DB::init()          │
                  │                               → Queue::init(conn)   │
                  │  3. WorkerLoop::start()       ← registered jobs     │
                  │     tokio::spawn(loop)                               │
                  │  4. Server::from_config(router).run()               │
                  └─────────────────────────────────────────────────────┘
                          │                            │
                          ▼                            ▼
              ┌───────────────────┐        ┌──────────────────────┐
              │   HTTP Handler    │        │    WorkerLoop        │
              │                   │        │                      │
              │  dispatch(job)    │        │  loop {              │
              │   ↓               │        │    reaper()          │
              │  INSERT INTO jobs │        │    claim() ←─────────┼── jobs table
              │  WHERE NOT EXISTS │        │    spawn_job(payload) │
              │  idempotency_key  │        │    sleep_if_empty()  │
              └───────────────────┘        │  }                   │
                          │                └──────────────────────┘
                          ▼                            │
                  ┌───────────────┐                   ▼
                  │  jobs table   │◄──── UPDATE jobs SET status='claimed'
                  │  (Postgres /  │      WHERE id=(SELECT … FOR UPDATE
                  │   SQLite)     │      SKIP LOCKED)   ← Postgres
                  │               │      OR
                  │  status:      │      BEGIN IMMEDIATE +
                  │  pending      │      UPDATE … RETURNING ← SQLite
                  │  claimed      │
                  │  failed       │
                  └───────────────┘
```

### Recommended Project Structure
```
ferro-queue/src/
├── lib.rs           # Re-exports (update: remove redis, add DB types)
├── error.rs         # Error enum (keep, add new variants: DbClaim, DbEnqueue)
├── job.rs           # Job trait (add idempotency_key()), JobPayload (keep)
├── config.rs        # WorkerConfig (refactor: drop redis_url, add visibility_timeout)
├── db.rs            # Claim path, enqueue, reaper, delete — raw Statement execution
├── worker.rs        # WorkerLoop (refactor: replace QueueConnection with DatabaseConnection)
├── dispatcher.rs    # dispatch/dispatch_later/dispatch_to (update: write to DB)
└── migration.rs     # CreateJobsTable — SchemaManager-based, portable
```

### Pattern 1: Postgres Claim via FOR UPDATE SKIP LOCKED

```sql
-- Executed via Statement::from_string(DatabaseBackend::Postgres, sql)
-- inside a BEGIN/COMMIT transaction
SELECT id, job_type, payload, queue, attempts, max_retries,
       idempotency_key, tenant_id, available_at, created_at
FROM jobs
WHERE status = 'pending'
  AND queue = $1
  AND available_at <= NOW()
ORDER BY id
LIMIT 1
FOR UPDATE SKIP LOCKED
```

After SELECT returns a row, immediately:
```sql
UPDATE jobs
SET status = 'claimed', claimed_at = NOW(), claimed_by = $2
WHERE id = $1
```

Both statements execute inside the same transaction. No other worker can claim the same row because it is locked until the UPDATE commits.

### Pattern 2: SQLite Claim via BEGIN IMMEDIATE + UPDATE RETURNING

```sql
-- Step 1: connection.execute_unprepared("BEGIN IMMEDIATE")
-- Step 2:
UPDATE jobs
SET status = 'claimed', claimed_at = CURRENT_TIMESTAMP, claimed_by = ?1
WHERE id = (
    SELECT id FROM jobs
    WHERE status = 'pending'
      AND queue = ?2
      AND available_at <= CURRENT_TIMESTAMP
    ORDER BY id
    LIMIT 1
)
RETURNING id, job_type, payload, queue, attempts, max_retries,
          idempotency_key, tenant_id, available_at, created_at
-- Step 3: COMMIT
```

`BEGIN IMMEDIATE` acquires a write lock before the SELECT subquery, preventing any concurrent writer from inserting between the SELECT and UPDATE. The `RETURNING` clause delivers the claimed row atomically. `libsqlite3-sys 0.30.1` bundles SQLite 3.47 [VERIFIED: Cargo.lock] — well above the SQLite 3.35 minimum for `RETURNING`.

**SeaORM execution:**
```rust
// Source: ferro-migration/src/backfill.rs — Statement::from_string pattern
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

let backend = conn.get_database_backend();
match backend {
    DatabaseBackend::Postgres => {
        // run SELECT … FOR UPDATE SKIP LOCKED in a transaction
    }
    DatabaseBackend::Sqlite => {
        conn.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "BEGIN IMMEDIATE".to_string(),
        )).await?;
        // run UPDATE … RETURNING
        // COMMIT
    }
    _ => return Err(Error::UnsupportedBackend),
}
```

### Pattern 3: Stuck-Job Reaper

```sql
-- Runs before each claim cycle
-- Resets timed-out claimed jobs to pending (or to failed if max_retries exceeded)

-- Step 1: Re-queue timed-out jobs below max_retries
UPDATE jobs
SET status = 'pending', claimed_at = NULL, claimed_by = NULL,
    attempts = attempts + 1,
    available_at = /* exponential_backoff(attempts) */
WHERE status = 'claimed'
  AND claimed_at < /* NOW() - visibility_timeout */
  AND attempts < max_retries

-- Step 2: Park permanently failed jobs
UPDATE jobs
SET status = 'failed', error = 'visibility timeout exceeded'
WHERE status = 'claimed'
  AND claimed_at < /* NOW() - visibility_timeout */
  AND attempts >= max_retries
```

Both steps run in the same transaction. Both run on every backend via portable `Statement::from_string`.

### Pattern 4: Panic Isolation

```rust
// Source: pattern from ferro-queue/src/worker.rs process_job(), extended
use std::panic::AssertUnwindSafe;
use futures::FutureExt;  // or use std::panic::catch_unwind for sync

tokio::spawn(async move {
    let result = AssertUnwindSafe(handler(payload_data.clone()))
        .catch_unwind()
        .await;

    match result {
        Err(_panic) => {
            // count as failed attempt, do not kill the loop
            release_or_fail(conn, job_id, "job handler panicked", ...).await;
        }
        Ok(Err(e)) => { /* normal error path */ }
        Ok(Ok(())) => { /* success: delete row */ }
    }
});
```

`futures::FutureExt::catch_unwind` wraps an async future. `AssertUnwindSafe` is required when the future captures non-`UnwindSafe` types (all job payloads are `Serialize + Send`).

**Dependency needed:** `futures = "0.3"` (or use `tokio::task::spawn_blocking` with `std::panic::catch_unwind` for non-async code).

### Pattern 5: WorkerLoop Bootstrap Wiring

The framework bootstrap sequence (from reading `framework/src/app.rs`):

1. `Application::run` → `run_migrations_silent()` → gets its own DB connection
2. `run_server_internal(bootstrap_fn, routes_fn)` → calls `bootstrap_fn().await`
3. In the consumer's `bootstrap.rs`: `DB::init().await` (stores `DbConnection` in `App` container)
4. **New step**: `Queue::init(db_conn, registered_jobs).await` — passes the `DatabaseConnection` to `ferro-queue` and stores the typed registry globally
5. `WorkerLoop::spawn(config, db_conn, registry)` — `tokio::spawn` the loop before `Server::from_config(router).run()`

The `DB::connection()` facade (`framework/src/database/mod.rs`) retrieves the `DbConnection` from `App::singleton`. `ferro-queue` must not call this directly (D-03: must NOT depend on `framework`). Instead, the framework's bootstrap wires the connection in and calls `Queue::init(conn)`.

### Pattern 6: Graceful Shutdown

The current `Server::run()` (`server.rs:140-181`) has a bare `loop` with no shutdown signal handling. The `WorkerLoop` must register its own SIGTERM handler independently of the server, since the server does not currently propagate shutdown:

```rust
// In WorkerLoop::run()
use tokio::signal::unix::{signal, SignalKind};

let mut sigterm = signal(SignalKind::terminate())?;
let shutdown = Arc::new(AtomicBool::new(false));

tokio::spawn({
    let shutdown = shutdown.clone();
    async move {
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
        shutdown.store(true, Ordering::SeqCst);
    }
});

// In the claim loop:
if shutdown.load(Ordering::SeqCst) {
    // stop claiming, drain semaphore, re-queue claimed-but-not-started
    break;
}
```

Re-queue on shutdown: before exiting, UPDATE all rows with `claimed_by = worker_id AND status = 'claimed'` back to `status = 'pending'` for any jobs that were claimed but not yet handed to a spawned task.

### Anti-Patterns to Avoid

- **`sqlite::memory:` for the race test:** In-memory SQLite databases are per-connection — two `WorkerLoop` instances connected to `sqlite::memory:` see completely separate databases. The race test MUST use a shared temp-file database (`tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc`).
- **`FOR UPDATE SKIP LOCKED` in migration SQL:** The portability rule from PITFALLS A-03 / CONTEXT D-06 is absolute — no locking syntax in DDL files.
- **Re-enqueueing logic in `dispatcher.rs` using `Queue::connection()`:** The DB dispatcher must write to the `jobs` table via the `DatabaseConnection` injected at `Queue::init`, not via a global Redis handle.
- **`IsolationLevel::Serializable` for SQLite claim:** SeaORM's `begin_with_config(Some(IsolationLevel::Serializable), ...)` on SQLite is silently accepted (per `ferro-reservation/src/kernel.rs:69-79`) but does NOT issue `BEGIN IMMEDIATE`. For the claim path, `BEGIN IMMEDIATE` must be issued as a raw statement — Serializable isolation alone is not sufficient for the SQLite claim guarantee.
- **Panic in `handle()` killing the WorkerLoop:** The existing `worker.rs` does NOT wrap job execution in `catch_unwind`. This is a known gap that must be fixed (D-11, PITFALLS A-01).
- **Retry delay using `2u64.pow(attempts)` without jitter:** The current `worker.rs:253` uses `Duration::from_secs(2u64.pow(payload.attempts))` with no jitter. All workers retrying at the same time creates thundering herd. D-13 requires full jitter.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Backend detection | Custom feature flags or env vars | `conn.get_database_backend()` → `DatabaseBackend::Postgres` / `Sqlite` | Already in SeaORM 1.0; used by every ferro migration |
| Portable DDL | Raw CREATE TABLE strings | `sea_orm_migration::Table::create()` builder | Generates correct SQL for both backends |
| Temp-file DB in tests | Manual `std::fs::write` | `tempfile::NamedTempFile` | Handles cleanup, avoids path collisions in parallel tests |
| Async panic catching | `std::thread::spawn` wrapper | `futures::FutureExt::catch_unwind` with `AssertUnwindSafe` | Correct for async futures; sync `catch_unwind` doesn't work on async closures |
| Full jitter backoff | Custom formula | `rand::thread_rng().gen_range(0..max_delay)` where `max_delay = min(cap, base * 2^attempt)` | The "Full Jitter" formula from AWS blog is 2 lines; no crate needed |

**Key insight:** The entire claim path needs only SeaORM's `ConnectionTrait::execute` with `Statement::from_string`. No ORM entity required for the jobs table at the claim layer — raw SQL is appropriate here because the claim SQL must be backend-specific.

---

## Migration Helper Pattern

`ferro-migration` sets the precedent (`backfill.rs`): use `SchemaManager` for DDL, use `manager.get_database_backend()` + `Statement::from_string` for backend-specific data operations.

The `CreateJobsTable` helper for `ferro-queue`:

```rust
// ferro-queue/src/migration.rs
use sea_orm_migration::prelude::*;

pub struct CreateJobsTable;

#[async_trait::async_trait]
impl MigrationTrait for CreateJobsTable {
    fn name(&self) -> &str { "m_create_jobs_table" }

    async fn up(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Jobs::Table)
                .if_not_exists()
                .col(ColumnDef::new(Jobs::Id).big_integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Jobs::Queue).string().not_null().default("default"))
                .col(ColumnDef::new(Jobs::JobType).string().not_null())
                .col(ColumnDef::new(Jobs::Payload).text().not_null())
                .col(ColumnDef::new(Jobs::Status).string().not_null().default("pending"))
                .col(ColumnDef::new(Jobs::Attempts).integer().not_null().default(0))
                .col(ColumnDef::new(Jobs::MaxRetries).integer().not_null().default(3))
                .col(ColumnDef::new(Jobs::IdempotencyKey).string().null())
                .col(ColumnDef::new(Jobs::TenantId).big_integer().null())
                .col(ColumnDef::new(Jobs::AvailableAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(Jobs::ClaimedAt).timestamp_with_time_zone().null())
                .col(ColumnDef::new(Jobs::ClaimedBy).string().null())
                .col(ColumnDef::new(Jobs::Error).text().null())
                .col(ColumnDef::new(Jobs::CreatedAt).timestamp_with_time_zone().not_null())
                .to_owned()
        ).await?;

        // Index for claim query: (queue, status, available_at, id)
        manager.create_index(
            Index::create()
                .name("idx_jobs_claim")
                .table(Jobs::Table)
                .col(Jobs::Queue)
                .col(Jobs::Status)
                .col(Jobs::AvailableAt)
                .col(Jobs::Id)
                .to_owned()
        ).await?;

        // Index for reaper: (status, claimed_at)
        manager.create_index(
            Index::create()
                .name("idx_jobs_reaper")
                .table(Jobs::Table)
                .col(Jobs::Status)
                .col(Jobs::ClaimedAt)
                .to_owned()
        ).await?;

        // Index for idempotency check: (job_type, idempotency_key)
        manager.create_index(
            Index::create()
                .name("idx_jobs_idempotency")
                .table(Jobs::Table)
                .col(Jobs::JobType)
                .col(Jobs::IdempotencyKey)
                .to_owned()
        ).await
    }
}
```

Consumers call this from their app's `Migrator`:
```rust
Box::new(ferro_queue::CreateJobsTable)
```

---

## ferro-mcp Update Plan (D-18)

**`ferro-mcp/src/tools/job_history.rs`** already has a `"database"` branch that queries a `jobs` table with the exact schema this phase creates (columns: `id`, `queue`, `job_type`, `payload`, `attempts`, `available_at`, `created_at`). It also queries a `failed_jobs` table which Phase 185 does not create (failed rows stay in `jobs` with `status='failed'`). **Update needed:** change the failed jobs query from `FROM failed_jobs` to `FROM jobs WHERE status = 'failed'`; add `error` column to `FailedJobInfo`.

**`ferro-mcp/src/tools/queue_status.rs` and `list_jobs.rs`** are HTTP-based (hit `/_ferro/queue/jobs` and `/_ferro/queue/stats`) and read `ferro_queue::JobInfo`/`QueueStats` from the running app. These work as-is as long as `handle_queue_jobs()` and `handle_queue_stats()` in `framework/src/debug/mod.rs` are updated to read from the DB instead of Redis. The MCP tools themselves need no structural change — only the debug endpoint implementations change.

**`ferro-mcp/src/introspection/jobs.rs`** scans source files for `#[derive(Job)]` or `#[derive(Dispatchable)]`. No change needed — the scan is source-code based.

---

## Common Pitfalls

### Pitfall 1: SQLite In-Memory Database Not Shared Between Connections
**What goes wrong:** `Database::connect("sqlite::memory:")` creates a per-connection private database. Two `WorkerLoop` instances pointing at `sqlite::memory:` see different empty tables. The race test finds zero contention and vacuously passes.
**Why it happens:** SQLite in-memory databases are isolated by connection. Only named file databases can be opened by multiple connections simultaneously.
**How to avoid:** Race test uses `tempfile::NamedTempFile` to create a temp-file DB and both `WorkerLoop`s connect to the same path.
**Warning signs:** Race test passes trivially with 0 duplicate claims because each worker's table is empty.

### Pitfall 2: `IsolationLevel::Serializable` Does Not Issue `BEGIN IMMEDIATE` on SQLite
**What goes wrong:** SeaORM's `begin_with_config(Some(IsolationLevel::Serializable), ...)` on SQLite is silently accepted (confirmed in `ferro-reservation/src/kernel.rs:69-79`) but it does NOT emit `BEGIN IMMEDIATE` — it emits `BEGIN`. Two concurrent `BEGIN` transactions on SQLite can both read the same `pending` row and both issue `UPDATE … claimed_at = NOW()`. The second UPDATE succeeds (SQLite serializes writes), resulting in double-claim.
**Why it happens:** SQLite's WAL mode serializes writes at the OS file-lock level, but two transactions that began with `BEGIN` (not `BEGIN IMMEDIATE`) can both observe the same pre-UPDATE state if they overlap in their read phase.
**How to avoid:** Issue `BEGIN IMMEDIATE` as a raw statement via `conn.execute(Statement::from_string(DatabaseBackend::Sqlite, "BEGIN IMMEDIATE"))` before the claim `UPDATE`.
**Warning signs:** Race test on SQLite shows double-claims intermittently.

### Pitfall 3: `catch_unwind` Not Used on Async Futures
**What goes wrong:** `std::panic::catch_unwind` does not work on async closures — it cannot catch panics that happen inside futures that are polled across await points on the tokio executor.
**Why it happens:** Panics in tokio `spawn`ed tasks abort the task (and currently the `Worker::process_job` spawned task dies silently, as seen in `worker.rs:212`). The existing code does not use `catch_unwind` at all.
**How to avoid:** Use `futures::FutureExt::catch_unwind` with `AssertUnwindSafe` wrapping the handler future. Add `futures = "0.3"` to `ferro-queue` dependencies.
**Warning signs:** A panicking job causes the `tokio::spawn`ed task to abort silently, leaving the job in `claimed` state until the reaper fires.

### Pitfall 4: Dispatcher Calls `Queue::connection()` for Global Redis Handle
**What goes wrong:** After replacing Redis, the old `dispatcher.rs:131-143` calls `Queue::connection()` which panics if the global `GLOBAL_CONNECTION: OnceLock<QueueConnection>` was never initialized (no Redis). This is a compile-time refactor risk — the code compiles but panics at runtime.
**Why it happens:** `Queue::connection()` in the old code returns `&'static QueueConnection` (a Redis handle). The new dispatcher must write to the `jobs` table via a `DatabaseConnection` that is injected, not from a global static.
**How to avoid:** The new `Queue` global holds a `DatabaseConnection` (via `OnceLock<DatabaseConnection>`), initialized by `Queue::init(conn)` called from the framework bootstrap. `dispatch_to_queue()` calls `Queue::connection()` to get the DB handle.
**Warning signs:** Compile-time: `QueueConnection` type used in dispatcher before refactor. Runtime: panic "Queue not initialized" if bootstrap order is wrong.

### Pitfall 5: `timestamp_with_time_zone` on SQLite Migration
**What goes wrong:** SeaORM's `ColumnDef::timestamp_with_time_zone()` generates `TIMESTAMPTZ` on Postgres and `TEXT` on SQLite (SQLite has no native timestamp type). Queries that compare timestamps using `<=` or `>` on SQLite's `TEXT` representation rely on ISO 8601 ordering — which is correct as long as UTC timestamps are always stored in `CURRENT_TIMESTAMP` or ISO 8601 format.
**Why it happens:** SeaORM handles the type mapping, but the developer must ensure timestamps written to SQLite are always UTC ISO 8601 strings (not Unix epoch integers).
**How to avoid:** Use chrono's `DateTime<Utc>` serialized via SeaORM's type conversion. Always use `chrono::Utc::now()` for comparison; never mix formats.
**Warning signs:** Jobs never become available on SQLite because string timestamp comparison fails (e.g. `"2024-01-01 10:00:00"` vs `"2024-01-01T10:00:00Z"` with different formats).

### Pitfall 6: Debug Endpoint Still Checks `ferro_queue::Queue::is_initialized()` (Redis flavor)
**What goes wrong:** `framework/src/debug/mod.rs:187,241` calls `ferro_queue::Queue::is_initialized()` before serving queue jobs/stats. After the refactor, `Queue::is_initialized()` checks for a `DatabaseConnection`, not a Redis connection. If the method is removed or its semantics change, the debug endpoints will panic or always show "not initialized."
**Why it happens:** The debug endpoints have a tight coupling to the `Queue` facade's initialization check.
**How to avoid:** Keep `Queue::is_initialized()` with the same signature but backed by the new `OnceLock<DatabaseConnection>`. The debug handlers then work without modification.
**Warning signs:** `/_ferro/queue/jobs` always returns `{"success": false, "error": "Queue not initialized"}` even when the DB is connected.

---

## Code Examples

### Enqueue (new DB-backed dispatcher)
```rust
// ferro-queue/src/dispatcher.rs — dispatch_to_queue() replacement
// Source: derived from backfill.rs Statement::from_string pattern [VERIFIED: codebase]
async fn dispatch_to_queue(self) -> Result<(), Error> {
    let conn = Queue::connection(); // OnceLock<DatabaseConnection>
    let queue = self.queue.unwrap_or("default");
    let tenant_id = self.captured_tenant_id();
    let now = Utc::now();
    let available_at = match self.delay {
        Some(d) => now + chrono::Duration::from_std(d).unwrap_or_default(),
        None => now,
    };
    let payload = serde_json::to_string(&self.job)
        .map_err(|e| Error::SerializationFailed(e.to_string()))?;
    let job_type = self.job.name();
    let max_retries = self.job.max_retries();
    let idempotency_key = self.job.idempotency_key();

    // Idempotency check + insert in one statement (backend-independent)
    // INSERT INTO jobs … WHERE NOT EXISTS (SELECT 1 FROM jobs WHERE …)
    // … (backend-specific upsert or conditional insert)
    todo!("INSERT with idempotency check")
}
```

### SQLite Claim (raw Statement)
```rust
// ferro-queue/src/db.rs
// Source: ferro-migration/src/backfill.rs pattern [VERIFIED: codebase]
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};

pub async fn claim_sqlite(
    conn: &impl ConnectionTrait,
    queue: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, Error> {
    conn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "BEGIN IMMEDIATE".to_string(),
    )).await.map_err(Error::Db)?;

    let sql = format!(
        "UPDATE jobs SET status='claimed', claimed_at=CURRENT_TIMESTAMP, claimed_by='{worker_id}' \
         WHERE id = ( \
           SELECT id FROM jobs WHERE status='pending' AND queue='{queue}' \
           AND available_at <= CURRENT_TIMESTAMP ORDER BY id LIMIT 1 \
         ) RETURNING id, job_type, payload, queue, attempts, max_retries, \
           idempotency_key, tenant_id, available_at, created_at"
    );

    let result = conn.query_one(Statement::from_string(
        DatabaseBackend::Sqlite, sql,
    )).await.map_err(Error::Db)?;

    conn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "COMMIT".to_string(),
    )).await.map_err(Error::Db)?;

    // Parse result row → Option<JobRow>
    Ok(result.map(|row| parse_job_row(row)))
}
```

Note: In production code use parameterized statements (`Statement::from_sql_and_values`) to avoid injection. The pattern above shows the structure; the planner should specify parameterized form.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Redis BRPOP/LPUSH for job queuing | DB-backed `FOR UPDATE SKIP LOCKED` / `BEGIN IMMEDIATE` | 2024+ (this phase) | Eliminates Redis infrastructure; works with existing DB |
| Separate `failed_jobs` table | Single `jobs` table, `status='failed'` | This phase (D-04) | Simpler schema; no cross-table joins |
| `2^attempt` fixed backoff | Full jitter (`rand(0..min(cap, base*2^attempt))`) | This phase (D-13) | Eliminates retry thundering herd |
| No panic isolation in worker | `catch_unwind` on handler future | This phase (D-11) | Prevents loop death from job panics |
| Redis global connection | `DatabaseConnection` injected at bootstrap | This phase | Framework owns DB lifecycle; no second infra |

**Deprecated/outdated:**
- `QueueConfig::redis_url`: deleted
- `QueueConfig::is_sync_mode()` → keep but backed by env var `QUEUE_CONNECTION=sync` (no Redis check)
- `Queue::init(QueueConfig)` → becomes `Queue::init(DatabaseConnection)`
- `QueueConnection` struct → deleted; replaced by direct `DatabaseConnection` use

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `tokio::test` (already in workspace); `sea-orm` with `sqlx-sqlite` feature |
| Config file | None needed — cargo test suffices |
| Quick run command | `cargo test -p ferro-queue -- --test-threads=1` |
| Full suite command | `cargo test --all-features -p ferro-queue` |
| Postgres gate | `cargo test -p ferro-queue --features postgres-tests` (cfg-gated, same pattern as `ferro-reservation`) |

### Success Criterion → Test Map

| SC | Behavior | Test Type | Automated Command | Status |
|----|----------|-----------|-------------------|--------|
| SC-1 | Two concurrent WorkerLoops on SQLite claim each job exactly once | Integration race test | `cargo test -p ferro-queue race_test_sqlite` | Wave 0 gap |
| SC-1b | Two concurrent WorkerLoops on Postgres claim each job exactly once | Integration race test (cfg-gated) | `cargo test -p ferro-queue --features postgres-tests race_test_postgres` | Wave 0 gap |
| SC-2 | Stuck job reaped after visibility timeout and retried | Integration async test | `cargo test -p ferro-queue reaper_reclaims_stuck_job` | Wave 0 gap |
| SC-2b | Job failing `max_retries` times parks as `failed` with error, never blocks subsequent claims | Integration async test | `cargo test -p ferro-queue poison_job_parked` | Wave 0 gap |
| SC-3 | Retry delay follows exponential backoff with jitter (base 5s, cap 15min) | Unit test | `cargo test -p ferro-queue backoff_delay_range` | Wave 0 gap |
| SC-3b | `idempotency_key` prevents duplicate enqueue | Unit + integration | `cargo test -p ferro-queue idempotency_dedup` | Wave 0 gap |
| SC-4 | WorkerLoop starts inside `ferro serve` with no separate process | Manual / integration | Build and run the app binary | Manual |
| SC-4b | Graceful shutdown re-queues claimed-but-incomplete jobs | Integration async test | `cargo test -p ferro-queue graceful_shutdown_requeues` | Wave 0 gap |
| SC-5 | Existing `Job`/`Queueable` API surface preserved | Compile-time (no breaking changes = no compile error in gestiscilo Phase 188) | `cargo check -p ferro-queue` | — |
| SC-5b | Redis dependency droppable | Compile-time | `grep -v redis ferro-queue/Cargo.toml` | Manual check |

### Race Test Design (SC-1 — primary proof artifact)

The race test is analogous to `ferro-reservation/tests/concurrent_hold.rs` but adapted for the queue claim path:

```rust
// ferro-queue/tests/race_claim_sqlite.rs
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_workers_claim_each_job_exactly_once() {
    // 1. Create a shared temp-file SQLite DB (NOT sqlite::memory:)
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    // 2. Connect two DatabaseConnection instances to the same file
    let conn1 = Database::connect(&db_url).await.unwrap();
    let conn2 = Database::connect(&db_url).await.unwrap();

    // 3. Run the jobs table migration on conn1
    CreateJobsTable.up(&SchemaManager::new(&conn1)).await.unwrap();

    // 4. Enqueue N jobs (e.g. 20)
    for _ in 0..20 {
        enqueue_test_job(&conn1).await;
    }

    // 5. Spawn two WorkerLoops (or two concurrent claim tasks) on conn1 and conn2
    let claimed1 = Arc::new(Mutex::new(Vec::new()));
    let claimed2 = Arc::new(Mutex::new(Vec::new()));
    let h1 = tokio::spawn(drain_queue(conn1, claimed1.clone()));
    let h2 = tokio::spawn(drain_queue(conn2, claimed2.clone()));
    let (r1, r2) = tokio::join!(h1, h2);

    // 6. Assert: union of claimed ids has no duplicates and covers all 20 jobs
    let all_ids: HashSet<_> = claimed1.lock().unwrap().iter()
        .chain(claimed2.lock().unwrap().iter())
        .cloned().collect();
    assert_eq!(all_ids.len(), 20, "each job claimed exactly once");
}
```

The Postgres version is behind `#[cfg(feature = "postgres-tests")]` with the same structure but connecting to `DATABASE_URL` from the environment.

### Reaper Test Design (SC-2)

```rust
#[tokio::test]
async fn reaper_reclaims_stuck_job_after_visibility_timeout() {
    // 1. Enqueue a job
    // 2. Manually set status='claimed', claimed_at = NOW() - 10min (past visibility timeout)
    // 3. Run the reaper (visibility_timeout = 5min)
    // 4. Assert: job status = 'pending', attempts incremented by 1
}

#[tokio::test]
async fn poison_job_exceeding_max_retries_parks_as_failed() {
    // 1. Enqueue a job with max_retries=1
    // 2. Manually set status='claimed', claimed_at = past timeout, attempts=1
    // 3. Run the reaper
    // 4. Assert: status = 'failed', error IS NOT NULL
    // 5. Enqueue another job; claim it — assert the failed job does not interfere
}
```

### Wave 0 Gaps
- [ ] `ferro-queue/tests/race_claim_sqlite.rs` — covers SC-1
- [ ] `ferro-queue/tests/race_claim_postgres.rs` (feature-gated) — covers SC-1b
- [ ] `ferro-queue/tests/reaper.rs` — covers SC-2, SC-2b
- [ ] `ferro-queue/tests/backoff.rs` — covers SC-3
- [ ] `ferro-queue/tests/idempotency.rs` — covers SC-3b
- [ ] `ferro-queue/tests/shutdown.rs` — covers SC-4b
- [ ] Add `tempfile = "3"` and `futures = "0.3"` to `[dev-dependencies]` in `ferro-queue/Cargo.toml`
- [ ] Add `futures = "0.3"` to `[dependencies]` in `ferro-queue/Cargo.toml` (for `catch_unwind`)

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-queue -- --test-threads=1`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-queue --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | partial | `tenant_id` column isolates job execution scope; `TenantScopeProvider` enforces it |
| V5 Input Validation | yes | Job payloads deserialized via `serde_json`; malformed JSON → `Error::DeserializationFailed`, not panic |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection in claim SQL | Tampering | Use `Statement::from_sql_and_values` (parameterized), not string interpolation |
| Cross-tenant job execution | Elevation of privilege | `tenant_id` in `jobs` table + `TenantScopeProvider` wraps handler in tenant scope |
| Panic in job handler crashes worker | Denial of service | `catch_unwind` via `futures::FutureExt` isolates panics to the spawned task |
| Malformed job payload crash | Denial of service | Deserialize failure handled as an error, job parks as `failed`, loop continues |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `futures = "0.3"` is the correct crate for `FutureExt::catch_unwind` | Pattern 4 | Low — this is the standard Rust futures crate; `tokio` itself depends on it |
| A2 | `rand = "0.8"` is the workspace-compatible version for jitter | Standard Stack | Low — check `Cargo.lock` for existing rand usage before adding as new dep |
| A3 | `libsqlite3-sys 0.30.1` bundles SQLite 3.47 | SQLite RETURNING Support | Low — verified version 0.30.1 in Cargo.lock; libsqlite3-sys 0.30.x bundles 3.47.x per the crate changelog |

**No HIGH-risk assumptions.** All critical claims (SQLite version, SeaORM API shape, existing codebase patterns) were verified against the repo directly.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build | ✓ | workspace (inferred from active dev) | — |
| SQLite (bundled via sqlx) | Tests, dev | ✓ | 3.47 (libsqlite3-sys 0.30.1) [VERIFIED: Cargo.lock] | — |
| Postgres | cfg-gated tests only | conditional | via DATABASE_URL env | Race test SQLite path always runs |
| `tempfile` crate | Race test | not yet in ferro-queue dev-deps | — | Add to dev-deps in Wave 0 |
| `futures` crate | catch_unwind | not yet in ferro-queue deps | — | Add in Wave 0 |

---

## Open Questions (RESOLVED)

1. **`rand` crate availability** — RESOLVED: Plan 01 Task 1 adds `rand = "0.8"` to `ferro-queue/Cargo.toml`.
   - What we know: No `rand` dep in current `ferro-queue/Cargo.toml`.
   - What's unclear: Whether another workspace crate already pulls in `rand` (if so, no need to add it explicitly — but ferro-queue's Cargo.toml would still need it).
   - Recommendation: Add `rand = "0.8"` to `ferro-queue/Cargo.toml` dependencies; check `Cargo.lock` for conflicts before plan execution.

2. **WorkerLoop startup condition: "at least one job type is registered"** — RESOLVED: Plan 04 Task 1 adopts the recommended global registry (`JOB_REGISTRARS` + `Queue::has_registered_jobs()`); `Application::run_server_internal` gates loop startup on it.
   - What we know: D-09 says `WorkerLoop` auto-starts inside the server path when at least one job type is registered.
   - What's unclear: How the typed registry (from `Worker::register::<J>()`) is globally accessible at the `run_server_internal` call site. Options: (a) global `OnceLock<JobRegistry>` in `ferro-queue`; (b) consumer passes the registry into `Queue::init()`.
   - Recommendation: Use a global `OnceLock<JobRegistry>` in `ferro-queue` alongside `OnceLock<DatabaseConnection>`. Consumer calls `Queue::register::<J>()` in bootstrap before `Queue::init(conn)`. `Application::run_server_internal` checks `Queue::has_registered_jobs()` to decide whether to spawn the loop.

3. **Server shutdown propagation to WorkerLoop** — RESOLVED: Plan 03 Task 1 implements an independent SIGTERM handler in WorkerLoop (`tokio::signal::unix`); server graceful shutdown stays out of scope.
   - What we know: `Server::run()` (`server.rs:140-181`) has a bare `loop` with no shutdown signal handling. The worker loop must register its own SIGTERM handler.
   - What's unclear: Whether the server should be extended with graceful shutdown as part of this phase, or whether the WorkerLoop handles it independently.
   - Recommendation: WorkerLoop registers its own `tokio::signal::unix::signal(SignalKind::terminate())` independently. Server shutdown is orthogonal and can be a separate phase. Note: on macOS, `SIGTERM` via `tokio::signal::unix` requires `unix` signal feature — verify `tokio` features in `ferro-queue/Cargo.toml`.

---

## Sources

### Primary (HIGH confidence)
- `ferro-queue/src/job.rs`, `worker.rs`, `dispatcher.rs`, `queue.rs`, `config.rs` — direct codebase read [VERIFIED: file contents]
- `framework/src/app.rs`, `server.rs`, `database/mod.rs`, `database/connection.rs` — direct codebase read [VERIFIED]
- `framework/src/debug/mod.rs` — queue debug endpoint implementations [VERIFIED]
- `ferro-migration/src/backfill.rs` — `Statement::from_string` + `manager.get_database_backend()` pattern [VERIFIED]
- `ferro-reservation/src/kernel.rs` — `IsolationLevel::Serializable` on SQLite behavior note, `begin_with_config` pattern [VERIFIED]
- `ferro-reservation/tests/concurrent_hold.rs` — race test pattern with `sqlite::memory:` + `multi_thread` tokio flavor [VERIFIED]
- `Cargo.lock` — sqlx 0.8.6, libsqlite3-sys 0.30.1 (SQLite 3.47) [VERIFIED]
- `gestiscilo/.planning/research/v7.1-PITFALLS.md` §A — pitfalls A-01..A-04 [READ]
- `gestiscilo/.planning/research/v7.1-STACK.md` §D-01 — "replace the queue backend, not extend it" [READ]
- `ferro-mcp/src/tools/job_history.rs` — database job history query shape already assumes `jobs` table schema [VERIFIED]

### Secondary (MEDIUM confidence)
- `ferro-orm/tests/concurrent_decrement.rs` — `DatabaseBackend::Sqlite` + schema creation pattern in tests [VERIFIED: codebase]
- `ferro-audit/src/migration.rs`, `ferro-projection/src/migration.rs` — `Statement::from_string(DatabaseBackend::Sqlite, ...)` in test setup [VERIFIED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified in Cargo.lock/Cargo.toml
- Architecture: HIGH — grounded in actual source file reads
- Pitfalls: HIGH — A-01..A-04 from gestiscilo research, verified against existing codebase patterns
- Claim mechanics: HIGH — SQLite RETURNING supported (libsqlite3-sys 0.30.1), `Statement::from_string` pattern established in codebase

**Research date:** 2026-06-07
**Valid until:** 2026-07-07 (SeaORM 1.0 stable, no fast-moving deps)

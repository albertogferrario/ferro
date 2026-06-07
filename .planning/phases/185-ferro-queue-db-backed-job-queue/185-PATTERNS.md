# Phase 185: ferro::queue — DB-Backed Job Queue - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 11 new/modified files
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-queue/src/db.rs` (new) | service | CRUD + event-driven | `ferro-migration/src/backfill.rs` + `ferro-reservation/src/kernel.rs` | role-match |
| `ferro-queue/src/migration.rs` (new) | migration | batch | `ferro-audit/src/migration.rs` + `ferro-projection/src/migration.rs` | exact |
| `ferro-queue/src/worker.rs` (refactor) | service | event-driven | `ferro-queue/src/worker.rs` (current) | exact |
| `ferro-queue/src/dispatcher.rs` (refactor) | utility | request-response | `ferro-queue/src/dispatcher.rs` (current) | exact |
| `ferro-queue/src/config.rs` (refactor) | config | — | `ferro-queue/src/config.rs` (current) | exact |
| `ferro-queue/src/job.rs` (refactor) | model | — | `ferro-queue/src/job.rs` (current) | exact |
| `ferro-queue/src/queue.rs` (delete, replaced) | service | — | `ferro-queue/src/queue.rs` (current — delete entirely) | exact |
| `ferro-queue/src/error.rs` (refactor) | utility | — | `ferro-queue/src/error.rs` (current) | exact |
| `ferro-queue/src/lib.rs` (refactor) | config | — | `ferro-queue/src/lib.rs` (current) | exact |
| `framework/src/lib.rs` (refactor lines 194-199) | config | — | `framework/src/lib.rs` (current flat re-exports) | exact |
| `framework/src/debug/mod.rs` (refactor) | service | request-response | `framework/src/debug/mod.rs` (current) | exact |
| `ferro-mcp/src/tools/job_history.rs` (refactor) | utility | request-response | `ferro-mcp/src/tools/job_history.rs` (current) | exact |
| `ferro-queue/tests/race_claim_sqlite.rs` (new) | test | event-driven | `ferro-reservation/tests/concurrent_hold.rs` | exact |
| `ferro-queue/tests/race_claim_postgres.rs` (new) | test | event-driven | `ferro-reservation/tests/concurrent_hold_postgres.rs` | exact |
| reaper/poison tests (inline `#[cfg(test)]` in `ferro-queue/src/db.rs`) | test | batch | `ferro-reservation/src/kernel.rs` inline tests | role-match |

---

## Pattern Assignments

### `ferro-queue/src/db.rs` (new — DB claim path, reaper, enqueue, fail, delete)

**Primary analog:** `ferro-migration/src/backfill.rs` (Statement::from_string pattern)
**Secondary analog:** `ferro-reservation/src/kernel.rs` (DatabaseConnection ownership, state-machine transitions)

**Imports pattern** (`ferro-migration/src/backfill.rs` lines 1-6):
```rust
use sea_orm::{DbBackend, DbErr, Statement};
use sea_orm_migration::prelude::*;
```

For `db.rs` the equivalent is:
```rust
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use crate::error::Error;
```

**Backend-branch pattern** (`ferro-migration/src/backfill.rs` lines 43-57):
```rust
match backend {
    DbBackend::Sqlite => Ok(format!(
        "UPDATE \"{table}\" SET ..."
    )),
    DbBackend::Postgres => Ok(format!(
        "UPDATE \"{table}\" SET ..."
    )),
    DbBackend::MySql => Err(Error::UnsupportedBackend(...)),
}
```

**Statement execution pattern** (`ferro-migration/src/backfill.rs` lines 26-30):
```rust
manager
    .get_connection()
    .execute(Statement::from_string(backend, sql))
    .await
    .map(|_| ())
```

In `db.rs` use `conn.execute(...)` and `conn.query_one(...)` directly (no SchemaManager).

**Serializable transaction + state transition pattern** (`ferro-reservation/src/kernel.rs` lines 69-79, 195-205):
```rust
// SQLite: BEGIN IMMEDIATE via raw Statement (NOT begin_with_config Serializable —
// Serializable on SQLite does NOT emit BEGIN IMMEDIATE, per kernel.rs comment lines 69-79)
conn.execute(Statement::from_string(
    DatabaseBackend::Sqlite,
    "BEGIN IMMEDIATE".to_string(),
)).await.map_err(Error::Db)?;
// ... UPDATE ... RETURNING ...
conn.execute(Statement::from_string(
    DatabaseBackend::Sqlite,
    "COMMIT".to_string(),
)).await.map_err(Error::Db)?;

// Postgres: transaction with FOR UPDATE SKIP LOCKED
let txn = conn.begin().await.map_err(Error::Db)?;
// SELECT ... FOR UPDATE SKIP LOCKED
// UPDATE jobs SET status='claimed' ...
txn.commit().await.map_err(Error::Db)?;
```

**Error mapping convention** (`ferro-reservation/src/kernel.rs` lines 141-151):
```rust
.await.map_err(|e| {
    // translate specific DB errors to domain errors
    ReservationError::Db(e)
})?;
```

---

### `ferro-queue/src/migration.rs` (new — CreateJobsTable portable migration)

**Primary analog:** `ferro-audit/src/migration.rs` (full file — exact structure to copy)
**Secondary analog:** `ferro-projection/src/migration.rs` (manual `MigrationName` impl)

**File structure** (`ferro-audit/src/migration.rs` lines 16-101):
```rust
use sea_orm_migration::prelude::*;

// Option A: DeriveMigrationName macro (ferro-audit pattern)
#[derive(DeriveMigrationName)]
pub struct Migration;

// Option B: manual MigrationName impl (ferro-projection pattern, lines 17-21)
pub struct CreateJobsTable;

impl sea_orm_migration::MigrationName for CreateJobsTable {
    fn name(&self) -> &str { "m_create_jobs_table" }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateJobsTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Jobs::Table)
                .if_not_exists()
                .col(ColumnDef::new(Jobs::Id).big_integer().not_null().auto_increment().primary_key())
                // ... all columns
                .to_owned()
        ).await?;

        // Indexes follow create_table (ferro-audit lines 51-77)
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

        // additional indexes...
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Jobs::Table).to_owned()).await
    }
}

// DeriveIden enum for column names (ferro-audit lines 86-101)
#[derive(DeriveIden)]
enum Jobs {
    Table,
    Id,
    Queue,
    JobType,
    Payload,
    Status,
    Attempts,
    MaxRetries,
    IdempotencyKey,
    TenantId,
    AvailableAt,
    ClaimedAt,
    ClaimedBy,
    Error,
    CreatedAt,
}
```

**Migration test pattern** (`ferro-audit/src/migration.rs` lines 103-176):
```rust
#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;
    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    #[tokio::test]
    async fn migration_creates_table_and_indexes() {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("run migration up");

        let table_row = conn.query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table' AND name='audit_log'".to_string(),
        )).await.expect("query sqlite_master for table");
        assert!(table_row.is_some(), "table not created by migration");
    }
}
```

**CRITICAL:** The `down()` method must drop indexes before the table or SeaORM generates incorrect SQL. See `ferro-audit/src/migration.rs` line 79 — `drop_table` only; indexes are dropped automatically by Postgres/SQLite when the table is dropped. No separate `drop_index` calls needed.

---

### `ferro-queue/src/worker.rs` (refactor — WorkerLoop replacing Worker)

**Analog:** `ferro-queue/src/worker.rs` (current, full file — reuse structure, replace innards)

**Preserved types** (current `worker.rs` lines 20-27, 31-67, 70-71):
```rust
// TenantScopeProvider trait — copy verbatim (lines 20-27)
#[async_trait]
pub trait TenantScopeProvider: Send + Sync {
    async fn with_scope(
        &self,
        tenant_id: i64,
        f: Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>,
    ) -> Result<(), Error>;
}

// WorkerConfig — keep fields, add visibility_timeout + worker_id
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub queues: Vec<String>,
    pub max_jobs: usize,
    pub sleep_duration: Duration,
    pub stop_on_error: bool,
    // NEW: pub visibility_timeout: Duration,  // default 5min
    // NEW: pub worker_id: String,             // uuid-based
}

// JobHandler type alias — copy verbatim (line 70-71)
type JobHandler =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>;
```

**Job handler registration pattern** (current `worker.rs` lines 120-135):
```rust
pub fn register<J>(&mut self)
where
    J: Job + serde::de::DeserializeOwned + 'static,
{
    let type_name = std::any::type_name::<J>().to_string();
    let handler: JobHandler = Arc::new(move |data: String| {
        Box::pin(async move {
            let job: J = serde_json::from_str(&data)
                .map_err(|e| Error::DeserializationFailed(e.to_string()))?;
            job.handle().await
        })
    });
    self.handlers.insert(type_name, handler);
}
```

**Semaphore + spawn pattern** (current `worker.rs` lines 202-212):
```rust
async fn process_job(&self, payload: JobPayload) -> Result<(), Error> {
    let permit = self.semaphore.clone().acquire_owned().await.unwrap();
    let handlers = self.handlers.clone();
    let job_type = payload.job_type.clone();
    let tenant_scope = self.tenant_scope.clone();
    let tenant_id = payload.tenant_id;

    tokio::spawn(async move {
        let _permit = permit; // Hold permit until job completes
        // ... dispatch to handler, tenant scope wrap
    });
    Ok(())
}
```

**Tenant scope dispatch pattern** (current `worker.rs` lines 232-238):
```rust
let job_result = match (&tenant_scope, tenant_id) {
    (Some(scope), Some(id)) => {
        let job_fut = Box::pin(handler(payload.data.clone()));
        scope.with_scope(id, job_fut).await
    }
    _ => handler(payload.data.clone()).await,
};
```

**NEW: Panic isolation** — add around handler dispatch (does not exist in current worker.rs — this is a gap to fill per D-11 and PITFALLS A-01):
```rust
use futures::FutureExt;
use std::panic::AssertUnwindSafe;

// In the spawned task:
let result = AssertUnwindSafe(handler(payload.data.clone()))
    .catch_unwind()
    .await;
match result {
    Err(_panic) => { /* count as failed attempt */ }
    Ok(Err(e)) => { /* normal error path */ }
    Ok(Ok(())) => { /* success: delete row */ }
}
```

**Shutdown signal pattern** — current `worker.rs` uses `Arc<tokio::sync::Notify>` (line 84). New version must add SIGTERM:
```rust
// Pattern from RESEARCH.md Pattern 6:
use tokio::signal::unix::{signal, SignalKind};
use std::sync::atomic::{AtomicBool, Ordering};

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
```

**Shutdown drain pattern** (current `worker.rs` lines 172-173):
```rust
// Wait for all in-flight jobs to complete
let _ = self.semaphore.acquire_many(self.config.max_jobs as u32).await;
```

---

### `ferro-queue/src/dispatcher.rs` (refactor — swap Redis sink for DB)

**Analog:** `ferro-queue/src/dispatcher.rs` (current, full file)

**Preserved verbatim** (lines 1-171 minus `dispatch_to_queue`):
- `TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>>` (line 11)
- `register_tenant_capture_hook` (lines 18-20)
- `PendingDispatch<J>` struct and all builder methods: `new`, `on_queue`, `delay`, `for_tenant`, `captured_tenant_id`, `dispatch`, `dispatch_immediately`, `dispatch_now` (lines 26-171)
- `QueueConfig::is_sync_mode()` check in `dispatch()` (line 86)
- All three free functions: `dispatch`, `dispatch_to`, `dispatch_later` (lines 192-215)

**Changed:** `dispatch_to_queue` (current lines 131-144) — swap `Queue::connection()` (which returns `&QueueConnection` Redis handle) for `Queue::connection()` (which will return `&DatabaseConnection` after refactor). The function body writes to `jobs` table via `Statement::from_string` instead of `conn.push(payload)`.

**Global OnceLock pattern** (current `queue.rs` lines 428-448, 450):
```rust
static GLOBAL_CONNECTION: std::sync::OnceLock<QueueConnection> = std::sync::OnceLock::new();

pub struct Queue;
impl Queue {
    pub fn connection() -> &'static QueueConnection {
        GLOBAL_CONNECTION.get().expect("Queue not initialized. Call Queue::init() first.")
    }
    pub async fn init(config: QueueConfig) -> Result<(), Error> {
        let conn = QueueConnection::new(config).await?;
        GLOBAL_CONNECTION.set(conn).map_err(|_| Error::custom("Queue already initialized"))?;
        Ok(())
    }
    pub fn is_initialized() -> bool {
        GLOBAL_CONNECTION.get().is_some()
    }
}
```

The new `queue.rs` / `db.rs` replaces `OnceLock<QueueConnection>` with `OnceLock<DatabaseConnection>` but preserves the `Queue::connection()`, `Queue::init()`, `Queue::is_initialized()` signatures (the last two change their parameter types but the method names stay — see Pitfall 6 in RESEARCH.md).

---

### `ferro-queue/src/config.rs` (refactor — drop Redis fields, add visibility_timeout)

**Analog:** `ferro-queue/src/config.rs` (current, full file)

**Preserved pattern** (lines 107-115 — keep `is_sync_mode()`):
```rust
pub fn is_sync_mode() -> bool {
    env::var("QUEUE_CONNECTION")
        .map(|v| v.to_lowercase() == "sync")
        .unwrap_or(true) // Default to sync for development
}
```

**Preserved builder pattern** (lines 117-138 — consuming `mut self -> Self`):
```rust
pub fn default_queue(mut self, queue: impl Into<String>) -> Self {
    self.default_queue = queue.into();
    self
}
pub fn max_concurrent_jobs(mut self, count: usize) -> Self {
    self.max_concurrent_jobs = count;
    self
}
```

**Delete:** All Redis-specific fields and methods: `redis_url`, `prefix`, `block_timeout`, `delayed_job_poll_interval`, `build_redis_url()`, `queue_key()`, `delayed_key()`, `reserved_key()`, `failed_key()`.

**Add:** `default_queue: String`, `max_concurrent_jobs: usize`, `sleep_duration: Duration`, `visibility_timeout: Duration` (default 5min).

---

### `ferro-queue/src/job.rs` (refactor — add `idempotency_key`)

**Analog:** `ferro-queue/src/job.rs` (current, full file — minimal changes)

**Preserved verbatim:** Entire `Job` trait (lines 43-72), `JobPayload` struct (lines 75-100), all `JobPayload` methods (lines 102-168), all tests (lines 170-265).

**Add to `Job` trait** (after `timeout()` at line 71):
```rust
/// Idempotency key for deduplication on enqueue.
///
/// When `Some`, enqueue skips insertion if a pending or claimed row with
/// the same `(job_type, idempotency_key)` already exists (D-15).
fn idempotency_key(&self) -> Option<String> {
    None
}
```

**Change in `retry_delay` default** — current (line 58-61) returns `Duration::from_secs(5)` flat; new default implements full jitter (D-13):
```rust
fn retry_delay(&self, attempt: u32) -> std::time::Duration {
    // Full jitter: rand(0 .. min(cap, base * 2^attempt))
    // Base 5s, factor 2^attempt, cap 15min
    use rand::Rng;
    let base_secs: u64 = 5;
    let cap_secs: u64 = 15 * 60;
    let max_delay = cap_secs.min(base_secs.saturating_mul(2u64.saturating_pow(attempt)));
    let jitter = rand::thread_rng().gen_range(0..=max_delay);
    std::time::Duration::from_secs(jitter)
}
```

---

### `ferro-queue/src/error.rs` (refactor — replace Redis variant with Db)

**Analog:** `ferro-queue/src/error.rs` (current, full file)

**Preserved pattern** (lines 3-114 — thiserror derive, factory methods):
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("...")] ConnectionFailed(String),
    #[error("...")] SerializationFailed(String),
    #[error("...")] DeserializationFailed(String),
    // ... keep all non-Redis variants
    #[error("Tenant not found for job: tenant_id={tenant_id}")]
    TenantNotFound { tenant_id: i64 },
    #[error("{0}")] Custom(String),
}

impl Error {
    pub fn job_failed(job: impl Into<String>, message: impl Into<String>) -> Self { ... }
    pub fn tenant_not_found(id: i64) -> Self { ... }
    pub fn custom(message: impl Into<String>) -> Self { ... }
}
```

**Delete:** `Redis(#[from] redis::RedisError)` variant (line 58-59).

**Add:**
```rust
#[error("Database error: {0}")]
Db(#[from] sea_orm::DbErr),

#[error("Unsupported database backend")]
UnsupportedBackend,
```

---

### `ferro-queue/src/lib.rs` (refactor — namespaced re-exports, add migration)

**Analog:** `ferro-queue/src/lib.rs` (current, full file)

**Module declarations** (current lines 45-51 — delete `queue` module, add `db` and `migration`):
```rust
mod config;
mod dispatcher;
mod error;
mod job;
mod db;        // NEW: replaces queue.rs
mod migration; // NEW
mod worker;
```

**Public exports** (current lines 52-94):
- Keep: `QueueConfig`, dispatcher free functions, `Error`, `Job`, `JobPayload`, `TenantScopeProvider`, `Worker`, `WorkerConfig`, `Queueable` blanket impl
- Keep: `FailedJobInfo`, `JobInfo`, `JobState`, `Queue`, `QueueStats`, `SingleQueueStats` (from `db.rs` now, not `queue.rs`)
- Remove: `QueueConnection` (deleted with Redis backend)
- Add: `pub use migration::CreateJobsTable;`
- Remove: `QueueConfig` re-export of Redis connection field

**`Queueable` blanket trait** (lines 67-94 — copy verbatim):
```rust
pub trait Queueable: Job + serde::Serialize + serde::de::DeserializeOwned {
    fn dispatch(self) -> PendingDispatch<Self> where Self: Sized {
        PendingDispatch::new(self)
    }
    fn delay(self, duration: std::time::Duration) -> PendingDispatch<Self> where Self: Sized {
        PendingDispatch::new(self).delay(duration)
    }
    fn on_queue(self, queue: &'static str) -> PendingDispatch<Self> where Self: Sized {
        PendingDispatch::new(self).on_queue(queue)
    }
}
impl<T> Queueable for T where T: Job + serde::Serialize + serde::de::DeserializeOwned {}
```

---

### `framework/src/lib.rs` lines 194-199 (refactor — namespaced `pub mod queue`)

**Analog:** `framework/src/lib.rs` (current flat re-exports at lines 194-199)

**Current (delete):**
```rust
pub use ferro_queue::{
    dispatch as queue_dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook,
    Error as QueueError, Job, JobPayload, PendingDispatch, Queue, QueueConfig, QueueConnection,
    Queueable, TenantScopeProvider, Worker, WorkerConfig,
};
```

**Replacement (D-02 — namespaced module):**
```rust
/// Background job queue. Use `ferro::queue::Job`, `ferro::queue::dispatch`, etc.
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook,
        CreateJobsTable, Error, FailedJobInfo, Job, JobInfo, JobPayload,
        JobState, PendingDispatch, Queue, QueueConfig, QueueStats, Queueable,
        SingleQueueStats, TenantScopeProvider, Worker, WorkerConfig,
    };
}
```

Pattern for `pub mod` wrapping a crate re-export: check `framework/src/lib.rs` line 9 area — the framework already uses `pub mod` for its own submodules. The `pub mod queue { pub use ferro_queue::... }` pattern is the idiomatic way to create a namespaced re-export without a separate file.

---

### `framework/src/debug/mod.rs` (refactor — queue endpoints read from DB)

**Analog:** `framework/src/debug/mod.rs` (current, lines 173-267)

**Preserved pattern** (lines 173-198, 228-252 — debug guard + initialized check):
```rust
pub async fn handle_queue_jobs() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(DebugErrorResponse { ... }, 403);
    }
    if !ferro_queue::Queue::is_initialized() {
        return json_response(DebugErrorResponse {
            success: false,
            error: "Queue not initialized ...".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }, 503);
    }
    // ... fetch and respond
}
```

**Changed:** Replace `conn.get_pending_jobs(...)`, `conn.get_delayed_jobs(...)`, `conn.get_failed_jobs(...)` (Redis-backed) with equivalent DB-backed calls on the new `Queue::connection()` which returns `&DatabaseConnection`. The `get_pending_jobs`, `get_delayed_jobs`, `get_stats` methods move to `db.rs` and accept a `&DatabaseConnection`.

**`Queue::is_initialized()` contract preserved** — must keep this method in `queue.rs`/`db.rs` (see RESEARCH.md Pitfall 6).

---

### `ferro-mcp/src/tools/job_history.rs` (refactor — fix failed_jobs query)

**Analog:** `ferro-mcp/src/tools/job_history.rs` (current, full file)

**Preserved pattern** (lines 75-193 — `get_database_job_history` structure):
```rust
async fn get_database_job_history(
    project_root: &Path,
    queue_filter: Option<&str>,
    limit: usize,
) -> Result<JobHistoryInfo> {
    let db: DatabaseConnection = Database::connect(&database_url).await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    let rows = db.query_all(Statement::from_string(
        db.get_database_backend(),
        query_string,
    )).await;
    // ...row mapping to JobInfo structs
}
```

**Changed:** failed jobs query (current lines 130-136) queries `FROM failed_jobs` — must change to:
```rust
// OLD (delete):
"SELECT * FROM failed_jobs WHERE queue = '{queue}' ORDER BY failed_at DESC LIMIT {limit}"

// NEW:
"SELECT * FROM jobs WHERE status = 'failed' \
 AND queue = '{queue}' ORDER BY created_at DESC LIMIT {limit}"
```

**Add `error` column to `FailedJobInfo`** (current struct lines 28-35 lacks it):
```rust
pub struct FailedJobInfo {
    pub id: String,
    pub queue: String,
    pub job_type: String,
    pub payload_preview: String,
    pub exception: String,  // keep existing field name for API compat
    pub failed_at: String,
    // Map from jobs.error column in the query
}
```

In the row mapping, `exception` is populated from `row.try_get_by("error")` (was `row.try_get_by("exception")` on `failed_jobs`).

---

### `ferro-queue/tests/race_claim_sqlite.rs` (new — SC-1 proof artifact)

**Primary analog:** `ferro-reservation/tests/concurrent_hold.rs` (full file — structure to copy)

**Test harness pattern** (`concurrent_hold.rs` lines 96-135):
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_workers_claim_each_job_exactly_once() {
    // CRITICAL: use NamedTempFile, NOT sqlite::memory: (RESEARCH.md Pitfall 1)
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn1 = Database::connect(&db_url).await.unwrap();
    let conn2 = Database::connect(&db_url).await.unwrap();

    // Run migration on conn1 (both see same file)
    // CreateJobsTable.up(...).await.unwrap();

    // Enqueue N jobs
    // Spawn two concurrent claim tasks on conn1 and conn2
    // Assert: union of claimed IDs has no duplicates, covers all N jobs
}
```

**Concurrent task pattern** (`concurrent_hold.rs` lines 111-134):
```rust
let mut handles = Vec::with_capacity(2);
for _ in 0..2 {
    let conn = conn.clone(); // DatabaseConnection is Clone
    handles.push(tokio::spawn(async move {
        // claim loop
    }));
}
// join all handles
for h in handles {
    match h.await.expect("join") {
        Ok(id) => { ... }
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}
```

---

### `ferro-queue/tests/race_claim_postgres.rs` (new — SC-1b cfg-gated)

**Primary analog:** `ferro-reservation/tests/concurrent_hold_postgres.rs` (full file — copy structure exactly)

**Cfg gate** (line 27 — copy verbatim):
```rust
#![cfg(feature = "postgres-tests")]
```

**Feature declaration in Cargo.toml** (`ferro-reservation/Cargo.toml` lines 32-35):
```toml
[features]
sqlx-postgres = ["sea-orm/sqlx-postgres", "dep:sqlx"]
postgres-tests = ["sqlx-postgres"]
```

**Graceful DATABASE_URL skip pattern** (`concurrent_hold_postgres.rs` lines 63-71, 127-129):
```rust
async fn fresh_pg_db() -> Option<DatabaseConnection> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let conn = Database::connect(&url).await.expect("connect to postgres");
    let _ = TestMigrator::down(&conn, None).await;
    TestMigrator::up(&conn, None).await.expect("migrate");
    Some(conn)
}

// In test:
if std::env::var("DATABASE_URL").is_err() {
    eprintln!("DATABASE_URL not set — skipping postgres race test");
    return;
}
```

**`--test-threads=1` requirement** (`concurrent_hold_postgres.rs` lines 9-18 — doc comment to copy):
```
//! Run with:
//!   DATABASE_URL=postgres://... \
//!     cargo test -p ferro-queue --features postgres-tests \
//!     -- --test-threads=1
```

---

## Shared Patterns

### Global OnceLock Initialization
**Source:** `ferro-queue/src/queue.rs` lines 428-450
**Apply to:** `ferro-queue/src/db.rs` (or inline in `queue.rs` replacement)
```rust
static GLOBAL_CONNECTION: std::sync::OnceLock<DatabaseConnection> = std::sync::OnceLock::new();

pub struct Queue;
impl Queue {
    pub fn connection() -> &'static DatabaseConnection {
        GLOBAL_CONNECTION.get().expect("Queue not initialized. Call Queue::init() first.")
    }
    pub async fn init(conn: sea_orm::DatabaseConnection) -> Result<(), Error> {
        GLOBAL_CONNECTION.set(conn).map_err(|_| Error::custom("Queue already initialized"))?;
        Ok(())
    }
    pub fn is_initialized() -> bool {
        GLOBAL_CONNECTION.get().is_some()
    }
}
```
Note: `Queue::init` signature changes from `(config: QueueConfig)` to `(conn: DatabaseConnection)` per D-03.

### Statement::from_string Backend Branch
**Source:** `ferro-migration/src/backfill.rs` lines 23-30 + 43-57
**Apply to:** `ferro-queue/src/db.rs` claim path, reaper, enqueue
```rust
let backend = conn.get_database_backend();
let sql = match backend {
    DatabaseBackend::Sqlite => "... SQLite SQL ...",
    DatabaseBackend::Postgres => "... Postgres SQL ...",
    _ => return Err(Error::UnsupportedBackend),
};
conn.execute(Statement::from_string(backend, sql.to_string())).await.map_err(Error::Db)?
```

### Tenant ID Capture Hook
**Source:** `ferro-queue/src/dispatcher.rs` lines 11-20, 73-76
**Apply to:** `ferro-queue/src/dispatcher.rs` (preserve unchanged)
```rust
static TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>> = OnceLock::new();

pub fn register_tenant_capture_hook(f: fn() -> Option<i64>) {
    let _ = TENANT_ID_HOOK.set(f);
}

fn captured_tenant_id(&self) -> Option<i64> {
    self.tenant_id
        .or_else(|| TENANT_ID_HOOK.get().and_then(|f| f()))
}
```

### thiserror Error Enum
**Source:** `ferro-queue/src/error.rs` lines 3-113
**Apply to:** `ferro-queue/src/error.rs` (refactor in place — drop Redis variant, add Db variant)
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // ... keep existing variants except Redis
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),
}
```

### Consuming `with_*` Builder Pattern
**Source:** `ferro-queue/src/config.rs` lines 117-138, `ferro-queue/src/job.rs` lines 123-137
**Apply to:** `WorkerConfig` new fields, `QueueConfig` refactored struct
```rust
pub fn with_visibility_timeout(mut self, d: Duration) -> Self {
    self.visibility_timeout = d;
    self
}
```

### Migration Struct Export for Consumers
**Source:** `ferro-audit/src/migration.rs` lines 6-14 (doc comment showing consumer usage)
**Apply to:** `ferro-queue/src/migration.rs` + `ferro-queue/src/lib.rs`
```rust
// Consumers register in their Migrator:
// vec![
//     Box::new(ferro_queue::CreateJobsTable),
//     // ...
// ]
pub use migration::CreateJobsTable;
```

### Semaphore-Based Concurrency Limit
**Source:** `ferro-queue/src/worker.rs` lines 82, 91-92, 202-204, 172-173
**Apply to:** `ferro-queue/src/worker.rs` (WorkerLoop — preserve this pattern)
```rust
let semaphore = Arc::new(Semaphore::new(config.max_jobs));
// Per job spawn:
let permit = self.semaphore.clone().acquire_owned().await.unwrap();
tokio::spawn(async move { let _permit = permit; /* job */ });
// Shutdown drain:
let _ = self.semaphore.acquire_many(self.config.max_jobs as u32).await;
```

### Debug Endpoint Guard Pattern
**Source:** `framework/src/debug/mod.rs` lines 39-47, 61-70
**Apply to:** `framework/src/debug/mod.rs` queue handlers (unchanged structure)
```rust
if !is_debug_enabled() {
    return json_response(DebugErrorResponse { success: false, error: "...", timestamp: ... }, 403);
}
if !ferro_queue::Queue::is_initialized() {
    return json_response(DebugErrorResponse { ... }, 503);
}
```

---

## Anti-Patterns Documented (from RESEARCH.md)

| Do Not | Do Instead | Source |
|---|---|---|
| `Database::connect("sqlite::memory:")` in race test | `tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc` | RESEARCH.md Pitfall 1 |
| `begin_with_config(Serializable)` for SQLite claim | `Statement::from_string(Sqlite, "BEGIN IMMEDIATE")` | RESEARCH.md Pitfall 2, `ferro-reservation/src/kernel.rs` lines 69-79 |
| `std::panic::catch_unwind` on async futures | `futures::FutureExt::catch_unwind` + `AssertUnwindSafe` | RESEARCH.md Pitfall 3 |
| `FOR UPDATE SKIP LOCKED` in migration DDL | Only in claim runtime SQL | RESEARCH.md / CONTEXT D-06 |
| `2u64.pow(attempt)` retry with no jitter | `rand::thread_rng().gen_range(0..min(cap, base*2^attempt))` | `worker.rs` line 253 (current gap) |

---

## Metadata

**Analog search scope:** `ferro-queue/src/`, `ferro-migration/src/`, `ferro-reservation/src/`, `ferro-reservation/tests/`, `ferro-audit/src/`, `ferro-projection/src/`, `framework/src/`, `ferro-mcp/src/tools/`
**Files scanned:** 15
**Pattern extraction date:** 2026-06-07

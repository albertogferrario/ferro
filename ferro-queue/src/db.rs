//! DB engine for ferro-queue: dual-backend atomic claim, idempotent enqueue,
//! stuck-job reaper, job lifecycle ops, and the `Queue` global.
//!
//! Supports SQLite (raw `BEGIN IMMEDIATE` claim) and Postgres
//! (`FOR UPDATE SKIP LOCKED` inside a transaction). All dynamic values are
//! bound via `Statement::from_sql_and_values` — no string interpolation of
//! caller-supplied data (T-185-01).

use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait, Value,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

// ---------------------------------------------------------------------------
// Queue global
// ---------------------------------------------------------------------------

static GLOBAL_CONNECTION: std::sync::OnceLock<DatabaseConnection> = std::sync::OnceLock::new();

/// Registered job-type applier functions, collected before the server starts.
type RegisterFn = Box<dyn Fn(&mut crate::WorkerLoop) + Send + Sync>;
static JOB_REGISTRARS: std::sync::Mutex<Vec<RegisterFn>> =
    std::sync::Mutex::new(Vec::new());

/// Global handle to the queue's database connection.
///
/// Initialise once at application start with [`Queue::init`]; worker loops
/// and dispatcher call [`Queue::connection`] to get the static reference.
pub struct Queue;

impl Queue {
    /// Return a reference to the global `DatabaseConnection`.
    ///
    /// # Panics
    ///
    /// Panics if [`Queue::init`] has not been called yet.
    pub fn connection() -> &'static DatabaseConnection {
        GLOBAL_CONNECTION
            .get()
            .expect("Queue not initialized. Call Queue::init() first.")
    }

    /// Store `conn` as the global connection.
    ///
    /// Returns `Err` if called more than once.
    pub async fn init(conn: DatabaseConnection) -> Result<(), Error> {
        GLOBAL_CONNECTION
            .set(conn)
            .map_err(|_| Error::custom("Queue already initialized"))?;
        Ok(())
    }

    /// Returns `true` if [`Queue::init`] has been called.
    pub fn is_initialized() -> bool {
        GLOBAL_CONNECTION.get().is_some()
    }

    /// Register a job type for auto-start by the framework's WorkerLoop.
    ///
    /// Call this in your application bootstrap before the server starts.
    /// The framework's server boot path inspects [`Queue::has_registered_jobs`]
    /// and spawns a `WorkerLoop` automatically when at least one type is registered.
    pub fn register<J>()
    where
        J: crate::Job + serde::de::DeserializeOwned + 'static,
    {
        JOB_REGISTRARS
            .lock()
            .unwrap()
            .push(Box::new(|w: &mut crate::WorkerLoop| w.register::<J>()));
    }

    /// Returns `true` if at least one job type has been registered via [`Queue::register`].
    pub fn has_registered_jobs() -> bool {
        !JOB_REGISTRARS.lock().unwrap().is_empty()
    }

    /// Apply all registered job types to the given `WorkerLoop`.
    ///
    /// Used internally by [`WorkerLoop::from_registry`].
    pub(crate) fn apply_registrars(w: &mut crate::WorkerLoop) {
        for r in JOB_REGISTRARS.lock().unwrap().iter() {
            r(w);
        }
    }
}

// ---------------------------------------------------------------------------
// JobRow — a claimed row from the jobs table
// ---------------------------------------------------------------------------

/// A row read from the `jobs` table during a claim operation.
#[derive(Debug, Clone)]
pub struct JobRow {
    /// Primary key.
    pub id: i64,
    /// Fully-qualified type name of the job (used to route to the correct handler).
    pub job_type: String,
    /// JSON-serialized job data.
    pub payload: String,
    /// Name of the queue this job belongs to.
    pub queue: String,
    /// Number of execution attempts so far.
    pub attempts: u32,
    /// Maximum attempts before the job is parked as failed.
    pub max_retries: u32,
    /// Optional deduplication key.
    pub idempotency_key: Option<String>,
    /// Optional tenant scope.
    pub tenant_id: Option<i64>,
    /// Earliest time the job may be claimed.
    pub available_at: DateTime<Utc>,
    /// Insertion timestamp.
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Introspection types (moved from queue.rs)
// ---------------------------------------------------------------------------

/// Job status for introspection queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Waiting to be claimed.
    Pending,
    /// Scheduled for a future time.
    Delayed,
    /// Permanently failed after exhausting retries.
    Failed,
}

/// Summary of a single job for introspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    /// Primary key.
    pub id: i64,
    /// Fully-qualified job type name.
    pub job_type: String,
    /// Queue name.
    pub queue: String,
    /// Execution attempt count.
    pub attempts: u32,
    /// Maximum retry count.
    pub max_retries: u32,
    /// Insertion timestamp (RFC 3339).
    pub created_at: String,
    /// Earliest eligible claim time (RFC 3339).
    pub available_at: String,
    /// Logical state.
    pub state: JobState,
}

/// Per-queue pending/delayed counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleQueueStats {
    /// Queue name.
    pub name: String,
    /// Jobs with `status='pending'` and `available_at <= now`.
    pub pending: usize,
    /// Jobs with `status='pending'` and `available_at > now`.
    pub delayed: usize,
}

/// Aggregate stats across all queues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStats {
    /// Per-queue breakdown.
    pub queues: Vec<SingleQueueStats>,
    /// Total jobs with `status='failed'`.
    pub total_failed: usize,
}

/// A failed job with the error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedJobInfo {
    /// Core job fields.
    pub job: JobInfo,
    /// Error message stored at failure time.
    pub error: String,
    /// When the job was parked as failed (same as `created_at` for reaper-parked).
    pub failed_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Map a `QueryResult` row to a `JobRow`.
fn parse_job_row(row: &sea_orm::QueryResult) -> Result<JobRow, Error> {
    let id: i64 = row
        .try_get_by::<i64, _>("id")
        .map_err(|e| Error::custom(format!("parse id: {e}")))?;
    let job_type: String = row
        .try_get_by::<String, _>("job_type")
        .map_err(|e| Error::custom(format!("parse job_type: {e}")))?;
    let payload: String = row
        .try_get_by::<String, _>("payload")
        .map_err(|e| Error::custom(format!("parse payload: {e}")))?;
    let queue: String = row
        .try_get_by::<String, _>("queue")
        .map_err(|e| Error::custom(format!("parse queue: {e}")))?;
    let attempts: i32 = row
        .try_get_by::<i32, _>("attempts")
        .map_err(|e| Error::custom(format!("parse attempts: {e}")))?;
    let max_retries: i32 = row
        .try_get_by::<i32, _>("max_retries")
        .map_err(|e| Error::custom(format!("parse max_retries: {e}")))?;
    let idempotency_key: Option<String> = row
        .try_get_by::<Option<String>, _>("idempotency_key")
        .map_err(|e| Error::custom(format!("parse idempotency_key: {e}")))?;
    let tenant_id: Option<i64> = row
        .try_get_by::<Option<i64>, _>("tenant_id")
        .map_err(|e| Error::custom(format!("parse tenant_id: {e}")))?;

    // available_at / created_at — SQLite stores as ISO-8601 text, Postgres as timestamptz.
    // SeaORM maps both to DateTime<Utc> when the column is timestamp_with_time_zone; on
    // SQLite we fall back to string parsing if the native mapping fails.
    let available_at = parse_timestamp(row, "available_at")?;
    let created_at = parse_timestamp(row, "created_at")?;

    Ok(JobRow {
        id,
        job_type,
        payload,
        queue,
        attempts: attempts as u32,
        max_retries: max_retries as u32,
        idempotency_key,
        tenant_id,
        available_at,
        created_at,
    })
}

/// Parse a timestamp column that may arrive as `DateTime<Utc>` (Postgres) or
/// as an ISO-8601 string (SQLite).
fn parse_timestamp(row: &sea_orm::QueryResult, col: &str) -> Result<DateTime<Utc>, Error> {
    // Try native DateTime<Utc> first (Postgres timestamptz).
    if let Ok(dt) = row.try_get_by::<DateTime<Utc>, _>(col) {
        return Ok(dt);
    }
    // Fall back to string parsing (SQLite TEXT column).
    let s: String = row
        .try_get_by::<String, _>(col)
        .map_err(|e| Error::custom(format!("parse {col}: {e}")))?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::custom(format!("parse {col} as rfc3339 ('{s}'): {e}")))
}

/// SQL placeholder style: Postgres uses `$N`, SQLite uses `?N`.
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),
    }
}

// ---------------------------------------------------------------------------
// claim — atomic work-stealing claim (dual-backend)
// ---------------------------------------------------------------------------

/// Atomically claim one pending job from `queue`.
///
/// Returns `None` when the queue is empty or all eligible jobs are locked by
/// another worker (Postgres). Uses:
/// - Postgres: `SELECT … FOR UPDATE SKIP LOCKED` + `UPDATE` inside a transaction.
/// - SQLite: raw `BEGIN IMMEDIATE` + `UPDATE … RETURNING` (no `begin_with_config`
///   which does NOT emit `BEGIN IMMEDIATE` on SQLite — Pitfall 2).
pub async fn claim(
    conn: &DatabaseConnection,
    queue: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, Error> {
    match conn.get_database_backend() {
        DatabaseBackend::Postgres => claim_postgres(conn, queue, worker_id).await,
        DatabaseBackend::Sqlite => claim_sqlite(conn, queue, worker_id).await,
        _ => Err(Error::UnsupportedBackend),
    }
}

async fn claim_postgres(
    conn: &DatabaseConnection,
    queue: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, Error> {
    let txn = conn.begin().await.map_err(Error::Db)?;

    let select = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT id, job_type, payload, queue, attempts, max_retries, idempotency_key, \
         tenant_id, available_at, created_at FROM jobs \
         WHERE status = 'pending' AND queue = $1 AND available_at <= NOW() \
         ORDER BY id LIMIT 1 FOR UPDATE SKIP LOCKED",
        [Value::String(Some(Box::new(queue.to_string())))],
    );

    let row = txn.query_one(select).await.map_err(Error::Db)?;
    let Some(row) = row else {
        txn.commit().await.map_err(Error::Db)?;
        return Ok(None);
    };
    let job = parse_job_row(&row)?;

    let upd = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "UPDATE jobs SET status = 'claimed', claimed_at = NOW(), claimed_by = $2 WHERE id = $1",
        [
            Value::BigInt(Some(job.id)),
            Value::String(Some(Box::new(worker_id.to_string()))),
        ],
    );
    txn.execute(upd).await.map_err(Error::Db)?;
    txn.commit().await.map_err(Error::Db)?;

    Ok(Some(job))
}

async fn claim_sqlite(
    conn: &DatabaseConnection,
    queue: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, Error> {
    let now_iso = Utc::now().to_rfc3339();

    // Raw BEGIN IMMEDIATE — not begin_with_config(Serializable) which does NOT
    // emit BEGIN IMMEDIATE on SQLite (see ferro-reservation/src/kernel.rs lines 69-79).
    conn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "BEGIN IMMEDIATE".to_string(),
    ))
    .await
    .map_err(Error::Db)?;

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

    // Capture result before COMMIT so we can always commit the transaction.
    let row_result = conn.query_one(stmt).await;

    conn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "COMMIT".to_string(),
    ))
    .await
    .map_err(Error::Db)?;

    let row = row_result.map_err(Error::Db)?;
    row.map(|r| parse_job_row(&r)).transpose()
}

// ---------------------------------------------------------------------------
// reaper — re-queue stuck claimed rows; park exhausted ones as failed
// ---------------------------------------------------------------------------

/// Re-queue claimed rows that have been held longer than `visibility_timeout`.
///
/// Rows below `max_retries` are reset to `pending` with `attempts + 1`.
/// Rows at or above `max_retries` are parked as `failed` (T-185-04).
pub async fn reaper(
    conn: &DatabaseConnection,
    queue: &str,
    visibility_timeout: std::time::Duration,
) -> Result<(), Error> {
    let now = Utc::now();
    let duration = chrono::Duration::from_std(visibility_timeout)
        .map_err(|e| Error::custom(format!("visibility_timeout out of range: {e}")))?;
    let cutoff = (now - duration).to_rfc3339();
    let now_iso = now.to_rfc3339();

    let backend = conn.get_database_backend();
    let txn = conn.begin().await.map_err(Error::Db)?;

    // Step 1: re-queue rows below max_retries.
    let (p1, p2, p3) = (ph(backend, 1), ph(backend, 2), ph(backend, 3));
    let requeue_sql = format!(
        "UPDATE jobs SET status='pending', claimed_at=NULL, claimed_by=NULL, \
         attempts = attempts + 1, available_at = {p1} \
         WHERE status='claimed' AND claimed_at < {p2} \
         AND attempts < max_retries AND queue = {p3}"
    );
    let requeue = Statement::from_sql_and_values(
        backend,
        &requeue_sql,
        [
            Value::String(Some(Box::new(now_iso.clone()))),
            Value::String(Some(Box::new(cutoff.clone()))),
            Value::String(Some(Box::new(queue.to_string()))),
        ],
    );
    txn.execute(requeue).await.map_err(Error::Db)?;

    // Step 2: park exhausted rows as failed.
    // Use fresh placeholders ?1 and ?2 (independent statement, not continuing p1..p4).
    let (pp1, pp2) = (ph(backend, 1), ph(backend, 2));
    let park_sql = format!(
        "UPDATE jobs SET status='failed', error='visibility timeout exceeded' \
         WHERE status='claimed' AND claimed_at < {pp1} \
         AND attempts >= max_retries AND queue = {pp2}"
    );
    let park = Statement::from_sql_and_values(
        backend,
        &park_sql,
        [
            Value::String(Some(Box::new(cutoff))),
            Value::String(Some(Box::new(queue.to_string()))),
        ],
    );
    txn.execute(park).await.map_err(Error::Db)?;

    txn.commit().await.map_err(Error::Db)
}

// ---------------------------------------------------------------------------
// enqueue — idempotent insert
// ---------------------------------------------------------------------------

/// Insert a new job into the queue.
///
/// When `idempotency_key` is `Some`, skips the insert if a `pending` or
/// `claimed` row with the same `(job_type, idempotency_key)` already exists
/// (D-15, T-185-01).
#[allow(clippy::too_many_arguments)]
pub async fn enqueue(
    conn: &DatabaseConnection,
    queue: &str,
    job_type: &str,
    payload: &str,
    max_retries: u32,
    idempotency_key: Option<&str>,
    tenant_id: Option<i64>,
    available_at: DateTime<Utc>,
) -> Result<(), Error> {
    let backend = conn.get_database_backend();
    let now_iso = Utc::now().to_rfc3339();
    let available_iso = available_at.to_rfc3339();

    if let Some(idem) = idempotency_key {
        // Idempotent insert: skip if a pending/claimed row with the same key exists.
        let (p1, p2, p3, p4, p5, p6, p7, p8) = (
            ph(backend, 1),
            ph(backend, 2),
            ph(backend, 3),
            ph(backend, 4),
            ph(backend, 5),
            ph(backend, 6),
            ph(backend, 7),
            ph(backend, 8),
        );
        let sql = format!(
            "INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries, \
             idempotency_key, tenant_id, available_at, created_at) \
             SELECT {p1}, {p2}, {p3}, 'pending', 0, {p4}, {p5}, {p6}, {p7}, {p8} \
             WHERE NOT EXISTS ( \
               SELECT 1 FROM jobs WHERE job_type = {p2} AND idempotency_key = {p5} \
               AND status IN ('pending','claimed') \
             )"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [
                Value::String(Some(Box::new(queue.to_string()))),
                Value::String(Some(Box::new(job_type.to_string()))),
                Value::String(Some(Box::new(payload.to_string()))),
                Value::Int(Some(max_retries as i32)),
                Value::String(Some(Box::new(idem.to_string()))),
                tenant_id.map_or(Value::BigInt(None), |id| Value::BigInt(Some(id))),
                Value::String(Some(Box::new(available_iso))),
                Value::String(Some(Box::new(now_iso))),
            ],
        );
        conn.execute(stmt).await.map_err(Error::Db)?;
    } else {
        // Plain insert — no deduplication guard.
        let (p1, p2, p3, p4, p5, p6, p7) = (
            ph(backend, 1),
            ph(backend, 2),
            ph(backend, 3),
            ph(backend, 4),
            ph(backend, 5),
            ph(backend, 6),
            ph(backend, 7),
        );
        let sql = format!(
            "INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries, \
             tenant_id, available_at, created_at) \
             VALUES ({p1}, {p2}, {p3}, 'pending', 0, {p4}, {p5}, {p6}, {p7})"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [
                Value::String(Some(Box::new(queue.to_string()))),
                Value::String(Some(Box::new(job_type.to_string()))),
                Value::String(Some(Box::new(payload.to_string()))),
                Value::Int(Some(max_retries as i32)),
                tenant_id.map_or(Value::BigInt(None), |id| Value::BigInt(Some(id))),
                Value::String(Some(Box::new(available_iso))),
                Value::String(Some(Box::new(now_iso))),
            ],
        );
        conn.execute(stmt).await.map_err(Error::Db)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Lifecycle operations
// ---------------------------------------------------------------------------

/// Delete a successfully-completed job row (D-04 delete-on-success).
pub async fn delete_job(conn: &DatabaseConnection, id: i64) -> Result<(), Error> {
    let backend = conn.get_database_backend();
    let p1 = ph(backend, 1);
    let sql = format!("DELETE FROM jobs WHERE id = {p1}");
    let stmt = Statement::from_sql_and_values(backend, &sql, [Value::BigInt(Some(id))]);
    conn.execute(stmt).await.map_err(Error::Db)?;
    Ok(())
}

/// Park a job as `failed` with an error message.
pub async fn fail_job(conn: &DatabaseConnection, id: i64, error: &str) -> Result<(), Error> {
    let backend = conn.get_database_backend();
    let (p1, p2) = (ph(backend, 1), ph(backend, 2));
    let sql = format!("UPDATE jobs SET status='failed', error={p1} WHERE id = {p2}");
    let stmt = Statement::from_sql_and_values(
        backend,
        &sql,
        [
            Value::String(Some(Box::new(error.to_string()))),
            Value::BigInt(Some(id)),
        ],
    );
    conn.execute(stmt).await.map_err(Error::Db)?;
    Ok(())
}

/// Reset a job to `pending`, bump its attempt count, and set a new
/// `available_at` (used by the worker after a retryable failure).
pub async fn release_job(
    conn: &DatabaseConnection,
    id: i64,
    attempts: u32,
    available_at: DateTime<Utc>,
) -> Result<(), Error> {
    let backend = conn.get_database_backend();
    let (p1, p2, p3) = (ph(backend, 1), ph(backend, 2), ph(backend, 3));
    let sql = format!(
        "UPDATE jobs SET status='pending', claimed_at=NULL, claimed_by=NULL, \
         attempts={p1}, available_at={p2} WHERE id = {p3}"
    );
    let stmt = Statement::from_sql_and_values(
        backend,
        &sql,
        [
            Value::Int(Some(attempts as i32)),
            Value::String(Some(Box::new(available_at.to_rfc3339()))),
            Value::BigInt(Some(id)),
        ],
    );
    conn.execute(stmt).await.map_err(Error::Db)?;
    Ok(())
}

/// Reset all jobs claimed by `worker_id` back to `pending` (D-10 shutdown
/// re-queue — called by the worker loop before the process exits).
pub async fn requeue_claimed_by(conn: &DatabaseConnection, worker_id: &str) -> Result<(), Error> {
    let backend = conn.get_database_backend();
    let p1 = ph(backend, 1);
    let sql = format!(
        "UPDATE jobs SET status='pending', claimed_at=NULL, claimed_by=NULL \
         WHERE status='claimed' AND claimed_by={p1}"
    );
    let stmt = Statement::from_sql_and_values(
        backend,
        &sql,
        [Value::String(Some(Box::new(worker_id.to_string())))],
    );
    conn.execute(stmt).await.map_err(Error::Db)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Introspection / stat queries
// ---------------------------------------------------------------------------

/// Return up to `limit` pending (immediately eligible) jobs in `queue`.
pub async fn get_pending_jobs(
    conn: &DatabaseConnection,
    queue: &str,
    limit: u64,
) -> Result<Vec<JobInfo>, Error> {
    let backend = conn.get_database_backend();
    let now_iso = Utc::now().to_rfc3339();
    let (p1, p2, p3) = (ph(backend, 1), ph(backend, 2), ph(backend, 3));
    let sql = format!(
        "SELECT id, job_type, queue, attempts, max_retries, created_at, available_at \
         FROM jobs WHERE status='pending' AND queue={p1} AND available_at <= {p2} \
         ORDER BY id LIMIT {p3}"
    );
    let stmt = Statement::from_sql_and_values(
        backend,
        &sql,
        [
            Value::String(Some(Box::new(queue.to_string()))),
            Value::String(Some(Box::new(now_iso))),
            Value::BigInt(Some(limit as i64)),
        ],
    );
    let rows = conn.query_all(stmt).await.map_err(Error::Db)?;
    rows.iter()
        .map(|r| parse_job_info(r, JobState::Pending))
        .collect()
}

/// Return up to `limit` delayed (not-yet-eligible) jobs in `queue`.
pub async fn get_delayed_jobs(
    conn: &DatabaseConnection,
    queue: &str,
    limit: u64,
) -> Result<Vec<JobInfo>, Error> {
    let backend = conn.get_database_backend();
    let now_iso = Utc::now().to_rfc3339();
    let (p1, p2, p3) = (ph(backend, 1), ph(backend, 2), ph(backend, 3));
    let sql = format!(
        "SELECT id, job_type, queue, attempts, max_retries, created_at, available_at \
         FROM jobs WHERE status='pending' AND queue={p1} AND available_at > {p2} \
         ORDER BY id LIMIT {p3}"
    );
    let stmt = Statement::from_sql_and_values(
        backend,
        &sql,
        [
            Value::String(Some(Box::new(queue.to_string()))),
            Value::String(Some(Box::new(now_iso))),
            Value::BigInt(Some(limit as i64)),
        ],
    );
    let rows = conn.query_all(stmt).await.map_err(Error::Db)?;
    rows.iter()
        .map(|r| parse_job_info(r, JobState::Delayed))
        .collect()
}

/// Return up to `limit` failed jobs (across all queues).
pub async fn get_failed_jobs(
    conn: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<FailedJobInfo>, Error> {
    let backend = conn.get_database_backend();
    let p1 = ph(backend, 1);
    let sql = format!(
        "SELECT id, job_type, queue, attempts, max_retries, created_at, available_at, error \
         FROM jobs WHERE status='failed' ORDER BY created_at DESC LIMIT {p1}"
    );
    let stmt = Statement::from_sql_and_values(backend, &sql, [Value::BigInt(Some(limit as i64))]);
    let rows = conn.query_all(stmt).await.map_err(Error::Db)?;
    rows.iter().map(parse_failed_job_info).collect()
}

/// Return aggregate pending/delayed counts per queue and the total failed count.
pub async fn get_stats(conn: &DatabaseConnection, queues: &[&str]) -> Result<QueueStats, Error> {
    let backend = conn.get_database_backend();
    let now_iso = Utc::now().to_rfc3339();
    let mut queue_stats = Vec::new();

    for &q in queues {
        let p1 = ph(backend, 1);
        let p2 = ph(backend, 2);
        let p3 = ph(backend, 3);
        let pending_sql = format!(
            "SELECT COUNT(*) as cnt FROM jobs \
             WHERE status='pending' AND queue={p1} AND available_at <= {p2}"
        );
        let pending_stmt = Statement::from_sql_and_values(
            backend,
            &pending_sql,
            [
                Value::String(Some(Box::new(q.to_string()))),
                Value::String(Some(Box::new(now_iso.clone()))),
            ],
        );
        let pending_row = conn
            .query_one(pending_stmt)
            .await
            .map_err(Error::Db)?
            .ok_or_else(|| Error::custom("stats: no row returned for pending count"))?;
        let pending: i64 = pending_row
            .try_get_by::<i64, _>("cnt")
            .map_err(|e| Error::custom(format!("stats pending cnt: {e}")))?;

        let delayed_sql = format!(
            "SELECT COUNT(*) as cnt FROM jobs \
             WHERE status='pending' AND queue={p1} AND available_at > {p3}"
        );
        let delayed_stmt = Statement::from_sql_and_values(
            backend,
            &delayed_sql,
            [
                Value::String(Some(Box::new(q.to_string()))),
                Value::String(Some(Box::new(now_iso.clone()))),
            ],
        );
        let delayed_row = conn
            .query_one(delayed_stmt)
            .await
            .map_err(Error::Db)?
            .ok_or_else(|| Error::custom("stats: no row returned for delayed count"))?;
        let delayed: i64 = delayed_row
            .try_get_by::<i64, _>("cnt")
            .map_err(|e| Error::custom(format!("stats delayed cnt: {e}")))?;

        queue_stats.push(SingleQueueStats {
            name: q.to_string(),
            pending: pending as usize,
            delayed: delayed as usize,
        });
    }

    // Total failed across all queues.
    let failed_sql = "SELECT COUNT(*) as cnt FROM jobs WHERE status='failed'";
    let failed_stmt = Statement::from_string(backend, failed_sql.to_string());
    let failed_row = conn
        .query_one(failed_stmt)
        .await
        .map_err(Error::Db)?
        .ok_or_else(|| Error::custom("stats: no row returned for failed count"))?;
    let total_failed: i64 = failed_row
        .try_get_by::<i64, _>("cnt")
        .map_err(|e| Error::custom(format!("stats failed cnt: {e}")))?;

    Ok(QueueStats {
        queues: queue_stats,
        total_failed: total_failed as usize,
    })
}

// ---------------------------------------------------------------------------
// Internal parse helpers for introspection types
// ---------------------------------------------------------------------------

fn parse_job_info(row: &sea_orm::QueryResult, state: JobState) -> Result<JobInfo, Error> {
    let id: i64 = row
        .try_get_by::<i64, _>("id")
        .map_err(|e| Error::custom(format!("parse id: {e}")))?;
    let job_type: String = row
        .try_get_by::<String, _>("job_type")
        .map_err(|e| Error::custom(format!("parse job_type: {e}")))?;
    let queue: String = row
        .try_get_by::<String, _>("queue")
        .map_err(|e| Error::custom(format!("parse queue: {e}")))?;
    let attempts: i32 = row
        .try_get_by::<i32, _>("attempts")
        .map_err(|e| Error::custom(format!("parse attempts: {e}")))?;
    let max_retries: i32 = row
        .try_get_by::<i32, _>("max_retries")
        .map_err(|e| Error::custom(format!("parse max_retries: {e}")))?;
    let created_at = parse_timestamp(row, "created_at")?.to_rfc3339();
    let available_at = parse_timestamp(row, "available_at")?.to_rfc3339();

    Ok(JobInfo {
        id,
        job_type,
        queue,
        attempts: attempts as u32,
        max_retries: max_retries as u32,
        created_at,
        available_at,
        state,
    })
}

fn parse_failed_job_info(row: &sea_orm::QueryResult) -> Result<FailedJobInfo, Error> {
    let job = parse_job_info(row, JobState::Failed)?;
    let error: String = row
        .try_get_by::<Option<String>, _>("error")
        .map_err(|e| Error::custom(format!("parse error: {e}")))?
        .unwrap_or_default();
    let failed_at = parse_timestamp(row, "created_at")?;

    Ok(FailedJobInfo {
        job,
        error,
        failed_at,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::CreateJobsTable)]
        }
    }

    /// Spin up an in-memory SQLite DB and run the jobs migration.
    async fn setup() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite::memory:");
        TestMigrator::up(&conn, None)
            .await
            .expect("run CreateJobsTable migration");
        conn
    }

    /// Insert a job row directly (bypassing enqueue) for test setup.
    #[allow(clippy::too_many_arguments)]
    async fn insert_job(
        conn: &DatabaseConnection,
        queue: &str,
        job_type: &str,
        status: &str,
        attempts: i32,
        max_retries: i32,
        claimed_at: Option<&str>,
        available_at: &str,
    ) -> i64 {
        let now = Utc::now().to_rfc3339();
        let claimed_at_sql = match claimed_at {
            Some(ts) => format!("'{ts}'"),
            None => "NULL".to_string(),
        };
        let sql = format!(
            "INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries, \
             available_at, claimed_at, created_at) \
             VALUES ('{queue}', '{job_type}', '{{}}', '{status}', {attempts}, {max_retries}, \
             '{available_at}', {claimed_at_sql}, '{now}') \
             RETURNING id"
        );
        let row = conn
            .query_one(Statement::from_string(DatabaseBackend::Sqlite, sql))
            .await
            .expect("insert_job query")
            .expect("insert_job row");
        row.try_get_by::<i64, _>("id").expect("insert id")
    }

    #[tokio::test]
    async fn claim_returns_pending_job() {
        let conn = setup().await;
        let now = Utc::now().to_rfc3339();

        // Enqueue one job via direct INSERT.
        insert_job(&conn, "default", "MyJob", "pending", 0, 3, None, &now).await;

        // First claim should return the job.
        let job = claim(&conn, "default", "worker-1")
            .await
            .expect("claim failed");
        assert!(job.is_some(), "expected Some(job), got None");
        let job = job.unwrap();
        assert_eq!(job.job_type, "MyJob");

        // Second claim on the same queue should return None (job is now claimed).
        let second = claim(&conn, "default", "worker-2")
            .await
            .expect("second claim failed");
        assert!(second.is_none(), "second claim should return None");
    }

    #[tokio::test]
    async fn idempotency_dedup() {
        let conn = setup().await;
        let now = Utc::now().to_rfc3339();
        let available_at = DateTime::parse_from_rfc3339(&now)
            .unwrap()
            .with_timezone(&Utc);

        // Enqueue with an idempotency key twice.
        enqueue(
            &conn,
            "default",
            "MyJob",
            "{}",
            3,
            Some("key-abc"),
            None,
            available_at,
        )
        .await
        .expect("first enqueue");
        enqueue(
            &conn,
            "default",
            "MyJob",
            "{}",
            3,
            Some("key-abc"),
            None,
            available_at,
        )
        .await
        .expect("second enqueue (should be a no-op)");

        // Verify COUNT = 1.
        let row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as cnt FROM jobs WHERE job_type='MyJob' AND idempotency_key='key-abc'".to_string(),
            ))
            .await
            .expect("count query")
            .expect("count row");
        let cnt: i64 = row.try_get_by::<i64, _>("cnt").expect("cnt");
        assert_eq!(
            cnt, 1,
            "idempotency key should deduplicate (expected 1 row)"
        );

        // Enqueue without idempotency key twice — both must insert.
        enqueue(
            &conn,
            "default",
            "OtherJob",
            "{}",
            3,
            None,
            None,
            available_at,
        )
        .await
        .expect("first plain enqueue");
        enqueue(
            &conn,
            "default",
            "OtherJob",
            "{}",
            3,
            None,
            None,
            available_at,
        )
        .await
        .expect("second plain enqueue");

        let row2 = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as cnt FROM jobs WHERE job_type='OtherJob'".to_string(),
            ))
            .await
            .expect("count query 2")
            .expect("count row 2");
        let cnt2: i64 = row2.try_get_by::<i64, _>("cnt").expect("cnt2");
        assert_eq!(cnt2, 2, "plain enqueue should insert both rows");
    }

    #[tokio::test]
    async fn reaper_reclaims_stuck_job() {
        let conn = setup().await;

        // Insert a claimed job with claimed_at 10 minutes ago, attempts=0, max_retries=3.
        let ten_min_ago = (Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();
        let now = Utc::now().to_rfc3339();
        let id = insert_job(
            &conn,
            "default",
            "StuckJob",
            "claimed",
            0,
            3,
            Some(&ten_min_ago),
            &now,
        )
        .await;

        // Run reaper with 5-minute visibility timeout.
        reaper(&conn, "default", std::time::Duration::from_secs(5 * 60))
            .await
            .expect("reaper failed");

        // Verify status = 'pending' and attempts = 1.
        let row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT status, attempts FROM jobs WHERE id={id}"),
            ))
            .await
            .expect("select after reaper")
            .expect("row after reaper");

        let status: String = row.try_get_by::<String, _>("status").expect("status");
        let attempts: i32 = row.try_get_by::<i32, _>("attempts").expect("attempts");
        assert_eq!(
            status, "pending",
            "reaper should reset stuck job to pending"
        );
        assert_eq!(attempts, 1, "reaper should increment attempts");
    }

    #[tokio::test]
    async fn poison_job_parked() {
        let conn = setup().await;

        // Insert an exhausted job (attempts == max_retries) claimed 10 min ago.
        let ten_min_ago = (Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();
        let now = Utc::now().to_rfc3339();
        let id = insert_job(
            &conn,
            "default",
            "PoisonJob",
            "claimed",
            3,
            3,
            Some(&ten_min_ago),
            &now,
        )
        .await;

        // Run reaper.
        reaper(&conn, "default", std::time::Duration::from_secs(5 * 60))
            .await
            .expect("reaper failed");

        // Verify status = 'failed' and error IS NOT NULL.
        let row = conn
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT status, error FROM jobs WHERE id={id}"),
            ))
            .await
            .expect("select after reaper")
            .expect("row");

        let status: String = row.try_get_by::<String, _>("status").expect("status");
        let error: Option<String> = row.try_get_by::<Option<String>, _>("error").expect("error");
        assert_eq!(status, "failed", "exhausted job should be parked as failed");
        assert!(error.is_some(), "failed job should have an error message");

        // A fresh pending job should still be claimable (parked row does not block).
        let available = Utc::now().to_rfc3339();
        insert_job(
            &conn, "default", "FreshJob", "pending", 0, 3, None, &available,
        )
        .await;

        let claimed = claim(&conn, "default", "worker-1")
            .await
            .expect("claim after poison park");
        assert!(
            claimed.is_some(),
            "fresh job should be claimable after poison job is parked"
        );
        let claimed = claimed.unwrap();
        assert_eq!(claimed.job_type, "FreshJob");
    }
}

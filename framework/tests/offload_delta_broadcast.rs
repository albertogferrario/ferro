//! Integration suite proving the offload broadcast loop (OFFLOAD-04 D-12).
//!
//! Drives the full enqueue → WorkerLoop drain → broadcast delta → subscriber
//! receive chain against two `Broadcaster` instances sharing one
//! `InMemoryTransport` bus (multi-replica shape).
//!
//! The four scenarios are named sub-functions invoked in sequence from a single
//! `#[tokio::test]` to avoid the global-init race from `Queue`'s `OnceLock`
//! and `OFFLOAD_BROADCASTER`'s `OnceLock` — identical to the pattern used in
//! `offload_result_round_trip.rs`. Between scenarios the two tables are cleared.
//!
//! The function names bound to VALIDATION.md rows are:
//! - `cross_replica_delta`        — SC#1
//! - `request_returns_before_worker` — SC#2
//! - `offload_failed_delta_is_redacted` — D-05
//! - `resolve_already_complete`   — D-09
//!
//! The env-gated `redis_cross_replica` test (feature `redis-transport`) is an
//! additional `#[tokio::test]` that skips when `REDIS_URL` is unset.

extern crate ferro_rs as ferro;

use ferro::offload::{
    enqueue_and_mark_pending, read_result, register_offload_hooks_with_broadcaster, resolve,
    OffloadResult, OFFLOAD_PROJECTION_NAME,
};
use ferro_broadcast::{transport::memory::InMemoryTransport, Broadcaster, ServerMessage};
use ferro_queue::{
    async_trait, Error, Job, JobRegistrarEntry, Offloadable, Queue, WorkerConfig, WorkerLoop,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

// ---------------------------------------------------------------------------
// TestMigrator
// ---------------------------------------------------------------------------

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_queue::CreateJobsTable),
            Box::new(ferro_projection::CreateProjectionSnapshotsTable),
        ]
    }
}

// ---------------------------------------------------------------------------
// Harness helpers
// ---------------------------------------------------------------------------

/// Create a temp-file SQLite DB with both migrations applied.
///
/// Uses a temp file (not `sqlite::memory:`) because the WorkerLoop opens
/// multiple pool connections; per-connection in-memory databases are invisible
/// across connections.
async fn setup_db() -> (DatabaseConnection, tempfile::NamedTempFile) {
    let db_file = tempfile::NamedTempFile::new().expect("create temp SQLite file");
    let url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let conn = Database::connect(&url)
        .await
        .expect("connect to temp SQLite file");
    TestMigrator::up(&conn, None)
        .await
        .expect("run both migrations");
    (conn, db_file)
}

/// Delete all rows from jobs and projection_snapshots.
async fn clear_tables(db: &DatabaseConnection) {
    for sql in ["DELETE FROM jobs", "DELETE FROM projection_snapshots"] {
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string(),
        ))
        .await
        .expect("clear table");
    }
}

/// Drain all pending jobs and wait for hook writes to complete.
async fn drain() {
    let worker = WorkerLoop::from_registry(WorkerConfig {
        sleep_duration: Duration::from_millis(10),
        ..WorkerConfig::default()
    });
    worker
        .drain_for_test()
        .await
        .expect("drain_for_test completed without fatal error");
    // Allow spawned job tasks time to complete hook writes and snapshot persistence.
    tokio::time::sleep(Duration::from_millis(200)).await;
}

// ---------------------------------------------------------------------------
// Subscribe helper
// ---------------------------------------------------------------------------

/// Add a client to `broadcaster` subscribed to the offload result channel for
/// `handle_key`. Returns the socket id and mpsc receiver for message assertions.
async fn subscribe_client(
    broadcaster: &Broadcaster,
    handle_key: &str,
) -> (String, tokio::sync::mpsc::Receiver<ServerMessage>) {
    let socket_id = format!("test-client-{}", uuid::Uuid::new_v4());
    let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key);
    let (tx, rx) = tokio::sync::mpsc::channel::<ServerMessage>(16);
    broadcaster.add_client(socket_id.clone(), tx);
    broadcaster
        .subscribe(&socket_id, &channel, None, None)
        .await
        .expect("subscribe client");
    (socket_id, rx)
}

// ---------------------------------------------------------------------------
// Test jobs
// ---------------------------------------------------------------------------

/// A job that returns a known i64 value — used in SC#1, SC#2, and D-09.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValueJob {
    value: i64,
}

#[async_trait]
impl Job for ValueJob {
    async fn handle(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_with_value(&self) -> Result<Option<serde_json::Value>, Error> {
        let v = serde_json::to_value(self.value)
            .map_err(|e| Error::job_failed(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(Some(v))
    }
}

inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<ValueJob>(); },
        name: "ValueJob",
        queue: None,
    }
}

impl Offloadable for ValueJob {
    type Output = i64;
}

/// A job that always fails with a known error string — used in D-05.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailingJob;

#[async_trait]
impl Job for FailingJob {
    fn max_retries(&self) -> u32 {
        1
    }

    fn retry_delay(&self, _attempt: u32) -> Duration {
        Duration::ZERO
    }

    async fn handle(&self) -> Result<(), Error> {
        Err(Error::job_failed("FailingJob", "sensitive-secret-value"))
    }
}

inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<FailingJob>(); },
        name: "FailingJob",
        queue: None,
    }
}

impl Offloadable for FailingJob {
    type Output = ();
}

// ---------------------------------------------------------------------------
// Scenario functions
// ---------------------------------------------------------------------------

/// SC#1: A client on Broadcaster B receives the delta from a worker on
/// Broadcaster A via the shared `InMemoryTransport` (multi-replica shape).
async fn cross_replica_delta(
    broadcaster_a: &Arc<Broadcaster>,
    broadcaster_b: &Broadcaster,
    db: &DatabaseConnection,
) {
    clear_tables(db).await;

    let handle = ValueJob { value: 42 }
        .offload()
        .await
        .expect("cross_replica_delta: offload dispatch");
    let key = handle.key().to_string();

    // Subscribe on Broadcaster B BEFORE draining (subscribe-first, Pitfall 3).
    let (_socket_id, mut rx) = subscribe_client(broadcaster_b, &key).await;

    // Drain — simulates the worker replica completing the job.
    drain().await;

    let msg = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("cross_replica_delta: timed out waiting for delta")
        .expect("cross_replica_delta: channel closed before receiving delta");

    match msg {
        ServerMessage::Event(bm) => {
            assert_eq!(bm.event, "offload.result", "event must be offload.result");
            assert_eq!(
                bm.channel,
                format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, key),
                "channel must match handle key"
            );
            assert_eq!(
                bm.data["status"], "completed",
                "data.status must be completed"
            );
            assert_eq!(bm.data["value"], 42, "data.value must be 42");
        }
        other => panic!("cross_replica_delta: expected ServerMessage::Event, got {other:?}"),
    }

    // Suppress unused-variable warning; broadcaster_a is used to set the hook.
    let _ = broadcaster_a;
}

/// SC#2: `enqueue_and_mark_pending` returns before the worker runs; the
/// pending snapshot exists immediately and the completed result appears after drain.
async fn request_returns_before_worker(db: &DatabaseConnection) {
    clear_tables(db).await;

    let start = Instant::now();
    let handle = enqueue_and_mark_pending(ValueJob { value: 7 }, db)
        .await
        .expect("request_returns_before_worker: enqueue_and_mark_pending");
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "enqueue_and_mark_pending must return before the worker runs; elapsed: {elapsed:?}"
    );

    let key = handle.key().to_string();

    // The pending marker must exist immediately (D-07).
    let before_drain = read_result::<i64>(&key, db)
        .await
        .expect("request_returns_before_worker: read_result before drain");
    assert!(
        matches!(before_drain, Some(OffloadResult::Pending)),
        "snapshot must be Pending before drain; got {before_drain:?}"
    );

    drain().await;

    let after_drain = read_result::<i64>(&key, db)
        .await
        .expect("request_returns_before_worker: read_result after drain")
        .expect("snapshot must exist after drain");
    assert!(
        matches!(after_drain, OffloadResult::Completed { value: 7 }),
        "snapshot must be Completed {{ value: 7 }} after drain; got {after_drain:?}"
    );
}

/// D-05: The failed delta carries no raw error; the authoritative snapshot does.
async fn offload_failed_delta_is_redacted(broadcaster_b: &Broadcaster, db: &DatabaseConnection) {
    clear_tables(db).await;

    let handle = FailingJob
        .offload()
        .await
        .expect("offload_failed_delta_is_redacted: offload dispatch");
    let key = handle.key().to_string();

    // Subscribe on Broadcaster B before draining (subscribe-first).
    let (_socket_id, mut rx) = subscribe_client(broadcaster_b, &key).await;

    drain().await;

    let msg = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("offload_failed_delta_is_redacted: timed out waiting for failed delta")
        .expect("offload_failed_delta_is_redacted: channel closed");

    match msg {
        ServerMessage::Event(bm) => {
            assert_eq!(bm.event, "offload.result");
            assert_eq!(bm.data["status"], "failed", "delta status must be failed");

            // Raw error must NOT appear in the delta (D-05 / T-247-info-disclosure).
            assert!(
                bm.data.get("error").is_none(),
                "failed delta must NOT carry an error field; got data = {:?}",
                bm.data
            );
            let delta_str = bm.data.to_string();
            assert!(
                !delta_str.contains("sensitive-secret-value"),
                "raw error must not appear in the delta payload; got: {delta_str}"
            );
        }
        other => {
            panic!("offload_failed_delta_is_redacted: expected ServerMessage::Event, got {other:?}")
        }
    }

    // The authoritative snapshot must still contain the raw error (D-06).
    let snapshot = read_result::<()>(&key, db)
        .await
        .expect("offload_failed_delta_is_redacted: read_result after drain")
        .expect("failed snapshot must exist");
    match snapshot {
        OffloadResult::Failed { error } => {
            assert!(
                error.contains("sensitive-secret-value"),
                "snapshot must retain the raw error; got: {error:?}"
            );
        }
        other => {
            panic!("offload_failed_delta_is_redacted: expected Failed snapshot, got {other:?}")
        }
    }
}

/// D-09: `resolve()` short-circuits a handle that already has a terminal result
/// via the read-back step — no delta needed.
async fn resolve_already_complete(broadcaster_b: &Arc<Broadcaster>, db: &DatabaseConnection) {
    clear_tables(db).await;

    let handle = ValueJob { value: 99 }
        .offload()
        .await
        .expect("resolve_already_complete: offload dispatch");

    // Drain BEFORE calling resolve — the result is already in the snapshot.
    drain().await;

    let result = resolve(&handle, broadcaster_b, db, Some(Duration::from_secs(5)))
        .await
        .expect("resolve_already_complete: resolve failed");

    assert!(
        matches!(result, OffloadResult::Completed { value: 99 }),
        "resolve must return Completed {{ value: 99 }} via short-circuit; got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Main integration test — all four scenarios in one test function
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial]
async fn offload_delta_broadcast_suite() {
    std::env::set_var("QUEUE_CONNECTION", "db");

    // Shared in-memory transport — connects the two replica broadcasters.
    let bus = Arc::new(InMemoryTransport::new(64));
    let broadcaster_a = Arc::new(Broadcaster::new().with_transport(bus.clone())); // worker
    let broadcaster_b = Arc::new(Broadcaster::new().with_transport(bus)); // client

    // Register the broadcaster-aware result hook once for the whole suite.
    register_offload_hooks_with_broadcaster(broadcaster_a.clone());

    let (conn, _db_file) = setup_db().await; // _db_file must outlive this scope
    Queue::init(conn).await.expect("Queue::init");
    let db = Queue::connection();

    cross_replica_delta(&broadcaster_a, &broadcaster_b, db).await;
    request_returns_before_worker(db).await;
    offload_failed_delta_is_redacted(&broadcaster_b, db).await;
    resolve_already_complete(&broadcaster_b, db).await;
}

// ---------------------------------------------------------------------------
// Env-gated live-redis cross-replica variant (Task 3)
// ---------------------------------------------------------------------------

#[cfg(feature = "redis-transport")]
mod redis_tests {
    use super::*;
    use ferro_broadcast::transport::redis::RedisTransport;

    fn redis_url() -> Option<String> {
        std::env::var("REDIS_URL").ok().filter(|s| !s.is_empty())
    }

    /// Cross-replica delta delivery over a live Redis bus.
    ///
    /// Skips when `REDIS_URL` is unset or empty. Mirrors `cross_replica_delta`
    /// but uses `RedisTransport` for both Broadcaster A and B, proving that the
    /// multi-replica delta path works over a real Redis pub/sub channel.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[serial_test::serial]
    async fn redis_cross_replica() {
        let Some(url) = redis_url() else {
            eprintln!("REDIS_URL not set — skipping redis_cross_replica");
            return;
        };

        std::env::set_var("QUEUE_CONNECTION", "db");

        // Unique channel per run to avoid interference.
        let channel = format!("ferro:offload:test:{}", uuid::Uuid::new_v4());

        let bus_a = Arc::new(
            RedisTransport::new(&url, channel.clone())
                .await
                .expect("RedisTransport A"),
        );
        let bus_b = Arc::new(
            RedisTransport::new(&url, channel)
                .await
                .expect("RedisTransport B"),
        );

        let broadcaster_a = Arc::new(Broadcaster::new().with_transport(bus_a));
        let broadcaster_b = Broadcaster::new().with_transport(bus_b);

        register_offload_hooks_with_broadcaster(broadcaster_a.clone());

        let (conn, _db_file) = setup_db().await;
        Queue::init(conn).await.expect("Queue::init");
        let db = Queue::connection();

        clear_tables(db).await;

        let handle = ValueJob { value: 55 }
            .offload()
            .await
            .expect("redis_cross_replica: offload dispatch");
        let key = handle.key().to_string();

        let (_socket_id, mut rx) = subscribe_client(&broadcaster_b, &key).await;

        // Brief pause to let the Redis subscription propagate before drain.
        tokio::time::sleep(Duration::from_millis(150)).await;

        drain().await;

        let msg = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("redis_cross_replica: timed out waiting for delta")
            .expect("redis_cross_replica: channel closed before receiving delta");

        match msg {
            ServerMessage::Event(bm) => {
                assert_eq!(bm.event, "offload.result");
                assert_eq!(
                    bm.channel,
                    format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, key)
                );
                assert_eq!(bm.data["status"], "completed");
                assert_eq!(bm.data["value"], 55);
            }
            other => {
                panic!("redis_cross_replica: expected ServerMessage::Event, got {other:?}")
            }
        }
    }
}

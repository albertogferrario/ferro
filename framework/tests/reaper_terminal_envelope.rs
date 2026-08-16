//! SC#2 integration test: reaper-parked rows with a handle_key write a terminal
//! `failed` envelope, and a subscribed caller observes the failure delta.
//!
//! Covers the reaper edge (edge b) of OFFLOAD-03 / WR-02:
//! - Signal A (persisted): after the reaper parks a timed-out claimed row that
//!   carries a known handle_key, `read_result` returns `Some(Failed { error })`
//!   where `error` contains `"visibility timeout exceeded"`.
//! - Signal B (observed): a subscriber registered BEFORE the reaper runs receives
//!   an `offload.result` `ServerMessage::Event` with `data["status"] == "failed"`
//!   within a bounded timeout (caller wakes rather than waiting to its own timeout).
//! - Redaction (D-05): the observed failed delta carries no raw `error` field; the
//!   raw string lives only in the server-side snapshot (Signal A).
//!
//! The reaper is driven directly via `ferro_queue::reaper` + the per-row
//! `ferro_queue::persist_offload_outcome` loop, exactly mirroring the updated
//! worker call-site. This avoids the `reap_startup_claims` call inside
//! `drain_for_test` which would park the test row before the reaper sees it.
//!
//! All scenarios run from a single `#[tokio::test]` to avoid the global-init race
//! from `Queue`'s `OnceLock` and `OFFLOAD_BROADCASTER`'s `OnceLock` — same pattern
//! as `offload_delta_broadcast.rs` and `offload_result_round_trip.rs`.

extern crate ferro_rs as ferro;

use ferro::offload::{
    read_result, register_offload_hooks_with_broadcaster, OffloadResult, OFFLOAD_PROJECTION_NAME,
};
use ferro_broadcast::{Broadcaster, ServerMessage};
use ferro_queue::{persist_offload_outcome, reaper, Queue};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use std::{sync::Arc, time::Duration};

// ---------------------------------------------------------------------------
// TestMigrator
// ---------------------------------------------------------------------------

struct TestMigrator;

#[async_trait::async_trait]
impl sea_orm_migration::MigratorTrait for TestMigrator {
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

/// Insert a claimed job row with a known `handle_key` directly (bypassing enqueue).
///
/// A local copy — the `#[cfg(test)]` helper in `ferro-queue/src/db.rs` is not
/// reachable from this integration test.
#[allow(clippy::too_many_arguments)]
async fn insert_job_with_handle(
    conn: &DatabaseConnection,
    queue: &str,
    job_type: &str,
    status: &str,
    attempts: i32,
    max_retries: i32,
    claimed_at: Option<&str>,
    available_at: &str,
    handle_key: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    let claimed_at_sql = match claimed_at {
        Some(ts) => format!("'{ts}'"),
        None => "NULL".to_string(),
    };
    let sql = format!(
        "INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries, \
         available_at, claimed_at, created_at, handle_key) \
         VALUES ('{queue}', '{job_type}', '{{}}', '{status}', {attempts}, {max_retries}, \
         '{available_at}', {claimed_at_sql}, '{now}', '{handle_key}') \
         RETURNING id"
    );
    conn.query_one(Statement::from_string(DatabaseBackend::Sqlite, sql))
        .await
        .expect("insert_job_with_handle query")
        .expect("insert_job_with_handle row");
}

/// Trigger the reaper and run the per-row persistence loop, mirroring the
/// updated worker call-site exactly (RESEARCH.md reaper edge fix data flow).
///
/// Uses a 1 ms visibility timeout so any claimed row with a past `claimed_at`
/// is immediately eligible.
async fn run_reaper_with_persistence(db: &'static DatabaseConnection, queue: &str) {
    let parked = reaper(db, queue, Duration::from_millis(1))
        .await
        .expect("reaper");
    for (_, handle_key) in parked {
        persist_offload_outcome(
            Some(&handle_key),
            Err("visibility timeout exceeded".to_string()),
            db,
        )
        .await;
    }
}

// ---------------------------------------------------------------------------
// Scenario A: read_result returns Failed after reaper parks the row
// ---------------------------------------------------------------------------

async fn reaper_parks_write_failed_envelope(db: &'static DatabaseConnection) {
    clear_tables(db).await;

    let key = ferro_queue::HandleKey::new();
    let ten_min_ago = (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();
    let now = chrono::Utc::now().to_rfc3339();
    insert_job_with_handle(
        db,
        "default",
        "BoundaryJob",
        "claimed",
        2, // attempts — at max_retries-1, so the reaper parks (not requeues) it
        3, // max_retries
        Some(&ten_min_ago),
        &now,
        key.as_str(),
    )
    .await;

    run_reaper_with_persistence(db, "default").await;

    let snapshot = read_result::<()>(key.as_str(), db)
        .await
        .expect("read_result after reaper")
        .expect("SC#2-A: failed envelope must exist after reaper parks a handle-key row");

    match snapshot {
        OffloadResult::Failed { error } => assert!(
            error.contains("visibility timeout exceeded"),
            "SC#2-A: snapshot error must match reaper string; got: {error:?}"
        ),
        other => panic!("SC#2-A: expected Failed snapshot, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Scenario B: subscriber observes the failure delta (wakes rather than timing out)
// ---------------------------------------------------------------------------

async fn subscriber_observes_reaper_failure(
    broadcaster: &Broadcaster,
    db: &'static DatabaseConnection,
) {
    clear_tables(db).await;

    let key = ferro_queue::HandleKey::new();
    let ten_min_ago = (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();
    let now = chrono::Utc::now().to_rfc3339();
    insert_job_with_handle(
        db,
        "default",
        "BoundaryJob",
        "claimed",
        2, // attempts
        3, // max_retries
        Some(&ten_min_ago),
        &now,
        key.as_str(),
    )
    .await;

    // Subscribe BEFORE triggering the reaper so the delta is not missed.
    let (_socket_id, mut rx) = subscribe_client(broadcaster, key.as_str()).await;

    run_reaper_with_persistence(db, "default").await;

    let msg = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("SC#2-B: timed out waiting for failed delta")
        .expect("SC#2-B: channel closed");

    match msg {
        ServerMessage::Event(bm) => {
            assert_eq!(bm.event, "offload.result");
            assert_eq!(bm.data["status"], "failed", "delta status must be failed");
            // D-05 redaction: raw error must NOT appear in the client-facing delta.
            assert!(
                bm.data.get("error").is_none(),
                "D-05: failed delta must not carry a raw error field; got {:?}",
                bm.data
            );
        }
        other => panic!("SC#2-B: expected ServerMessage::Event, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Main integration test — both scenarios in one test function
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial]
async fn reaper_terminal_envelope() {
    std::env::set_var("QUEUE_CONNECTION", "db");

    let broadcaster = Arc::new(Broadcaster::new());
    register_offload_hooks_with_broadcaster(broadcaster.clone());

    let (conn, _db_file) = setup_db().await;
    Queue::init(conn).await.expect("Queue::init");
    let db = Queue::connection();

    // SC#2-A: read_result returns Failed after reaper parks the row.
    reaper_parks_write_failed_envelope(db).await;

    // SC#2-B: subscribed caller observes the failure delta within a bounded timeout.
    subscriber_observes_reaper_failure(&broadcaster, db).await;
}

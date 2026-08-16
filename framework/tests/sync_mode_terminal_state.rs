//! Integration test for SC#1: sync-mode dispatch persists a terminal envelope.
//!
//! Verifies that `PendingDispatch` in sync mode (`QUEUE_CONNECTION=sync`) writes
//! a terminal `completed` or `failed` snapshot under the offload handle key, so
//! the caller never reads `Ok(None)` after dispatch completes.
//!
//! Success criteria:
//! - SC#1-success: `read_result::<i32>` returns `Some(Completed { value: 42 })`.
//! - SC#1-failure: `dispatch()` returns `Err` (D-03 dual signal) AND
//!   `read_result::<()>` returns `Some(Failed { .. })`.
//!
//! All scenarios run under a single `#[tokio::test]` to avoid the global `Queue::init`
//! `OnceLock` race that arises when multiple async test functions run concurrently.
//! Distinct `HandleKey`s are used per scenario so rows do not collide.

extern crate ferro_rs as ferro;

use ferro::offload::{read_result, register_offload_hooks_with_broadcaster, OffloadResult};
use ferro_broadcast::Broadcaster;
use ferro_queue::{async_trait, Error, HandleKey, Offloadable, PendingDispatch, Queue};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// TestMigrator — both migrations required for snapshot persistence
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

/// Create a temp-file SQLite DB and run both migrations.
///
/// Uses `NamedTempFile` (not `:memory:`) so all pool connections see the
/// same database — in-memory databases are per-connection and invisible
/// across pool members.
async fn setup_db() -> (DatabaseConnection, tempfile::NamedTempFile) {
    let db_file = tempfile::NamedTempFile::new().expect("create temp SQLite file");
    let url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let conn = Database::connect(&url)
        .await
        .expect("connect to temp SQLite file");
    TestMigrator::up(&conn, None)
        .await
        .expect("run both migrations (CreateJobsTable + CreateProjectionSnapshotsTable)");
    (conn, db_file)
}

/// Delete all rows from both tables.
#[allow(dead_code)]
async fn clear_tables(db: &DatabaseConnection) {
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "DELETE FROM jobs".to_string(),
    ))
    .await
    .expect("clear jobs");
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "DELETE FROM projection_snapshots".to_string(),
    ))
    .await
    .expect("clear projection_snapshots");
}

// ---------------------------------------------------------------------------
// RAII guard — ensures QUEUE_CONNECTION is removed on drop even on panic
// ---------------------------------------------------------------------------

struct EnvGuard {
    vars: Vec<String>,
}

impl EnvGuard {
    fn set(key: &str, value: &str) -> Self {
        std::env::set_var(key, value);
        Self {
            vars: vec![key.to_string()],
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for var in &self.vars {
            std::env::remove_var(var);
        }
    }
}

// ---------------------------------------------------------------------------
// Test jobs (no `inventory::submit!` — WorkerLoop is not used in sync mode)
// ---------------------------------------------------------------------------

/// A job that returns a known i32 value in `handle_with_value`.
///
/// Mirrors what the `#[offload]` macro emits for a `-> i32` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncSuccessJob {
    expected_value: i32,
}

#[async_trait]
impl ferro_queue::Job for SyncSuccessJob {
    async fn handle(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_with_value(&self) -> Result<Option<serde_json::Value>, Error> {
        let v = serde_json::to_value(self.expected_value)
            .map_err(|e| Error::job_failed(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(Some(v))
    }
}

impl Offloadable for SyncSuccessJob {
    type Output = i32;
}

/// A job that always fails on `handle` (SC#1-failure).
///
/// `max_retries = 1` makes the first failure terminal immediately.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncFailingJob;

#[async_trait]
impl ferro_queue::Job for SyncFailingJob {
    fn max_retries(&self) -> u32 {
        1
    }

    fn retry_delay(&self, _attempt: u32) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    async fn handle(&self) -> Result<(), Error> {
        Err(Error::job_failed("SyncFailingJob", "sync job always fails"))
    }
}

impl Offloadable for SyncFailingJob {
    type Output = ();
}

// ---------------------------------------------------------------------------
// Integration test — SC#1 success and failure scenarios in one test function
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn sync_mode_terminal_state() {
    // Init once for both scenarios. The OnceLock in Queue::init means a second
    // call in the same process is silently ignored; all scenarios reuse this init.
    let (conn, _db_file) = setup_db().await;
    Queue::init(conn).await.expect("Queue::init");
    register_offload_hooks_with_broadcaster(Arc::new(Broadcaster::new()));
    let db = Queue::connection();

    // Guard ensures QUEUE_CONNECTION is removed even if a scenario panics.
    let _env = EnvGuard::set("QUEUE_CONNECTION", "sync");

    // ------------------------------------------------------------------
    // SC#1-success: sync dispatch of a returning job persists Completed.
    //
    // Uses a distinct HandleKey so this scenario's row is independent of
    // the failure scenario's row (no clear_tables needed between them).
    // ------------------------------------------------------------------
    {
        let key = HandleKey::new();
        PendingDispatch::new(SyncSuccessJob { expected_value: 42 })
            .with_handle_key(key.as_str().to_string())
            .dispatch()
            .await
            .expect("SC#1-success: sync dispatch must not return Err");

        let result = read_result::<i32>(key.as_str(), db)
            .await
            .expect("read_result")
            .expect("SC#1-success: completed envelope must be present after sync dispatch");

        match result {
            OffloadResult::Completed { value } => {
                assert_eq!(value, 42, "SC#1-success: value must match job output");
            }
            other => panic!("SC#1-success: expected Completed, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // SC#1-failure: sync dispatch of a failing job persists Failed AND
    // dispatch() returns Err (D-03 dual signal).
    // ------------------------------------------------------------------
    {
        let key = HandleKey::new();
        let dispatch_res = PendingDispatch::new(SyncFailingJob)
            .with_handle_key(key.as_str().to_string())
            .dispatch()
            .await;

        assert!(
            dispatch_res.is_err(),
            "D-03: sync dispatch must still return Err on failure"
        );

        let result = read_result::<()>(key.as_str(), db)
            .await
            .expect("read_result")
            .expect("SC#1-failure: failed envelope must be present after sync dispatch");

        match result {
            OffloadResult::Failed { .. } => {}
            other => panic!("SC#1-failure: expected Failed, got {other:?}"),
        }
    }
}

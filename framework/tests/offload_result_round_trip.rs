//! End-to-end integration harness for the offload result round-trip.
//!
//! Drives the full enqueue → claim → WorkerLoop drain → persist → read_result chain
//! against a temporary file-based SQLite database with both migrations registered.
//!
//! Success criteria exercised:
//! - SC1: after the worker completes a job, a completed envelope is persisted.
//! - SC2: the envelope is retrievable by the caller's handle key (key equality seam).
//! - SC3a: a job returning Err until max_retries is exhausted leaves a failed envelope.
//! - SC3b: a panicking job leaves a failed envelope (no silent drop).
//!
//! All four SC assertions run inside a SINGLE `#[tokio::test]` function, which
//! avoids the global-init race that would arise from concurrent async test tasks.
//! Between scenarios the two tables are cleared via DELETE to give each assertion
//! a clean state.
//!
//! Worker execution uses `WorkerLoop::drain_for_test` rather than `WorkerLoop::run`
//! to bypass the SIGTERM/Ctrl-C signal handler that `run()` installs and that can
//! fire spuriously in `cargo test` environments.

extern crate ferro_rs as ferro;

use ferro::offload::{read_result, register_offload_hooks, OffloadResult};
use ferro_queue::{
    async_trait, Error, Job, JobRegistrarEntry, Offloadable, Queue, WorkerConfig, WorkerLoop,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TestMigrator — registers BOTH required migrations
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
// Harness setup
// ---------------------------------------------------------------------------

/// Create the shared DB and run both migrations.
///
/// Uses a temporary file-based SQLite so all pool connections share the same
/// database. Plain `sqlite::memory:` gives each pool connection its own
/// isolated in-memory DB; the migrated tables would be invisible to the
/// connections opened by the WorkerLoop.
async fn setup_db() -> (DatabaseConnection, tempfile::NamedTempFile) {
    let db_file = tempfile::NamedTempFile::new().expect("create temp SQLite file");
    let url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let conn = Database::connect(&url)
        .await
        .expect("connect to temp SQLite file");
    TestMigrator::up(&conn, None)
        .await
        .expect("run both migrations (CreateJobsTable + CreateProjectionSnapshotsTable)");
    (conn, db_file) // db_file must stay alive for the test duration
}

/// Delete all rows from both tables so each scenario starts from a clean state.
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
// drain — claim+execute cycles via drain_for_test (no signal handler)
// ---------------------------------------------------------------------------

/// Drain all pending jobs from the queue.
///
/// Uses `WorkerLoop::drain_for_test` which runs the claim/execute cycle without
/// the SIGTERM/Ctrl-C signal handler. That handler can fire spuriously in
/// `cargo test` environments, causing `run()` to exit before processing any jobs.
///
/// After `drain_for_test` returns, sleeps 200 ms to let spawned job tasks complete
/// their hook writes before the test reads results back.
async fn drain() {
    let worker = WorkerLoop::from_registry(WorkerConfig {
        sleep_duration: std::time::Duration::from_millis(10),
        ..WorkerConfig::default()
    });
    worker
        .drain_for_test()
        .await
        .expect("drain_for_test completed without fatal error");
    // Give spawned job tasks time to complete hook writes and snapshot persistence.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

// ---------------------------------------------------------------------------
// Test jobs
// ---------------------------------------------------------------------------

/// A job that returns a known i32 value (success path — SC1 + SC2).
///
/// Overrides `handle_with_value` to capture and serialize the return value,
/// mirroring what the `#[offload]` macro emits for a `-> i32` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuccessJob {
    expected_value: i32,
}

#[async_trait]
impl Job for SuccessJob {
    // name() intentionally omitted: the default returns std::any::type_name::<Self>(),
    // which matches the key stored by WorkerLoop::register::<SuccessJob>().
    // Overriding to a short string would break the handler lookup in spawn_job.

    async fn handle(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_with_value(&self) -> Result<Option<serde_json::Value>, Error> {
        let v = serde_json::to_value(self.expected_value)
            .map_err(|e| Error::job_failed(std::any::type_name::<Self>(), e.to_string()))?;
        Ok(Some(v))
    }
}

// Register via inventory so WorkerLoop::from_registry picks up the handler.
inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<SuccessJob>(); },
        name: "SuccessJob",
        queue: None,
    }
}

impl Offloadable for SuccessJob {
    type Output = i32;
}

/// A job that always returns Err (SC3a).
///
/// `max_retries = 1` makes the first failure terminal immediately, avoiding
/// retry-delay waits in the test.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlwaysErrJob;

#[async_trait]
impl Job for AlwaysErrJob {
    // name() omitted: default returns std::any::type_name::<Self>(), matching register::<AlwaysErrJob>().

    fn max_retries(&self) -> u32 {
        1
    }

    fn retry_delay(&self, _attempt: u32) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    async fn handle(&self) -> Result<(), Error> {
        Err(Error::job_failed("AlwaysErrJob", "always fails"))
    }
}

inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<AlwaysErrJob>(); },
        name: "AlwaysErrJob",
        queue: None,
    }
}

impl Offloadable for AlwaysErrJob {
    type Output = ();
}

/// A job that always panics (SC3b).
///
/// `max_retries = 1` makes the first panic terminal immediately.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlwaysPanicJob;

#[async_trait]
impl Job for AlwaysPanicJob {
    // name() omitted: default returns std::any::type_name::<Self>(), matching register::<AlwaysPanicJob>().

    fn max_retries(&self) -> u32 {
        1
    }

    fn retry_delay(&self, _attempt: u32) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    async fn handle(&self) -> Result<(), Error> {
        panic!("intentional test panic");
    }
}

inventory::submit! {
    JobRegistrarEntry {
        register: |w: &mut WorkerLoop| { w.register::<AlwaysPanicJob>(); },
        name: "AlwaysPanicJob",
        queue: None,
    }
}

impl Offloadable for AlwaysPanicJob {
    type Output = ();
}

// ---------------------------------------------------------------------------
// Integration test — four SC assertions in one test function
//
// Running all scenarios in a single `#[tokio::test]` eliminates the
// global-init race: `Queue::init` is called exactly once, and each scenario
// runs sequentially with a fresh DB state (tables cleared between scenarios).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn offload_result_round_trip() {
    // Set DB mode so dispatch() inserts into the jobs table instead of running
    // synchronously in the calling task.
    std::env::set_var("QUEUE_CONNECTION", "db");

    // Initialise once for the whole test function.
    let (conn, _db_file) = setup_db().await; // _db_file keeps temp SQLite alive
    Queue::init(conn).await.expect("Queue::init");
    // Register the persistence hook before any WorkerLoop drain.
    register_offload_hooks();

    let db = Queue::connection();

    // ------------------------------------------------------------------
    // SC1: a completed job leaves a Completed envelope in the snapshot table.
    // SC2: the envelope is retrievable by the caller's handle key.
    // ------------------------------------------------------------------
    clear_tables(db).await;
    {
        let handle = SuccessJob { expected_value: 42 }
            .offload()
            .await
            .expect("SC1/SC2: offload dispatch");

        let key = handle.key().to_string();

        // Before drain: no snapshot yet (D-08 — no pending row).
        let before: Option<OffloadResult<i32>> = read_result::<i32>(&key, db)
            .await
            .expect("read_result before drain");
        assert!(before.is_none(), "D-08: no snapshot before worker drains");

        drain().await;

        // SC1: completed envelope persisted.
        // SC2: retrievable by the same handle.key() the caller holds.
        let result: OffloadResult<i32> = read_result::<i32>(&key, db)
            .await
            .expect("read_result after drain")
            .expect("SC1: completed envelope must be present after worker drains");

        match result {
            OffloadResult::Completed { value } => {
                assert_eq!(
                    value, 42,
                    "SC1: completed value must match the job's output"
                );
            }
            OffloadResult::Failed { error } => {
                panic!("SC1: expected Completed, got Failed {{ error: {error:?} }}")
            }
            OffloadResult::Pending => {
                panic!("SC1: expected Completed, got Pending")
            }
        }
    }

    // ------------------------------------------------------------------
    // retrieve_by_handle_after_complete (SC2 focused):
    // The key from handle.key() equals the projection_snapshots key.
    // ------------------------------------------------------------------
    clear_tables(db).await;
    {
        let handle = SuccessJob { expected_value: 99 }
            .offload()
            .await
            .expect("SC2: offload dispatch");

        let key = handle.key().to_string();
        drain().await;

        let result: OffloadResult<i32> = read_result::<i32>(&key, db)
            .await
            .expect("read_result by handle key")
            .expect("SC2: snapshot must be present by handle.key()");

        assert!(
            matches!(result, OffloadResult::Completed { value: 99 }),
            "SC2: read_result by handle.key() returned the correct completed value"
        );
    }

    // ------------------------------------------------------------------
    // SC3a: Err until max_retries exhausted → Failed envelope (no silent drop).
    // ------------------------------------------------------------------
    clear_tables(db).await;
    {
        let handle = AlwaysErrJob
            .offload()
            .await
            .expect("SC3a: offload dispatch");
        let key = handle.key().to_string();

        drain().await;

        let result: OffloadResult<()> = read_result::<()>(&key, db)
            .await
            .expect("read_result after Err exhaustion")
            .expect("SC3a: failed envelope must be present after retry exhaustion");

        match result {
            OffloadResult::Failed { error } => {
                assert!(
                    !error.is_empty(),
                    "SC3a: failed envelope must carry a non-empty error string"
                );
                assert!(
                    error.contains("always fails"),
                    "SC3a: error string must reference the original failure message, got: {error:?}"
                );
            }
            OffloadResult::Completed { .. } => {
                panic!("SC3a: expected OffloadResult::Failed, got Completed")
            }
            OffloadResult::Pending => {
                panic!("SC3a: expected OffloadResult::Failed, got Pending")
            }
        }
    }

    // ------------------------------------------------------------------
    // SC3b: panic until max_retries exhausted → Failed envelope (no silent drop).
    // ------------------------------------------------------------------
    clear_tables(db).await;
    {
        let handle = AlwaysPanicJob
            .offload()
            .await
            .expect("SC3b: offload dispatch");
        let key = handle.key().to_string();

        drain().await;

        let result: OffloadResult<()> = read_result::<()>(&key, db)
            .await
            .expect("read_result after panic exhaustion")
            .expect("SC3b: failed envelope must be present after panic — no silent drop");

        match result {
            OffloadResult::Failed { error } => {
                assert!(
                    error.contains("panicked"),
                    "SC3b: failed envelope must reference the panic; got: {error:?}"
                );
            }
            OffloadResult::Completed { .. } => {
                panic!(
                    "SC3b: expected OffloadResult::Failed, got Completed — panic silently dropped"
                )
            }
            OffloadResult::Pending => {
                panic!("SC3b: expected OffloadResult::Failed, got Pending")
            }
        }
    }
}

//! DISPATCH-KEY-01 — a real #[offload]-DERIVED job declared in a non-crate-root
//! module is enqueued on the db path, claimed, and dispatched by a WorkerLoop
//! exactly once. RED before the offload.rs name() fix (release-loop), GREEN after.
#![allow(dead_code)]

extern crate ferro_rs as ferro;

use std::sync::atomic::{AtomicBool, Ordering};

use ferro::{async_trait, service};
use ferro_queue::{dispatch, CreateJobsTable, Queue, WorkerConfig, WorkerLoop};
use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Serializable param type + module-static dispatch flag
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month(pub u32);

pub static DISPATCHED: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// The derived job — declared in a nested module so that type_name includes
// the module segment, making it differ from the bare struct ident.
// ---------------------------------------------------------------------------

mod reports {
    use super::*;

    #[derive(Default)]
    pub struct ReportBuilder;

    // Attribute ordering: #[service(..)] FIRST, then #[async_trait].
    #[service(ReportBuilder)]
    #[async_trait]
    pub trait Reports {
        #[offload]
        async fn build_monthly(&self, month: Month);
    }

    #[async_trait]
    impl Reports for ReportBuilder {
        async fn build_monthly(&self, _month: Month) {
            DISPATCHED.store(true, Ordering::SeqCst);
        }
    }
}

// ---------------------------------------------------------------------------
// Inline migrator — jobs table only
// ---------------------------------------------------------------------------

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

// ---------------------------------------------------------------------------
// Integration test — single outer tokio::test, one Queue::init per binary
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial]
async fn dispatch_key_derived_job_enqueue_claim_dispatch_suite() {
    // (a) Force the db path — default QUEUE_CONNECTION is "sync".
    std::env::set_var("QUEUE_CONNECTION", "db");

    // (b) Temp-file SQLite (never sqlite::memory: — pool connections would not share it).
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let conn = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn, None).await.unwrap();

    // (c) Global queue connection (OnceLock — once per binary).
    Queue::init(conn).await.expect("Queue::init");

    // (d) Activate the #[service] binding so the derived handle() can resolve `dyn Reports`.
    ferro::container::provider::bootstrap();

    // (e) Register the derived handler by type_name (deterministic; keys by type_name::<J>()).
    let mut worker = WorkerLoop::new(WorkerConfig {
        sleep_duration: std::time::Duration::from_millis(100),
        ..WorkerConfig::default()
    });
    worker.register::<reports::ReportsBuildMonthlyJob>();

    // (f) Enqueue the DERIVED job on the db path — writes job_type = job.name().
    dispatch(reports::ReportsBuildMonthlyJob { month: Month(7) })
        .await
        .expect("enqueue failed");

    // (g) Drain: claim + execute.
    worker.drain_for_test().await.expect("drain_for_test");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // (h) Dispatched exactly once: the method body ran.
    assert!(
        DISPATCHED.load(Ordering::SeqCst),
        "DISPATCH-KEY-01: derived job handle() never ran — the job was release-looped, \
         so the enqueue key (Job::name) does not equal the worker's type_name lookup key"
    );

    // (i) Not release-looped: the row was claimed+deleted, so jobs is empty.
    let conn = Queue::connection();
    let count = conn
        .query_one(Statement::from_string(
            conn.get_database_backend(),
            "SELECT COUNT(*) AS c FROM jobs".to_string(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "c")
        .unwrap();
    assert_eq!(
        count, 0,
        "job row still present — it was not claimed+deleted (release-loop)"
    );
}

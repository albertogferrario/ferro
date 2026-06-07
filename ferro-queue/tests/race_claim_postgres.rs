//! SC-1b: Postgres-gated mirror of `race_claim_sqlite.rs`.
//!
//! Run with:
//!   DATABASE_URL=postgres://user:pass@localhost:5432/ferro_test \
//!     cargo test -p ferro-queue --features postgres-tests \
//!     -- --test-threads=1
//!
//! `--test-threads=1` is REQUIRED for the Postgres path. Each test calls
//! `TestMigrator::down`/`up` on the shared database, which drops and recreates
//! the `jobs` table. With parallel test execution, two tests race on schema
//! operations and fail. The SQLite test uses isolated `NamedTempFile` databases
//! so parallel execution is safe there; Postgres tests share the live database
//! identified by `DATABASE_URL` and must serialize at the test harness level.
//!
//! Without the `postgres-tests` feature this file compiles to an empty module
//! and contributes zero tests to the default `cargo test` run.

#![cfg(feature = "postgres-tests")]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use ferro_queue::{claim, delete_job, enqueue, CreateJobsTable};

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

/// Connect to the Postgres instance at DATABASE_URL, drop and recreate the
/// `jobs` table, and return a fresh `DatabaseConnection`.
///
/// Returns `None` when `DATABASE_URL` is unset (typical CI without a Postgres
/// service), so callers can skip the test gracefully instead of panicking.
///
/// WARNING — DESTRUCTIVE: when `DATABASE_URL` is set, this calls
/// `TestMigrator::down` then `up`. The `jobs` table is dropped and recreated
/// on every invocation. NEVER point at a production database.
async fn fresh_pg_db() -> Option<DatabaseConnection> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let conn = Database::connect(&url).await.expect("connect to postgres");
    // down is a no-op when the table does not exist.
    let _ = TestMigrator::down(&conn, None).await;
    TestMigrator::up(&conn, None).await.expect("migrate");
    Some(conn)
}

/// SC-1b (Postgres): two workers claim each of N jobs exactly once.
///
/// `multi_thread` flavor: both workers run on distinct OS threads, stressing
/// the `FOR UPDATE SKIP LOCKED` path under true concurrency.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_workers_claim_each_job_exactly_once_postgres() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("DATABASE_URL not set — skipping postgres race test");
        return;
    }

    let conn_setup = fresh_pg_db().await.expect("DATABASE_URL checked above");

    // Enqueue N=20 jobs.
    const N: usize = 20;
    let now = chrono::Utc::now();
    for _ in 0..N {
        enqueue(
            &conn_setup,
            "default",
            "TestJobPg",
            "{}",
            3,
            None,
            None,
            now,
        )
        .await
        .expect("enqueue failed");
    }

    // Open two independent connections to the same Postgres database.
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let conn1 = Database::connect(&db_url)
        .await
        .expect("connect conn1 to postgres");
    let conn2 = Database::connect(&db_url)
        .await
        .expect("connect conn2 to postgres");

    async fn drain(
        conn: sea_orm::DatabaseConnection,
        worker_id: &'static str,
        out: Arc<Mutex<Vec<i64>>>,
    ) {
        loop {
            match claim(&conn, "default", worker_id).await {
                Ok(Some(row)) => {
                    out.lock().unwrap().push(row.id);
                    let _ = delete_job(&conn, row.id).await;
                }
                Ok(None) => break,
                Err(e) => panic!("claim error: {e:?}"),
            }
        }
    }

    let c1: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
    let c2: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));

    let (h1, h2) = (
        tokio::spawn(drain(conn1, "w1", c1.clone())),
        tokio::spawn(drain(conn2, "w2", c2.clone())),
    );
    let _ = tokio::join!(h1, h2);

    let mut all: Vec<i64> = c1.lock().unwrap().clone();
    all.extend(c2.lock().unwrap().iter().cloned());
    let unique: HashSet<i64> = all.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        all.len(),
        "a job was claimed more than once (total claimed: {}, unique: {})",
        all.len(),
        unique.len()
    );
    assert_eq!(
        unique.len(),
        N,
        "not all jobs were claimed exactly once (expected {N}, got {})",
        unique.len()
    );
}

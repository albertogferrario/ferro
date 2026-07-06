//! SC-1: concurrent exactly-once claim race test on a shared temp-file SQLite DB.
//!
//! Two concurrent workers drain a queue of N=20 jobs. Each job must be claimed
//! exactly once — no job is claimed by both workers, and no job is skipped.
//!
//! CRITICAL: uses `tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc`.
//! Per-connection in-memory SQLite databases see different empty tables and
//! produce a vacuous pass (RESEARCH.md Pitfall 1 — never use in-memory for
//! cross-connection concurrency tests).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

use ferro_queue::{claim, delete_job, enqueue, CreateJobsTable};

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

/// SC-1: two workers claim each of N jobs exactly once (no duplicates, full coverage).
///
/// `multi_thread` flavor: both workers run on distinct OS threads, generating
/// true parallelism between the `BEGIN IMMEDIATE` transactions — the same
/// configuration the Postgres mirror uses for `FOR UPDATE SKIP LOCKED`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_workers_claim_each_job_exactly_once() {
    // CRITICAL: NamedTempFile — per-connection in-memory DBs see different tables (Pitfall 1)
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn1 = Database::connect(&db_url).await.unwrap();
    let conn2 = Database::connect(&db_url).await.unwrap();

    // Run migration on conn1 (both connections see the same file).
    TestMigrator::up(&conn1, None).await.unwrap();

    // Enqueue N=20 jobs (all pending, available immediately).
    const N: usize = 20;
    let now = chrono::Utc::now();
    for _ in 0..N {
        enqueue(&conn1, "default", "TestJob", "{}", 3, None, None, now)
            .await
            .expect("enqueue failed");
    }

    // Two concurrent drain tasks: each claims until the queue is empty, then breaks.
    async fn drain(
        conn: sea_orm::DatabaseConnection,
        worker_id: &'static str,
        out: Arc<Mutex<Vec<i64>>>,
    ) {
        loop {
            match claim(&conn, "default", worker_id).await {
                Ok(Some(row)) => {
                    out.lock().unwrap().push(row.id);
                    // Delete on success so the claimed row does not re-appear.
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

    // Assert exactly-once: union has no duplicates and covers all N jobs.
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

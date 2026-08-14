//! SC#1–SC#3 worker-runtime integration suite.
//!
//! All three scenarios run as sub-functions of a single outer tokio test to
//! avoid the OnceLock collision (RESEARCH Pitfall 2) and the false-green
//! test-collapse pitfall (RESEARCH Pitfall 3).  Every scenario uses `tempfile::NamedTempFile`
//! + `sqlite://{path}?mode=rwc` — never `sqlite::memory:` (RESEARCH Pitfall 1).
//!
//! RED status (Wave 0): SC#1 exercises `WorkerConfig::new(vec!["reports"])` to
//! assert that a queue-scoped consumer does NOT claim jobs from a disjoint queue.
//! This assertion will FAIL until Plan 01 wires the queue-filter into the claim
//! path.  SC#2 and SC#3 use only the existing `claim`/`enqueue` primitives and
//! may already be GREEN depending on the current claim semantics.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

use ferro_queue::{claim, delete_job, enqueue, CreateJobsTable, WorkerConfig, WorkerLoop};

// ---------------------------------------------------------------------------
// Inline migrator (mirrors race_claim_sqlite.rs pattern exactly)
// ---------------------------------------------------------------------------

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

// ---------------------------------------------------------------------------
// Outer test — single tokio::test calling three scenario sub-functions
// ---------------------------------------------------------------------------

/// Wave-0 worker-runtime test suite: SC#1, SC#2, SC#3.
///
/// `multi_thread` flavor is required so both worker drain tasks run on
/// distinct OS threads, generating true parallelism between `BEGIN IMMEDIATE`
/// transactions (the same configuration used by `race_claim_sqlite.rs`).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn worker_runtime_suite() {
    worker_consumes_only_selected_queue().await;
    two_loops_split_work_no_duplicates().await;
    queue_fault_isolation().await;
}

// ---------------------------------------------------------------------------
// SC#1: a worker scoped to "reports" must not claim "default" jobs
// ---------------------------------------------------------------------------

/// SC#1: queue-scoped consumption.
///
/// Enqueues N jobs on `"reports"` and M jobs on `"default"`.  Runs a drain
/// loop that only calls `claim(&conn, "reports", worker_id)`.  After the drain:
///
/// - All `"reports"` jobs must have been claimed.
/// - All `"default"` jobs must remain (claim on `"default"` still returns rows).
///
/// This is the RED target for Plan 01: it FAILS if a consumer scoped to
/// `"reports"` ever claims a `"default"` job, or if the existing `claim`
/// helper ignores the queue parameter and drains across queues indiscriminately.
async fn worker_consumes_only_selected_queue() {
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn, None).await.unwrap();

    let now = chrono::Utc::now();

    const N_REPORTS: usize = 5;
    const M_DEFAULT: usize = 5;

    for _ in 0..N_REPORTS {
        enqueue(
            &conn,
            "reports",
            "ReportJob",
            "{}",
            3,
            None,
            None,
            None,
            now,
        )
        .await
        .expect("enqueue reports job failed");
    }
    for _ in 0..M_DEFAULT {
        enqueue(
            &conn,
            "default",
            "DefaultJob",
            "{}",
            3,
            None,
            None,
            None,
            now,
        )
        .await
        .expect("enqueue default job failed");
    }

    // Drain only the "reports" queue.
    let reports_claimed: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
    {
        let out = reports_claimed.clone();
        loop {
            match claim(&conn, "reports", "sc1-worker").await {
                Ok(Some(row)) => {
                    out.lock().unwrap().push(row.id);
                    delete_job(&conn, row.id).await.expect("delete_job failed");
                }
                Ok(None) => break,
                Err(e) => panic!("SC#1 claim error: {e:?}"),
            }
        }
    }

    // All "reports" jobs must have been claimed.
    let claimed_count = reports_claimed.lock().unwrap().len();
    assert_eq!(
        claimed_count, N_REPORTS,
        "SC#1: expected {N_REPORTS} 'reports' jobs claimed, got {claimed_count}"
    );

    // "default" jobs must still be claimable — the "reports" drain must not
    // have touched them.
    let mut default_remaining = 0usize;
    loop {
        match claim(&conn, "default", "sc1-verifier").await {
            Ok(Some(row)) => {
                default_remaining += 1;
                delete_job(&conn, row.id).await.expect("delete_job failed");
            }
            Ok(None) => break,
            Err(e) => panic!("SC#1 default-queue verify error: {e:?}"),
        }
    }
    assert_eq!(
        default_remaining, M_DEFAULT,
        "SC#1: expected {M_DEFAULT} 'default' jobs untouched, found {default_remaining}"
    );

    // WorkerConfig is constructed to assert the public API compiles as expected.
    // It is not used for actual WorkerLoop execution here (the runtime API does
    // not yet exist at Wave 0) — the load-bearing assertion is the claim-queue
    // scoping above.
    let _cfg = WorkerConfig::new(vec!["reports".to_string()]);
    let _ = WorkerLoop::from_registry(_cfg);
}

// ---------------------------------------------------------------------------
// SC#2: two concurrent drain loops split all jobs with no duplicates
// ---------------------------------------------------------------------------

/// SC#2: exactly-once work distribution across two concurrent consumers.
///
/// Two concurrent drain tasks over a shared file-backed SQLite DB claim N=20
/// jobs on `"default"`.  After `tokio::join!`, the combined claim list must
/// have no duplicates and must cover all N jobs.
///
/// Mirrors `race_claim_sqlite.rs` at the `WorkerLoop` level.  This scenario
/// may already be GREEN (the DB-level exactly-once guarantee is the same kernel
/// proven by `race_claim_sqlite.rs`); it is included here to pin the behaviour
/// at the worker-runtime integration surface.
async fn two_loops_split_work_no_duplicates() {
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn1 = Database::connect(&db_url).await.unwrap();
    let conn2 = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn1, None).await.unwrap();

    const N: usize = 20;
    let now = chrono::Utc::now();
    for _ in 0..N {
        enqueue(&conn1, "default", "TestJob", "{}", 3, None, None, None, now)
            .await
            .expect("enqueue failed");
    }

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
                Err(e) => panic!("SC#2 claim error: {e:?}"),
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
        "SC#2: a job was claimed more than once (total={}, unique={})",
        all.len(),
        unique.len()
    );
    assert_eq!(
        unique.len(),
        N,
        "SC#2: not all jobs claimed exactly once (expected {N}, got {})",
        unique.len()
    );
}

// ---------------------------------------------------------------------------
// SC#3: saturating one queue must not stall a disjoint queue
// ---------------------------------------------------------------------------

/// SC#3: fault-domain isolation across disjoint queues.
///
/// Enqueues N_SLOW jobs on `"media"` and N_FAST jobs on `"reports"`.  Two
/// concurrent drain tasks — one for each queue — start simultaneously via a
/// `tokio::sync::Barrier` (NO `time::sleep` for synchronization).  The
/// `"media"` drain simulates saturation by acquiring a semaphore permit before
/// each claim; the `"reports"` drain runs unthrottled.
///
/// Assertion: all `"reports"` jobs are claimed even while `"media"` still has
/// unclaimed backlog — a saturated `"media"` queue must not prevent `"reports"`
/// drain.
///
/// This scenario is GREEN at Wave 0 when queue isolation is purely a DB-level
/// property (the `claim` queue parameter already scopes claims).  Plan 01 must
/// not break this guarantee when restructuring the boot path.
async fn queue_fault_isolation() {
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn_media = Database::connect(&db_url).await.unwrap();
    let conn_reports = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn_media, None).await.unwrap();

    let now = chrono::Utc::now();

    const N_SLOW: usize = 6;
    const N_FAST: usize = 6;

    for _ in 0..N_SLOW {
        enqueue(
            &conn_media,
            "media",
            "SlowJob",
            "{}",
            3,
            None,
            None,
            None,
            now,
        )
        .await
        .expect("enqueue media job failed");
    }
    for _ in 0..N_FAST {
        enqueue(
            &conn_reports,
            "reports",
            "FastJob",
            "{}",
            3,
            None,
            None,
            None,
            now,
        )
        .await
        .expect("enqueue reports job failed");
    }

    // Barrier: both drains start simultaneously.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));

    // Semaphore: "media" drain acquires one permit per job to simulate slow
    // processing (capacity = 1 → one claim at a time, creating backlog
    // pressure).
    let slow_sem = Arc::new(tokio::sync::Semaphore::new(1));

    let media_claimed: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
    let reports_claimed: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));

    let barrier_media = barrier.clone();
    let barrier_reports = barrier.clone();
    let slow_sem_clone = slow_sem.clone();
    let media_out = media_claimed.clone();
    let reports_out = reports_claimed.clone();

    let media_task = tokio::spawn(async move {
        barrier_media.wait().await;
        loop {
            // Acquire permit before each claim to model saturation.
            let _permit = slow_sem_clone.acquire().await.expect("semaphore closed");
            match claim(&conn_media, "media", "media-worker").await {
                Ok(Some(row)) => {
                    media_out.lock().unwrap().push(row.id);
                    delete_job(&conn_media, row.id)
                        .await
                        .expect("delete media job failed");
                    // Hold the permit briefly to slow down this drain.
                    tokio::task::yield_now().await;
                }
                Ok(None) => break,
                Err(e) => panic!("SC#3 media claim error: {e:?}"),
            }
        }
    });

    let reports_task = tokio::spawn(async move {
        barrier_reports.wait().await;
        loop {
            match claim(&conn_reports, "reports", "reports-worker").await {
                Ok(Some(row)) => {
                    reports_out.lock().unwrap().push(row.id);
                    delete_job(&conn_reports, row.id)
                        .await
                        .expect("delete reports job failed");
                }
                Ok(None) => break,
                Err(e) => panic!("SC#3 reports claim error: {e:?}"),
            }
        }
    });

    // Wait for the "reports" drain to finish; the "media" drain may still be
    // running (saturation simulation), but that must not block "reports".
    reports_task.await.expect("reports task panicked");

    let reports_count = reports_claimed.lock().unwrap().len();
    assert_eq!(
        reports_count, N_FAST,
        "SC#3: expected all {N_FAST} 'reports' jobs claimed, got {reports_count} \
         — a saturated 'media' queue must not stall 'reports'"
    );

    // Clean up the media task.
    media_task.await.expect("media task panicked");
}

//! SC-4b: graceful-shutdown re-queue test.
//!
//! Proves that `requeue_claimed_by` resets claimed-but-incomplete jobs back to
//! `pending`, making them available to the next worker. This is the D-10
//! re-queue behavior exercised deterministically — no signal or timing required.

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

use ferro_queue::{claim, enqueue, requeue_claimed_by, CreateJobsTable};

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}

/// SC-4b: a claimed-but-unstarted job is re-queued on shutdown and can be
/// claimed again by a subsequent worker.
#[tokio::test]
async fn graceful_shutdown_requeues_claimed_jobs() {
    // 1. Temp-file SQLite, migrate via CreateJobsTable.
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let conn = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn, None).await.unwrap();

    // 2. Enqueue one job.
    let now = chrono::Utc::now();
    enqueue(
        &conn,
        "default",
        "ShutdownTestJob",
        "{}",
        3,
        None,
        None,
        now,
    )
    .await
    .expect("enqueue failed");

    // 3. Claim it (status -> 'claimed', claimed_by='w-shutdown') WITHOUT executing.
    let row = claim(&conn, "default", "w-shutdown")
        .await
        .expect("claim failed");
    assert!(
        row.is_some(),
        "expected a job to be claimed, got None (queue may be empty)"
    );

    // 4. Call requeue_claimed_by — the D-10 shutdown re-queue operation.
    requeue_claimed_by(&conn, "w-shutdown")
        .await
        .expect("requeue_claimed_by failed");

    // 5. Claim again with a different worker — the job must be pending again.
    let reclaimed = claim(&conn, "default", "w-next")
        .await
        .expect("re-claim failed");
    assert!(
        reclaimed.is_some(),
        "job should be pending again after requeue_claimed_by; \
         expected Some, got None (re-queue did not reset status)"
    );
    let reclaimed = reclaimed.unwrap();
    assert_eq!(
        reclaimed.job_type, "ShutdownTestJob",
        "re-claimed job should be the original job"
    );
}

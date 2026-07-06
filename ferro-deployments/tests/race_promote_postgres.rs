//! SC-2b: Postgres-gated mirror of `race_promote_sqlite.rs`.
//!
//! Run with:
//!   DATABASE_URL=postgres://user:pass@localhost:5432/ferro_test \
//!     cargo test -p ferro-deployments --features postgres-tests \
//!     -- --test-threads=1
//!
//! `--test-threads=1` is REQUIRED for the Postgres path. Each test calls
//! `TestMigrator::down`/`up` on the shared database, which drops and recreates
//! both tables. With parallel test execution, two tests race on schema
//! operations and fail.
//!
//! Without the `postgres-tests` feature this file compiles to an empty module
//! and contributes zero tests to the default `cargo test` run.

#![cfg(feature = "postgres-tests")]

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;

use ferro_deployments::{CreateDeploymentPointersTable, CreateDeploymentsTable, Deployments};

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(CreateDeploymentsTable),
            Box::new(CreateDeploymentPointersTable),
        ]
    }
}

/// Connect to the Postgres instance at DATABASE_URL, drop and recreate both
/// tables, and return a fresh `DatabaseConnection`.
///
/// Returns `None` when `DATABASE_URL` is unset (typical CI without a Postgres
/// service), so callers can skip the test gracefully instead of panicking.
///
/// WARNING — DESTRUCTIVE: when `DATABASE_URL` is set, this calls
/// `TestMigrator::down` then `up`. Both tables are dropped and recreated on
/// every invocation. NEVER point at a production database.
async fn fresh_pg_db() -> Option<DatabaseConnection> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let conn = Database::connect(&url).await.expect("connect to postgres");
    // down is a no-op when the tables do not exist.
    let _ = TestMigrator::down(&conn, None).await;
    TestMigrator::up(&conn, None).await.expect("migrate");
    Some(conn)
}

/// SC-2b (Postgres): two concurrent promoters for the same owner_key both
/// complete without error; the final pointer is in a consistent state.
///
/// `multi_thread` flavor: both promoters run on distinct OS threads, stressing
/// the ON CONFLICT DO UPDATE path under true concurrency.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_promoters_last_write_wins_postgres() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("DATABASE_URL not set — skipping postgres race test");
        return;
    }

    let conn_setup = fresh_pg_db().await.expect("DATABASE_URL checked above");

    // Create two ready deployments.
    let d_setup = Deployments::new(conn_setup.clone());
    let dep_a = d_setup.create("owner:1", None).await.expect("create dep_a");
    let dep_b = d_setup.create("owner:1", None).await.expect("create dep_b");
    d_setup
        .mark_ready(dep_a.id, "a/", 1)
        .await
        .expect("mark dep_a ready");
    d_setup
        .mark_ready(dep_b.id, "b/", 2)
        .await
        .expect("mark dep_b ready");

    let dep_a_id = dep_a.id;
    let dep_b_id = dep_b.id;

    // Open two independent connections to the same Postgres database.
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let conn1 = Database::connect(&db_url)
        .await
        .expect("connect conn1 to postgres");
    let conn2 = Database::connect(&db_url)
        .await
        .expect("connect conn2 to postgres");

    let d1 = Deployments::new(conn1.clone());
    let d2 = Deployments::new(conn2);

    let (h1, h2) = (
        tokio::spawn(async move { d1.promote("owner:1", dep_a_id).await }),
        tokio::spawn(async move { d2.promote("owner:1", dep_b_id).await }),
    );
    let (r1, r2) = tokio::join!(h1, h2);

    let r1 = r1.expect("h1 panicked");
    let r2 = r2.expect("h2 panicked");

    eprintln!("promoter 1 result: {r1:?}");
    eprintln!("promoter 2 result: {r2:?}");

    // Query the pointer row directly.
    let pointer_row = conn1
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT deployment_id, previous_deployment_id \
             FROM deployment_pointers WHERE owner_key = 'owner:1'"
                .to_string(),
        ))
        .await
        .expect("query pointer row")
        .expect("pointer row must exist after at least one successful promote");

    let final_id: i64 = pointer_row
        .try_get_by::<i64, _>("deployment_id")
        .expect("deployment_id");
    let prev_id: Option<i64> = pointer_row
        .try_get_by::<Option<i64>, _>("previous_deployment_id")
        .expect("previous_deployment_id");

    assert!(
        final_id == dep_a_id || final_id == dep_b_id,
        "pointer.deployment_id ({final_id}) must be dep_a ({dep_a_id}) or dep_b ({dep_b_id})"
    );

    if let Some(prev) = prev_id {
        assert_ne!(
            prev, final_id,
            "torn state: previous_deployment_id == deployment_id ({final_id})"
        );
    }
}

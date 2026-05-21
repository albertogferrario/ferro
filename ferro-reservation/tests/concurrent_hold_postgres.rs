//! Postgres-gated mirror of `concurrent_hold.rs` (Phase 177 SC-1, SC-5).
//!
//! Run with:
//!   DATABASE_URL=postgres://user:pass@localhost:5432/ferro_test \
//!     cargo test -p ferro-reservation --features postgres-tests \
//!     -- --test-threads=1
//!
//! `--test-threads=1` is REQUIRED for the Postgres path. Each test calls
//! `TestMigrator::down`/`up` on the shared database, which creates and drops
//! the `reservations` and `audit_entries` tables along with their backing
//! Postgres types. With cargo's default parallel test execution, two tests
//! race on `pg_catalog.pg_type` and fail with SQLSTATE 23505
//! ("duplicate key value violates unique constraint pg_type_typname_nsp_index").
//! Cargo's default parallelism is fine for the SQLite tests (each `fresh_db`
//! creates a new in-memory connection), but Postgres tests share the live
//! database identified by `DATABASE_URL` and must serialize at the test
//! harness level.
//!
//! Requires:
//!   - A reachable Postgres instance at `DATABASE_URL`.
//!   - The database is empty (the test calls `TestMigrator::up` to create
//!     `audit_entries` and `reservations` tables fresh on each test).
//!
//! Without the `postgres-tests` feature, this file compiles to an empty
//! module and contributes zero tests to the default `cargo test` run.

#![cfg(feature = "postgres-tests")]

use async_trait::async_trait;
use ferro_reservation::{ReservationContext, ReservationError, ReservationKernel, Resource};
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use std::time::Duration;

struct TestMigrator;

#[async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_audit::CreateAuditLogTable),
            Box::new(ferro_reservation::CreateReservationsTable),
        ]
    }
}

/// Connect to the Postgres instance pointed at by DATABASE_URL, drop and
/// recreate the two tables this crate owns, and return a fresh
/// `DatabaseConnection`.
///
/// WARNING — DESTRUCTIVE: this function calls `TestMigrator::down` then `up`
/// on whatever `DATABASE_URL` points at. The `audit_entries` and
/// `reservations` tables are dropped and recreated on every test invocation.
/// NEVER run with `DATABASE_URL` pointing at a production database, a shared
/// staging database, or any database whose contents you wish to preserve.
/// Use a dedicated test database (typically localhost via docker-compose).
async fn fresh_pg_db() -> DatabaseConnection {
    let url = std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must be set for the postgres-tests feature. \
         WARNING: this test is DESTRUCTIVE — it drops and recreates the \
         `audit_entries` and `reservations` tables. Use a dedicated test \
         database (e.g. postgres://test:test@localhost:5432/ferro_test). \
         Never point at production or shared staging.",
    );
    let conn = Database::connect(&url).await.expect("connect to postgres");
    // Down-then-up so re-running the test on the same DB does not collide
    // on already-existing tables. `down` is a no-op if tables do not exist.
    let _ = TestMigrator::down(&conn, None).await;
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}

/// Identical to the SQLite-side `TestResource` in `concurrent_hold.rs`.
#[derive(Clone)]
struct TestResource {
    capacity_value: u32,
}

#[async_trait]
impl Resource for TestResource {
    type Key = String;
    type Window = ();
    const KIND: &'static str = "test.concurrent_hold_postgres";

    async fn capacity<C: ConnectionTrait>(
        &self,
        _conn: &C,
        _key: &Self::Key,
        _window: &Self::Window,
    ) -> Result<u32, ReservationError> {
        Ok(self.capacity_value)
    }

    async fn held<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        _window: &Self::Window,
    ) -> Result<u32, ReservationError> {
        use ferro_reservation::ReservationEntity;
        let key_json = serde_json::to_value(key)?;
        let rows = ReservationEntity::find()
            .filter(<ReservationEntity as EntityTrait>::Column::ResourceKind.eq(Self::KIND))
            .filter(<ReservationEntity as EntityTrait>::Column::ResourceKey.eq(key_json))
            .filter(
                <ReservationEntity as EntityTrait>::Column::Status.is_in(vec!["held", "committed"]),
            )
            .all(conn)
            .await
            .map_err(ReservationError::Db)?;
        let total: i32 = rows.iter().map(|r| r.quantity).sum();
        Ok(total.max(0) as u32)
    }
}

/// SC-1 (Postgres): 50 iterations of (2 tasks race on capacity=1) ->
/// exactly 1 Ok + 1 Insufficient. Validates SERIALIZABLE isolation +
/// SQLSTATE 40001 -> ReservationError::Insufficient translation.
///
/// `multi_thread` flavor: Postgres SSI contention is more faithfully
/// stressed when racing tasks run on distinct OS threads. `current_thread`
/// can mask races by serializing `.await` resumption on the cooperative
/// scheduler.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_capacity_1_exactly_one_succeeds_postgres() {
    for iteration in 0..50 {
        let conn = Arc::new(fresh_pg_db().await);
        let kernel = Arc::new(ReservationKernel::new(
            (*conn).clone(),
            TestResource { capacity_value: 1 },
        ));
        // Distinct key per iteration so we do not collide on the prior
        // iteration's row (fresh_pg_db wiped + recreated tables, but
        // using unique keys is belt-and-braces against migration surprises).
        let key = format!("race_key_pg_{iteration}");

        let mut handles = Vec::with_capacity(2);
        for _ in 0..2 {
            let kernel = kernel.clone();
            let conn = conn.clone();
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                let ctx = ReservationContext::system();
                kernel
                    .hold(&*conn, key, (), 1, Duration::from_secs(60), &ctx)
                    .await
            }));
        }

        let mut successes = 0usize;
        let mut insufficient = 0usize;
        for h in handles {
            match h.await.expect("join") {
                Ok(_) => successes += 1,
                Err(ReservationError::Insufficient { .. }) => insufficient += 1,
                Err(e) => panic!(
                    "iteration {iteration}: expected Ok or Insufficient, got {e:?} \
                     (SQLSTATE 40001 should be translated to Insufficient by the kernel)"
                ),
            }
        }

        assert_eq!(successes, 1, "iteration {iteration}: expected exactly 1 Ok");
        assert_eq!(
            insufficient, 1,
            "iteration {iteration}: expected exactly 1 Insufficient"
        );
    }
}

/// SC-5 (Postgres): after a capacity=1 race resolves, exactly 1
/// reservation row exists. The conflict-losing task's row and audit row
/// were rolled back with its serializable transaction.
/// `multi_thread` flavor: matches the SC-1 race test rationale — distinct
/// OS threads stress Postgres SSI contention faithfully.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_audit_atomicity_exactly_one_row_postgres() {
    let conn = Arc::new(fresh_pg_db().await);
    let kernel = Arc::new(ReservationKernel::new(
        (*conn).clone(),
        TestResource { capacity_value: 1 },
    ));
    let key = "audit_race_pg".to_string();

    let mut handles = Vec::with_capacity(2);
    for _ in 0..2 {
        let kernel = kernel.clone();
        let conn = conn.clone();
        let key = key.clone();
        handles.push(tokio::spawn(async move {
            let ctx = ReservationContext::system();
            kernel
                .hold(&*conn, key, (), 1, Duration::from_secs(60), &ctx)
                .await
        }));
    }

    let mut successful_ids = Vec::new();
    for h in handles {
        match h.await.expect("join") {
            Ok(handle) => successful_ids.push(handle.id),
            Err(ReservationError::Insufficient { .. }) => {}
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    assert_eq!(
        successful_ids.len(),
        1,
        "expected exactly 1 successful hold"
    );

    for id in &successful_ids {
        let history = ferro_audit::history_for_target(
            &ferro_audit::AuditTarget::new("reservation", id.to_string()),
            &*conn,
        )
        .await
        .expect("audit query");
        assert_eq!(
            history.len(),
            1,
            "expected exactly 1 audit entry for reservation {id}"
        );
        assert_eq!(history[0].action, "reservation.held");
    }

    use ferro_reservation::ReservationEntity;
    let all_reservations = ReservationEntity::find()
        .all(&*conn)
        .await
        .expect("count all reservations");
    assert_eq!(
        all_reservations.len(),
        1,
        "Postgres DB must contain exactly 1 reservation row — \
         conflict-loser row rolled back with its transaction"
    );
}

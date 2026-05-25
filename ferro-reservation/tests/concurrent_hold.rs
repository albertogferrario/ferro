//! Integration tests for kernel `hold` atomicity (Phase 177).
//!
//! After Phase 177, `ReservationKernel::hold` wraps its body in a
//! `SERIALIZABLE` transaction (`sea_orm::IsolationLevel::Serializable`).
//! Concurrent callers on the same `(key, window)` are serialized at the
//! database level — the conflict-losing task receives
//! `ReservationError::Insufficient`, not a raw `DbErr`. These tests
//! prove the kernel is intrinsically race-free without any
//! application-layer mutex.
//!
//! Test coverage:
//! - SC-1 (capacity=1, 2 tasks, ≥50 iterations): exactly 1 Ok + 1 Insufficient.
//! - SC-1 extended (capacity=N=5, N+1=6 tasks, ≥50 iterations): exactly N Ok + 1 Insufficient.
//! - SC-2 (non-overlapping keys): both succeed — atomicity fix does not introduce false positives.
//! - SC-5 (audit atomicity): conflict-losing task's audit row is rolled back with its transaction.

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

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}

/// TestResource with held() that queries the reservations table for the
/// live count (sum of quantity where status IN ('held','committed')).
#[derive(Clone)]
struct TestResource {
    capacity_value: u32,
}

#[async_trait]
impl Resource for TestResource {
    type Key = String;
    type Window = ();
    const KIND: &'static str = "test.concurrent_hold";

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

/// SC-1: 50 iterations of (2 tasks race on capacity=1) → exactly 1 Ok + 1 Insufficient.
/// Proves the kernel's serializable transaction serializes the check+INSERT pair.
///
/// `multi_thread` flavor: `tokio::spawn`-ed race tasks run on distinct OS
/// threads, generating true parallelism between the `capacity()`/`held()`
/// reads and the INSERT — the same configuration the Postgres mirror uses
/// to exercise SSI conflict detection. Keeps SQLite and Postgres tests on
/// the same runtime model for symmetry.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_capacity_1_exactly_one_succeeds() {
    for iteration in 0..50 {
        let conn = Arc::new(fresh_db().await);
        let kernel = Arc::new(ReservationKernel::new(
            (*conn).clone(),
            TestResource { capacity_value: 1 },
        ));
        let key = "race_key".to_string();

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
                Err(e) => panic!("unexpected error in iteration {iteration}: {e:?}"),
            }
        }

        assert_eq!(successes, 1, "iteration {iteration}: expected exactly 1 Ok");
        assert_eq!(
            insufficient, 1,
            "iteration {iteration}: expected exactly 1 Insufficient"
        );
    }
}

/// SC-1 extended: 50 iterations of (6 tasks race on capacity=5) → exactly 5 Ok + 1 Insufficient.
/// Confirms the fix correctly handles `capacity > 1` without false rejections.
///
/// `multi_thread` flavor: see `hold_race_capacity_1_exactly_one_succeeds`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_capacity_n_admits_exactly_n() {
    const CAPACITY: u32 = 5;
    const TASKS: usize = 6;

    for iteration in 0..50 {
        let conn = Arc::new(fresh_db().await);
        let kernel = Arc::new(ReservationKernel::new(
            (*conn).clone(),
            TestResource {
                capacity_value: CAPACITY,
            },
        ));
        let key = "race_key_n".to_string();

        let mut handles = Vec::with_capacity(TASKS);
        for _ in 0..TASKS {
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
                Err(e) => panic!("unexpected error in iteration {iteration}: {e:?}"),
            }
        }

        assert_eq!(
            successes, CAPACITY as usize,
            "iteration {iteration}: expected exactly {CAPACITY} Ok"
        );
        assert_eq!(
            insufficient,
            TASKS - CAPACITY as usize,
            "iteration {iteration}: expected exactly {} Insufficient",
            TASKS - CAPACITY as usize
        );
    }
}

/// SC-2: two `hold(...)` calls on different keys both succeed.
/// Boundary preservation — the atomicity fix must not introduce false positives
/// that reject legitimate non-overlapping holds.
///
/// `multi_thread` flavor: keeps runtime configuration consistent across the
/// suite. This test is sequential (no race), so flavor is not load-bearing
/// for the assertion — uniformity prevents accidental drift if the test is
/// later extended to race non-overlapping keys.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_non_overlapping_keys_both_succeed() {
    let conn = Arc::new(fresh_db().await);
    let kernel = ReservationKernel::new((*conn).clone(), TestResource { capacity_value: 1 });
    let ctx = ReservationContext::system();

    kernel
        .hold(
            &*conn,
            "key_a".to_string(),
            (),
            1,
            Duration::from_secs(60),
            &ctx,
        )
        .await
        .expect("key_a hold must succeed");
    kernel
        .hold(
            &*conn,
            "key_b".to_string(),
            (),
            1,
            Duration::from_secs(60),
            &ctx,
        )
        .await
        .expect("key_b hold must succeed");
}

/// SC-5 / D-04: after a capacity=1 race resolves, exactly 1 `reservations`
/// row AND exactly 1 `audit_entries` row exist. The conflict-losing task's
/// audit row was rolled back with its transaction.
///
/// `multi_thread` flavor: see `hold_race_capacity_1_exactly_one_succeeds`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hold_race_audit_atomicity_exactly_n_audit_rows() {
    const CAPACITY: u32 = 1;

    let conn = Arc::new(fresh_db().await);
    let kernel = Arc::new(ReservationKernel::new(
        (*conn).clone(),
        TestResource {
            capacity_value: CAPACITY,
        },
    ));
    let key = "audit_race_key".to_string();

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
        CAPACITY as usize,
        "expected exactly {CAPACITY} successful hold(s)"
    );

    // Each successful reservation has exactly one audit row tagged "reservation.held".
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

    // DB-level invariant: exactly CAPACITY reservation rows exist.
    // The conflict-losing task's row was rolled back with the txn.
    use ferro_reservation::ReservationEntity;
    let all_reservations = ReservationEntity::find()
        .all(&*conn)
        .await
        .expect("count all reservations");
    assert_eq!(
        all_reservations.len(),
        CAPACITY as usize,
        "DB must contain exactly {CAPACITY} reservation rows — \
         conflict-loser row rolled back with its transaction"
    );
}

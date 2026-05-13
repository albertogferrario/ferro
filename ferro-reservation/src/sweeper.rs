//! `SweepReport` + `ReservationKernel::run_sweep_once` (D-21..D-24).
//!
//! The sweeper is consumer-scheduled (D-22 — no `ferro-queue` runtime
//! dependency). Three idiomatic scheduling patterns:
//!
//! 1. **ferro-queue `Job`** — implement a `Job` that calls
//!    `kernel.run_sweep_once().await` and schedule via the queue.
//! 2. **`tokio::time::interval` task** — spawn a 60-second loop on
//!    application start.
//! 3. **Cron-driven CLI** — `your-app reservation:sweep` calls
//!    `kernel.run_sweep_once()` and exits.
//!
//! Per-row contention is safe under concurrent sweepers: each
//! transition uses `GuardedUpdate::exec_at_most_one` (D-24), which
//! treats 0-rows-affected as a normal outcome (the other sweeper won).

use chrono::{DateTime, Utc};
use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_orm::{GuardedUpdate, Value};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde_json::json;

use crate::entity as reservations;
use crate::error::ReservationError;
use crate::event::ReservationEvent;
use crate::kernel::ReservationKernel;
use crate::resource::Resource;

/// Result of one sweeper invocation (D-21). Consumers typically log
/// this for observability; high `expired_count` values indicate a
/// sweep backlog and a need to schedule sweeps more frequently.
#[derive(Clone, Debug)]
pub struct SweepReport {
    pub expired_count: u32,
    pub scanned_at: DateTime<Utc>,
}

impl<R: Resource> ReservationKernel<R> {
    /// Scan and expire held rows whose `expires_at` is in the past.
    /// Per-row idempotent under concurrent sweepers (D-24).
    ///
    /// Cap: 500 rows per call (D-21). Schedule subsequent sweeps if a
    /// backlog persists.
    ///
    /// Emits one [`ReservationEvent::Expired`] + one
    /// [`ferro_audit::AuditEntry`] (action `"reservation.expired"`,
    /// actor [`AuditActor::System`] per D-23) per row that transitioned.
    /// Audit failure is logged at `tracing::warn!` and does NOT propagate
    /// (the DB transition is already committed). Event-dispatch failure
    /// is also logged at `tracing::warn!` and does NOT propagate (D-26).
    pub async fn run_sweep_once(&self) -> Result<SweepReport, ReservationError> {
        let scanned_at = Utc::now();
        let now_naive = scanned_at.naive_utc();

        let expired_rows = reservations::Entity::find()
            .filter(reservations::Column::Status.eq("held"))
            .filter(reservations::Column::ExpiresAt.lt(now_naive))
            .limit(500)
            .all(&self.db)
            .await
            .map_err(ReservationError::Db)?;

        let mut expired_count: u32 = 0;

        for row in &expired_rows {
            let result = GuardedUpdate::new(reservations::Entity)
                .filter(reservations::Column::Id.eq(row.id))
                .filter(reservations::Column::Status.eq("held"))
                .set_value(
                    reservations::Column::Status,
                    Value::String(Some(Box::new("expired".to_string()))),
                )
                .exec_at_most_one(&self.db)
                .await;

            match result {
                Ok(true) => {
                    expired_count += 1;

                    // Audit (D-23 + D-28)
                    let mut audit = AuditEntry::record("reservation.expired")
                        .actor(AuditActor::System)
                        .target(AuditTarget::new("reservation", row.id.to_string()))
                        .before(json!({"status": "held", "quantity": row.quantity}))
                        .after(json!({"status": "expired"}));
                    if let Some(tid) = row.tenant_id.as_deref() {
                        audit = audit.tenant(tid);
                    }
                    if let Err(e) = audit.write(&self.db).await {
                        // D-30: audit failure surfaces but DB state is committed
                        tracing::warn!(
                            reservation_id = %row.id,
                            error = %e,
                            "audit write failed for reservation.expired — state is committed"
                        );
                    }

                    // Event dispatch (D-26 best-effort)
                    if let Err(e) = ferro_events::dispatch(ReservationEvent::Expired {
                        id: row.id,
                        resource_kind: row.resource_kind.clone(),
                        resource_key: row.resource_key.clone(),
                    })
                    .await
                    {
                        tracing::warn!(
                            reservation_id = %row.id,
                            error = %e,
                            "event dispatch failed after reservation.expired — state is committed"
                        );
                    }
                }
                Ok(false) => {
                    // Concurrent sweeper won; skip silently per D-24.
                }
                Err(e) => {
                    // DB error or GuardedError — log and continue; do not abort the sweep.
                    tracing::warn!(
                        reservation_id = %row.id,
                        error = %e,
                        "sweeper guarded update db error — skipping row"
                    );
                }
            }
        }

        Ok(SweepReport {
            expired_count,
            scanned_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration as ChronoDuration;
    use ferro_audit::CreateAuditLogTable;
    use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, Database, DatabaseConnection};
    use sea_orm_migration::MigratorTrait;
    use uuid::Uuid;

    struct TestMigrator;

    #[async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![
                Box::new(CreateAuditLogTable),
                Box::new(crate::migration::Migration),
            ]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    #[derive(Clone)]
    struct TestResource;

    #[async_trait]
    impl Resource for TestResource {
        type Key = String;
        type Window = ();
        const KIND: &'static str = "test.sweep";

        async fn capacity<C: ConnectionTrait>(
            &self,
            _conn: &C,
            _key: &Self::Key,
            _window: &Self::Window,
        ) -> Result<u32, ReservationError> {
            Ok(100)
        }

        async fn held<C: ConnectionTrait>(
            &self,
            _conn: &C,
            _key: &Self::Key,
            _window: &Self::Window,
        ) -> Result<u32, ReservationError> {
            Ok(0)
        }
    }

    async fn insert_expired_held_row(conn: &DatabaseConnection) -> Uuid {
        let id = Uuid::new_v4();
        let past = (Utc::now() - ChronoDuration::seconds(60)).naive_utc();
        let am = reservations::ActiveModel {
            id: ActiveValue::Set(id),
            resource_kind: ActiveValue::Set("test.sweep".to_string()),
            resource_key: ActiveValue::Set(serde_json::json!({"k": "v"})),
            window: ActiveValue::Set(None),
            quantity: ActiveValue::Set(1),
            status: ActiveValue::Set("held".to_string()),
            expires_at: ActiveValue::Set(past),
            held_at: ActiveValue::Set((Utc::now() - ChronoDuration::seconds(120)).naive_utc()),
            committed_at: ActiveValue::Set(None),
            released_at: ActiveValue::Set(None),
            release_reason: ActiveValue::Set(None),
            tenant_id: ActiveValue::Set(None),
        };
        am.insert(conn).await.expect("insert expired held row");
        id
    }

    /// D-47-8: 3 rows with expires_at < now → all transition to expired.
    #[tokio::test]
    async fn sweep_expires_rows() {
        let conn = fresh_db().await;
        let kernel = ReservationKernel::new(conn.clone(), TestResource);

        let mut ids = vec![];
        for _ in 0..3 {
            ids.push(insert_expired_held_row(&conn).await);
        }

        let report = kernel.run_sweep_once().await.expect("sweep ok");
        assert_eq!(report.expired_count, 3, "expected 3 rows expired");

        for id in ids {
            let row = reservations::Entity::find_by_id(id)
                .one(&conn)
                .await
                .expect("query")
                .expect("found");
            assert_eq!(row.status, "expired", "row {id} should be expired");
        }
    }

    /// D-47-9: no eligible rows → report.expired_count = 0.
    #[tokio::test]
    async fn sweep_noop() {
        let conn = fresh_db().await;
        let kernel = ReservationKernel::new(conn.clone(), TestResource);
        let report = kernel.run_sweep_once().await.expect("sweep ok");
        assert_eq!(report.expired_count, 0);
    }
}

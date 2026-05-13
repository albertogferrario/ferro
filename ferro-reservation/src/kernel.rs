//! `ReservationKernel<R>` — typed hold/commit/release/extend orchestrator.
//!
//! Composes [`ferro_orm::GuardedUpdate`] (atomic state transitions),
//! [`ferro_audit::AuditEntry`] (unconditional audit emission per D-28),
//! and [`ferro_events::dispatch`] (best-effort domain events per D-26).
//!
//! The kernel owns a `DatabaseConnection` for the sweeper path
//! (`run_sweep_once`, plan 154-06) and accepts a `&C: ConnectionTrait` on
//! per-call methods so consumers can run transitions inside their own
//! transactions.

use chrono::{Duration as ChronoDuration, Utc};
use ferro_audit::{AuditEntry, AuditTarget};
use ferro_orm::{GuardedError, GuardedUpdate, Value};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

use crate::context::ReservationContext;
use crate::entity::{self as reservations};
use crate::error::ReservationError;
use crate::event::{ReleaseReason, ReservationEvent};
use crate::handle::ReservationHandle;
use crate::resource::Resource;

/// Generic hold/commit/release/extend orchestrator over a consumer's
/// [`Resource`] implementation. Construct with [`ReservationKernel::new`].
///
/// The struct carries an owned `DatabaseConnection` for the sweeper path
/// (which has no caller-supplied connection); per-call methods accept an
/// explicit `&C: ConnectionTrait` so consumers can run them inside their
/// own transactions.
pub struct ReservationKernel<R: Resource> {
    pub(crate) db: DatabaseConnection,
    pub(crate) resource: R,
}

impl<R: Resource> ReservationKernel<R> {
    /// Construct a kernel from an owned `DatabaseConnection` and a
    /// consumer `Resource` impl. Cloning is cheap (`DatabaseConnection`
    /// is `Clone` by SeaORM's design).
    pub fn new(db: DatabaseConnection, resource: R) -> Self {
        Self { db, resource }
    }

    /// Hold `quantity` units of the resource for `ttl`. See D-10 for the
    /// seven-step sequence.
    ///
    /// Returns [`ReservationError::Insufficient`] if `held + quantity >
    /// capacity` (with `capacity` and `available` populated for telemetry).
    /// Returns [`ReservationError::Db`] / [`ReservationError::Json`] /
    /// [`ReservationError::Audit`] on the corresponding subsystem failure.
    pub async fn hold<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: R::Key,
        window: R::Window,
        quantity: u32,
        ttl: Duration,
        ctx: &ReservationContext,
    ) -> Result<ReservationHandle, ReservationError> {
        // Step 1: generate id
        let id = Uuid::new_v4();

        // Steps 2–3: capacity check (consumer-defined)
        let capacity = self.resource.capacity(conn, &key, &window).await?;
        let held = self.resource.held(conn, &key, &window).await?;
        let available = capacity.saturating_sub(held);

        // Step 4: enforce invariant
        if quantity > available {
            return Err(ReservationError::Insufficient {
                requested: quantity,
                available,
                capacity,
            });
        }

        // Step 5: INSERT reservations row
        let key_json = serde_json::to_value(&key)?;
        let window_json_raw = serde_json::to_value(&window)?;
        let window_json: Option<serde_json::Value> = if window_json_raw.is_null() {
            None
        } else {
            Some(window_json_raw)
        };

        let now = Utc::now();
        let expires_at = now
            + ChronoDuration::from_std(ttl).map_err(|e| {
                ReservationError::Db(sea_orm::DbErr::Custom(format!(
                    "reservation: ttl overflow: {e}"
                )))
            })?;

        let am = reservations::ActiveModel {
            id: ActiveValue::Set(id),
            resource_kind: ActiveValue::Set(R::KIND.to_string()),
            resource_key: ActiveValue::Set(key_json.clone()),
            window: ActiveValue::Set(window_json.clone()),
            quantity: ActiveValue::Set(quantity as i32),
            status: ActiveValue::Set("held".to_string()),
            expires_at: ActiveValue::Set(expires_at.naive_utc()),
            held_at: ActiveValue::Set(now.naive_utc()),
            committed_at: ActiveValue::Set(None),
            released_at: ActiveValue::Set(None),
            release_reason: ActiveValue::Set(None),
            tenant_id: ActiveValue::Set(ctx.tenant_id.clone()),
        };
        am.insert(conn).await.map_err(ReservationError::Db)?;

        // Step 6: AuditEntry::record("reservation.held").write(conn)
        let mut audit = AuditEntry::record("reservation.held")
            .actor(ctx.actor.clone())
            .target(AuditTarget::new("reservation", id.to_string()))
            .before(json!(null))
            .after(json!({
                "status": "held",
                "quantity": quantity,
                "resource_kind": R::KIND,
            }));
        if let Some(cid) = ctx.correlation_id {
            audit = audit.correlation(cid);
        }
        if let Some(tid) = ctx.tenant_id.as_deref() {
            audit = audit.tenant(tid);
        }
        if let Some(reason) = ctx.reason.as_deref() {
            audit = audit.reason(reason);
        }
        audit.write(conn).await.map_err(ReservationError::Audit)?;

        // Build handle
        let handle = ReservationHandle {
            id,
            resource_kind: R::KIND.to_string(),
            resource_key: key_json,
            window: window_json,
            quantity,
            held_at: now,
            expires_at,
            tenant_id: ctx.tenant_id.clone(),
        };

        // Step 7: dispatch event (best-effort, D-26)
        if let Err(e) = ferro_events::dispatch(ReservationEvent::Held {
            id,
            resource_kind: R::KIND.to_string(),
            resource_key: handle.resource_key.clone(),
            window: handle.window.clone(),
            quantity,
            expires_at,
        })
        .await
        {
            tracing::warn!(
                reservation_id = %id,
                error = %e,
                "event dispatch failed after reservation.held — state is committed"
            );
        }

        Ok(handle)
    }

    /// Commit a held reservation. Transitions `held → committed` via
    /// [`GuardedUpdate`]; the row's `committed_at` is set to `Utc::now()`.
    ///
    /// `handle` is taken by value to enforce use-once at the type level (D-11).
    pub async fn commit<C: ConnectionTrait>(
        &self,
        conn: &C,
        handle: ReservationHandle,
        ctx: &ReservationContext,
    ) -> Result<(), ReservationError> {
        let now = Utc::now();

        // GuardedUpdate held → committed (D-12 / D-46)
        GuardedUpdate::new(reservations::Entity)
            .filter(reservations::Column::Id.eq(handle.id))
            .filter(reservations::Column::Status.eq("held"))
            .set_value(
                reservations::Column::Status,
                Value::String(Some(Box::new("committed".to_string()))),
            )
            .set_value(
                reservations::Column::CommittedAt,
                Value::ChronoDateTime(Some(Box::new(now.naive_utc()))),
            )
            .exec_one(conn)
            .await
            .map_err(|e| match e {
                GuardedError::NoRowsAffected => ReservationError::ConflictingState {
                    id: handle.id,
                    expected: "held",
                },
                other => ReservationError::Guarded(other),
            })?;

        // AuditEntry (D-28)
        let mut audit = AuditEntry::record("reservation.committed")
            .actor(ctx.actor.clone())
            .target(AuditTarget::new("reservation", handle.id.to_string()))
            .before(json!({"status": "held", "quantity": handle.quantity}))
            .after(json!({"status": "committed"}));
        if let Some(cid) = ctx.correlation_id {
            audit = audit.correlation(cid);
        }
        if let Some(tid) = ctx.tenant_id.as_deref() {
            audit = audit.tenant(tid);
        }
        if let Some(reason) = ctx.reason.as_deref() {
            audit = audit.reason(reason);
        }
        audit.write(conn).await.map_err(ReservationError::Audit)?;

        // Event dispatch (D-26 best-effort)
        if let Err(e) = ferro_events::dispatch(ReservationEvent::Committed {
            id: handle.id,
            resource_kind: handle.resource_kind.clone(),
            resource_key: handle.resource_key.clone(),
        })
        .await
        {
            tracing::warn!(
                reservation_id = %handle.id,
                error = %e,
                "event dispatch failed after reservation.committed — state is committed"
            );
        }

        Ok(())
    }

    /// Release a held reservation with a typed [`ReleaseReason`].
    ///
    /// `handle` is taken by value to enforce use-once at the type level (D-11).
    pub async fn release<C: ConnectionTrait>(
        &self,
        conn: &C,
        handle: ReservationHandle,
        reason: ReleaseReason,
        ctx: &ReservationContext,
    ) -> Result<(), ReservationError> {
        let now = Utc::now();

        // Serialize reason tag for the release_reason column
        let reason_tag = match &reason {
            ReleaseReason::UserCancelled => "user_cancelled".to_string(),
            ReleaseReason::PaymentFailed => "payment_failed".to_string(),
            ReleaseReason::AdminOverride => "admin_override".to_string(),
            ReleaseReason::Other(_) => "other".to_string(),
        };

        // GuardedUpdate held → released (D-12 / D-46)
        GuardedUpdate::new(reservations::Entity)
            .filter(reservations::Column::Id.eq(handle.id))
            .filter(reservations::Column::Status.eq("held"))
            .set_value(
                reservations::Column::Status,
                Value::String(Some(Box::new("released".to_string()))),
            )
            .set_value(
                reservations::Column::ReleasedAt,
                Value::ChronoDateTime(Some(Box::new(now.naive_utc()))),
            )
            .set_value(
                reservations::Column::ReleaseReason,
                Value::String(Some(Box::new(reason_tag))),
            )
            .exec_one(conn)
            .await
            .map_err(|e| match e {
                GuardedError::NoRowsAffected => ReservationError::ConflictingState {
                    id: handle.id,
                    expected: "held",
                },
                other => ReservationError::Guarded(other),
            })?;

        let reason_json = serde_json::to_value(&reason)?;
        let mut audit = AuditEntry::record("reservation.released")
            .actor(ctx.actor.clone())
            .target(AuditTarget::new("reservation", handle.id.to_string()))
            .before(json!({"status": "held", "quantity": handle.quantity}))
            .after(json!({"status": "released", "release_reason": reason_json}));
        if let Some(cid) = ctx.correlation_id {
            audit = audit.correlation(cid);
        }
        if let Some(tid) = ctx.tenant_id.as_deref() {
            audit = audit.tenant(tid);
        }
        if let Some(reason_str) = ctx.reason.as_deref() {
            audit = audit.reason(reason_str);
        }
        audit.write(conn).await.map_err(ReservationError::Audit)?;

        // Event dispatch (D-26 best-effort)
        if let Err(e) = ferro_events::dispatch(ReservationEvent::Released {
            id: handle.id,
            resource_kind: handle.resource_kind.clone(),
            resource_key: handle.resource_key.clone(),
            reason,
        })
        .await
        {
            tracing::warn!(
                reservation_id = %handle.id,
                error = %e,
                "event dispatch failed after reservation.released — state is committed"
            );
        }

        Ok(())
    }

    /// Extend the TTL of a held reservation by `by`. Optimistic-lock
    /// semantics: the GuardedUpdate filters on `status='held'`,
    /// `expires_at = handle.expires_at` (exact match), and `expires_at >
    /// now()` (D-13 — cannot extend an already-expired-but-not-swept row).
    ///
    /// If a concurrent extend or sweeper has already moved the row, the
    /// call returns [`ReservationError::ConflictingState`]. The caller can
    /// retry with a fresh handle.
    ///
    /// No upper cap on cumulative extension (D-32) — consumer responsibility.
    ///
    /// `handle` is taken by value to enforce use-once at the type level (D-11).
    pub async fn extend<C: ConnectionTrait>(
        &self,
        conn: &C,
        handle: ReservationHandle,
        by: Duration,
        ctx: &ReservationContext,
    ) -> Result<(), ReservationError> {
        let now = Utc::now();
        let by_chrono = ChronoDuration::from_std(by).map_err(|e| {
            ReservationError::Db(sea_orm::DbErr::Custom(format!(
                "reservation: extend duration overflow: {e}"
            )))
        })?;
        let new_expires = handle.expires_at + by_chrono;

        // Optimistic-lock: filter on Status='held' AND ExpiresAt = handle.expires_at
        // AND ExpiresAt > now() (D-13 — cannot extend already-expired-but-not-swept row)
        GuardedUpdate::new(reservations::Entity)
            .filter(reservations::Column::Id.eq(handle.id))
            .filter(reservations::Column::Status.eq("held"))
            .filter(reservations::Column::ExpiresAt.eq(handle.expires_at.naive_utc()))
            .filter(reservations::Column::ExpiresAt.gt(now.naive_utc()))
            .set_value(
                reservations::Column::ExpiresAt,
                Value::ChronoDateTime(Some(Box::new(new_expires.naive_utc()))),
            )
            .exec_one(conn)
            .await
            .map_err(|e| match e {
                GuardedError::NoRowsAffected => ReservationError::ConflictingState {
                    id: handle.id,
                    expected: "held",
                },
                other => ReservationError::Guarded(other),
            })?;

        let mut audit = AuditEntry::record("reservation.extended")
            .actor(ctx.actor.clone())
            .target(AuditTarget::new("reservation", handle.id.to_string()))
            .before(json!({"expires_at": handle.expires_at.to_rfc3339()}))
            .after(json!({"expires_at": new_expires.to_rfc3339()}));
        if let Some(cid) = ctx.correlation_id {
            audit = audit.correlation(cid);
        }
        if let Some(tid) = ctx.tenant_id.as_deref() {
            audit = audit.tenant(tid);
        }
        if let Some(reason) = ctx.reason.as_deref() {
            audit = audit.reason(reason);
        }
        audit.write(conn).await.map_err(ReservationError::Audit)?;

        // No ReservationEvent variant for "Extended" in v0 (CONTEXT D-25
        // declares only four variants: Held, Committed, Released, Expired).
        // The audit log records the extension; consumers needing extension
        // events can subscribe to the audit log.

        Ok(())
    }
}

impl<R: Resource + Clone> Clone for ReservationKernel<R> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            resource: self.resource.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
    use sea_orm_migration::{MigrationTrait, MigratorTrait, SchemaManager};

    // ---- Test harness -------------------------------------------------------

    // `DeriveMigrationName` on both ferro_audit::migration::Migration and
    // crate::migration::Migration generates the name "migration" (file stem
    // of src/migration.rs). When both are registered in one Migrator the
    // seaql_migrations table gets a UNIQUE constraint violation on the version
    // column. We wrap our migration with a distinct name to avoid the collision.
    struct ReservationMigrationWrapper;

    impl sea_orm_migration::MigrationName for ReservationMigrationWrapper {
        fn name(&self) -> &str {
            "create_reservations_table"
        }
    }

    #[async_trait]
    impl MigrationTrait for ReservationMigrationWrapper {
        async fn up(&self, manager: &SchemaManager) -> Result<(), sea_orm::DbErr> {
            crate::migration::Migration.up(manager).await
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), sea_orm::DbErr> {
            crate::migration::Migration.down(manager).await
        }
    }

    struct TestMigrator;

    #[async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![
                Box::new(ferro_audit::CreateAuditLogTable),
                Box::new(ReservationMigrationWrapper),
            ]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    /// Resource impl that queries the reservations table for the live held count.
    #[derive(Clone)]
    struct TestResource {
        capacity_value: u32,
    }

    #[async_trait]
    impl Resource for TestResource {
        type Key = String;
        type Window = ();
        const KIND: &'static str = "test.resource";

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
            // Sum quantity for rows where status IN ('held','committed')
            // and resource_key = json(key) and resource_kind = Self::KIND
            let key_json = serde_json::to_value(key)?;
            let rows = reservations::Entity::find()
                .filter(reservations::Column::ResourceKind.eq(Self::KIND))
                .filter(reservations::Column::ResourceKey.eq(key_json))
                .filter(
                    reservations::Column::Status
                        .is_in(vec!["held".to_string(), "committed".to_string()]),
                )
                .all(conn)
                .await
                .map_err(ReservationError::Db)?;
            let total: i32 = rows.iter().map(|r| r.quantity).sum();
            Ok(total.max(0) as u32)
        }
    }

    async fn fresh_kernel() -> (DatabaseConnection, ReservationKernel<TestResource>) {
        let conn = fresh_db().await;
        let kernel = ReservationKernel::new(conn.clone(), TestResource { capacity_value: 10 });
        (conn, kernel)
    }

    fn ttl(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    // ---- Tests (D-47-1 through D-47-7) + audit smoke test ------------------

    #[tokio::test]
    async fn hold_happy_path() {
        // D-47-1: capacity 10, request 3 → handle returned, row persisted
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        let handle = kernel
            .hold(&conn, "k1".into(), (), 3, ttl(900), &ctx)
            .await
            .expect("hold should succeed");

        assert_eq!(handle.quantity, 3);
        assert_eq!(handle.resource_kind, "test.resource");

        let row = reservations::Entity::find_by_id(handle.id)
            .one(&conn)
            .await
            .expect("query")
            .expect("found");
        assert_eq!(row.status, "held");
        assert_eq!(row.quantity, 3);
        assert!(
            row.expires_at > row.held_at,
            "expires_at should be after held_at"
        );
    }

    #[tokio::test]
    async fn hold_insufficient() {
        // D-47-2: held + quantity > capacity → Insufficient
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        // Pre-fill 8 of 10
        for _ in 0..8 {
            kernel
                .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
                .await
                .expect("ok");
        }
        // Request of quantity=3 should fail (held=8, capacity=10, available=2)
        let err = kernel
            .hold(&conn, "k1".into(), (), 3, ttl(900), &ctx)
            .await
            .expect_err("should fail");
        match err {
            ReservationError::Insufficient {
                requested,
                available,
                capacity,
            } => {
                assert_eq!(requested, 3);
                assert_eq!(available, 2);
                assert_eq!(capacity, 10);
            }
            other => panic!("expected Insufficient, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn commit_happy_path() {
        // D-47-3: held → committed; committed_at set
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        let handle = kernel
            .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
            .await
            .expect("hold");
        let id = handle.id;

        kernel
            .commit(&conn, handle, &ctx)
            .await
            .expect("commit should succeed");

        let row = reservations::Entity::find_by_id(id)
            .one(&conn)
            .await
            .expect("query")
            .expect("found");
        assert_eq!(row.status, "committed");
        assert!(row.committed_at.is_some());
    }

    #[tokio::test]
    async fn commit_conflicting_state() {
        // D-47-4: commit on already-committed → ConflictingState
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        let handle = kernel
            .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
            .await
            .expect("hold");
        let id = handle.id;

        // commit consumes the handle by value; manually build a clone for the second call
        let handle_clone = ReservationHandle {
            id,
            resource_kind: handle.resource_kind.clone(),
            resource_key: handle.resource_key.clone(),
            window: handle.window.clone(),
            quantity: handle.quantity,
            held_at: handle.held_at,
            expires_at: handle.expires_at,
            tenant_id: handle.tenant_id.clone(),
        };

        kernel
            .commit(&conn, handle, &ctx)
            .await
            .expect("first commit ok");
        let err = kernel
            .commit(&conn, handle_clone, &ctx)
            .await
            .expect_err("second commit must fail");
        match err {
            ReservationError::ConflictingState { id: eid, expected } => {
                assert_eq!(eid, id);
                assert_eq!(expected, "held");
            }
            other => panic!("expected ConflictingState, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn release_all_reasons() {
        // D-47-5: release happy path with each ReleaseReason variant
        for reason in [
            ReleaseReason::UserCancelled,
            ReleaseReason::PaymentFailed,
            ReleaseReason::AdminOverride,
            ReleaseReason::Other("custom".into()),
        ] {
            let (conn, kernel) = fresh_kernel().await;
            let ctx = ReservationContext::system();
            let handle = kernel
                .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
                .await
                .expect("hold");
            let id = handle.id;
            kernel
                .release(&conn, handle, reason.clone(), &ctx)
                .await
                .expect("release ok");
            let row = reservations::Entity::find_by_id(id)
                .one(&conn)
                .await
                .expect("query")
                .expect("found");
            assert_eq!(row.status, "released");
            assert!(row.released_at.is_some());
            assert!(
                row.release_reason.is_some(),
                "release_reason should be set for {reason:?}"
            );
        }
    }

    #[tokio::test]
    async fn extend_happy_path() {
        // D-47-6: extend → expires_at increases by `by`
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        let handle = kernel
            .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
            .await
            .expect("hold");
        let old_expires = handle.expires_at;
        let id = handle.id;

        kernel
            .extend(&conn, handle, ttl(600), &ctx)
            .await
            .expect("extend ok");

        let row = reservations::Entity::find_by_id(id)
            .one(&conn)
            .await
            .expect("query")
            .expect("found");
        let new_expires = row.expires_at;
        // SQLite truncates timestamp precision; compare via a small epsilon
        let delta = new_expires - old_expires.naive_utc();
        assert!(
            delta.num_seconds() >= 600,
            "expected at least 600s extension, got {} s",
            delta.num_seconds()
        );
    }

    #[tokio::test]
    async fn extend_on_expired() {
        // D-47-7: extend on expired-but-not-swept row → ConflictingState
        // Force expires_at to be in the past via raw update, then call extend.
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::system();
        let handle = kernel
            .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
            .await
            .expect("hold");
        let id = handle.id;

        // Force expires_at into the past via a direct UPDATE using GuardedUpdate
        // (can't use ActiveModel ..Default::default() — DeriveEntityModel does not
        // derive Default; construct only the fields we need to change)
        let past = Utc::now() - ChronoDuration::seconds(60);
        GuardedUpdate::new(reservations::Entity)
            .filter(reservations::Column::Id.eq(id))
            .set_value(
                reservations::Column::ExpiresAt,
                Value::ChronoDateTime(Some(Box::new(past.naive_utc()))),
            )
            .exec_one(&conn)
            .await
            .expect("force expires_at into past");

        // Now the handle still has the original expires_at; extend uses
        // the handle's expires_at in the filter, which no longer matches
        // the row → NoRowsAffected → ConflictingState
        let err = kernel
            .extend(&conn, handle, ttl(900), &ctx)
            .await
            .expect_err("should fail");
        assert!(
            matches!(err, ReservationError::ConflictingState { .. }),
            "expected ConflictingState, got {err:?}"
        );
    }

    /// Audit / event emission smoke test — additional coverage for D-26 / D-28.
    /// The full cross-crate integration test lands in plan 154-06.
    #[tokio::test]
    async fn hold_emits_audit_entry() {
        let (conn, kernel) = fresh_kernel().await;
        let ctx = ReservationContext::user("u_42").with_correlation(Uuid::new_v4());
        let handle = kernel
            .hold(&conn, "k1".into(), (), 1, ttl(900), &ctx)
            .await
            .expect("hold");

        // Query audit_log for an entry on this reservation
        let history = ferro_audit::history_for_target(
            &ferro_audit::AuditTarget::new("reservation", handle.id.to_string()),
            &conn,
        )
        .await
        .expect("query audit");
        assert!(
            !history.is_empty(),
            "audit_log should contain at least one entry for reservation.held"
        );
        assert_eq!(history.last().unwrap().action, "reservation.held");
    }
}

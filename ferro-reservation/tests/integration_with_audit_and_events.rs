//! D-50 cross-crate integration test for ferro-reservation.
//!
//! Proves the three-crate composition end-to-end:
//! - hold + commit/release a reservation
//! - Assert two ReservationEvent instances dispatched (Held + Committed/Released)
//! - Assert two AuditEntry rows persisted with matching correlation_id
//! - Assert reconstruct_state on the audit history reproduces the final state
//! - Assert tenant_id is preserved across both audit entries and the reservation row
//!
//! This is the showcase test for the v11.11 milestone.
//!
//! **Test isolation:** `ferro_events::global_dispatcher()` is a process-level
//! singleton. Tests that register listeners must serialize to avoid cross-test
//! interference (listeners from test A firing during test B's dispatches).
//! A process-global `tokio::sync::Mutex` (`DISPATCH_LOCK`) ensures only one
//! listener-registration + dispatch sequence runs at a time.

use async_trait::async_trait;
use ferro_audit::AuditTarget;
use ferro_reservation::{
    ReleaseReason, ReservationContext, ReservationError, ReservationEvent, ReservationKernel,
    Resource,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::MigratorTrait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

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

#[derive(Clone)]
struct TestResource;

#[async_trait]
impl Resource for TestResource {
    type Key = String;
    type Window = ();
    const KIND: &'static str = "test.integration";

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

/// Process-global mutex serializing all tests that touch the global dispatcher.
/// Ensures listener registration + dispatch sequence is isolated per test.
static DISPATCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dispatch_lock() -> &'static Mutex<()> {
    DISPATCH_LOCK.get_or_init(|| Mutex::new(()))
}

/// D-50: hold + commit emits 2 events + 2 audit entries with matching correlation_id.
/// reconstruct_state reproduces {"status": "committed"}.
#[tokio::test]
async fn hold_commit_emits_two_events_and_two_audit_entries() {
    let _lock = dispatch_lock().lock().await;

    // Clear any stale listeners from prior test runs in this process
    ferro_events::global_dispatcher().forget::<ReservationEvent>();

    let held_count = Arc::new(AtomicU32::new(0));
    let committed_count = Arc::new(AtomicU32::new(0));
    let released_count = Arc::new(AtomicU32::new(0));

    let hc = held_count.clone();
    let cc = committed_count.clone();
    let rc = released_count.clone();

    ferro_events::global_dispatcher().on::<ReservationEvent, _, _>(move |ev: ReservationEvent| {
        let hc = hc.clone();
        let cc = cc.clone();
        let rc = rc.clone();
        async move {
            match ev {
                ReservationEvent::Held { .. } => {
                    hc.fetch_add(1, Ordering::SeqCst);
                }
                ReservationEvent::Committed { .. } => {
                    cc.fetch_add(1, Ordering::SeqCst);
                }
                ReservationEvent::Released { .. } => {
                    rc.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
            Ok(())
        }
    });

    let conn = fresh_db().await;
    let kernel = ReservationKernel::new(conn.clone(), TestResource);

    let correlation = Uuid::new_v4();
    let ctx = ReservationContext::user("u_42").with_correlation(correlation);

    let handle = kernel
        .hold(&conn, "key1".into(), (), 3, Duration::from_secs(900), &ctx)
        .await
        .expect("hold");
    let id = handle.id;

    kernel.commit(&conn, handle, &ctx).await.expect("commit");

    // Assertion (a): two events dispatched
    assert_eq!(held_count.load(Ordering::SeqCst), 1, "Held event count");
    assert_eq!(
        committed_count.load(Ordering::SeqCst),
        1,
        "Committed event count"
    );
    assert_eq!(
        released_count.load(Ordering::SeqCst),
        0,
        "Released event count (none expected)"
    );

    // Assertion (b): two audit entries with matching correlation_id
    let target = AuditTarget::new("reservation", id.to_string());
    let history = ferro_audit::history_for_target(&target, &conn)
        .await
        .expect("audit query");
    assert_eq!(history.len(), 2, "expected exactly 2 audit entries");
    assert_eq!(history[0].action, "reservation.held");
    assert_eq!(history[1].action, "reservation.committed");
    assert_eq!(
        history[0].correlation_id,
        Some(correlation),
        "first entry correlation_id"
    );
    assert_eq!(
        history[1].correlation_id,
        Some(correlation),
        "second entry correlation_id"
    );

    // Assertion (c): reconstruct_state shows status="committed"
    let final_state =
        ferro_audit::reconstruct_state(&history).expect("reconstruct_state should produce a value");
    let obj = final_state
        .as_object()
        .expect("reconstructed state should be a JSON object");
    assert_eq!(
        obj.get("status").and_then(|v| v.as_str()),
        Some("committed"),
        "reconstructed final status should be 'committed', got: {final_state:?}"
    );

    // Cleanup: remove listeners so the next test starts clean
    ferro_events::global_dispatcher().forget::<ReservationEvent>();
}

/// D-50 variant: hold + release emits 2 events + 2 audit entries.
/// reconstruct_state reproduces {"status": "released", ...}.
#[tokio::test]
async fn hold_release_emits_two_events_and_two_audit_entries() {
    let _lock = dispatch_lock().lock().await;
    ferro_events::global_dispatcher().forget::<ReservationEvent>();

    let held_count = Arc::new(AtomicU32::new(0));
    let released_count = Arc::new(AtomicU32::new(0));

    let hc = held_count.clone();
    let rc = released_count.clone();

    ferro_events::global_dispatcher().on::<ReservationEvent, _, _>(move |ev: ReservationEvent| {
        let hc = hc.clone();
        let rc = rc.clone();
        async move {
            match ev {
                ReservationEvent::Held { .. } => {
                    hc.fetch_add(1, Ordering::SeqCst);
                }
                ReservationEvent::Released { .. } => {
                    rc.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
            Ok(())
        }
    });

    let conn = fresh_db().await;
    let kernel = ReservationKernel::new(conn.clone(), TestResource);
    let correlation = Uuid::new_v4();
    let ctx = ReservationContext::user("u_42").with_correlation(correlation);

    let handle = kernel
        .hold(&conn, "key2".into(), (), 1, Duration::from_secs(900), &ctx)
        .await
        .expect("hold");
    let id = handle.id;

    kernel
        .release(&conn, handle, ReleaseReason::UserCancelled, &ctx)
        .await
        .expect("release");

    assert_eq!(held_count.load(Ordering::SeqCst), 1, "Held event count");
    assert_eq!(
        released_count.load(Ordering::SeqCst),
        1,
        "Released event count"
    );

    let target = AuditTarget::new("reservation", id.to_string());
    let history = ferro_audit::history_for_target(&target, &conn)
        .await
        .expect("audit query");
    assert_eq!(history.len(), 2, "expected exactly 2 audit entries");
    assert_eq!(history[1].action, "reservation.released");

    // Reconstructed state should include status="released"
    let final_state = ferro_audit::reconstruct_state(&history).expect("reconstruct_state");
    let obj = final_state.as_object().expect("object");
    assert_eq!(
        obj.get("status").and_then(|v| v.as_str()),
        Some("released"),
        "reconstructed final status should be 'released', got: {final_state:?}"
    );

    ferro_events::global_dispatcher().forget::<ReservationEvent>();
}

/// D-50 variant: tenant_id propagates through reservation row AND both audit entries.
#[tokio::test]
async fn tenant_id_is_preserved_across_audit_entries() {
    let _lock = dispatch_lock().lock().await;
    ferro_events::global_dispatcher().forget::<ReservationEvent>();

    let conn = fresh_db().await;
    let kernel = ReservationKernel::new(conn.clone(), TestResource);
    let ctx = ReservationContext::user("u_42").with_tenant("tenant_a");

    let handle = kernel
        .hold(&conn, "key3".into(), (), 1, Duration::from_secs(900), &ctx)
        .await
        .expect("hold");
    let id = handle.id;

    kernel.commit(&conn, handle, &ctx).await.expect("commit");

    let target = AuditTarget::new("reservation", id.to_string());
    let history = ferro_audit::history_for_target(&target, &conn)
        .await
        .expect("audit query");
    assert_eq!(history.len(), 2, "expected exactly 2 audit entries");

    // Each audit entry should have tenant_id = "tenant_a"
    for (i, entry) in history.iter().enumerate() {
        assert_eq!(
            entry.tenant_id.as_deref(),
            Some("tenant_a"),
            "audit entry {i} should have tenant_id = 'tenant_a', got {:?}",
            entry.tenant_id
        );
    }

    // The reservation row also has tenant_id set
    use ferro_reservation::ReservationEntity;
    let row = ReservationEntity::find_by_id(id)
        .one(&conn)
        .await
        .expect("query")
        .expect("found");
    assert_eq!(
        row.tenant_id.as_deref(),
        Some("tenant_a"),
        "reservation row should have tenant_id = 'tenant_a'"
    );

    ferro_events::global_dispatcher().forget::<ReservationEvent>();
}

//! D-47 milestone-completing showcase test.
//!
//! v11.11 = ferro-orm (Phase 152) + ferro-audit (Phase 153) +
//! ferro-reservation (Phase 154) + ferro-projection (Phase 155).
//!
//! This test composes the live-read-model half:
//! - ferro-reservation emits `ReservationEvent` on every state transition
//!   (hold / commit / release / expire)
//! - ferro-events routes those events through the global dispatcher
//! - ferro-projection's `ProjectionRuntime` registers a listener; each
//!   event folds into a per-resource_kind counter state row
//! - ferro-broadcast fans the delta to subscribers on
//!   `projection.reservations.counters.{resource_kind}` channels
//!
//! A maintainer reading this test should understand the four-primitive
//! composition without leaving the file.

mod common;

use common::BroadcastCapture;
use ferro_events::Event;
use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
use ferro_reservation::{ReleaseReason, ReservationEvent};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use uuid::Uuid;

static DISPATCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dispatch_lock() -> &'static Mutex<()> {
    DISPATCH_LOCK.get_or_init(|| Mutex::new(()))
}

// Per-resource_kind counters — held / committed / released.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct ReservationCounters {
    held: u32,
    committed: u32,
    released: u32,
}

#[derive(Clone, Serialize, Debug)]
struct CountersDelta {
    held: u32,
    committed: u32,
    released: u32,
}

struct ReservationCountProjection;

impl Projection for ReservationCountProjection {
    type Event = ReservationEvent;
    type State = ReservationCounters;
    type Delta = CountersDelta;
    const NAME: &'static str = "reservations.counters";

    fn key(&self, event: &Self::Event) -> ProjectionKey {
        // Group counters by resource_kind. Each variant of
        // ReservationEvent carries `resource_kind`.
        let rk = match event {
            ReservationEvent::Held { resource_kind, .. } => resource_kind,
            ReservationEvent::Committed { resource_kind, .. } => resource_kind,
            ReservationEvent::Released { resource_kind, .. } => resource_kind,
            ReservationEvent::Expired { resource_kind, .. } => resource_kind,
        };
        ProjectionKey::new(rk.clone())
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        match event {
            ReservationEvent::Held { .. } => state.held += 1,
            ReservationEvent::Committed { .. } => state.committed += 1,
            ReservationEvent::Released { .. } => state.released += 1,
            ReservationEvent::Expired { .. } => state.released += 1,
        }
        CountersDelta {
            held: state.held,
            committed: state.committed,
            released: state.released,
        }
    }
}

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_reservation::CreateReservationsTable),
            Box::new(ferro_projection::CreateProjectionSnapshotsTable),
        ]
    }
}

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}

fn fake_uuid() -> Uuid {
    Uuid::new_v4()
}

fn fake_key() -> JsonValue {
    serde_json::json!({ "id": 1 })
}

#[tokio::test]
async fn reservation_events_fold_into_per_resource_kind_counters() {
    let _lock = dispatch_lock().lock().await;

    // Clean any stale listeners
    ferro_events::global_dispatcher().forget::<ReservationEvent>();

    let db = fresh_db().await;
    let channel = "projection.reservations.counters.inventory.unit";
    let mut capture = BroadcastCapture::subscribe(channel).await;

    // Compose the four primitives in one Arc + register call:
    // ferro-reservation events → ferro-events dispatcher →
    // ferro-projection runtime → ferro-broadcast delta fanout
    let runtime = Arc::new(ProjectionRuntime::new(
        db,
        capture.broadcaster.clone(),
        ReservationCountProjection,
    ));
    runtime.clone().register();

    let resource_kind = "inventory.unit".to_string();

    // 3 holds — simulate what ferro-reservation::Kernel::hold would emit
    for _ in 0..3 {
        ReservationEvent::Held {
            id: fake_uuid(),
            resource_kind: resource_kind.clone(),
            resource_key: fake_key(),
            window: None,
            quantity: 1,
            expires_at: chrono::Utc::now(),
        }
        .dispatch()
        .await
        .expect("dispatch Held");
    }

    // 1 commit — what Kernel::commit would emit
    ReservationEvent::Committed {
        id: fake_uuid(),
        resource_kind: resource_kind.clone(),
        resource_key: fake_key(),
    }
    .dispatch()
    .await
    .expect("dispatch Committed");

    // 1 release — what Kernel::release would emit
    ReservationEvent::Released {
        id: fake_uuid(),
        resource_kind: resource_kind.clone(),
        resource_key: fake_key(),
        reason: ReleaseReason::UserCancelled,
    }
    .dispatch()
    .await
    .expect("dispatch Released");

    // Yield to listener tasks
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    // Assert: projection state for "inventory.unit" key shows the
    // composed counter rollup driven by ReservationEvent variants
    let state = runtime
        .read(&ProjectionKey::new(resource_kind.clone()))
        .await
        .expect("read")
        .expect("state");
    assert_eq!(state.held, 3, "expected 3 holds");
    assert_eq!(state.committed, 1, "expected 1 commit");
    assert_eq!(state.released, 1, "expected 1 release");

    // Assert: broadcast frames captured (≥ 5 — one per event, channel
    // matches the resource_kind key)
    let frames = capture.drain();
    assert!(
        frames.len() >= 5,
        "expected >= 5 broadcast frames, got {}",
        frames.len()
    );
    for frame in &frames {
        assert_eq!(frame.channel, channel);
        assert_eq!(frame.event, "delta");
    }

    // Cleanup
    ferro_events::global_dispatcher().forget::<ReservationEvent>();
}

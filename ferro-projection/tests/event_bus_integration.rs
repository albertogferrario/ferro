//! D-46 integration test: auto-register listener path.
//!
//! Proves the killer-feature wiring works end-to-end: dispatch an
//! event → ProjectionListener.handle → ProjectionRuntime.apply_event →
//! snapshot upsert → broadcast frame captured by BroadcastCapture.
//!
//! Uses `DISPATCH_LOCK` + `global_dispatcher().forget::<E>()` to
//! isolate the global dispatcher from other integration tests
//! (mirrors ferro-reservation/tests/integration_with_audit_and_events.rs:78-93).

mod common;

use common::BroadcastCapture;
use ferro_events::Event;
use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

static DISPATCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dispatch_lock() -> &'static Mutex<()> {
    DISPATCH_LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Clone, Serialize, Deserialize)]
struct CountEvent {
    delta: i32,
}

impl Event for CountEvent {
    fn name(&self) -> &'static str {
        "CountEvent"
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct CountState {
    total: i64,
}

#[derive(Clone, Serialize, Debug)]
struct CountDelta {
    new_total: i64,
}

struct CountingProjection;

impl Projection for CountingProjection {
    type Event = CountEvent;
    type State = CountState;
    type Delta = CountDelta;
    const NAME: &'static str = "counting.test";

    fn key(&self, _event: &Self::Event) -> ProjectionKey {
        ProjectionKey::new("test-key")
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        state.total += event.delta as i64;
        CountDelta {
            new_total: state.total,
        }
    }
}

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(
            ferro_projection::CreateProjectionSnapshotsTable,
        )]
    }
}

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}

#[tokio::test]
async fn register_path_dispatches_through_runtime_and_broadcasts_5_frames() {
    let _lock = dispatch_lock().lock().await;

    // Clear any stale listeners from prior runs in this process
    ferro_events::global_dispatcher().forget::<CountEvent>();

    let db = fresh_db().await;
    let channel = "projection.counting.test.test-key";
    let mut capture = BroadcastCapture::subscribe(channel).await;

    let runtime = Arc::new(ProjectionRuntime::new(
        db,
        capture.broadcaster.clone(),
        CountingProjection,
    ));
    runtime.clone().register();

    // Dispatch 5 events through the global dispatcher
    for i in 1..=5 {
        CountEvent { delta: i }
            .dispatch()
            .await
            .expect("dispatch");
    }

    // Yield to let async listener tasks complete
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Drain broadcast frames
    let frames = capture.drain();
    assert_eq!(
        frames.len(),
        5,
        "expected 5 broadcast frames, got {}",
        frames.len()
    );
    for frame in &frames {
        assert_eq!(frame.channel, channel);
        assert_eq!(frame.event, "delta");
    }

    // Assert final state via read
    let state = runtime
        .read(&ProjectionKey::new("test-key"))
        .await
        .expect("read")
        .expect("state");
    assert_eq!(state.total, 1 + 2 + 3 + 4 + 5);

    // Cleanup for the next test in this process
    ferro_events::global_dispatcher().forget::<CountEvent>();
}

//! D-48 concurrency integration test.
//!
//! Proves per-key serialization + cross-key parallelism (D-32, D-31):
//! 20 tokio tasks across 5 distinct keys (4 tasks per key), each
//! calling `apply_event(CounterEvent { delta: 1 })`. Final state per
//! key must be exactly 4 (no lost increments). Total snapshot rows
//! must be 5 (one per key).
//!
//! Does NOT touch the global dispatcher (bypasses register, calls
//! apply_event directly). Tests the shard-lock-drop-before-await
//! pattern from plan 05 under contention.

use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime, ProjectionSnapshotEntity};
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
struct CounterEvent {
    key_idx: u8,
    delta: i32,
}

impl ferro_events::Event for CounterEvent {
    fn name(&self) -> &'static str {
        "CounterEvent"
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct CounterState {
    total: i64,
}

#[derive(Clone, Serialize)]
struct CounterDelta {
    new_total: i64,
}

struct KeyedCounter;

impl Projection for KeyedCounter {
    type Event = CounterEvent;
    type State = CounterState;
    type Delta = CounterDelta;
    const NAME: &'static str = "test.keyed_concurrent";

    fn key(&self, event: &Self::Event) -> ProjectionKey {
        ProjectionKey::new(format!("key-{}", event.key_idx))
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        state.total += event.delta as i64;
        CounterDelta {
            new_total: state.total,
        }
    }
}

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(ferro_projection::CreateProjectionSnapshotsTable)]
    }
}

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_apply_20_tasks_5_keys_serializes_per_key() {
    let conn = fresh_db().await;
    // Clone connection before moving into runtime — SeaORM DatabaseConnection
    // is Arc-backed so clones are cheap and share the same underlying pool.
    let conn_for_assert = conn.clone();
    let broadcaster = Arc::new(ferro_broadcast::Broadcaster::new());
    let runtime = Arc::new(ProjectionRuntime::new(conn, broadcaster, KeyedCounter));

    let mut handles = Vec::with_capacity(20);
    for key_idx in 0..5u8 {
        for _ in 0..4 {
            let rt = runtime.clone();
            let h =
                tokio::spawn(
                    async move { rt.apply_event(&CounterEvent { key_idx, delta: 1 }).await },
                );
            handles.push(h);
        }
    }
    for h in handles {
        h.await.expect("join").expect("apply");
    }

    // Per-key total must be exactly 4 (no lost increments under contention)
    for key_idx in 0..5u8 {
        let state = runtime
            .read(&ProjectionKey::new(format!("key-{key_idx}")))
            .await
            .expect("read")
            .expect("state");
        assert_eq!(
            state.total, 4,
            "key-{}: expected total=4, got {}",
            key_idx, state.total
        );
    }

    // 5 distinct snapshot rows (one per key)
    let all = ProjectionSnapshotEntity::find()
        .all(&conn_for_assert)
        .await
        .expect("query all");
    assert_eq!(all.len(), 5, "expected 5 snapshot rows, got {}", all.len());
}

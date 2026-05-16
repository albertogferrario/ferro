//! D-49 property-based tests.
//!
//! Three properties verify replay correctness:
//! - **P1 (apply determinism):** sequential apply_event yields the
//!   same final state as a pure-fold reference.
//! - **P2 (replay equivalence):** N sequential applies = one rebuild
//!   call with the same events.
//! - **P3 (cross-key independence):** per-key final states are
//!   independent of the interleaving order between keys.

use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
use proptest::prelude::*;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Builder as RuntimeBuilder;

#[derive(Clone, Serialize, Deserialize)]
struct Ev {
    key_idx: u8,
    delta: i32,
}

impl ferro_events::Event for Ev {
    fn name(&self) -> &'static str {
        "Ev"
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct St {
    total: i64,
}

#[derive(Clone, Serialize)]
struct De {
    new_total: i64,
}

struct P;

impl Projection for P {
    type Event = Ev;
    type State = St;
    type Delta = De;
    const NAME: &'static str = "proptest.p";

    fn key(&self, event: &Self::Event) -> ProjectionKey {
        ProjectionKey::new(format!("k-{}", event.key_idx))
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        state.total += event.delta as i64;
        De {
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

async fn fresh_runtime() -> ProjectionRuntime<P> {
    let conn: DatabaseConnection = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    let broadcaster = Arc::new(ferro_broadcast::Broadcaster::new());
    ProjectionRuntime::new(conn, broadcaster, P)
}

fn pure_fold_single_key(deltas: &[i32]) -> St {
    let mut s = St::default();
    for d in deltas {
        s.total += *d as i64;
    }
    s
}

fn build_runtime() -> tokio::runtime::Runtime {
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime")
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn proptest_apply_determinism(deltas in proptest::collection::vec(-50i32..50i32, 0..50)) {
        let expected = pure_fold_single_key(&deltas);
        let rt_tokio = build_runtime();
        rt_tokio.block_on(async {
            let runtime = fresh_runtime().await;
            for d in &deltas {
                runtime.apply_event(&Ev { key_idx: 0, delta: *d }).await.expect("apply");
            }
            let key = ProjectionKey::new("k-0");
            let observed = runtime.read(&key).await.expect("read");
            match observed {
                None => prop_assert_eq!(expected, St::default()),
                Some(s) => prop_assert_eq!(s, expected),
            }
            Ok::<(), TestCaseError>(())
        })?;
    }

    #[test]
    fn proptest_replay_equivalence(deltas in proptest::collection::vec(-50i32..50i32, 1..50)) {
        let rt_tokio = build_runtime();
        rt_tokio.block_on(async {
            // Path A: sequential applies
            let rt_a = fresh_runtime().await;
            for d in &deltas {
                rt_a.apply_event(&Ev { key_idx: 0, delta: *d }).await.expect("apply a");
            }
            let key = ProjectionKey::new("k-0");
            let state_a = rt_a.read(&key).await.expect("read a").expect("state a");

            // Path B: single rebuild
            let rt_b = fresh_runtime().await;
            let events: Vec<Ev> = deltas.iter().map(|d| Ev { key_idx: 0, delta: *d }).collect();
            let state_b = rt_b.rebuild(&key, events).await.expect("rebuild b");

            prop_assert_eq!(state_a, state_b);
            Ok::<(), TestCaseError>(())
        })?;
    }

    #[test]
    fn proptest_cross_key_independence(
        events_a in proptest::collection::vec((0u8..5u8, -20i32..20i32), 0..20),
        events_b in proptest::collection::vec((0u8..5u8, -20i32..20i32), 0..20),
    ) {
        let rt_tokio = build_runtime();
        rt_tokio.block_on(async {
            // Interleaving 1: a then b
            let rt_1 = fresh_runtime().await;
            for (k, d) in &events_a {
                rt_1.apply_event(&Ev { key_idx: *k, delta: *d }).await.expect("a1");
            }
            for (k, d) in &events_b {
                rt_1.apply_event(&Ev { key_idx: *k, delta: *d }).await.expect("b1");
            }

            // Interleaving 2: b then a (same total event set)
            let rt_2 = fresh_runtime().await;
            for (k, d) in &events_b {
                rt_2.apply_event(&Ev { key_idx: *k, delta: *d }).await.expect("b2");
            }
            for (k, d) in &events_a {
                rt_2.apply_event(&Ev { key_idx: *k, delta: *d }).await.expect("a2");
            }

            // Per-key final states must be identical
            for k in 0u8..5 {
                let key = ProjectionKey::new(format!("k-{k}"));
                let s1 = rt_1.read(&key).await.expect("read 1");
                let s2 = rt_2.read(&key).await.expect("read 2");
                prop_assert_eq!(s1, s2);
            }
            Ok::<(), TestCaseError>(())
        })?;
    }
}

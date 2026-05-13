//! `ProjectionRuntime<P>` — orchestrator owning the DB connection, the
//! broadcaster handle, the projection impl, and the per-key Mutex
//! registry (D-13).
//!
//! ## Apply algorithm (D-19, 7 steps)
//!
//! 1. Compute the key via `P::key(event)`.
//! 2. Acquire the per-key Mutex from `DashMap<String, Arc<Mutex<()>>>`,
//!    dropping the DashMap RefMut BEFORE the Mutex `.await`.
//! 3. Load the snapshot via composite-PK `Entity::find_by_id`.
//!    Default to `P::State::default()` if absent.
//! 4. Apply the event: `let delta = self.projection.apply(&mut state, event);`.
//! 5. Persist via SeaORM `OnConflict::columns([ProjectionName, Key])`
//!    upsert. `version += 1`, `updated_at = Utc::now()`.
//! 6. Broadcast on `projection.{NAME}.{key}` channel. Broadcast failure
//!    does NOT roll back state — `tracing::warn!` + surface
//!    `ProjectionError::Broadcast`.
//! 7. Release the Mutex (RAII).
//!
//! Cross-key concurrency: DashMap shards allow concurrent access; each
//! key's `Arc<Mutex<()>>` is independent. The proof is `concurrent_apply`
//! in plan 06's integration tests.

use std::sync::Arc;

use chrono::Utc;
use dashmap::DashMap;
use sea_orm::{
    sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait,
};

use crate::entity::{ActiveModel, Column, Entity};
use crate::error::ProjectionError;
use crate::key::ProjectionKey;
use crate::projection::Projection;

/// Live read-model runtime owning the DB connection, the broadcaster
/// handle, the projection impl, and the per-key Mutex registry (D-13).
pub struct ProjectionRuntime<P: Projection> {
    pub(crate) db: DatabaseConnection,
    pub(crate) broadcaster: Arc<ferro_broadcast::Broadcaster>,
    pub(crate) projection: P,
    pub(crate) locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl<P: Projection> ProjectionRuntime<P> {
    /// Construct a new runtime. Consumers typically wrap in `Arc` for
    /// sharing across tokio tasks (and `register` requires `Arc<Self>`
    /// to wire into the global dispatcher).
    pub fn new(
        db: DatabaseConnection,
        broadcaster: Arc<ferro_broadcast::Broadcaster>,
        projection: P,
    ) -> Self {
        Self {
            db,
            broadcaster,
            projection,
            locks: DashMap::new(),
        }
    }

    /// Read the persisted snapshot for `key`. Returns `Ok(None)` if no
    /// row exists. Does NOT take the per-key Mutex (D-33) — readers
    /// see either the pre- or post-upsert state, no torn reads.
    pub async fn read(
        &self,
        key: &ProjectionKey,
    ) -> Result<Option<P::State>, ProjectionError> {
        let row = Entity::find_by_id((P::NAME.to_string(), key.0.clone()))
            .one(&self.db)
            .await?;
        match row {
            None => Ok(None),
            Some(model) => {
                let state: P::State = serde_json::from_value(model.state)?;
                Ok(Some(state))
            }
        }
    }

    /// Read with a hard error if the snapshot is absent (D-30). Wraps
    /// `read`; consumers wanting `Result<State, _>` use this instead
    /// of `Result<Option<State>, _>`.
    pub async fn read_required(
        &self,
        key: &ProjectionKey,
    ) -> Result<P::State, ProjectionError> {
        self.read(key).await?.ok_or_else(|| ProjectionError::StateNotFound {
            name: P::NAME,
            key: key.0.clone(),
        })
    }

    /// Apply a single event. Implements the 7-step D-19 sequence
    /// inside a per-key `tokio::sync::Mutex`. Cross-key applies run
    /// in parallel; same-key applies serialize.
    pub async fn apply_event(
        &self,
        event: &P::Event,
    ) -> Result<(), ProjectionError> {
        // Step 1: compute key
        let key = self.projection.key(event);

        // Step 2: acquire per-key Mutex with shard-lock-drop-before-await
        // (mirrors ferro-broadcast/src/broadcaster.rs:271 pattern —
        // RefMut MUST drop before .await or cross-key concurrency
        // collapses through the DashMap shard).
        let lock_arc: Arc<tokio::sync::Mutex<()>> = {
            self.locks
                .entry(key.0.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        }; // <-- RefMut drops here, shard unlocked
        let _guard = lock_arc.lock().await;

        // Step 3: load the snapshot
        let existing = Entity::find_by_id((P::NAME.to_string(), key.0.clone()))
            .one(&self.db)
            .await?;

        let (mut state, new_version) = match existing {
            Some(model) => {
                let s: P::State = serde_json::from_value(model.state)?;
                (s, model.version + 1)
            }
            None => (P::State::default(), 1_i64),
        };

        // Step 4: apply (sync — must not block; inside the per-key Mutex)
        let delta = self.projection.apply(&mut state, event);

        // Step 5: upsert via OnConflict on the composite PK
        let state_json = serde_json::to_value(&state)?;
        let now = Utc::now().naive_utc();
        let am = ActiveModel {
            projection_name: ActiveValue::Set(P::NAME.to_string()),
            key: ActiveValue::Set(key.0.clone()),
            state: ActiveValue::Set(state_json),
            version: ActiveValue::Set(new_version),
            updated_at: ActiveValue::Set(now),
        };

        Entity::insert(am)
            .on_conflict(
                OnConflict::columns([Column::ProjectionName, Column::Key])
                    .update_columns([Column::State, Column::Version, Column::UpdatedAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        // Step 6: broadcast — failure does NOT roll back state (D-21)
        let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
        let event_name = self.projection.broadcast_event_name();
        let send_result = ferro_broadcast::Broadcast::new(self.broadcaster.clone())
            .channel(channel_name.clone())
            .event(event_name)
            .data(delta)
            .send()
            .await;

        if let Err(e) = send_result {
            tracing::warn!(
                error = %e,
                channel = %channel_name,
                "projection broadcast failed; snapshot persisted"
            );
            return Err(ProjectionError::from(e));
        }

        // Step 7: Mutex released on drop of `_guard` after this return
        Ok(())
    }

    /// Register a `ProjectionListener<P>` into the global event
    /// dispatcher (D-15). Killer-feature one-line wiring; every
    /// `P::Event::dispatch().await` now flows through this projection.
    ///
    /// **Not idempotent on `Arc` identity (D-36).** Calling `register`
    /// twice on the same `Arc<Self>` registers TWO listeners; both
    /// fire on each dispatch (same semantic as Laravel's
    /// `Event::listen`). Register once at app startup.
    pub fn register(self: Arc<Self>) {
        let listener = crate::listener::ProjectionListener {
            runtime: self.clone(),
        };
        ferro_events::global_dispatcher().listen::<P::Event, _>(listener);
    }

    /// Discard the persisted snapshot for `key`, fold the supplied event
    /// sequence through `P::State::default()` via `P::apply`, persist
    /// the final state, broadcast ONE `"rebuild"` frame, and return
    /// the rebuilt state (D-17).
    ///
    /// Acquires the same per-key Mutex as `apply_event`, so rebuild
    /// serializes against in-flight applies.
    ///
    /// **Not transactional** (D-44). DELETE + folded INSERT under the
    /// per-key Mutex; if the process crashes mid-rebuild, the snapshot
    /// is gone but a subsequent `apply_event` re-initializes from
    /// `Default`.
    ///
    /// Empty iterator wipes the snapshot row (D-43): returns
    /// `P::State::default()` with no insert and no broadcast.
    ///
    /// The broadcast event name is `"rebuild"` (overrides
    /// `P::broadcast_event_name`); payload is the final state. Clients
    /// reset their local state on receipt of a `"rebuild"` frame
    /// (D-41).
    pub async fn rebuild<I>(
        &self,
        key: &ProjectionKey,
        events: I,
    ) -> Result<P::State, ProjectionError>
    where
        I: IntoIterator<Item = P::Event>,
    {
        // Acquire per-key Mutex (same shard-drop pattern as apply_event)
        let lock_arc: Arc<tokio::sync::Mutex<()>> = {
            self.locks
                .entry(key.0.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        }; // <-- RefMut drops here
        let _guard = lock_arc.lock().await;

        // Step 1: DELETE the existing snapshot row (D-42)
        Entity::delete_by_id((P::NAME.to_string(), key.0.clone()))
            .exec(&self.db)
            .await?;

        // Step 2: fold events through Default
        let mut state = P::State::default();
        let mut count: i64 = 0;
        for event in events {
            let _delta = self.projection.apply(&mut state, &event);
            count += 1;
        }

        // Step 3: empty iterator → wipe semantic (D-43)
        if count == 0 {
            return Ok(state);
        }

        // Step 4: persist final state with version = count (D-42)
        let state_json = serde_json::to_value(&state)?;
        let now = Utc::now().naive_utc();
        let am = ActiveModel {
            projection_name: ActiveValue::Set(P::NAME.to_string()),
            key: ActiveValue::Set(key.0.clone()),
            state: ActiveValue::Set(state_json),
            version: ActiveValue::Set(count),
            updated_at: ActiveValue::Set(now),
        };
        Entity::insert(am).exec(&self.db).await?;

        // Step 5: broadcast ONE "rebuild" frame with full final state (D-41)
        let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
        let send_result = ferro_broadcast::Broadcast::new(self.broadcaster.clone())
            .channel(channel_name.clone())
            .event("rebuild")
            .data(state.clone())
            .send()
            .await;

        if let Err(e) = send_result {
            tracing::warn!(
                error = %e,
                channel = %channel_name,
                "projection rebuild broadcast failed; snapshot persisted"
            );
            return Err(ProjectionError::from(e));
        }

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;
    use serde::{Deserialize, Serialize};

    // Minimal Projection impl used across all runtime tests.
    #[derive(Clone, Serialize, Deserialize)]
    struct CounterEvent {
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

    #[derive(Clone, Serialize, Debug, PartialEq)]
    struct CounterDelta {
        new_total: i64,
    }

    struct CounterProjection;

    impl Projection for CounterProjection {
        type Event = CounterEvent;
        type State = CounterState;
        type Delta = CounterDelta;
        const NAME: &'static str = "test.counter";

        fn key(&self, _e: &Self::Event) -> ProjectionKey {
            ProjectionKey::new("default-key")
        }

        fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
            state.total += event.delta as i64;
            CounterDelta {
                new_total: state.total,
            }
        }
    }

    // Variant projection: uses event.delta-as-key for cross-key tests.
    struct KeyedCounterProjection;

    impl Projection for KeyedCounterProjection {
        type Event = CounterEvent;
        type State = CounterState;
        type Delta = CounterDelta;
        const NAME: &'static str = "test.keyed_counter";

        fn key(&self, event: &Self::Event) -> ProjectionKey {
            ProjectionKey::new(format!("k-{}", event.delta))
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
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_runtime<P: Projection>(
        projection: P,
    ) -> ProjectionRuntime<P> {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        let broadcaster = Arc::new(ferro_broadcast::Broadcaster::new());
        ProjectionRuntime::new(conn, broadcaster, projection)
    }

    // D-45 #4: runtime construction
    #[tokio::test]
    async fn new_returns_owned_runtime_arc_is_send_sync() {
        let rt = fresh_runtime(CounterProjection).await;
        let arc: Arc<ProjectionRuntime<CounterProjection>> = Arc::new(rt);
        fn assert_send_sync<T: Send + Sync>(_: &T) {}
        assert_send_sync(&arc);
    }

    // D-45 #5: apply happy path — empty table → one event
    #[tokio::test]
    async fn apply_event_initial_writes_version_1() {
        let rt = fresh_runtime(CounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 5 })
            .await
            .expect("apply");

        let key = ProjectionKey::new("default-key");
        let state = rt.read(&key).await.expect("read").expect("state");
        assert_eq!(state.total, 5);

        // Verify version=1 via direct entity query
        let row = Entity::find_by_id((
            CounterProjection::NAME.to_string(),
            "default-key".to_string(),
        ))
        .one(&rt.db)
        .await
        .expect("query")
        .expect("row");
        assert_eq!(row.version, 1);
    }

    // D-45 #6: second apply on the same key folds onto loaded state
    #[tokio::test]
    async fn apply_event_second_call_folds_and_bumps_version() {
        let rt = fresh_runtime(CounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 5 })
            .await
            .expect("first apply");
        rt.apply_event(&CounterEvent { delta: 3 })
            .await
            .expect("second apply");

        let key = ProjectionKey::new("default-key");
        let state = rt.read(&key).await.expect("read").expect("state");
        assert_eq!(state.total, 8);

        let row = Entity::find_by_id((
            CounterProjection::NAME.to_string(),
            "default-key".to_string(),
        ))
        .one(&rt.db)
        .await
        .expect("query")
        .expect("row");
        assert_eq!(row.version, 2);
    }

    // D-45 #7: new key initializes from Default
    #[tokio::test]
    async fn apply_event_new_key_initializes_from_default() {
        let rt = fresh_runtime(KeyedCounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 7 })
            .await
            .expect("apply key 7");
        rt.apply_event(&CounterEvent { delta: 9 })
            .await
            .expect("apply key 9");

        let s7 = rt
            .read(&ProjectionKey::new("k-7"))
            .await
            .expect("read 7")
            .expect("state 7");
        let s9 = rt
            .read(&ProjectionKey::new("k-9"))
            .await
            .expect("read 9")
            .expect("state 9");
        assert_eq!(s7.total, 7);
        assert_eq!(s9.total, 9);
    }

    // D-45 #8: read returns None for absent key, Some after apply
    #[tokio::test]
    async fn read_returns_none_for_absent_key() {
        let rt = fresh_runtime(CounterProjection).await;
        let key = ProjectionKey::new("absent");
        let r = rt.read(&key).await.expect("read");
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn read_returns_some_after_apply() {
        let rt = fresh_runtime(CounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 1 })
            .await
            .expect("apply");
        let r = rt
            .read(&ProjectionKey::new("default-key"))
            .await
            .expect("read");
        assert!(r.is_some());
    }

    // read_required returns StateNotFound for absent key
    #[tokio::test]
    async fn read_required_returns_state_not_found_for_absent() {
        let rt = fresh_runtime(CounterProjection).await;
        let key = ProjectionKey::new("absent");
        let err = rt.read_required(&key).await.expect_err("should err");
        match err {
            ProjectionError::StateNotFound { name, key: k } => {
                assert_eq!(name, CounterProjection::NAME);
                assert_eq!(k, "absent");
            }
            other => panic!("expected StateNotFound, got {other:?}"),
        }
    }

    // version increments across multiple applies on same key
    #[tokio::test]
    async fn version_increments_per_apply_same_key() {
        let rt = fresh_runtime(CounterProjection).await;
        for _ in 0..5 {
            rt.apply_event(&CounterEvent { delta: 1 })
                .await
                .expect("apply");
        }
        let row = Entity::find_by_id((
            CounterProjection::NAME.to_string(),
            "default-key".to_string(),
        ))
        .one(&rt.db)
        .await
        .expect("query")
        .expect("row");
        assert_eq!(row.version, 5);
    }

    // updated_at advances across applies
    #[tokio::test]
    async fn updated_at_advances_per_apply() {
        let rt = fresh_runtime(CounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 1 })
            .await
            .expect("first");
        let row1 = Entity::find_by_id((
            CounterProjection::NAME.to_string(),
            "default-key".to_string(),
        ))
        .one(&rt.db)
        .await
        .expect("query")
        .expect("row");
        let t1 = row1.updated_at;

        // Sleep briefly to ensure timestamp difference (chrono ns
        // precision; SQLite stores to microsecond on the timestamp
        // column).
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        rt.apply_event(&CounterEvent { delta: 1 })
            .await
            .expect("second");
        let row2 = Entity::find_by_id((
            CounterProjection::NAME.to_string(),
            "default-key".to_string(),
        ))
        .one(&rt.db)
        .await
        .expect("query")
        .expect("row");
        assert!(row2.updated_at > t1, "updated_at must advance");
    }

    // Cross-key parallelism smoke test (light version of plan 06's
    // full concurrent_apply suite — D-32). Confirms two different
    // keys can be applied in sequence without sharing a Mutex.
    #[tokio::test]
    async fn cross_key_apply_does_not_share_lock() {
        let rt = fresh_runtime(KeyedCounterProjection).await;
        rt.apply_event(&CounterEvent { delta: 1 })
            .await
            .expect("k-1");
        rt.apply_event(&CounterEvent { delta: 2 })
            .await
            .expect("k-2");
        rt.apply_event(&CounterEvent { delta: 3 })
            .await
            .expect("k-3");

        // 3 distinct keys → 3 distinct lock entries
        assert_eq!(rt.locks.len(), 3);
    }

    // D-45 #9: rebuild equivalence with sequential applies
    #[tokio::test]
    async fn rebuild_three_events_equals_three_sequential_applies() {
        // Path A: 3 sequential apply_event calls
        let rt_a = fresh_runtime(CounterProjection).await;
        for d in [3, 5, 7] {
            rt_a.apply_event(&CounterEvent { delta: d })
                .await
                .expect("apply");
        }
        let state_a = rt_a
            .read(&ProjectionKey::new("default-key"))
            .await
            .expect("read a")
            .expect("state a");

        // Path B: one rebuild call
        let rt_b = fresh_runtime(CounterProjection).await;
        let events: Vec<CounterEvent> = vec![
            CounterEvent { delta: 3 },
            CounterEvent { delta: 5 },
            CounterEvent { delta: 7 },
        ];
        let state_b = rt_b
            .rebuild(&ProjectionKey::new("default-key"), events)
            .await
            .expect("rebuild");

        assert_eq!(state_a, state_b);
        assert_eq!(state_a.total, 15);
    }

    // D-45 #10: rebuild with empty iterator wipes row and returns Default
    #[tokio::test]
    async fn rebuild_empty_deletes_row_and_returns_default() {
        let rt = fresh_runtime(CounterProjection).await;
        // Seed a value
        rt.apply_event(&CounterEvent { delta: 7 })
            .await
            .expect("seed");
        let pre = rt
            .read(&ProjectionKey::new("default-key"))
            .await
            .expect("read pre")
            .expect("state pre");
        assert_eq!(pre.total, 7);

        // Rebuild empty
        let after = rt
            .rebuild(&ProjectionKey::new("default-key"), Vec::<CounterEvent>::new())
            .await
            .expect("rebuild empty");
        assert_eq!(after.total, 0);

        // Confirm the row is gone
        let post = rt
            .read(&ProjectionKey::new("default-key"))
            .await
            .expect("read post");
        assert!(post.is_none(), "row should be wiped");
    }
}

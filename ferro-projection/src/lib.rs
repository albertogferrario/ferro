//! # ferro-projection
//!
//! Live read-model runtime: subscribe to domain events, persist per-key
//! snapshots, broadcast deltas.
//!
//! **Not to be confused with [`ferro-projections`] (plural).** That crate
//! is the Service Projection abstraction (`ServiceDef → IntentGraph →
//! JsonUiRenderer`). This crate (`ferro-projection`, singular) is the
//! live read-model runtime that subscribes to domain events, maintains a
//! materialized state, and broadcasts deltas. The two abstractions are
//! orthogonal — most apps will use both for different reasons.
//!
//! ferro-projection is the *live-read-model* primitive. [`ferro-events`]
//! says *something happened*. [`ferro-broadcast`] says *something is
//! visible to clients*. ferro-projection composes the two: events fold
//! into per-key state, deltas land on `projection.{name}.{key}` channels.
//!
//! ## Per-key serialization
//!
//! ```text
//!  Event::dispatch() ─┐
//!                     │
//!       ProjectionListener<P> ──┐
//!                               │
//!                               ▼
//!  ┌── per-key Mutex (DashMap<String, Arc<Mutex<()>>>) ──┐
//!  │   1. load snapshot from projection_snapshots        │
//!  │   2. apply(&mut state, &event) → Delta              │
//!  │   3. upsert snapshot (state, version+1)             │
//!  │   4. broadcast on projection.{name}.{key}            │
//!  └─────────────────────────────────────────────────────┘
//!                                │
//!                                ▼
//!                  WebSocket clients receive the delta
//! ```
//!
//! ## Schema and migration
//!
//! ferro-projection ships a SeaORM migration as
//! [`CreateProjectionSnapshotsTable`]. Register it in your consumer-side
//! `Migrator`:
//!
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_projection::CreateProjectionSnapshotsTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```
//!
//! ## Operational footguns
//!
//! 1. **Broadcast failure does NOT roll back state.** If
//!    `Broadcast::send` returns `Err`, the snapshot row is already
//!    persisted; the runtime logs at `tracing::warn!` and surfaces
//!    `ProjectionError::Broadcast`. Subscribers reconcile by re-reading
//!    the snapshot.
//! 2. **Single-instance assumption.** v0 assumes a single application
//!    instance owns each projection's listener. Multi-instance
//!    deployments must elect a single projection-runner node or accept
//!    last-writer-wins behavior on concurrent applies to the same key
//!    from different nodes.
//! 3. **`register` is not idempotent on `Arc` identity.** Calling
//!    `Arc<ProjectionRuntime<P>>::register()` twice registers two
//!    listeners — both fire on each dispatch (same semantic as
//!    Laravel's `Event::listen`). Register once at app startup.
//!
//! [`ferro-projections`]: https://docs.rs/ferro-projections
//! [`ferro-events`]: https://docs.rs/ferro-events
//! [`ferro-broadcast`]: https://docs.rs/ferro-broadcast

mod entity;
mod error;
mod key;
mod listener;
mod migration;
mod projection;
mod runtime;

pub use error::ProjectionError;
pub use key::ProjectionKey;
pub use migration::Migration as CreateProjectionSnapshotsTable;
pub use projection::Projection;
pub use runtime::ProjectionRuntime;

// SeaORM entity re-exports for consumers needing native SeaORM query access.
pub use entity::{
    ActiveModel as ProjectionSnapshotActiveModel, Entity as ProjectionSnapshotEntity,
    Model as ProjectionSnapshotModel,
};

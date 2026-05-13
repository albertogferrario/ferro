# Phase 155: ferro-projection — Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 16 (10 new in `ferro-projection/`, 6 modified at workspace level)
**Analogs found:** 15 / 16 (1 file — the entity with composite PK — has no in-workspace precedent; uses cited SeaORM 1.x doc pattern)

The new crate is a **structural twin of `ferro-reservation/`** (Phase 154, Wave 1b). Every file in `ferro-projection/` maps 1:1 onto a `ferro-reservation/` file with surgical substitutions (deps, schema, trait shape). Where ferro-reservation lacks a needed pattern (composite PK, broadcast-capture test helper, `Broadcaster::new` runtime construction), the analog jumps to `ferro-audit/`, `framework/src/session/driver/database.rs`, or `ferro-broadcast/`.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-projection/Cargo.toml` (NEW) | config (crate manifest) | — | `ferro-reservation/Cargo.toml` | exact (Wave 1b sibling) |
| `ferro-projection/README.md` (NEW) | documentation (crate root readme) | — | `ferro-reservation/README.md` | exact |
| `ferro-projection/src/lib.rs` (NEW) | module index + public re-exports | — | `ferro-reservation/src/lib.rs` | exact |
| `ferro-projection/src/error.rs` (NEW) | error model | request-response (consumer error surface) | `ferro-reservation/src/error.rs` | exact |
| `ferro-projection/src/key.rs` (NEW) | newtype value object | — | `ferro-audit/src/actor.rs` (stringly-typed newtype precedent) + `ferro-reservation/src/handle.rs` (Serde-derived consumer-facing value type) | partial (no exact analog; constructed from the two) |
| `ferro-projection/src/projection.rs` (NEW) | trait definition (consumer-implemented) | — | `ferro-reservation/src/resource.rs` (consumer-implemented `Resource` trait with associated types + `const KIND`) | role-match (different bound shape) |
| `ferro-projection/src/runtime.rs` (NEW) | orchestrator (DB + broadcaster + locks) | event-driven (in) + CRUD upsert (DB) + pub-sub (out) | `ferro-reservation/src/kernel.rs` (state-transition orchestrator) | role-match (different state machine; same kernel shape) |
| `ferro-projection/src/listener.rs` (NEW) | event subscriber (dispatcher adapter) | event-driven | `ferro-events/src/dispatcher.rs` + RESEARCH.md §Technical Concerns #8 (no in-workspace `Listener` impl yet — Phase 155 is the first user of the struct-listener path) | partial |
| `ferro-projection/src/entity.rs` (NEW) | SeaORM entity (composite PK, JSON state) | CRUD | `ferro-audit/src/entity.rs` (JSON column + Model shape) + `framework/src/session/driver/database.rs:199` (`#[sea_orm(primary_key, auto_increment = false)]` precedent) | role-match (composite PK has no workspace precedent — SeaORM 1.x doc cited) |
| `ferro-projection/src/migration.rs` (NEW) | SeaORM migration | one-shot DDL | `ferro-audit/src/migration.rs` (composite-index migration with `DeriveMigrationName` + sqlite_master smoke test) + `ferro-reservation/src/migration.rs` (manual `MigrationName` impl) | exact (composite PK column declaration is the only surgical difference) |
| `ferro-projection/tests/common/mod.rs` (NEW) | test helper (`BroadcastCapture`) | event capture | `ferro-broadcast/src/broadcaster.rs` (`Broadcaster::new` + `add_client` + `subscribe` public API) | new pattern, derived from public API |
| `ferro-projection/tests/event_bus_integration.rs` (NEW) | integration test | event-driven | `ferro-reservation/tests/integration_with_audit_and_events.rs` (process-global `DISPATCH_LOCK` for `global_dispatcher()` isolation + `forget::<E>()` cleanup) | exact |
| `ferro-projection/tests/projection_over_reservation_events.rs` (NEW) | cross-crate showcase | event-driven | `ferro-reservation/tests/integration_with_audit_and_events.rs` | exact |
| `ferro-projection/tests/concurrent_apply.rs` (NEW) | concurrency test | event-driven (parallel tasks) | `ferro-reservation/tests/concurrent_hold.rs` (referenced in research; same `tokio::spawn` × N + JoinSet shape) | exact |
| `ferro-projection/tests/property_invariants.rs` (NEW) | property test | event-driven | `ferro-reservation/tests/property_invariants.rs` (proptest! macro + `RuntimeBuilder::new_current_thread`) | exact |
| `Cargo.toml` (root) (MODIFIED) | workspace manifest | — | Phase 154's same edit (add `"ferro-reservation"` to `[workspace.members]` + version bump) | exact |
| `.github/workflows/publish.yml` (MODIFIED) | CI publish workflow | — | Phase 154's same edit (`WAVE1B_CRATES` line) | exact |
| `CLAUDE.md` (MODIFIED) | workspace doc — `Workspace Structure` table | — | Existing `ferro-reservation` row at line 60 | exact |
| `README.md` (workspace root) (MODIFIED) | workspace doc — feature bullets | — | Existing `ferro-reservation` bullet at line 73 | exact |
| `docs/src/SUMMARY.md` (MODIFIED) | docs nav | — | Existing `Service Projections` entry at line 47 (sibling) | role-match |
| `docs/src/features/live-read-models.md` (NEW) | user-facing feature doc | — | `docs/src/database/reservations.md` (anti-pattern → replacement → state diagram → operational caveats narrative arc) | exact |
| `CHANGELOG.md` (MODIFIED) | release notes | — | `## ferro-reservation` section at line 6 | exact |

---

## Pattern Assignments

### `ferro-projection/Cargo.toml` (config)

**Analog:** `ferro-reservation/Cargo.toml` (full file, 31 lines).

**Metadata block to copy verbatim, replacing only the marked fields:**
```toml
[package]
name = "ferro-projection"   # (was "ferro-reservation")
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas (not the same as ferro-projections plural)"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["projection", "read-model", "events", "broadcast", "ferro"]
categories = ["database", "asynchronous", "web-programming"]
readme = "README.md"
homepage = "https://ferro-rs.dev"
```

**Dependencies pattern** (mirror `ferro-reservation/Cargo.toml:13-25`, swap runtime deps + add `dashmap` + `tokio`):
```toml
[dependencies]
sea-orm = "1.0"
sea-orm-migration = "1.0"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
async-trait = "0.1"
dashmap = "6"                                          # NEW vs ferro-reservation
tokio = { version = "1", features = ["sync", "rt"] }   # NEW vs ferro-reservation
ferro-events    = { path = "../ferro-events",    version = "0.2" }
ferro-broadcast = { path = "../ferro-broadcast", version = "0.2" }   # replaces ferro-orm + ferro-audit

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
ferro-reservation = { path = "../ferro-reservation", version = "0.2" }   # NEW — D-47 cross-crate showcase
```

**Notable surgical differences from ferro-reservation:**
- `categories` adds `"web-programming"` (projections fan to WebSockets via broadcast).
- `keywords` re-shuffled with `"read-model"` to mitigate ferro-projections/ferro-projection naming clash on crates.io search.
- `description` carries the disambiguation phrase verbatim (locked in CONTEXT.md D-02).
- Runtime ferro-* deps swap from `ferro-orm + ferro-events + ferro-audit` to `ferro-events + ferro-broadcast`.

---

### `ferro-projection/README.md` (documentation)

**Analog:** `ferro-reservation/README.md` (full file, 12 lines).

**Exact shape to mirror:**
```markdown
# ferro-projection

Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas.

[description paragraph: see CONTEXT.md Specifics §README/CLAUDE row text — single paragraph
naming the disambiguation from `ferro-projections` plural, plus the "what it does" sentence
and a pointer to crate docs for the full distinction]

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-projection

License: MIT
```

**Single deviation from ferro-reservation/README:** explicit "(not the same as `ferro-projections` plural)" sentence per D-02. The README is also surfaced on crates.io.

---

### `ferro-projection/src/lib.rs` (module index + rustdoc)

**Analog:** `ferro-reservation/src/lib.rs` (full file, 149 lines).

**Module-level rustdoc tone — opening pattern** (mirror `ferro-reservation/src/lib.rs:1-12`):
```rust
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
```

**State / pipeline diagram pattern** (mirror `ferro-reservation/src/lib.rs:20-33`):
```rust
//! ## Per-key serialization
//!
//! ```text
//!  Event::dispatch() ─┐
//!                     │
//!       ProjectionListener<P> ──┐
//!                               │
//!                               ▼
//!  ┌── per-key Mutex (DashMap<String, Arc<Mutex<()>>>) ──┐
//!  │   1. load snapshot                                  │
//!  │   2. apply(&mut state, &event) → Delta              │
//!  │   3. upsert snapshot (state, version+1)             │
//!  │   4. broadcast on projection.{name}.{key}            │
//!  └─────────────────────────────────────────────────────┘
//! ```
```

**Composition framing — mirror `ferro-reservation/src/lib.rs:13-18`:**
```rust
//! ferro-projection is the *live-read-model* primitive. [`ferro-events`]
//! says *something happened*. [`ferro-broadcast`] says *something is
//! visible to clients*. ferro-projection composes the two: events fold
//! into per-key state, deltas land on `projection.{name}.{key}` channels.
```

**Migration-as-public-re-export framing** (mirror `ferro-reservation/src/lib.rs:88-104`):
```rust
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
```

**Operational footguns section** — analogous to `ferro-reservation/src/lib.rs:72-87` (audit/event operational semantics), but enumerating ferro-projection's three footguns (D-21 broadcast failure, D-34 single-instance, D-36 register-twice-fires-twice).

**Module + re-export block — mirror `ferro-reservation/src/lib.rs:122-148`:**
```rust
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
    ActiveModel as ProjectionSnapshotActiveModel,
    Entity as ProjectionSnapshotEntity,
    Model as ProjectionSnapshotModel,
};
```

`ProjectionListener<P>` is NOT re-exported (implementation detail; Claude's Discretion in CONTEXT.md).

---

### `ferro-projection/src/error.rs` (error model)

**Analog:** `ferro-reservation/src/error.rs` (full file, 132 lines).

**File-level rustdoc + display-prefix convention** (mirror `ferro-reservation/src/error.rs:1-5`):
```rust
//! `ProjectionError` — the single error type for the ferro-projection crate.
//!
//! Every variant's `Display` impl prefixes `"projection: …"` so production
//! log greps stay surgical (matches `"guarded: …"`, `"audit: …"`,
//! `"reservation: …"`, `"config: …"`).
```

**Enum shape — locked by CONTEXT.md D-28, structurally identical to ferro-reservation's enum:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("projection: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("projection: json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Stringly-typed because `ferro_broadcast::Error` doesn't compose
    /// cleanly through `thiserror::From` — same pragmatic call Phase 149
    /// made for `ferro_notifications::Error::Broadcast(String)`.
    #[error("projection: broadcast error: {0}")]
    Broadcast(String),

    #[error("projection: events error: {0}")]
    Events(String),

    #[error("projection: state not found for {name}/{key}")]
    StateNotFound { name: &'static str, key: String },
}
```

**Hand-rolled `From` for stringly-typed variants** — the source `Error` enums don't compose through `#[from]`; pattern (no in-workspace `From<ferro_broadcast::Error>` precedent — CONTEXT.md D-29 references the Phase 149 precedent in `ferro-notifications/src/error.rs`):
```rust
impl From<ferro_broadcast::Error> for ProjectionError {
    fn from(e: ferro_broadcast::Error) -> Self {
        Self::Broadcast(e.to_string())
    }
}

impl From<ferro_events::Error> for ProjectionError {
    fn from(e: ferro_events::Error) -> Self {
        Self::Events(e.to_string())
    }
}
```

**Test pattern — display-prefix assertion** (mirror `ferro-reservation/src/error.rs:57-131`):
```rust
#[test]
fn db_from_sea_orm_dberr() {
    let db_err = sea_orm::DbErr::Custom("test".into());
    let e: ProjectionError = ProjectionError::from(db_err);
    assert!(matches!(e, ProjectionError::Db(_)));
    assert!(e.to_string().starts_with("projection: db error: "));
}
```
Replicate the same shape for every variant (one `#[test]` per variant asserting the `"projection: …"` prefix). Covers D-45 #2.

---

### `ferro-projection/src/key.rs` (newtype value object)

**Analog:** No exact in-workspace match. The stringly-typed-newtype convention is documented but no current ferro-* crate ships a public newtype identical in shape. Constructed from two partial analogs:

**Partial analog 1: `ferro-audit/src/actor.rs`** for `AuditActor::kind() -> &'static str` and the broader stringly-typed convention (CONTEXT.md D-11 explicitly cites this).

**Partial analog 2: `ferro-reservation/src/handle.rs`** for a Serde-derived consumer-facing value type co-living with the trait.

**Shape locked by CONTEXT.md D-11 — newtype around `String`:**
```rust
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionKey(String);

impl ProjectionKey {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ProjectionKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectionKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
```

**Test pattern** — three unit tests per D-45 #1 (roundtrip, Display, serde roundtrip).

---

### `ferro-projection/src/projection.rs` (trait)

**Analog:** `ferro-reservation/src/resource.rs` — the closest in-workspace precedent for "consumer-implemented async trait with associated types + `const KIND: &'static str`".

**Trait shape — locked by CONTEXT.md D-06..D-10:** the bound shape differs from `Resource` (no `<C: ConnectionTrait>` parameter; `apply` is sync) but the const-name + associated-types pattern is direct.

**Pattern to mirror — `const NAME` + associated types** (from `Resource`):
```rust
// ferro-reservation/src/resource.rs (paraphrased):
// type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
// type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;
// const KIND: &'static str;
```

**`Projection` trait — to write:**
```rust
use serde::de::DeserializeOwned;
use serde::Serialize;

#[async_trait::async_trait]
pub trait Projection: Send + Sync + 'static {
    type Event: ferro_events::Event + Serialize + DeserializeOwned;
    type State: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;
    type Delta: Serialize + Clone + Send + Sync + 'static;

    /// Dotted-namespace identifier, e.g. `"inventory.dashboard"`.
    const NAME: &'static str;

    /// Derive the per-row key from the event.
    fn key(&self, event: &Self::Event) -> crate::ProjectionKey;

    /// Fold the event into the running state and return the delta.
    /// Pure function — MUST NOT perform IO or block.
    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;

    /// Reserved for v0.x event-log-backed snapshot mode. Default 100.
    fn snapshot_interval(&self) -> u32 { 100 }

    /// Event name for the broadcast frame. Default `"delta"`.
    fn broadcast_event_name(&self) -> &'static str { "delta" }
}
```

**Disambiguation paragraph appears here too** (CONTEXT.md D-51 final-pass): module rustdoc opens with the "Not to be confused with `ferro-projections` (plural)" sentence.

**Doctest at module top — mirror `ferro-reservation/src/lib.rs:38-70` (consumer-side ignore'd example)** with the canonical `WarehouseProjection` from CONTEXT.md §Specifics lines 438-495.

---

### `ferro-projection/src/runtime.rs` (orchestrator)

**Analog:** `ferro-reservation/src/kernel.rs` — same role (state-transition orchestrator owning DB + extension handles + serialization primitives).

**Struct shape pattern** — `kernel::ReservationKernel<R>` owns `db: DatabaseConnection` + `resource: R`; `ProjectionRuntime<P>` adds two fields:
```rust
pub struct ProjectionRuntime<P: Projection> {
    db: sea_orm::DatabaseConnection,
    broadcaster: std::sync::Arc<ferro_broadcast::Broadcaster>,
    projection: P,
    locks: dashmap::DashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>,
}
```

**DashMap shard-lock-drop-before-await pattern** — directly mirror `ferro-broadcast/src/broadcaster.rs:271` (`drop(channel); // Release DashMap guard before await`). The runtime's apply path:
```rust
// Clone the per-key Arc<Mutex> INSIDE a narrow scope so the DashMap RefMut
// drops (releases the shard lock) before the per-key Mutex acquisition's
// .await. Failing to drop the RefMut first holds the shard lock across
// the entire apply path — cross-key concurrency collapses through the
// shard. Mirrors the pattern at ferro-broadcast/src/broadcaster.rs:271.
let lock_arc: Arc<tokio::sync::Mutex<()>> = {
    self.locks
        .entry(key.0.clone())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
};  // RefMut drops here
let _guard = lock_arc.lock().await;
```

**Apply algorithm — 7-step sequence locked by D-19.** No in-workspace precedent for the full pattern (the closest, `ReservationKernel::hold`, uses `GuardedUpdate` not SeaORM upsert). The `OnConflict` pattern itself has no workspace precedent and is the load-bearing technical concern (RESEARCH.md §Technical Concerns #2):
```rust
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

let am = crate::entity::ActiveModel {
    projection_name: ActiveValue::Set(P::NAME.to_string()),
    key: ActiveValue::Set(key.as_str().to_string()),
    state: ActiveValue::Set(serde_json::to_value(&new_state)?),
    version: ActiveValue::Set(new_version),
    updated_at: ActiveValue::Set(Utc::now().naive_utc()),
};

crate::entity::Entity::insert(am)
    .on_conflict(
        OnConflict::columns([
            crate::entity::Column::ProjectionName,
            crate::entity::Column::Key,
        ])
        .update_columns([
            crate::entity::Column::State,
            crate::entity::Column::Version,
            crate::entity::Column::UpdatedAt,
        ])
        .to_owned(),
    )
    .exec(&self.db)
    .await?;
```

**Broadcast call** — mirror `ferro-broadcast/src/broadcast.rs:25-46` (`Broadcast::new(broadcaster).channel(name).event(event_name).data(payload).send().await`):
```rust
use ferro_broadcast::Broadcast;

let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
Broadcast::new(self.broadcaster.clone())
    .channel(channel_name)
    .event(self.projection.broadcast_event_name())
    .data(delta)
    .send()
    .await
    .map_err(ProjectionError::from)?;
```

**Failure semantic — broadcast error does NOT roll back** — mirror `ferro-reservation/src/lib.rs:72-87` rustdoc framing: log at `tracing::warn!` and surface the error, but the snapshot row is already persisted.

**Test pattern: in-memory SQLite harness inline** — mirror `ferro-reservation/src/entity.rs:71-84`:
```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(crate::migration::Migration)]
    }
}

async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = sea_orm::Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```
Identical helper appears in `ferro-audit/src/entry.rs:204-208` and `ferro-audit/src/query.rs:101-105`. Each crate re-derives this inline (no shared `framework` dep, per CONTEXT.md D-50).

---

### `ferro-projection/src/listener.rs` (event subscriber)

**Analog:** `ferro-events/src/dispatcher.rs` (the listener-registration API at line 73-79) + `ferro-events/src/traits.rs` (the `Listener<E>` trait at line 131-151). **There is no existing in-workspace `Listener<E>` impl** as a `struct ... impl Listener<E> for ...` block — the existing tests use the `EventDispatcher::on(closure)` API. Phase 155 is the first user of the struct-listener path (RESEARCH.md §Technical Concerns #8 + §Established patterns explicitly flags this).

**API to invoke** (from `ferro-events/src/dispatcher.rs:73-79`):
```rust
pub fn listen<E, L>(&self, listener: L)
where
    E: Event,
    L: Listener<E>,
```

**`Listener<E>` trait shape to satisfy** (from `ferro-events/src/traits.rs:131-151`):
```rust
#[async_trait]
pub trait Listener<E: Event>: Send + Sync + 'static {
    async fn handle(&self, event: &E) -> Result<(), Error>;
    fn name(&self) -> &'static str { std::any::type_name::<Self>() }
    fn should_stop_propagation(&self) -> bool { false }
}
```

**Error constructor pattern** — `ferro_events::Error::listener_failed(listener, message)` at `ferro-events/src/error.rs:45-50`:
```rust
pub fn listener_failed(listener: impl Into<String>, message: impl Into<String>) -> Self {
    Self::ListenerFailed {
        listener: listener.into(),
        message: message.into(),
    }
}
```

**Listener implementation — locked by CONTEXT.md D-15:**
```rust
use std::sync::Arc;

pub(crate) struct ProjectionListener<P: crate::Projection> {
    pub(crate) runtime: Arc<crate::ProjectionRuntime<P>>,
}

#[async_trait::async_trait]
impl<P: crate::Projection> ferro_events::Listener<P::Event> for ProjectionListener<P> {
    async fn handle(&self, event: &P::Event) -> Result<(), ferro_events::Error> {
        self.runtime.apply_event(event).await.map_err(|e| {
            ferro_events::Error::listener_failed(
                std::any::type_name::<Self>(),
                e.to_string(),
            )
        })
    }
}
```

The `ProjectionListener<P>` struct is `pub(crate)` only — implementation detail of `register` (CONTEXT.md Claude's Discretion: NO public exposure).

**`register` method on `Arc<ProjectionRuntime<P>>`** — the API surface that wires the listener into the global dispatcher:
```rust
impl<P: Projection> ProjectionRuntime<P> {
    pub fn register(self: Arc<Self>) {
        let listener = crate::listener::ProjectionListener {
            runtime: self.clone(),
        };
        ferro_events::global_dispatcher().listen::<P::Event, _>(listener);
    }
}
```

---

### `ferro-projection/src/entity.rs` (SeaORM entity)

**Analog 1: `ferro-audit/src/entity.rs`** — full file, 67 lines — provides the JSON column + DeriveEntityModel + ActiveModelBehavior pattern.

**Analog 2: `ferro-reservation/src/entity.rs`** — full file, 132 lines — provides the round-trip test pattern at lines 86-131 and the `JsonValue` typing pattern.

**Analog 3: `framework/src/session/driver/database.rs:199`** — provides the `#[sea_orm(primary_key, auto_increment = false)]` precedent for single-column non-UUID PK.

**Composite PK has NO workspace precedent.** Two columns both annotated with `#[sea_orm(primary_key, auto_increment = false)]` — verified pattern from SeaORM 1.1.x docs (RESEARCH.md §Technical Concerns #1 citation).

**Model declaration pattern — synthesise from analogs:**
```rust
//! SeaORM `Entity` / `Model` / `ActiveModel` / `Column` / `Relation` for the
//! `projection_snapshots` table.
//!
//! Schema authority is `migration.rs` (`CreateProjectionSnapshotsTable`).
//! This module's `Model` shape must match the migration's column declarations
//! exactly.

use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projection_snapshots")]
pub struct Model {
    /// Projection logical name, e.g. `"inventory.dashboard"`. First half
    /// of the composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub projection_name: String,

    /// Per-row key inside the projection, e.g. `"warehouse-a"`. Second
    /// half of the composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,

    /// Serialized `P::State` (D-26 — JSON column).
    pub state: JsonValue,

    /// Monotonic counter (D-25); +1 per apply, reset on rebuild.
    pub version: i64,

    /// App-set `Utc::now()` inside the upsert (D-27).
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**Test pattern — round-trip ActiveModel** — direct mirror of `ferro-reservation/src/entity.rs:86-131`. The PK lookup form for composite PKs uses a tuple:
```rust
let fetched = Entity::find_by_id((name.to_string(), key.0.clone()))
    .one(&conn)
    .await
    .expect("query")
    .expect("found");
```

---

### `ferro-projection/src/migration.rs` (SeaORM migration)

**Analog 1: `ferro-audit/src/migration.rs`** — full file, 177 lines — provides the `DeriveMigrationName` (auto-named via the file's struct name), `Expr::current_timestamp()` default, sqlite_master smoke-test pattern.

**Analog 2: `ferro-reservation/src/migration.rs`** — full file, 194 lines — provides the manual `MigrationName` impl with explicit `"m20260513_000001_..."` name. (Phase 155 will choose `"m20260514_000001_create_projection_snapshots_table"`.)

**Migration declaration pattern — mirror `ferro-reservation/src/migration.rs:18-26`** (explicit `MigrationName`):
```rust
use sea_orm_migration::prelude::*;

pub struct Migration;

impl sea_orm_migration::MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260514_000001_create_projection_snapshots_table"
    }
}
```

**Schema pattern — mirror `ferro-reservation/src/migration.rs:28-107`** but with composite PK syntax:
```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectionSnapshots::Table)
                    .if_not_exists()
                    // projection_name VARCHAR NOT NULL — part of composite PK (D-24)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::ProjectionName)
                            .string()
                            .not_null(),
                    )
                    // key VARCHAR NOT NULL — part of composite PK (D-24)
                    .col(ColumnDef::new(ProjectionSnapshots::Key).string().not_null())
                    // state JSON NOT NULL (D-26)
                    .col(ColumnDef::new(ProjectionSnapshots::State).json().not_null())
                    // version BIGINT NOT NULL (D-25)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::Version)
                            .big_integer()
                            .not_null(),
                    )
                    // updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP (D-27)
                    .col(
                        ColumnDef::new(ProjectionSnapshots::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Composite primary key (D-24)
                    .primary_key(
                        Index::create()
                            .col(ProjectionSnapshots::ProjectionName)
                            .col(ProjectionSnapshots::Key),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectionSnapshots::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectionSnapshots {
    Table,
    ProjectionName,
    Key,
    State,
    Version,
    UpdatedAt,
}
```

**Smoke-test pattern — mirror `ferro-audit/src/migration.rs:103-176` and `ferro-reservation/src/migration.rs:133-193`:**
```rust
#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::Migration)]
        }
    }

    #[tokio::test]
    async fn migration_creates_projection_snapshots_table() {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate up");

        let row = conn
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='projection_snapshots'"
                    .to_string(),
            ))
            .await
            .expect("query sqlite_master");
        assert!(row.is_some(), "projection_snapshots table not created");
    }
}
```

---

### `ferro-projection/tests/common/mod.rs` (test helper)

**Analog:** No in-workspace `BroadcastCapture` helper exists. New pattern derived from `ferro-broadcast/src/broadcaster.rs` public API (lines 41-87 for `Broadcaster::new` + `add_client`; lines 102-188 for `subscribe`).

**Public API used (already exposed):**
- `Broadcaster::new() -> Self` — `ferro-broadcast/src/broadcaster.rs:43`
- `Broadcaster::add_client(socket_id: String, sender: mpsc::Sender<ServerMessage>)` — `:77`
- `Broadcaster::subscribe(socket_id, channel_name, auth, member_info) -> Result<(), Error>` — `:102`
- `BroadcastMessage` enum (server-pushed event payload) — `ferro-broadcast/src/message.rs` (`ServerMessage::Event(BroadcastMessage)`)

**Helper shape — RESEARCH.md §Technical Concerns #5 verbatim:**
```rust
use ferro_broadcast::{BroadcastMessage, Broadcaster, ServerMessage};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Drain-on-demand broadcast capture. Subscribes a single mock client to
/// the channel and stores received `BroadcastMessage` instances in a Vec
/// the test asserts against.
pub struct BroadcastCapture {
    pub broadcaster: Arc<Broadcaster>,
    rx: mpsc::Receiver<ServerMessage>,
}

impl BroadcastCapture {
    pub async fn subscribe(channel: &str) -> Self {
        let broadcaster = Arc::new(Broadcaster::new());
        let (tx, rx) = mpsc::channel(64);
        let socket_id = "test-client".to_string();
        broadcaster.add_client(socket_id.clone(), tx);
        broadcaster
            .subscribe(&socket_id, channel, None, None)
            .await
            .expect("subscribe to test channel");
        Self { broadcaster, rx }
    }

    pub fn drain(&mut self) -> Vec<BroadcastMessage> {
        let mut out = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            if let ServerMessage::Event(bm) = msg {
                out.push(bm);
            }
        }
        out
    }
}
```

`tests/common/mod.rs` is shared across all integration tests in the `tests/` directory. Unit tests inside `src/runtime.rs` cannot import it — they inline a smaller version (RESEARCH.md §Technical Concerns #5 note).

---

### `ferro-projection/tests/event_bus_integration.rs` (integration test)

**Analog:** `ferro-reservation/tests/integration_with_audit_and_events.rs` (full file shape — first 130 lines provide every load-bearing pattern).

**Global-dispatcher isolation pattern — mirror `ferro-reservation/tests/integration_with_audit_and_events.rs:78-84`:**
```rust
use std::sync::OnceLock;
use tokio::sync::Mutex;

/// Process-global mutex serializing all tests that touch the global dispatcher.
static DISPATCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dispatch_lock() -> &'static Mutex<()> {
    DISPATCH_LOCK.get_or_init(|| Mutex::new(()))
}
```

**Listener-cleanup pattern — mirror `ferro-reservation/tests/integration_with_audit_and_events.rs:90-93`:**
```rust
async fn each_test() {
    let _lock = dispatch_lock().lock().await;

    // Clear any stale listeners from prior test runs in this process
    ferro_events::global_dispatcher().forget::<MyEvent>();

    // ... test body
}
```

**Migrator pattern — mirror `ferro-reservation/tests/integration_with_audit_and_events.rs:32-48`:**
```rust
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
```

---

### `ferro-projection/tests/projection_over_reservation_events.rs` (cross-crate showcase)

**Analog:** `ferro-reservation/tests/integration_with_audit_and_events.rs` (full file shape).

The migrator includes BOTH crates' migrations:
```rust
fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
    vec![
        Box::new(ferro_reservation::CreateReservationsTable),
        Box::new(ferro_projection::CreateProjectionSnapshotsTable),
    ]
}
```

A tiny `ReservationCountProjection` folds `ferro_reservation::ReservationEvent::{Held, Committed, Released}` into per-`resource_kind` counts (CONTEXT.md D-47 spec). This is the **milestone-completing showcase test** — the full v11.11 chain in ~60 lines.

---

### `ferro-projection/tests/concurrent_apply.rs` (concurrency test)

**Analog:** `ferro-reservation/tests/concurrent_hold.rs` (listed in directory but not read in this pass — same pattern as the property-test file: `tokio::spawn` × N tasks + `JoinSet` collect, all driving the same underlying primitive).

The test spawns 20 `tokio::spawn`-ed tasks across 5 distinct keys (4 per key), each calling `runtime.apply_event(&event).await`. Final assertion: each key's persisted state shows exactly 4 events (proves per-key serialization + cross-key parallelism, D-32).

---

### `ferro-projection/tests/property_invariants.rs` (property test)

**Analog:** `ferro-reservation/tests/property_invariants.rs` (full file, first 120 lines).

**proptest! macro shape — mirror `ferro-reservation/tests/property_invariants.rs:99-104`:**
```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn proptest_apply_determinism(events in proptest::collection::vec(arb_event(), 0..50)) {
        let rt = build_runtime();
        rt.block_on(async {
            // ... pure-fold reference vs runtime-fold comparison
        });
    }
}
```

**Runtime construction inside proptest — mirror `ferro-reservation/tests/property_invariants.rs:92-97`:**
```rust
use tokio::runtime::Builder as RuntimeBuilder;

fn build_runtime() -> tokio::runtime::Runtime {
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime")
}
```

Three properties locked in CONTEXT.md D-49: apply determinism, replay equivalence (rebuild = sequential applies), cross-key independence.

---

### `Cargo.toml` (root, workspace manifest) — MODIFIED

**Analog:** Current `Cargo.toml` (35 lines).

**Two surgical edits** (mirroring Phase 154's exact pattern):
```diff
 [workspace]
 resolver = "2"
 members = [
     "framework",
     ...
     "ferro-reservation",
+    "ferro-projection",
 ]

 [workspace.package]
-version = "0.2.32"
+version = "0.2.33"
```

The ordering convention is "trailing add" (new crates appended at the end of `members`), matching Phase 154's placement of `ferro-reservation` at line 27 (last entry before `app` is implicitly handled by being in the workspace root).

---

### `.github/workflows/publish.yml` — MODIFIED

**Analog:** Current `publish.yml` line 236 (`WAVE1B_CRATES`).

**Single edit:**
```diff
-          WAVE1B_CRATES="ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation"
+          WAVE1B_CRATES="ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection"
```

**Operational note:** First publish requires manual personal-token bootstrap (CI token is `publish-update`-only — known operational reality flagged in CONTEXT.md D-55 / RESEARCH.md R7). The publish loop's "already exists, skipping" branch (lines 209-216 / 243-250) handles subsequent CI runs idempotently.

---

### `CLAUDE.md` — MODIFIED

**Analog:** Existing `ferro-reservation` row at line 60.

**Single edit — insert one row after line 60** (the disambiguation phrase is locked in CONTEXT.md Specifics line 522):
```diff
 | `ferro-reservation` | Generic hold/commit/release reservation kernel | `src/lib.rs` |
+| `ferro-projection` | Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas. **Not the same as `ferro-projections` (plural)** — see crate docs for the distinction. | `src/lib.rs` |
 | `app` | Sample application | Reference implementation |
```

---

### `README.md` (workspace root) — MODIFIED

**Analog:** Existing `ferro-reservation` bullet at line 73.

**Single edit — insert one bullet after line 73:**
```diff
 - **Resource reservations** — race-free hold/commit/release with TTL, audit, and event broadcast (`ferro-reservation`)
+- **Live read-models** — fold domain events into per-key materialized state with WebSocket delta broadcast (`ferro-projection` — not the same as `ferro-projections` plural)
```

---

### `docs/src/SUMMARY.md` — MODIFIED

**Analog:** Existing `Service Projections` entry at line 47 (which points to `features/projections.md` — the plural crate's doc page).

**Single edit — insert one entry as sibling under the Features section, nav text "Live Read-Models" NOT "Projections" (CONTEXT.md D-52 lock):**
```diff
 - [Service Projections](features/projections.md)
+- [Live Read-Models](features/live-read-models.md)
 - [AI & Confirmation](features/ai.md)
```

The placement adjacency (next to `Service Projections`) is deliberate — readers scanning the nav see both entries adjacently and the on-page disambiguation paragraph closes the loop.

---

### `docs/src/features/live-read-models.md` (NEW user-facing doc)

**Analog:** `docs/src/database/reservations.md` (full file shape, first 80 lines provide the canonical narrative arc).

**Narrative-arc pattern to mirror** (from `docs/src/database/reservations.md:1-80`):
1. Opening paragraph — what the crate is, who it's for.
2. **The Anti-Pattern** — show the hand-rolled fragile code consumers write without the kernel.
3. **The Replacement** — show the typed kernel one-liner.
4. **State Diagram** (or in projection's case, **Per-key serialization diagram**).
5. **Operational caveats** — the footguns enumerated.

**ferro-projection specifics layered on top:**
- Opening paragraph MUST lead with the disambiguation (locked D-52).
- Anti-pattern: hand-rolled `load → apply → persist → broadcast` cycle every event-driven dashboard reinvents.
- Replacement: `Arc::new(Runtime::new(...)).register()` one-liner from CONTEXT.md §Specifics line 484-486.
- Worked example: the `ReservationCountProjection` from D-47, expanded from showcase test into tutorial form.

---

### `CHANGELOG.md` — MODIFIED

**Analog:** `## ferro-reservation` section at `CHANGELOG.md:6-89` (existing).

**Insert NEW `## ferro-projection` section ABOVE the existing `## ferro-reservation` section** (CONTEXT.md D-56 + Phase 152 convention: newest crate goes at the top of the changelog).

**Structure to mirror — bullets from `CHANGELOG.md:14-89`:**
- `### [0.2.33] — 2026-05-14`
- `Initial release. Phase 155 — \`ferro-projection\` crate ...`
- `#### Added` bullet list summarising every public surface item per D-56.
- Closing milestone-completion line (CONTEXT.md §Specifics line 519): "v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm GuardedUpdate (Phase 152), ferro-audit (Phase 153), ferro-reservation (Phase 154), ferro-projection (Phase 155) now all shipped."

---

## Shared Patterns

### Display-prefix error convention

**Source:** `ferro-reservation/src/error.rs:1-5` (file-level rustdoc) + `:13` / `:25` / `:31` / `:35` (variants).

**Apply to:** `ferro-projection/src/error.rs`.

**Excerpt:**
```rust
//! Every variant's `Display` impl prefixes `"projection: …"` so production
//! log greps stay surgical (matches `"guarded: …"`, `"audit: …"`,
//! `"reservation: …"`, `"config: …"`).
```

Every variant's `#[error("…")]` string starts with `"projection: "`. Workspace convention.

---

### `#[from]` for `sea_orm::DbErr` + `serde_json::Error`; hand-`From` for cross-crate errors

**Source:** `ferro-reservation/src/error.rs:35-54` (uses `#[from]` for both); CONTEXT.md D-29 (cites Phase 149 precedent for hand-`From` on `Broadcast(String)`).

**Apply to:** `ferro-projection/src/error.rs`. `#[from]` for `DbErr` and `serde_json::Error`; hand-roll `From<ferro_broadcast::Error>` and `From<ferro_events::Error>` using `.to_string()`.

---

### In-memory SQLite test harness re-derived inline

**Source:** `ferro-reservation/src/entity.rs:71-84`, `ferro-audit/src/entry.rs:204-208`, `ferro-audit/src/query.rs:101-105`, `ferro-audit/src/migration.rs:120-122`. Every ferro-* crate re-derives this inline rather than depending on `framework`.

**Apply to:** Every test mod in `ferro-projection/src/*.rs` and every file in `ferro-projection/tests/`.

**Excerpt:**
```rust
async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = sea_orm::Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```

The local `TestMigrator` registers only the migrations needed for the test scope. The full pattern from `framework/src/database/testing.rs` (TestDatabase with container guard) is NOT used — Phase 155 stays minimal and `framework`-free (CONTEXT.md D-50).

---

### `global_dispatcher()` test isolation

**Source:** `ferro-reservation/tests/integration_with_audit_and_events.rs:78-93`.

**Apply to:** `ferro-projection/tests/event_bus_integration.rs`, `ferro-projection/tests/projection_over_reservation_events.rs`, and any other test that calls `Event::dispatch().await` against the global dispatcher.

**Excerpt:**
```rust
static DISPATCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dispatch_lock() -> &'static Mutex<()> {
    DISPATCH_LOCK.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn my_dispatch_test() {
    let _lock = dispatch_lock().lock().await;
    ferro_events::global_dispatcher().forget::<MyEvent>();
    // ... test body
}
```

Required because `global_dispatcher()` is a `OnceLock<EventDispatcher>` and listeners persist across tests in the same process.

---

### DashMap shard-lock-drop-before-await

**Source:** `ferro-broadcast/src/broadcaster.rs:271` (`drop(channel); // Release DashMap guard before await`).

**Apply to:** `ferro-projection/src/runtime.rs::apply_event` (per-key Mutex acquisition).

**Excerpt:**
```rust
// Clone the Arc INSIDE a short scope so the RefMut drops before we await.
let lock_arc: Arc<tokio::sync::Mutex<()>> = {
    self.locks
        .entry(key.0.clone())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
};  // RefMut drops here, shard unlocked
let _guard = lock_arc.lock().await;  // safe: shard not held during the await
```

Code comment referencing `ferro-broadcast/src/broadcaster.rs:271` belongs in the runtime source — explicit precedent pointer for future reviewers.

---

### Disambiguation paragraph everywhere it's reader-visible

**Source:** CONTEXT.md D-02 / D-51 / D-52 (locked decision spread across rustdoc, doc page, README, CLAUDE.md, CHANGELOG, Cargo.toml description).

**Apply to:**
- `ferro-projection/src/lib.rs` rustdoc (opening paragraph)
- `ferro-projection/src/projection.rs` rustdoc (final-pass D-51)
- `ferro-projection/README.md` (description sentence)
- `ferro-projection/Cargo.toml` `description` field
- Root `README.md` ferro-projection bullet
- Root `CLAUDE.md` ferro-projection table row
- `docs/src/features/live-read-models.md` opening section
- `CHANGELOG.md` `## ferro-projection` section

**Trigger phrase to grep for** (locked verbatim in CONTEXT.md): `"Not to be confused with"` (rustdoc) / `"Not the same as"` (tables, bullets). Plan-checker pre-commit assertion: `grep -q "Not to be confused with .ferro-projections. (plural)" ferro-projection/src/lib.rs`.

---

### SeaORM JSON column + `JsonValue` Model field

**Source:** `ferro-audit/src/entity.rs:43-47` (Before/After `Option<JsonValue>` columns) + `ferro-reservation/src/entity.rs:24-27` (`resource_key: JsonValue`, `window: Option<JsonValue>`).

**Apply to:** `ferro-projection/src/entity.rs::Model::state: JsonValue` (NOT nullable — every persisted row has a state, even `P::State::default()` serialized).

**Excerpt:**
```rust
use serde_json::Value as JsonValue;
...
pub state: JsonValue,
```

Migration uses `.json().not_null()` (cf. `ferro-reservation/src/migration.rs:50`).

---

### Workspace version bump in single edit to root `Cargo.toml`

**Source:** Phase 154's same edit (`0.2.31 → 0.2.32`).

**Apply to:** Phase 155's `0.2.32 → 0.2.33` edit at `Cargo.toml:31`.

The workspace version IS the publish version for every crate in the workspace (all crates use `version.workspace = true`). One edit, one tag.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `ferro-projection/src/entity.rs` (composite PK declaration) | SeaORM entity | CRUD | No workspace precedent for **composite** primary keys via SeaORM. All current `ferro-*` entities use single-column UUID PKs (`ferro-audit/src/entity.rs:19`, `ferro-reservation/src/entity.rs:17`) or single-column non-UUID PK (`framework/src/session/driver/database.rs:199`). The cited pattern (apply `#[sea_orm(primary_key, auto_increment = false)]` to each column) comes from SeaORM 1.1.x docs (RESEARCH.md §Technical Concerns #1). The migration's `.primary_key(Index::create().col(A).col(B))` syntax also has no in-workspace precedent. Plan-03's `migration_creates_table_with_composite_pk` smoke test is the proving artefact. |
| `ferro-projection/src/listener.rs` (`impl Listener<P::Event> for ProjectionListener<P>`) | event subscriber | event-driven | No workspace precedent for `struct ... impl Listener<E> for ...` — every existing ferro-events test uses the closure-based `EventDispatcher::on(closure)` API. Phase 155 is the first user of the struct-listener path. The bound shape is verified against `ferro-events/src/traits.rs:131-151` (the trait) and `ferro-events/src/dispatcher.rs:73-79` (the `listen<E, L>` API). Plan-06's `event_bus_integration.rs` integration test is the proving artefact. |
| `ferro-projection/tests/common/mod.rs` (`BroadcastCapture`) | test helper | event capture | No workspace precedent for capturing `Broadcaster` output in tests. The pattern uses the **production code path** — real `Broadcaster::new`, real `add_client(socket_id, mpsc::Sender)`, real `subscribe(...).await`, real drain via `try_recv` — no mock, no trait, no test-only fork. Shape locked in RESEARCH.md §Technical Concerns #5. |
| `ferro-projection/src/runtime.rs::apply_event` SeaORM `OnConflict` upsert | orchestrator | CRUD | No workspace precedent for `OnConflict::columns([..]).update_columns([..])`. The closest analogs — `ReservationKernel::hold` (uses `Entity::insert`) and `GuardedUpdate` (uses `WHERE`-predicate update) — neither demonstrate the conflict-target upsert pattern. Verified against sea-query 0.32 docs (RESEARCH.md §Technical Concerns #2). |

All four "no analog" cases are surfaced as load-bearing risks in RESEARCH.md (§Technical Concerns + §Risks R3/R4/R5) — the planner's smoke tests in plans 03, 05, 06 are the proving artefacts that close them out before the integration / property tests land.

---

## Metadata

**Analog search scope:**
- `ferro-reservation/` — full crate (closest Wave 1b structural twin)
- `ferro-audit/` — full crate (precedent for JSON column + migration + sqlite_master smoke test)
- `ferro-events/src/{dispatcher.rs, traits.rs, error.rs}` — listener registration API
- `ferro-broadcast/src/{broadcast.rs, broadcaster.rs}` — broadcast builder + Broadcaster construction + DashMap shard-lock pattern
- `framework/src/database/testing.rs` — in-memory SQLite harness reference
- `framework/src/session/driver/database.rs:199` — `#[sea_orm(primary_key, auto_increment = false)]` precedent
- Root `Cargo.toml`, `.github/workflows/publish.yml`, `CLAUDE.md`, `README.md`, `CHANGELOG.md`, `docs/src/SUMMARY.md`, `docs/src/database/reservations.md`

**Files scanned:** 19 (10 in `ferro-reservation/` + 5 in `ferro-audit/` + 3 in `ferro-events/` + 2 in `ferro-broadcast/` + workspace integration files).

**Pattern extraction date:** 2026-05-14

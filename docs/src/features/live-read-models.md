# Live Read-Models

**Not to be confused with `ferro-projections` (plural).** That crate is
the Service Projection abstraction (`ServiceDef → IntentGraph →
JsonUiRenderer`) — it shapes how data renders as UI. This page covers
`ferro-projection` (singular): the live read-model runtime that
subscribes to domain events, maintains a materialized per-key state,
and broadcasts deltas to WebSocket subscribers. The two abstractions
are orthogonal; most apps will use both for different reasons.

`ferro-projection` is the *live-read-model* primitive. Capacity-constrained
apps with live dashboards (operator views, real-time counters, queue depth
panels, kanban boards) all need the same code: subscribe to events → load
the current state → fold the event into state → persist → fan a delta to
the subscribed clients. Without a typed kernel, every consumer hand-rolls
this loop, gets the locking wrong, and ships races. `ferro-projection`
provides the kernel.

## The Anti-Pattern

Without a runtime, every consumer writes the same fragile code:

```rust,ignore
// BAD: load-check-write with manual broadcast — race-prone, easy to forget steps
let current: DashboardState = db
    .query("SELECT state FROM dashboards WHERE id = ?", id)
    .await?
    .unwrap_or_default();
let mut new_state = current.clone();
new_state.apply(&event);
db.execute("UPDATE dashboards SET state = ? WHERE id = ?", &new_state, id).await?;
broadcaster.send("dashboard-{id}", "delta", &new_state).await?;
```

Under concurrent load, two listeners can both read `current = old`,
both `.apply(&event)`, both `UPDATE`, and the second write clobbers
the first. The fix is invariant: serialize per-key, persist and broadcast
under one lock.

## The Replacement

```rust,ignore
use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
use ferro_events::Event;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Consumer event (already implements ferro_events::Event)
#[derive(Clone, Serialize, Deserialize)]
struct InventoryAdjusted { warehouse: String, sku: String, delta: i32 }

impl Event for InventoryAdjusted {
    fn name(&self) -> &'static str { "InventoryAdjusted" }
}

// Consumer projection state + delta
#[derive(Clone, Default, Serialize, Deserialize)]
struct WarehouseDashboard {
    totals: std::collections::HashMap<String, i64>,
}

#[derive(Clone, Serialize)]
struct WarehouseDelta { sku: String, new_total: i64 }

struct WarehouseProjection;

impl Projection for WarehouseProjection {
    type Event = InventoryAdjusted;
    type State = WarehouseDashboard;
    type Delta = WarehouseDelta;
    const NAME: &'static str = "inventory.dashboard";

    fn key(&self, event: &Self::Event) -> ProjectionKey {
        ProjectionKey::new(event.warehouse.clone())
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        let new_total = state.totals.entry(event.sku.clone()).or_insert(0);
        *new_total += event.delta as i64;
        WarehouseDelta { sku: event.sku.clone(), new_total: *new_total }
    }
}

// One-line wiring at application startup
let runtime = Arc::new(ProjectionRuntime::new(db.clone(), broadcaster.clone(), WarehouseProjection));
runtime.clone().register();

// Anywhere in the app:
InventoryAdjusted { warehouse: "a".into(), sku: "sku-1".into(), delta: 5 }
    .dispatch()
    .await?;

// Frontend subscribes to `projection.inventory.dashboard.a` and
// receives event `"delta"` with payload `{ "sku": "sku-1", "new_total": 5 }`.
```

## Per-Key Serialization

```text
 Event::dispatch() ─┐
                    │
      ProjectionListener<P> ──┐
                              │
                              ▼
 ┌── per-key Mutex (DashMap<String, Arc<Mutex<()>>>) ──┐
 │   1. load snapshot from projection_snapshots        │
 │   2. apply(&mut state, &event) → Delta              │
 │   3. upsert snapshot (state, version+1)             │
 │   4. broadcast on projection.{name}.{key}           │
 └─────────────────────────────────────────────────────┘
                              │
                              ▼
                WebSocket clients receive the delta
```

Same-key applies serialize through the per-key Mutex; different-key
applies run in parallel through different DashMap shards.

## The Projection Trait

```rust,ignore
pub trait Projection: Send + Sync + 'static {
    type Event: ferro_events::Event + Serialize + DeserializeOwned;
    type State: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;
    type Delta: Serialize + Clone + Send + Sync + 'static;

    const NAME: &'static str;

    fn key(&self, event: &Self::Event) -> ProjectionKey;
    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;

    // Defaulted:
    fn snapshot_interval(&self) -> u32 { 100 }
    fn broadcast_event_name(&self) -> &'static str { "delta" }
}
```

- **`NAME`** is a dotted-namespace constant: `"inventory.dashboard"`,
  `"checkout.cart"`, `"orders.recent"`. Same convention as
  `ferro-audit`'s action namespace.
- **`State: Default`** is required so a fresh key initializes from
  `State::default()` on first apply. If a state model has no sensible
  default, return an empty or zero variant from `Default`.
- **`apply` is synchronous.** It runs inside the per-key Mutex; an
  async `apply` would let the lock cross await boundaries and serialize
  unrelated work. Heavy work (HTTP fetches, additional DB queries) happens
  before dispatch or after receiving the broadcast delta on the client side.

Multi-tenancy: bake the tenant identifier into the key string
(`"tenant-7:warehouse-a"`); the runtime does not auto-scope by tenant.

## Constructing the Runtime

```rust,ignore
let runtime = ProjectionRuntime::new(
    db_connection,
    broadcaster_arc,
    my_projection,
);
```

The runtime owns the database connection, the broadcaster handle, the
projection impl, and the per-key Mutex registry. Wrap in `Arc` for
sharing across tokio tasks; `register` requires `Arc<Self>`.

## Two Entry Points

| Method | Use |
|--------|-----|
| `Arc<Runtime>::register()` | Auto-wires a listener into `ferro_events::global_dispatcher()`. Every `P::Event::dispatch().await` flows through the projection. The one-line wiring. |
| `runtime.apply_event(&event).await` | Manual entry point — bypass the global dispatcher. For tests, replay scripts, custom dispatchers. |

Both paths share the same per-key serialization, persistence, and
broadcast logic. Mixing both in the same runtime is safe — the manual
path does not interfere with the registered listener.

## The Read Path

```rust,ignore
// Returns Result<Option<State>, ProjectionError>
let maybe_state = runtime.read(&ProjectionKey::new("warehouse-a")).await?;

// Returns Result<State, ProjectionError> with StateNotFound on miss
let state = runtime.read_required(&ProjectionKey::new("warehouse-a")).await?;
```

`read` does NOT acquire the per-key Mutex. Concurrent `read` +
`apply_event` is safe; the SQL upsert is atomic at the DB level so
readers see either the pre- or post-upsert state, never a torn read.

## The Rebuild Path

```rust,ignore
let events: Vec<InventoryAdjusted> = audit_log_replay_for_key().collect();
let state = runtime
    .rebuild(&ProjectionKey::new("warehouse-a"), events)
    .await?;
```

`rebuild` discards the existing snapshot for the key, folds the
supplied event sequence through `State::default()`, persists the final
state, and broadcasts ONE `"rebuild"` frame carrying the full final
state (overriding the default `broadcast_event_name`). Clients reset
their local state on receipt of a `"rebuild"` frame.

Empty iterator wipes the snapshot row and returns `State::default()`
with no insert and no broadcast.

## Broadcast Channel Contract

- **Channel:** `format!("projection.{}.{}", P::NAME, key.as_str())`
  — example: `"projection.inventory.dashboard.warehouse-a"`.
- **Event name:** from `P::broadcast_event_name()`, default `"delta"`.
  Override to `"dashboard_updated"`, `"cart_changed"`, etc., if the
  frontend dispatches on event name.
- **Payload:** raw JSON-serialized `P::Delta`. No envelope, no wrapping
  object — frontends receive the delta directly.
- **Rebuild frame:** event name `"rebuild"`, payload is the full final
  state (not a delta).

## Operational Footguns

1. **Broadcast failure does NOT roll back state.** If `Broadcast::send`
   fails (no subscribers, network error), the snapshot row is already
   persisted; the runtime logs at `tracing::warn!` and returns
   `ProjectionError::Broadcast`. Subscribers reconcile by re-reading
   the snapshot via `runtime.read(...)`.
2. **Single-instance assumption.** v0 assumes a single application
   instance owns each projection's listener. Multi-instance deployments
   must elect a single projection-runner node or accept last-writer-wins
   behavior on concurrent applies to the same key from different nodes.
3. **`register` is not idempotent on `Arc` identity.** Calling
   `Arc<ProjectionRuntime<P>>::register()` twice registers two listeners
   — both fire on each dispatch (same semantic as registering a listener
   twice in any event bus). Register once at app startup.

## Worked Example: Reservation Counts Dashboard

A live counter dashboard that folds `ferro_reservation::ReservationEvent`
into per-`resource_kind` `{held, committed, released}` counters:

```rust,ignore
use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
use ferro_reservation::{ReleaseReason, ReservationEvent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default, Serialize, Deserialize)]
struct ReservationCounters {
    held: u32,
    committed: u32,
    released: u32,
}

#[derive(Clone, Serialize)]
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
        let rk = match event {
            ReservationEvent::Held { resource_kind, .. }
            | ReservationEvent::Committed { resource_kind, .. }
            | ReservationEvent::Released { resource_kind, .. }
            | ReservationEvent::Expired { resource_kind, .. } => resource_kind,
        };
        ProjectionKey::new(rk.clone())
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        match event {
            ReservationEvent::Held { .. } => state.held += 1,
            ReservationEvent::Committed { .. } => state.committed += 1,
            ReservationEvent::Released { .. } | ReservationEvent::Expired { .. } => {
                state.released += 1
            }
        }
        CountersDelta {
            held: state.held,
            committed: state.committed,
            released: state.released,
        }
    }
}

// App startup
let runtime = Arc::new(ProjectionRuntime::new(
    db.clone(),
    broadcaster.clone(),
    ReservationCountProjection,
));
runtime.clone().register();

// Every reservation transition (hold/commit/release/expire) emitted
// by ferro-reservation now flows into per-resource_kind counter state
// and a delta on `projection.reservations.counters.{resource_kind}`.
```

Each call to `Kernel::hold`, `Kernel::commit`, or `Kernel::release`
dispatches a `ReservationEvent`; the projection folds it; a counter-update
delta lands on the subscribed WebSocket channel; the operator dashboard
updates in real time.

## Schema

The migration creates a single `projection_snapshots` table:

```sql
projection_snapshots
├── projection_name VARCHAR NOT NULL     -- P::NAME ("inventory.dashboard")
├── key             VARCHAR NOT NULL     -- ProjectionKey.as_str()
├── state           JSON NOT NULL        -- serialized P::State
├── version         BIGINT NOT NULL      -- monotonic counter
├── updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
└── PRIMARY KEY (projection_name, key)   -- composite PK
```

Register the migration in your `Migrator`:

```rust,ignore
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ferro_projection::CreateProjectionSnapshotsTable),
            // ... your app migrations
        ]
    }
}
```

## Errors

`ProjectionError` is the single error enum:

| Variant | When |
|---------|------|
| `Db(#[from] sea_orm::DbErr)` | underlying database error |
| `Json(#[from] serde_json::Error)` | State serialization failure |
| `Broadcast(String)` | broadcast publish failed (state already persisted) |
| `Events(String)` | event-bus error from `ferro_events::Error` |
| `StateNotFound { name, key }` | `read_required` hit a missing key |

Display prefix is `"projection: …"` for grep-friendliness across the
workspace.

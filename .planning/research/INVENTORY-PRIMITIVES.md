# Resource Reservation & Live Read-Model Primitives — Design

**Date:** 2026-05-13
**Status:** Approved direction, phase planning pending
**Driver:** gestiscilo-it inventory monitoring field test (2026-05-13 audit). The gestiscilo concrete-intent design lives at `<gestiscilo-repo>/.planning/research/INVENTORY-MONITORING-DESIGN.md`. This file is the ferro-side slice — domain-neutral primitives that any consumer app can use.

---

## Killer feature

**Race-free reservations as a first-class framework primitive.** Apps with capacity constraints (booking, ticketing, checkout, queue admission, rate limiting) all hand-roll the same buggy `read → check → write` pattern. ferro-reservation turns it into a typed kernel with TTL, audit trail, and broadcast-ready events, race-free by construction.

Bonus: `ferro-orm::guarded` and `ferro-projection` are independently useful and unlock the same correctness gains in apps that don't need reservations (live dashboards, atomic counters).

---

## Why now

Two consumer milestones already need this:

1. **gestiscilo-it v6.3 Online Checkout** — already planned. Customer holds a slot for 15 min while paying via Stripe; webhook commits or releases. Today this would be hand-rolled per app.
2. **gestiscilo-it inventory monitoring** (current driver) — needs hold/commit/release semantics for bookings, plus live read-model for the Magazzino operator dashboard.

Building this in ferro once means both consumers get correctness for free, and any future app inherits it.

---

## Crates

### `ferro-reservation` (new)

Generic resource reservation. Knows nothing about inventory, products, or time-windows specifically — just "resource with capacity, hold a slice, commit or expire."

```rust
pub trait Resource: Send + Sync + 'static {
    type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
    type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;
    // Window = () for non-windowed resources (counters, atomic capacity)

    async fn capacity(&self, key: &Self::Key, window: &Self::Window) -> Result<u32>;
    async fn held(
        &self, key: &Self::Key, window: &Self::Window, db: &DatabaseConnection,
    ) -> Result<u32>;
}

pub struct ReservationKernel<R: Resource> { /* DB-backed */ }

#[derive(Debug, Error)]
pub enum HoldError {
    #[error("insufficient capacity (requested {requested}, available {available})")]
    Insufficient { requested: u32, available: u32 },
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),
}

impl<R: Resource> ReservationKernel<R> {
    pub async fn hold(
        &self, key: R::Key, window: R::Window, quantity: u32, ttl: Duration,
    ) -> Result<ReservationHandle, HoldError>;

    pub async fn commit(&self, handle: ReservationHandle) -> Result<()>;
    pub async fn release(&self, handle: ReservationHandle) -> Result<()>;
    pub async fn extend(&self, handle: ReservationHandle, by: Duration) -> Result<()>;
}
```

**Schema** (migration shipped with the crate):

```
reservations
├── id (uuid, pk)
├── resource_kind (string)        -- "inventory_unit", "checkout_slot", ...
├── resource_key (json)           -- serialized Resource::Key
├── window (json, nullable)       -- serialized Resource::Window
├── quantity (int)
├── status (string)               -- held | committed | released | expired
├── expires_at (timestamp)
├── held_at, committed_at, released_at (nullable)
├── tenant_id (uuid, nullable)    -- optional multi-tenant scoping
```

Indexes:
- `(resource_kind, resource_key, window, status)` — `held` lookup for capacity calc
- `(status, expires_at)` — sweeper

**Sweeper:** background ferro-queue job, runs every 60s. `SELECT … WHERE status='held' AND expires_at < now()` → transition to `expired`, emit `ReservationExpired` event.

**Events** (ferro-events):

```rust
pub enum ReservationEvent {
    Held { handle: ReservationHandle, key, window, quantity, expires_at },
    Committed { handle: ReservationHandle },
    Released { handle: ReservationHandle, reason: ReleaseReason },
    Expired { handle: ReservationHandle },
}
```

**Concurrency:** all state transitions use the same `WHERE status = $expected` predicate to make double-transition impossible. SQLite serial-writer guarantees this is atomic; on Postgres it's safe with `READ COMMITTED`.

---

### `ferro-orm::guarded` (extend ferro-orm)

```rust
// Atomic conditional update — exactly the SQL we need for race-free decrement.
GuardedUpdate::new(inventory_units::Entity)
    .filter(inventory_units::Column::Id.eq(unit_id))
    .filter(inventory_units::Column::Quantity.gte(needed))
    .set_expr(inventory_units::Column::Quantity,
              Expr::col(Column::Quantity).sub(needed))
    .exec_one(&txn).await?;     // errors if rows_affected != 1
```

```rust
pub struct GuardedUpdate<E: EntityTrait> { /* … */ }

impl<E: EntityTrait> GuardedUpdate<E> {
    pub fn new(entity: E) -> Self;
    pub fn filter(self, f: impl IntoCondition) -> Self;
    pub fn set_expr(self, col: E::Column, expr: SimpleExpr) -> Self;
    pub fn set_value(self, col: E::Column, value: Value) -> Self;
    pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError>;
    pub async fn exec_at_most_one<C: ConnectionTrait>(self, conn: &C) -> Result<bool, GuardedError>;
}

pub enum GuardedError {
    NoRowsAffected,                       // predicate failed
    TooManyRows { affected: u64 },        // never expected — index/uniqueness bug
    Db(DbErr),
}
```

One statement, one round-trip, race-free by construction. Replaces the `read-check-write` pattern wherever a single column's value is conditionally mutated.

---

### `ferro-projection` (new)

Live read-model pattern. Subscribes to domain events, maintains a materialized view, broadcasts deltas to UI subscribers via the existing `ferro-broadcast` channels.

```rust
pub trait Projection: Send + Sync + 'static {
    type Event: DomainEvent;
    type State: Clone + Serialize + DeserializeOwned;
    type Delta: Serialize;

    fn key(&self, event: &Self::Event) -> ProjectionKey;
    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;
    fn snapshot_interval(&self) -> u32 { 100 }    // events between snapshots
}

pub struct ProjectionRuntime<P: Projection> {
    /* subscribes to ferro-events, persists snapshots, fans deltas to ferro-broadcast */
}

impl<P: Projection> ProjectionRuntime<P> {
    pub fn register(self, app: &mut FerroApp);   // wire into event bus + broadcast channel
    pub async fn read(&self, key: ProjectionKey) -> Option<P::State>;
    pub async fn rebuild_from_events(&self, key: ProjectionKey) -> Result<()>;
}
```

**Snapshot strategy:**
- State written to `projection_snapshots(projection_name, key, state_json, event_offset)` every N events.
- On startup / cache miss, load latest snapshot + replay events past `event_offset`.
- `rebuild_from_events` walks the entire event log — used for schema changes or after audit-detected divergence.

**Broadcast contract:** every `apply()` emits a `Delta` that the broadcast channel sends to subscribed WebSocket clients. Subscribers maintain their own client-side state by applying deltas to the initial snapshot they fetched via `read`.

---

### `ferro-audit` (new)

Structured before/after audit log for state-changing operations.

```rust
audit_log!(
    actor: AuditActor::User(user_id),
    action: "inventory.stock.adjust",
    target: AuditTarget::InventoryUnit(unit_id),
    before: json!({ "quantity": old }),
    after: json!({ "quantity": new }),
    reason: "order_committed",
);
```

**Schema:**

```
audit_log
├── id (uuid)
├── tenant_id (nullable)
├── actor_kind, actor_id          -- User | System | Job | ApiClient
├── action (string)                -- dotted namespace: "inventory.stock.adjust"
├── target_kind, target_id
├── before (json, nullable)
├── after (json, nullable)
├── reason (string, nullable)
├── correlation_id (uuid, nullable) -- ties to request/job
├── created_at
```

Indexes on `(tenant_id, target_kind, target_id, created_at)` and `(tenant_id, actor_kind, actor_id, created_at)`.

**Replay:** any object's full state history is `SELECT * FROM audit_log WHERE target_kind=? AND target_id=? ORDER BY created_at`. Apply `before → after` diffs to reconstruct.

**Retention:** consumer-app choice. Crate ships a `prune_older_than` helper. Default: no retention policy (keep forever).

---

## Cross-crate relationships

```
ferro-reservation
    ├── uses ferro-orm (DB access)
    ├── uses ferro-orm::guarded (state transitions)
    ├── uses ferro-events (emits ReservationEvent)
    └── uses ferro-audit (logs every commit/release/expire)

ferro-projection
    ├── uses ferro-orm (snapshot persistence)
    ├── uses ferro-events (event subscription)
    └── uses ferro-broadcast (delta fanout)

ferro-audit
    └── uses ferro-orm (audit_log table)

ferro-orm::guarded
    └── extends ferro-orm (no new deps)
```

No circular deps. ferro-audit is the most foundational (no ferro deps beyond ferro-orm). ferro-reservation depends on everything else.

---

## Consumer integration shape

### gestiscilo-it (driving)

```rust
struct InventoryUnitResource { db: DatabaseConnection }

impl Resource for InventoryUnitResource {
    type Key = (TenantId, ProductId);
    type Window = BookingWindow;  // (date, time_range) for noleggio, () for vendita
    // ... capacity / held implementations query inventory_units + reservations
}

static KERNEL: ReservationKernel<InventoryUnitResource>;
```

### Future apps

A ticketing app reserving seats:

```rust
struct SeatResource { /* ... */ }
impl Resource for SeatResource {
    type Key = ShowId;
    type Window = ();
    // capacity = venue.seat_count, held = sum of held + committed reservations
}
```

A rate limiter:

```rust
struct ApiQuotaResource;
impl Resource for ApiQuotaResource {
    type Key = ApiClientId;
    type Window = MinuteBucket;
    // capacity = client.rate_limit, held = requests_in_bucket
}
```

---

## Testing strategy

### Per-crate unit

- `ferro-reservation` — hold-commit-release-expire transitions; concurrent hold on the last unit (only one succeeds); TTL sweeper transitions correctly; `extend` only works on `held`; serialization round-trip of `Resource::Key` / `Window`.
- `ferro-orm::guarded` — predicate succeeds → 1 row affected; predicate fails → 0 rows, error; condition matches >1 row → error (never expected); transaction support.
- `ferro-projection` — apply determinism (same events → same state); snapshot/replay equivalence; broadcast emits one delta per apply; rebuild from empty.
- `ferro-audit` — log writes, indexed queries, before/after JSON round-trip, replay reconstructs target state.

### Integration (cross-crate)

- `ReservationKernel.commit` writes audit log entry.
- `ReservationKernel` state changes emit events that a `Projection` consumes and broadcasts.
- Sweeper run while a commit is in-flight on the same handle: exactly one wins.

### Property-based

- For any random interleaving of N concurrent `hold` calls against capacity C, total committed quantity ≤ C.
- For any sequence of guarded updates against a counter starting at K, the counter never goes negative.
- Audit log replay produces the same final state as the live system.

---

## Migration / rollout

1. **`ferro-orm::guarded`** — additive, no breaking changes. Ship first.
2. **`ferro-audit`** — additive crate. Ship in parallel.
3. **`ferro-reservation`** — depends on 1 + 2. Ship after.
4. **`ferro-projection`** — independent of 1-3, but useful with them. Ship in parallel with 3.

After all four ship and auto-publish via the existing GH Actions workflow, gestiscilo-it bumps versions and proceeds with its inventory-monitoring milestone.

---

## Out of scope

- **Cross-process distributed locks.** SQLite single-writer + Postgres `READ COMMITTED` are sufficient for the targeted consumers. Don't pull in Redis / Raft.
- **Inventory-specific schema in ferro.** ferro doesn't know what "jet skis" are. The `Resource` trait is the only contract.
- **UI components for reservation status.** Apps render their own UI. ferro-projection's contract is "state + delta stream" — UI is the consumer's job.
- **Multi-region replication.** Single-region SQLite/Postgres. Replication is a future ferro concern.

# Reservations

`ferro-reservation` is a generic resource reservation kernel for the Ferro
framework. Capacity-constrained apps — booking, ticketing, checkout, queue
admission, rate limiting — all need the same primitive: hold N units of a
resource for a deadline, then commit or release. This crate provides it as a
typed, race-free state machine with automatic audit emission and event
broadcast. No hand-rolled `read → check → write` SQL required.

## The Anti-Pattern

Without a kernel, every consumer writes the same fragile code:

```rust,ignore
// BAD: read-check-write has a race window
let current = db.query("SELECT held FROM inventory WHERE id = ?", id).await?;
if current.held + qty <= capacity {
    db.execute("UPDATE inventory SET held = held + ? WHERE id = ?", qty, id).await?;
} else {
    return Err(NoCapacity);
}
```

Under concurrent load, two callers can both observe `held = 4` (capacity = 5)
and both execute the `UPDATE`, leaving `held = 6 > capacity`. The fix is
structural: fold the check into the UPDATE statement so the database enforces
atomicity. `ferro-reservation` uses [`ferro-orm::GuardedUpdate`] under the hood
to make every state transition race-free by construction.

## The Replacement

```rust,ignore
use ferro_reservation::{ReservationKernel, ReservationContext, Resource, ReleaseReason};
use std::time::Duration;

struct InventoryUnitResource { /* db reference, business rules */ }

#[async_trait::async_trait]
impl Resource for InventoryUnitResource {
    type Key = ProductId;
    type Window = BookingWindow;
    const KIND: &'static str = "inventory.unit";

    async fn capacity<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, _w: &Self::Window)
        -> Result<u32, ReservationError> { /* ... */ }
    async fn held<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, w: &Self::Window)
        -> Result<u32, ReservationError> { /* ... */ }
}

// Construct once at application start
let kernel = ReservationKernel::new(db.clone(), InventoryUnitResource::new());

// Hold during payment
let ctx = ReservationContext::user(user_id.to_string()).with_correlation(request_id);
let handle = kernel.hold(&conn, key, window, 1, Duration::from_secs(15 * 60), &ctx).await?;

// Commit or release based on payment outcome
match stripe_result {
    Ok(_)  => kernel.commit(&conn, handle, &ctx).await?,
    Err(_) => kernel.release(&conn, handle, ReleaseReason::PaymentFailed, &ctx).await?,
}
```

## State Diagram

```text
              hold()                  commit()
   ──────────────▶ held ──────────────────────▶ committed
                    │
                    │ release(reason)
                    ▼
                released

              hold()
   ──────────────▶ held ─── ttl ─────────────▶ expired
                                 run_sweep_once()
```

Terminal states (`committed`, `released`, `expired`) have no outgoing
transitions. Any attempt surfaces as `ReservationError::ConflictingState`.

## Resource Trait

Consumers implement `Resource` to define their capacity model:

```rust,ignore
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
    type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;

    const KIND: &'static str;

    async fn capacity<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;

    async fn held<C: ConnectionTrait>(
        &self,
        conn: &C,
        key: &Self::Key,
        window: &Self::Window,
    ) -> Result<u32, ReservationError>;
}
```

- **`Key`** identifies a resource instance (`ProductId`, `ShowId`,
  `ApiClientId`, ...).
- **`Window`** scopes capacity (a date+time range for booking, `()` for
  non-windowed resources).
- **`KIND`** is a `&'static str` dotted-namespace constant — convention mirrors
  `ferro-audit`'s action/target convention (`"inventory.unit"`,
  `"checkout.slot"`, `"api.quota"`).

Multi-tenancy is a `Key` concern: include the tenant identifier in `Key`, not
as a kernel-level parameter. The kernel does not scope queries by tenant
automatically.

## Lifecycle Methods

| Method | Transition | Returns |
|--------|------------|---------|
| `hold(&conn, key, window, qty, ttl, ctx)` | `(none) → held` | `ReservationHandle` |
| `commit(&conn, handle, ctx)` | `held → committed` | `()` |
| `release(&conn, handle, reason, ctx)` | `held → released` | `()` |
| `extend(&conn, handle, by, ctx)` | `held → held` (new `expires_at`) | `()` |
| `run_sweep_once()` | `held → expired` (TTL) | `SweepReport` |

`handle` is taken by value in `commit` / `release` / `extend` — use-once is a
compile-time guarantee. Reusing a handle after commit or release is a compile
error.

`hold` sequence:
1. Call `R::capacity(&conn, &key, &window)`.
2. Call `R::held(&conn, &key, &window)`.
3. If `held + quantity > capacity` → `Err(Insufficient { requested, available, capacity })`.
4. INSERT one `reservations` row with `status = 'held'`, `expires_at = now() + ttl`.
5. Emit `ReservationEvent::Held` via `ferro-events`.
6. Write one `AuditEntry` with `action = "reservation.held"` via `ferro-audit`.
7. Return `ReservationHandle`.

Note: the capacity check and the INSERT are two separate statements, not a
single atomic SQL operation. Under SQLite's serial-writer semantics concurrent
tasks should serialize `hold` calls at the application layer (e.g., a
`tokio::Mutex` per resource key). See the Consistency Model section.

## ReservationContext

Per-call audit metadata bundle:

```rust,ignore
pub struct ReservationContext {
    pub actor: AuditActor,
    pub correlation_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub reason: Option<String>,
}

impl ReservationContext {
    pub fn system() -> Self;
    pub fn user(user_id: impl Into<String>) -> Self;
    pub fn job(name: impl Into<String>) -> Self;
    pub fn anonymous() -> Self;

    pub fn with_correlation(self, id: Uuid) -> Self;
    pub fn with_tenant(self, t: impl Into<String>) -> Self;
    pub fn with_reason(self, r: impl Into<String>) -> Self;
}
```

The `actor` is recorded on every audit entry written during the state
transition. The optional fields propagate to the audit log when populated.
Builder methods follow the consuming `mut self → Self` convention used across
the framework.

## TTL and the Sweeper

`hold(...)` accepts a `ttl: Duration`. The persisted `expires_at` is computed at
hold time as `now() + ttl`. `run_sweep_once()` scans for held rows whose
`expires_at` is in the past, transitions them to `expired`, and emits one audit
entry and one event per row.

The crate has **no `ferro-queue` runtime dependency** — consumers schedule
sweeps with their preferred mechanism. Three idiomatic patterns:

### Pattern 1: ferro-queue Job

```rust,ignore
struct SweepReservations;

impl ferro_queue::Job for SweepReservations {
    async fn handle(self) -> Result<(), Error> {
        KERNEL.run_sweep_once().await?;
        Ok(())
    }
}

// Schedule every 60 seconds via your queue's recurring-job API
```

### Pattern 2: tokio::time::interval

```rust,ignore
tokio::spawn(async {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    loop {
        tick.tick().await;
        if let Err(e) = KERNEL.run_sweep_once().await {
            tracing::warn!(error = %e, "sweep failed");
        }
    }
});
```

### Pattern 3: Cron-driven CLI

```rust,ignore
// e.g. your-app reservation:sweep
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        KERNEL.run_sweep_once().await.expect("sweep");
    });
}
```

`SweepReport { expired_count, scanned_at }` is returned for observability. A
consistent non-zero `expired_count` across successive sweeps indicates a backlog;
schedule sweeps more frequently. The sweeper processes at most 500 rows per call;
subsequent calls drain the backlog.

The sweeper is idempotent under concurrent execution. If two sweeper tasks race
on the same expired row, only one wins the guarded `held → expired` transition;
the other gets zero rows affected and skips that row silently.

## ReservationEvent Subscription

Every state transition dispatches a `ReservationEvent` via `ferro-events`.
Subscribe with `ferro_events::global_dispatcher()`:

```rust,ignore
ferro_events::global_dispatcher()
    .on::<ReservationEvent, _, _>(|event: ReservationEvent| {
        Box::pin(async move {
            match event {
                ReservationEvent::Held { id, quantity, .. } => {
                    // notify a live dashboard, update a cache, etc.
                }
                ReservationEvent::Committed { id, .. } => { /* ... */ }
                ReservationEvent::Released { id, reason, .. } => { /* ... */ }
                ReservationEvent::Expired { id, .. } => { /* ... */ }
            }
            Ok(())
        })
    });
```

Event dispatch is **best-effort**. If dispatch fails, the kernel logs at
`tracing::warn!` and returns `Ok(())` — the state change is already committed
to the database. Consumers needing durable replay use the audit log.

## Audit Log Inspection

Every state transition writes one [`ferro_audit::AuditEntry`] with
`action = "reservation.{held|committed|released|expired|extended}"` and
`target = AuditTarget::new("reservation", id.to_string())`. The full history
for a reservation:

```rust,ignore
let target = ferro_audit::AuditTarget::new("reservation", id.to_string());
let history = ferro_audit::history_for_target(&target, &conn).await?;
let final_state = ferro_audit::reconstruct_state(&history);
```

`reconstruct_state` performs a shallow merge of `after` payloads in
`created_at ASC` order; the resulting value reflects the most recently written
state fields.

Audit emission is **unconditional**. If `AuditEntry::write` fails (e.g., the
`audit_log` table is missing), the kernel returns `ReservationError::Audit` but
the DB state transition is already committed. See Operational Footguns.

## Common Patterns

### Slot hold during online checkout

`Resource::Key = ProductId`. Hold for 15 minutes during Stripe payment; on
`payment_intent.succeeded` webhook commit, on `payment_intent.payment_failed`
release with `ReleaseReason::PaymentFailed`. If the user abandons, the sweeper
transitions the row to `expired` automatically.

### Ticket reservations

`Resource::Key = ShowId`, `Resource::Window = ()`. `capacity = venue.seat_count`,
`held = sum of held + committed reservations`. A short TTL (e.g., 5 minutes)
prevents zombie holds from blocking legitimate purchasers.

### API rate-limit buckets

`Resource::Key = ApiClientId`, `Resource::Window = MinuteBucket`.
`capacity = client.rate_limit`, `held = requests_in_bucket`. The kernel rejects
holds beyond capacity with `Insufficient`. Combine with `run_sweep_once` on a
per-minute cron to garbage-collect expired buckets.

## Schema

The migration creates a single `reservations` table:

```sql
-- 12 columns
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()  -- client-generated UUIDv4
resource_kind   VARCHAR NOT NULL      -- "inventory.unit", "checkout.slot"
resource_key    JSON NOT NULL         -- serialized Resource::Key
window          JSON NULL             -- serialized Resource::Window; NULL when Window = ()
quantity        INTEGER NOT NULL
status          VARCHAR NOT NULL      -- "held" | "committed" | "released" | "expired"
expires_at      TIMESTAMP NOT NULL
held_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
committed_at    TIMESTAMP NULL
released_at     TIMESTAMP NULL
release_reason  VARCHAR NULL
tenant_id       VARCHAR NULL

-- 2 indexes
idx_reservations_kind_key_window_status  -- (resource_kind, resource_key, window, status)
idx_reservations_status_expires           -- (status, expires_at) — sweeper scan path
```

Register the migration in your `Migrator` alongside `ferro-audit`'s:

```rust,ignore
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ferro_audit::CreateAuditLogTable),
            Box::new(ferro_reservation::CreateReservationsTable),
            // ... your app migrations
        ]
    }
}
```

## Errors

`ReservationError` is the single error enum:

| Variant | When |
|---------|------|
| `Insufficient { requested, available, capacity }` | Hold rejected — not enough capacity |
| `ConflictingState { id, expected }` | State-transition predicate failed (row already committed, released, or expired; or never existed) |
| `NotFound { id }` | Introspection or debug paths |
| `Db(#[from] sea_orm::DbErr)` | Underlying database error |
| `Guarded(#[from] ferro_orm::GuardedError)` | Guarded-update error other than `NoRowsAffected` (programming error) |
| `Audit(#[from] ferro_audit::AuditError)` | Audit emission failed AFTER state was already committed |
| `Json(#[from] serde_json::Error)` | `Resource::Key` or `Window` serialization failure |

Display prefix is `"reservation: …"` for log greppability across the workspace.

## Consistency Model

State transitions (`commit`, `release`, `extend`) are race-free on both SQLite
and Postgres: each is a single `UPDATE … WHERE status = 'held' AND id = ?`
statement executed via [`ferro-orm::GuardedUpdate`]. Per-statement atomicity
ensures concurrent callers cannot both succeed on the same row; one wins and the
other gets `ConflictingState`.

**`hold` on SQLite:** SQLite WAL mode serializes writers at the file level, but
the three-step hold sequence (capacity SELECT + held SELECT + INSERT) is not a
single statement. Under concurrent tokio tasks connecting to the same SQLite
database, the capacity check and the INSERT can interleave. Consumers running
concurrent holds against SQLite should serialize `hold` calls at the application
layer — a `tokio::sync::Mutex` per resource key is the idiomatic pattern.

**`hold` on Postgres:** Under Postgres `READ COMMITTED`, the capacity check has
a theoretical race window between the SELECT and the INSERT. The current crate is
SQLite-validated; Postgres correctness for the capacity check is on the roadmap
as a follow-up addition (`SELECT FOR UPDATE` or a counter column approach).
`commit`, `release`, and `extend` via `GuardedUpdate` are race-free on both
dialects.

## Operational Footguns

1. **Audit failure does not roll back state.** If `AuditEntry::write` fails for
   any reason (database connection lost, `audit_log` table missing), the kernel
   returns `ReservationError::Audit` AFTER the DB row has already been
   transitioned. The state change is committed; only the audit record is missing.
   Monitor for `ReservationError::Audit` in your error-handling layer.

2. **Event dispatch is best-effort.** `ferro_events::dispatch` failure logs at
   `tracing::warn!` but does not propagate. Use the audit log for durable replay
   of state transitions.

3. **No upper cap on `extend`.** A held reservation can be extended
   indefinitely. Consumers wanting a hard TTL ceiling must enforce it at the call
   site before calling `extend`.

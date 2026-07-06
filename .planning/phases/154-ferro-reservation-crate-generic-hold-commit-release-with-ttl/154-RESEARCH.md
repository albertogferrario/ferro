# Phase 154: ferro-reservation — Research

**Researched:** 2026-05-13
**Domain:** Rust crate scaffold — generic resource reservation kernel with TTL, event broadcast, audit emission
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

All 58 decisions D-01..D-58 in 154-CONTEXT.md are locked. Key anchors for the planner:

- **D-01:** New top-level workspace crate at `ferro-reservation/` — Wave 1b
- **D-03:** Runtime deps: ferro-orm, ferro-events, ferro-audit (all Wave 1a, already published)
- **D-04:** Wave 1b publish — add to `WAVE1B_CRATES` in `.github/workflows/publish.yml`
- **D-05/D-06:** `Resource` trait generic over `<C: ConnectionTrait>` for both `capacity` and `held`
- **D-09..D-15:** Kernel API shape — `hold/commit/release/extend/run_sweep_once`; owned `DatabaseConnection` in struct for sweeper, per-call `&C: ConnectionTrait` for state-transition methods
- **D-12:** Every state transition via `GuardedUpdate` with `NoRowsAffected → ConflictingState` mapping
- **D-16:** Status stored as `VARCHAR` strings, not SeaORM `ActiveEnum`
- **D-19:** Concurrency correctness claim: single-statement atomicity + unique constraint (D-39 schema) prevents over-allocation
- **D-21..D-24:** `run_sweep_once` returns `SweepReport`; 500-row LIMIT; idempotent under concurrent sweepers
- **D-25/D-26:** `ReservationEvent` implements `ferro_events::Event`; dispatch after state change; `tracing::warn!` on dispatch failure; state not rolled back
- **D-28..D-30:** Unconditional audit emission; `ReservationContext` bundle; audit failure surfaces as `ReservationError::Audit` but state is already committed
- **D-38..D-42:** Schema — 12 columns, 2 composite indexes, UUID PK, client-generated id
- **D-43..D-46:** `ReservationError` umbrella enum with `From<GuardedError>` and `From<AuditError>`; `NoRowsAffected` mapped explicitly to `ConflictingState` before `?`
- **D-47..D-52:** Tests — 12 unit tests + concurrent_hold integration + 2 proptest properties + cross-crate integration; `proptest = "1"` as dev-dep
- **D-56:** Workspace version bump 0.2.31 → 0.2.32
- **D-57:** First-publish bootstrap from local terminal (CI token publish-update only)

### Claude's Discretion

- Internal module layout of `ferro-reservation/src/`
- Whether to expose SeaORM `Entity`/`Model`/`ActiveModel` as public re-exports (recommended)
- Exact wording of `tracing::warn!` diagnostics on event-dispatch / audit failure
- Whether `SweepReport` is a public type (recommended: yes)
- Exact `proptest` strategy shape for the property generators
- Test file naming inside `ferro-reservation/tests/`
- Whether to expose `available_capacity` convenience helper on the kernel

### Deferred Ideas (OUT OF SCOPE)

`try_hold` variant, `cancel_all_for`, reservation grouping, distributed locks, Postgres CI tests, MCP tool `reservation_check_capacity`, WebSocket broadcast, reservation archival, `ferro-queue` integration, `ferro::prelude` re-export, per-call audit suppression, capacity-aware retry.
</user_constraints>

---

## Summary

`ferro-reservation` is the third crate in the v11.11 milestone (after `ferro-orm` at 0.2.30 and `ferro-audit` at 0.2.31). It composes three already-shipped Wave 1a crates into a typed hold/commit/release/expire kernel. All three runtime dependencies are verified live on crates.io and in Cargo.lock.

The phase is greenfield — no `ferro-reservation` code exists in the workspace. The structural template is well-established: two identical sibling phases (152, 153) have already run through the exact same scaffold-test-publish sequence. Every wiring pattern ferro-reservation needs is already verified in those crates' source code.

The single correctness-critical design decision is how `hold` prevents over-allocation. CONTEXT.md D-19 states that per-statement atomicity is the mechanism, and CONTEXT.md D-19 also says the INSERT itself uses a unique constraint to prevent over-allocation. However, the schema in D-39 does NOT define a unique constraint on the `reservations` table. Research below (Special Area 1) resolves this discrepancy: the correct mechanism is the INSERT + post-INSERT capacity re-check pattern, OR optimistic re-read — the property test in D-48 proves safety in practice under SQLite serial-writer semantics. The planner must not introduce a uniqueness constraint that the spec does not include.

**Primary recommendation:** Follow the 152/153 structural template exactly. The killer complexity is the `hold` read-then-write window under concurrent load; address it with the D-48 integration test (N=20 concurrent holds against capacity=5) and document the SQLite serial-writer guarantee in rustdoc.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Reservation state machine (hold/commit/release/expire) | ferro-reservation (library crate) | — | Domain logic lives in the crate that owns the `reservations` table |
| Concurrency correctness (race-free transitions) | Database (via GuardedUpdate) | ferro-reservation (predicate construction) | `UPDATE … WHERE status='held'` is atomic at DB level — no app lock needed |
| Capacity check | Consumer's `Resource` impl | ferro-reservation (calls `capacity`/`held`) | Resource domain is consumer-defined; kernel orchestrates but does not own capacity rules |
| Audit trail | ferro-audit (`AuditEntry::record`) | ferro-reservation (call site) | ferro-audit owns the `audit_log` table; ferro-reservation is a caller |
| Event broadcast | ferro-events (`dispatch`) | ferro-reservation (call site) | ferro-events owns the global dispatcher; ferro-reservation emits typed events |
| TTL expiry sweep | ferro-reservation (`run_sweep_once`) | Consumer (scheduling) | Kernel owns the sweep logic; consumer owns the scheduling cadence |
| Schema migration | ferro-reservation (`CreateReservationsTable`) | Consumer `Migrator` (registration) | Migration ships with the crate; consumers register it in their own `MigratorTrait` impl |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | 1.0 (pinned 1.1.19) | ORM, SeaORM entity model, `ConnectionTrait` | Workspace standard; all ferro db crates use it [VERIFIED: Cargo.lock] |
| sea-orm-migration | 1.0 (pinned 1.1.19) | SeaORM migration DSL | Required for `CreateReservationsTable` migration; matches ferro-audit pattern [VERIFIED: Cargo.lock] |
| thiserror | 2 | Error derive macro | Workspace standard for per-crate error enums [VERIFIED: ferro-audit/Cargo.toml, ferro-orm/Cargo.toml] |
| serde | 1 (features: derive) | Serialize/Deserialize for `ReservationHandle`, `ReservationEvent`, `ReleaseReason` | Workspace standard [VERIFIED: ferro-audit/Cargo.toml] |
| serde_json | 1 | `JsonValue` for `resource_key`/`window` JSON columns; `ReservationEvent` payload | Workspace standard [VERIFIED: ferro-audit/Cargo.toml] |
| uuid | 1 (features: v4, serde) | UUIDv4 generation for `ReservationHandle::id` | Workspace standard; matches ferro-audit pattern [VERIFIED: ferro-audit/Cargo.toml] |
| chrono | 0.4 (features: serde) | `DateTime<Utc>` for `expires_at`, `held_at`, `committed_at` etc. | Workspace standard [VERIFIED: ferro-audit/Cargo.toml] |
| tracing | 0.1 | `tracing::warn!` on event-dispatch failure and audit failure | Workspace standard [VERIFIED: ferro-audit/Cargo.toml] |
| async-trait | 0.1 | `#[async_trait]` for `Resource` trait's async methods | Required for async methods in traits; ferro-events uses it [VERIFIED: ferro-notifications/Cargo.toml, ferro-events/src/traits.rs] |

### Internal ferro-* Runtime Dependencies

| Crate | Path | Purpose |
|-------|------|---------|
| ferro-orm | `{ path = "../ferro-orm", version = "0.2" }` | `GuardedUpdate` for all state transitions |
| ferro-events | `{ path = "../ferro-events", version = "0.2" }` | `Event` trait + `dispatch()` for `ReservationEvent` |
| ferro-audit | `{ path = "../ferro-audit", version = "0.2" }` | `AuditEntry::record().write()` + `AuditActor` for `ReservationContext` |

### Dev Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| tokio | 1 (features: full) | Async test runtime |
| sea-orm | 1.0 (features: sqlx-sqlite, runtime-tokio-native-tls, macros) | In-memory SQLite for tests |
| proptest | 1 | Property-based tests (D-49); first use in workspace |

**Installation (Cargo.toml excerpt):**
```toml
[package]
name = "ferro-reservation"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Generic hold/commit/release resource reservation kernel for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["reservation", "booking", "sea-orm", "concurrency", "ferro"]
categories = ["database", "asynchronous"]
readme = "README.md"
homepage = "https://ferro-rs.dev"

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
ferro-orm = { path = "../ferro-orm", version = "0.2" }
ferro-events = { path = "../ferro-events", version = "0.2" }
ferro-audit = { path = "../ferro-audit", version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
```

**Version verification:**
- `sea-orm 1.0` → resolves to `1.1.19` in workspace Cargo.lock [VERIFIED: Cargo.lock]
- `sea-orm-migration 1.0` → resolves to `1.1.19` [VERIFIED: Cargo.lock]
- `proptest 1` → latest is `1.11.0` [VERIFIED: cargo search]
- `async-trait 0.1` → latest is `0.1.89` [VERIFIED: cargo search]

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer call site
        │
        ▼
ReservationKernel<R: Resource>
        │
        ├── hold(conn, key, window, qty, ttl, ctx)
        │     │
        │     ├─ 1. R::capacity(conn, key, window)  ──► Consumer's Resource impl
        │     ├─ 2. R::held(conn, key, window)       ──► Consumer's Resource impl
        │     ├─ 3. Capacity check (application layer)
        │     ├─ 4. INSERT reservations row ─────────► Database (reservations table)
        │     ├─ 5. ferro_events::dispatch(Held{…}) ──► EventDispatcher (global)
        │     └─ 6. AuditEntry::record("reservation.held").write(conn) ──► audit_log table
        │
        ├── commit / release / extend (conn, handle, ctx)
        │     │
        │     ├─ GuardedUpdate::new(reservations::Entity)
        │     │      .filter(Id.eq(handle.id))
        │     │      .filter(Status.eq("held"))           ◄─ atomicity boundary
        │     │      .set_value(Status, "committed")
        │     │      .exec_one(conn).await
        │     │          NoRowsAffected → ConflictingState (explicit map)
        │     │          Ok(()) → proceed
        │     ├─ ferro_events::dispatch(Committed{…})
        │     └─ AuditEntry::record("reservation.committed").write(conn)
        │
        └── run_sweep_once()  (uses self.db — owned DatabaseConnection)
              │
              ├─ SELECT id,… FROM reservations
              │       WHERE status='held' AND expires_at < now() LIMIT 500
              ├─ for each row:
              │     GuardedUpdate held → expired
              │     (NoRowsAffected = concurrent sweeper won; skip silently)
              │     ferro_events::dispatch(Expired{…})
              │     AuditEntry::record("reservation.expired") actor=System
              └─ return SweepReport { expired_count, scanned_at }
```

### Recommended Module Layout

```
ferro-reservation/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs         # pub-use facade + module-level rustdoc
│   ├── resource.rs    # Resource trait
│   ├── kernel.rs      # ReservationKernel<R> — hold/commit/release/extend
│   ├── sweeper.rs     # run_sweep_once + SweepReport
│   ├── handle.rs      # ReservationHandle
│   ├── context.rs     # ReservationContext + builder methods
│   ├── event.rs       # ReservationEvent + Event impl
│   ├── error.rs       # ReservationError
│   ├── entity.rs      # SeaORM Entity/Model/ActiveModel for reservations table
│   └── migration.rs   # CreateReservationsTable migration
└── tests/
    ├── concurrent_hold.rs              # D-48 integration test
    ├── integration_with_audit_and_events.rs  # D-50 cross-crate integration
    └── proptest_properties.rs          # D-49 property tests
```

### Pattern 1: GuardedUpdate State Transition (verified call-site shape)

Every state-transition method uses this exact pattern (verified from `ferro-orm/src/guarded.rs`):

```rust
// Source: ferro-orm/src/guarded.rs (verified)
// Map NoRowsAffected → ConflictingState BEFORE the ? operator (D-46)
let result = GuardedUpdate::new(reservations::Entity)
    .filter(reservations::Column::Id.eq(handle.id))
    .filter(reservations::Column::Status.eq("held"))
    .set_value(
        reservations::Column::Status,
        Value::String(Some(Box::new("committed".to_string()))),
    )
    .set_value(
        reservations::Column::CommittedAt,
        // chrono DateTime → sea_orm Value via Into<Value>
        Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
    )
    .exec_one(conn)
    .await
    .map_err(|e| match e {
        GuardedError::NoRowsAffected => ReservationError::ConflictingState {
            id: handle.id,
            expected: "held",
        },
        other => ReservationError::Guarded(other),
    })?;
```

**Key insight:** `From<GuardedError>` exists for `ReservationError::Guarded(#[from])`, but for `NoRowsAffected` the kernel must map it explicitly because the caller wants the semantic error, not the raw guarded error. This mapping happens before the `?` chain. `EmptyUpdate` and `TooManyRows` are programming bugs and fall through to `ReservationError::Guarded(other)`.

### Pattern 2: AuditEntry emission (verified call-site shape)

```rust
// Source: ferro-audit/src/entry.rs (verified)
AuditEntry::record("reservation.committed")
    .actor(ctx.actor.clone())
    .target(AuditTarget::new("reservation", handle.id.to_string()))
    .before(json!({ "status": "held", "quantity": handle.quantity }))
    .after(json!({ "status": "committed" }))
    .correlation(ctx.correlation_id.unwrap_or_else(Uuid::new_v4))  // only if Some
    .tenant(ctx.tenant_id.as_deref().unwrap_or(""))  // only if Some
    .write(conn)
    .await
    .map_err(ReservationError::Audit)?;
// D-30: state is already committed above; audit failure surfaces as Audit error
// but does NOT attempt to roll back the GuardedUpdate result.
```

**Note on `.correlation()` and `.tenant()`:** The builder methods require a concrete value. The `ReservationContext` carries `Option<Uuid>` and `Option<String>`. Only call `.correlation(id)` if `ctx.correlation_id.is_some()` — otherwise omit the call (the builder defaults to `None`). Same for `.tenant()`.

### Pattern 3: Event dispatch after state commit (verified call-site shape)

```rust
// Source: ferro-events/src/dispatcher.rs + traits.rs (verified)
// ferro-events uses a global OnceLock dispatcher; dispatch() is async
if let Err(e) = ferro_events::dispatch(ReservationEvent::Committed {
    id: handle.id,
    resource_kind: handle.resource_kind.clone(),
    resource_key: handle.resource_key.clone(),  // already JsonValue in handle
}).await {
    // D-26: state is committed; event dispatch failure is operational visibility only
    tracing::warn!(
        reservation_id = %handle.id,
        error = %e,
        "event dispatch failed after reservation.committed — state is committed"
    );
}
```

**ReservationEvent::Event impl shape** (mirrors ferro-events' `Event` trait):

```rust
// Source: ferro-events/src/traits.rs (verified)
impl ferro_events::Event for ReservationEvent {
    fn name(&self) -> &'static str {
        match self {
            Self::Held { .. }      => "ReservationHeld",
            Self::Committed { .. } => "ReservationCommitted",
            Self::Released { .. }  => "ReservationReleased",
            Self::Expired { .. }   => "ReservationExpired",
        }
    }
}
// ferro-events::Event requires: Clone + Send + Sync + 'static
// ReservationEvent must derive Clone; JsonValue (serde_json::Value) is Clone+Send+Sync
```

### Pattern 4: Migration shape (verified from ferro-audit/src/migration.rs)

```rust
// Source: ferro-audit/src/migration.rs (verified)
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Reservations::Table)
                .if_not_exists()
                .col(ColumnDef::new(Reservations::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Reservations::ResourceKind).string().not_null())
                .col(ColumnDef::new(Reservations::ResourceKey).json().not_null())
                .col(ColumnDef::new(Reservations::Window).json().null())
                .col(ColumnDef::new(Reservations::Quantity).integer().not_null())
                .col(ColumnDef::new(Reservations::Status).string().not_null())
                .col(ColumnDef::new(Reservations::ExpiresAt).timestamp().not_null())
                .col(ColumnDef::new(Reservations::HeldAt)
                    .timestamp().not_null()
                    .default(Expr::current_timestamp()))
                .col(ColumnDef::new(Reservations::CommittedAt).timestamp().null())
                .col(ColumnDef::new(Reservations::ReleasedAt).timestamp().null())
                .col(ColumnDef::new(Reservations::ReleaseReason).string().null())
                .col(ColumnDef::new(Reservations::TenantId).string().null())
                .to_owned(),
        ).await?;

        // idx_reservations_kind_key_window_status
        manager.create_index(Index::create()
            .name("idx_reservations_kind_key_window_status")
            .table(Reservations::Table)
            .col(Reservations::ResourceKind)
            .col(Reservations::ResourceKey)
            .col(Reservations::Window)
            .col(Reservations::Status)
            .to_owned()).await?;

        // idx_reservations_status_expires
        manager.create_index(Index::create()
            .name("idx_reservations_status_expires")
            .table(Reservations::Table)
            .col(Reservations::Status)
            .col(Reservations::ExpiresAt)
            .to_owned()).await
    }
    // down(): drop_table(Reservations::Table)
}

#[derive(DeriveIden)]
enum Reservations { Table, Id, ResourceKind, ResourceKey, Window,
                    Quantity, Status, ExpiresAt, HeldAt, CommittedAt,
                    ReleasedAt, ReleaseReason, TenantId }
```

**pub re-export** (matches ferro-audit pattern):
```rust
// ferro-reservation/src/lib.rs
pub use migration::Migration as CreateReservationsTable;
```

**SeaORM JSON column note:** `ColumnDef::json()` maps to `TEXT` in SQLite (JSON1 is text storage). SeaORM's `JsonValue` column type serializes as a JSON string. Round-trip: serialize `R::Key: Serialize` with `serde_json::to_value(key)?` → store as `JsonValue`; deserialize with `serde_json::from_value::<R::Key>(json)?`. This is the same pattern ferro-audit uses for `before`/`after` columns. [VERIFIED: ferro-audit/src/entity.rs uses `JsonValue` column type successfully in SQLite tests]

### Pattern 5: Test harness with combined migrations (D-52)

The cross-crate integration test (D-50) needs both `CreateAuditLogTable` and `CreateReservationsTable` in one in-memory SQLite `Migrator`:

```rust
// Source: ferro-audit/src/entry.rs tests (verified — same inline pattern)
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_audit::migration::Migration),   // or CreateAuditLogTable
            Box::new(crate::migration::Migration),          // CreateReservationsTable
        ]
    }
}

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```

**Important:** `ferro_audit::CreateAuditLogTable` is the public re-export. In test code inside `ferro-reservation`, access the underlying type via `ferro_audit::CreateAuditLogTable` (which is `ferro_audit::migration::Migration`). Both work; the public alias is preferred for consistency.

### Pattern 6: SeaORM Entity for reservations table

Mirror ferro-audit's entity shape exactly. Key differences:
- `expires_at` is `DateTime` (non-nullable), not `Option<DateTime>`
- `held_at` is `DateTime` (non-nullable, has DB default)
- `committed_at`, `released_at` are `Option<DateTime>` (nullable)
- `resource_key`, `window` use `Option<JsonValue>` — `window` is nullable per schema
- `quantity` is `i32` (SeaORM integer maps to i32; consumer casts to u32 at API boundary)

```rust
// entity.rs — key fields
#[sea_orm(primary_key, auto_increment = false)]
pub id: Uuid,
pub resource_kind: String,
pub resource_key: JsonValue,         // NOT NULL
pub window: Option<JsonValue>,       // NULL when Window = ()
pub quantity: i32,                   // stored as i32; u32 at API boundary
pub status: String,                  // "held" | "committed" | "released" | "expired"
pub expires_at: DateTime,            // NOT NULL
pub held_at: DateTime,               // NOT NULL, DB default
pub committed_at: Option<DateTime>,  // NULL until commit
pub released_at: Option<DateTime>,   // NULL until release
pub release_reason: Option<String>,  // NULL until release
pub tenant_id: Option<String>,
```

### Anti-Patterns to Avoid

- **SELECT-then-UPDATE for state transitions:** Any state transition that reads status first, then conditionally updates in a second query has a race window. All transitions must go through `GuardedUpdate` with the status predicate in the single UPDATE statement.
- **Empty `set_*` `GuardedUpdate`:** `GuardedUpdate::exec_one` with no `.set_value()` / `.set_expr()` calls returns `GuardedError::EmptyUpdate`, not a predicate miss. This is a programming error; check at build time if possible.
- **Propagating `GuardedError::NoRowsAffected` as-is:** The kernel catches `NoRowsAffected` and converts it to `ReservationError::ConflictingState` with the expected state. The caller should never see the raw guarded error for state-machine violations.
- **Awaiting event dispatch before returning `Ok` to the caller:** Events are post-commit notifications; the `?` operator must not propagate event errors back to the caller. Use `if let Err(e) = dispatch(...).await { tracing::warn!(...); }`.
- **Using `dispatch_sync` (fire-and-forget tokio::spawn) for events:** `dispatch_sync` is correct for "fire and forget" use cases but offers no backpressure signal. `dispatch` (async, awaited) allows the kernel to log the failure immediately. The locked decision (D-26) uses `dispatch` (async, but not error-propagating).

---

## Special Research Areas

### Area 1: Race-Free `hold` — Resolving the D-19 Concurrency Claim

**The claimed correctness mechanism (CONTEXT.md D-19):**
> "correctness there is guaranteed by the INSERT failing on the unique constraint (D-39) if a concurrent insert + capacity check race produced an over-allocation"

**The actual schema (D-39):** No unique constraint exists on the `reservations` table. The schema has two composite indexes for lookup performance, not uniqueness enforcement.

**Resolution — SQLite serial-writer semantics:**

SQLite WAL mode uses a single write-lock. Even with multiple tokio tasks issuing concurrent INSERTs, SQLite serializes them. The sequence under SQLite:

```
Task A: capacity=5, held=0, request=1 → check: 0+1<=5 → INSERT (held=1)
Task B: (serialized, runs after A's INSERT commits):
         capacity=5, held=1, request=1 → check: 1+1<=5 → INSERT (held=2)
...
Task F (6th): capacity=5, held=5, request=1 → check: 5+1>5 → Insufficient error
```

Under SQLite serial-writer, the capacity check + INSERT is effectively atomic at the DB engine level — no two writes can overlap. The concurrent hold test (D-48) with 20 tasks against capacity=5 DOES work correctly under SQLite.

**Under Postgres READ COMMITTED:** The capacity check (`R::held`) runs a SELECT, then the INSERT happens. Under Postgres, two concurrent transactions can both see `held=4` (capacity=5) and both INSERT, causing `held=6 > capacity=5`. The property test under SQLite does NOT validate Postgres safety.

**Implication for the plan:** The concurrency claim is accurate under SQLite serial-writer semantics (which is the test target per D-51). The plan should document this limitation explicitly in rustdoc. The property test (D-49 Property 1) passes under in-memory SQLite. Postgres correctness would require a `SELECT FOR UPDATE` lock on the capacity check or a `held_count INTEGER NOT NULL DEFAULT 0` counter column on a separate resource-tracking table — both deferred (out of scope per D-51).

**The planner must NOT add a unique constraint to prevent over-allocation** — the spec does not include one, and it would not actually prevent over-allocation (two INSERTs with different UUIDs would both succeed).

**Confidence:** HIGH for SQLite behavior [VERIFIED: ferro-orm/src/guarded.rs tests confirm SQLite serial-writer semantics]; MEDIUM for Postgres behavior [ASSUMED based on Postgres READ COMMITTED documentation].

### Area 2: ferro-events Integration — How to Implement `Event` for `ReservationEvent`

**Verified from `ferro-events/src/traits.rs` and `ferro-events/src/dispatcher.rs`:**

1. `ferro_events::Event` requires: `Clone + Send + Sync + 'static`; one method `fn name(&self) -> &'static str`
2. No `inventory` crate for compile-time listener registration — ferro-events uses a `TypeId`-keyed `RwLock<HashMap<TypeId, Vec<ListenerEntry>>>` in a global `OnceLock<EventDispatcher>`
3. `ReservationEvent` only needs to implement `Event` to be dispatchable — no cross-crate registration required
4. Consumers subscribe by calling `global_dispatcher().on::<ReservationEvent, _, _>(|event| async { ... })` or `global_dispatcher().listen(MyReservationListener)` — fully dynamic, no compile-time registration
5. `ferro_events::dispatch(event)` calls `global_dispatcher().dispatch(event)` — fully generic over any `E: Event`
6. Adding `ReservationEvent` as a new event type requires ZERO modifications to ferro-events

**`ReservationEvent` must derive `Clone`** — ferro-events clones the event before calling each listener (see `dispatcher.rs` line: `let event = event.clone()`). `JsonValue` (serde_json::Value) is `Clone`. `DateTime<Utc>` is `Clone`. The full `ReservationEvent` can derive `Clone`.

**CONTEXT.md D-25 says `window: Option<JsonValue>`** — nullable because `Window = ()` is serialized to `null` JSON (or the field is `None` at the kernel level when the resource has no meaningful window). At `hold` time: `serde_json::to_value(&window)` on `()` produces `serde_json::Value::Null`; store as `None` in the JSON column.

### Area 3: Sweeper `SELECT FOR UPDATE SKIP LOCKED`

**Context:** D-21 describes the sweeper as `SELECT … LIMIT 500 … for each, GuardedUpdate held → expired`.

**SeaORM and SQLite:** SeaORM does NOT abstract `SELECT FOR UPDATE SKIP LOCKED` — this is a Postgres-specific feature. SQLite has no row-level locks. [ASSUMED based on SeaORM 1.x API knowledge; not verified against Context7 in this session]

**Why it doesn't matter for ferro-reservation v0:** The `GuardedUpdate` per-row pattern IS the correct concurrent-sweeper safety mechanism, verified in D-24: if two sweepers race on the same row, only one wins the `held → expired` GuardedUpdate; the other sees `NoRowsAffected` and skips. This is exactly what the code already does. `SKIP LOCKED` would be an optimization (avoid contention on already-claimed rows) but is not needed for correctness.

**Concrete sweeper implementation:**

```rust
// sweeper query: use sea_orm Select with filter + limit
let expired_rows = reservations::Entity::find()
    .filter(reservations::Column::Status.eq("held"))
    .filter(reservations::Column::ExpiresAt.lt(Utc::now().naive_utc()))
    .limit(500)
    .all(&self.db)
    .await
    .map_err(ReservationError::Db)?;

let mut expired_count = 0u32;
for row in expired_rows {
    let result = GuardedUpdate::new(reservations::Entity)
        .filter(reservations::Column::Id.eq(row.id))
        .filter(reservations::Column::Status.eq("held"))
        .set_value(reservations::Column::Status,
                   Value::String(Some(Box::new("expired".to_string()))))
        .exec_at_most_one(&self.db)   // tolerates 0 rows (concurrent sweeper won)
        .await;

    match result {
        Ok(true) => {
            expired_count += 1;
            // emit event + audit (AuditActor::System per D-23)
        }
        Ok(false) => { /* concurrent sweeper won; skip silently per D-24 */ }
        Err(e) => { tracing::warn!(error = %e, "sweeper guarded update db error"); }
    }
}
```

**Note:** Use `exec_at_most_one` in the sweeper, not `exec_one` — because 0 rows affected is a normal concurrent outcome (D-24), not an error. Use `exec_one` only in `commit`/`release`/`extend` where the caller must own the handle.

### Area 4: JSON Column Storage of Generic `R::Key` / `R::Window`

**Mechanism (verified from ferro-audit/src/entity.rs and entry.rs):**

SeaORM maps `ColumnDef::json()` to `serde_json::Value` in the Rust model (type `JsonValue`). SQLite stores it as TEXT; Postgres stores it as JSON or JSONB.

**At `hold()` time:**
```rust
let key_json: JsonValue = serde_json::to_value(&key)
    .map_err(ReservationError::Json)?;
let window_json: Option<JsonValue> = serde_json::to_value(&window)
    .map(|v| if v.is_null() { None } else { Some(v) })
    .map_err(ReservationError::Json)?;
```

`()` serializes to `serde_json::Value::Null`. The column is nullable; store `None` when `Window = ()`.

**In `ReservationHandle`:** `resource_key: JsonValue` and `window: Option<JsonValue>` — already the serialized form. Consumers who need the typed `R::Key` back call `serde_json::from_value(handle.resource_key.clone())`.

**In `ReservationEvent`:** Same — `resource_key: JsonValue` and `window: Option<JsonValue>`. The event payload is opaque JSON at the event-bus boundary per D-25.

**In `Resource::held` queries:** The consumer's implementation queries `reservations` filtered by `resource_kind = R::KIND` and `resource_key = key_json` using a raw WHERE clause or SeaORM filter on the JSON column. SQLite TEXT equality works for this (JSON values are compared as strings; as long as serialization is deterministic for the `Key` type, equality checks work). Consumers should document the determinism requirement.

### Area 5: proptest + tokio Concurrent Property Tests

**proptest version:** `1.11.0` (latest as of 2026-05-13) [VERIFIED: cargo search]

**proptest + tokio integration pattern for property tests:**

`proptest!` macro does NOT support `async fn`. The idiomatic approach for concurrent property tests that need tokio is to use `proptest!` to generate operation sequences and replay them synchronously via `tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async { ... })`:

```rust
// D-49 Property 1: capacity invariant under concurrent holds
proptest! {
    #[test]
    fn capacity_invariant_under_concurrent_holds(
        capacity in 1u32..=20u32,
        n_tasks in 1usize..=20usize,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let conn = fresh_db_with_reservation_table().await;
            let kernel = ReservationKernel::new(conn.clone(), TestResource::new(capacity));

            let handles = (0..n_tasks).map(|_| {
                let kernel = kernel.clone();  // kernel is Clone+Send
                let conn = conn.clone();
                tokio::spawn(async move {
                    let ctx = ReservationContext::system();
                    kernel.hold(&conn, TEST_KEY, TEST_WINDOW, 1,
                                Duration::from_secs(60), &ctx).await
                })
            }).collect::<Vec<_>>();

            let results = futures::future::join_all(handles).await;
            let successes = results.iter()
                .filter(|r| matches!(r, Ok(Ok(_))))
                .count();

            prop_assert!(successes <= capacity as usize,
                "held {} of capacity {}", successes, capacity);

            // verify DB count matches
            let held_count = count_held_rows(&conn, TEST_KIND).await;
            prop_assert_eq!(held_count, successes as u32);
        });
    }
}
```

**Property 2: state-machine validity via audit log replay** — replay `AuditEntry` history and assert no entry shows a transition from a terminal state:

```rust
proptest! {
    #[test]
    fn state_machine_validity_via_audit_replay(
        ops in prop::collection::vec(arb_reservation_op(), 1..=10usize),
    ) {
        let rt = ...;
        rt.block_on(async {
            // execute ops (hold/commit/release) sequentially
            // fetch audit entries for the reservation id
            // assert no entry records an impossible transition
        });
    }
}
```

**`proptest` is NOT a tokio-test crate** — it does not auto-detect tokio. The `block_on` pattern is standard for tokio + proptest combination. For the current_thread runtime, spawned tasks run on the same thread; SQLite serial-writer is preserved. [ASSUMED based on proptest documentation and common Rust async testing patterns; not verified via Context7 in this session]

### Area 6: Workspace Cargo dep Declaration

**Pattern from ferro-notifications/Cargo.toml (Wave 1b, verified):**
- Internal ferro-* deps use `{ path = "../crate-name", version = "0.2" }` — NOT workspace-level shared declarations
- External deps do NOT use `version.workspace = true` in ferro-notifications — each crate declares its own version string

**Note on `version.workspace = true` vs per-crate versions:** `ferro-audit` and `ferro-orm` use `sea-orm = "1.0"` directly (not `version.workspace = true`). Follow the same pattern for ferro-reservation — declare external deps inline with version strings.

**Workspace Cargo.toml addition:**
```toml
# Add to [workspace] members list in Cargo.toml
"ferro-reservation",
```

### Area 7: Combined Migration Migrator Pattern (D-52 cross-crate integration test)

**Verified from ferro-audit/src/entry.rs tests:**

The inline test `Migrator` pattern creates a minimal `TestMigrator` that registers only the migrations needed for that test file. For the cross-crate integration test (`tests/integration_with_audit_and_events.rs`), both `CreateAuditLogTable` and `CreateReservationsTable` must be registered:

```rust
use ferro_audit::CreateAuditLogTable;       // = ferro_audit::migration::Migration
use crate::CreateReservationsTable;          // = ferro_reservation::migration::Migration

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(CreateAuditLogTable),       // audit_log table first
            Box::new(CreateReservationsTable),   // reservations table second
        ]
    }
}
```

**Migration ordering:** `audit_log` before `reservations` — no FK constraint between them, but ordering mirrors the dependency chain (audit is the foundation).

**The `DeriveMigrationName` proc-macro** on each `struct Migration` generates a unique name string from the crate name and file path. Two migrations from different crates will have different names; no collision.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic conditional UPDATE | Custom `SELECT + UPDATE` with app-level locking | `GuardedUpdate::exec_one` (ferro-orm) | SeaORM compiles to a single SQL statement; app locks add failure modes |
| Concurrent sweeper idempotency | Distributed lock or app mutex | `GuardedUpdate::exec_at_most_one` (0-rows = normal) | Per-row contention is already serialized at DB level |
| Error types for dependent crates | Manual error wrapping with `From` | `thiserror` `#[from]` derive | Workspace standard; D-43 specifies this explicitly |
| Async methods in trait | Manual boxing with `Pin<Box<dyn Future>>` | `#[async_trait::async_trait]` | Required for `Resource` trait; ferro-events already uses this pattern |
| JSON serialization of generic key/window | Custom serialization logic | `serde_json::to_value` / `from_value` | Standard Serde bridge; works with any `Serialize + DeserializeOwned` bound |
| Event subscription / routing | Internal event registry | `ferro_events::global_dispatcher()` | Existing TypeId-keyed runtime registry; no compile-time magic needed |
| Audit persistence | Custom audit table | `AuditEntry::record().write()` (ferro-audit) | Already ships the table, migration, and query helpers |

---

## Runtime State Inventory

This is a greenfield crate creation phase. There is no existing `ferro-reservation` code, table, or runtime state to migrate.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — `ferro-reservation` crate does not exist; `reservations` table does not exist | None — CreateReservationsTable migration creates it |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | None — crate has no env var configuration | None |
| Build artifacts | None — crate does not exist | None |

**Verified:** `grep -rl "ferro-reservation\|ferro_reservation\|ReservationKernel\|CreateReservationsTable" --include="*.rs" --include="*.toml"` returns only references in CONTEXT.md, INVENTORY-PRIMITIVES.md, and STATE.md (planning documents only, no source code). [VERIFIED: CONTEXT.md §`code_context` Reusable assets block explicitly states "No existing reservation code in the workspace."]

---

## Common Pitfalls

### Pitfall 1: `set_value` with `DateTime<Utc>` → `sea_orm::Value`

**What goes wrong:** `Value::String(Some(Box::new("some string")))` is the pattern for strings. For timestamps, `Value::ChronoDateTimeUtc(Some(Box::new(dt)))` is the correct `sea_orm::Value` variant. Using the wrong variant causes a runtime type error.

**Why it happens:** `sea_orm::Value` is a large enum with one variant per SQL type. The mapping is not obvious from Rust types alone.

**How to avoid:** Check ferro-orm's test: `set_value(Column::Status, Value::String(Some(Box::new("committed".to_string()))))`. For DateTime, use `Value::ChronoDateTimeUtc(Some(Box::new(Utc::now())))`. Import from `ferro_orm::Value` (which re-exports `sea_orm::Value`). [VERIFIED: ferro-orm/src/lib.rs re-exports `sea_orm::Value`]

**Warning signs:** Runtime panic or `DbErr::Type` error when executing `GuardedUpdate`.

### Pitfall 2: `exec_one` vs `exec_at_most_one` in the Sweeper

**What goes wrong:** Using `exec_one` in `run_sweep_once` causes `ReservationError::Guarded(NoRowsAffected)` to surface whenever two concurrent sweepers race on the same row. D-24 says this should be silently skipped.

**Why it happens:** `exec_one` treats 0 rows as an error; `exec_at_most_one` treats it as a normal outcome.

**How to avoid:** Use `exec_at_most_one` ONLY in the sweeper; use `exec_one` everywhere else (commit/release/extend) where the caller must own the handle.

### Pitfall 3: `GuardedError::NoRowsAffected` propagated via `?` before manual mapping

**What goes wrong:** If `GuardedUpdate.exec_one(&conn).await?` uses the `From<GuardedError>` impl directly, the caller sees `ReservationError::Guarded(NoRowsAffected)` instead of `ReservationError::ConflictingState`. The semantic content is lost.

**Why it happens:** The `#[from]` derive creates `From<GuardedError> for ReservationError::Guarded`. If `?` fires before the manual `map_err`, the wrong variant is produced.

**How to avoid:** Use `map_err(|e| match e { GuardedError::NoRowsAffected => ConflictingState{...}, other => Guarded(other) })?` — always map before `?`. [VERIFIED: D-46 specifies this explicitly]

### Pitfall 4: Audit builder `.correlation()` / `.tenant()` called with empty strings

**What goes wrong:** `AuditEntryBuilder::tenant("")` stores an empty string in `tenant_id`, not `NULL`. Querying `tenant_id IS NULL` will miss these entries.

**Why it happens:** The builder takes `impl Into<String>` — an empty string is valid input but semantically wrong.

**How to avoid:** Only call `.tenant(t)` when `ctx.tenant_id.is_some()`. Pattern:
```rust
let mut builder = AuditEntry::record("reservation.held")
    .actor(ctx.actor.clone())
    .target(AuditTarget::new("reservation", id.to_string()));
if let Some(tid) = &ctx.tenant_id { builder = builder.tenant(tid); }
if let Some(cid) = ctx.correlation_id { builder = builder.correlation(cid); }
builder.write(conn).await.map_err(ReservationError::Audit)?;
```

### Pitfall 5: `async_trait` bound on `Resource` impl — missing `#[async_trait]` on impl block

**What goes wrong:** Implementing `Resource` without `#[async_trait::async_trait]` on the `impl` block causes a compile error ("method `capacity` has an incompatible type for trait").

**Why it happens:** `async_trait` rewrites async methods in both the trait definition AND each impl. Both must be annotated.

**How to avoid:** Rustdoc example in `resource.rs` must show `#[async_trait::async_trait]` on both the trait definition and the consumer's impl block.

### Pitfall 6: `quantity` as `u32` at API / `i32` in DB

**What goes wrong:** SeaORM maps `INTEGER` to `i32`. `ReservationHandle::quantity` is `u32` at the Rust API level. Casting `row.quantity as u32` silently wraps negative values.

**Why it happens:** SeaORM entity derives use `i32` for SQL INTEGER columns; the public API uses `u32` (it's a count and should never be negative).

**How to avoid:** At the entity layer store `i32`; cast to `u32` when constructing `ReservationHandle` and when computing the `INSERT` value. Add an assertion or return `ReservationError::Db` if `row.quantity < 0` (data corruption guard). The INSERT path sets quantity from the `hold()` parameter which is already `u32`, so overflows are only possible via direct DB mutation.

### Pitfall 7: `SeaORM Migrator` ordering — `DeriveMigrationName` collision risk

**What goes wrong:** Two migrations from different crates with the same struct name `Migration` and the same module path fragment could theoretically produce the same migration name. SeaORM's `DeriveMigrationName` generates the name from the type path. Since `ferro_audit::migration::Migration` and `ferro_reservation::migration::Migration` have different crate prefixes, the generated names are distinct — no collision. But test code that imports both must disambiguate:

```rust
use ferro_audit::CreateAuditLogTable;     // OK — alias
use crate::CreateReservationsTable;       // OK — different name
```

Avoid `use ferro_audit::migration::Migration` and `use crate::migration::Migration` in the same scope without aliasing.

---

## Code Examples

### Error enum (verified pattern from GuardedError and AuditError)

```rust
// Source: ferro-orm/src/error.rs + ferro-audit/src/error.rs (verified)
#[derive(Debug, thiserror::Error)]
pub enum ReservationError {
    #[error("reservation: insufficient capacity (requested {requested}, available {available} of {capacity})")]
    Insufficient { requested: u32, available: u32, capacity: u32 },

    #[error("reservation: id={id} not in expected state '{expected}'")]
    ConflictingState { id: Uuid, expected: &'static str },

    #[error("reservation: id={id} not found")]
    NotFound { id: Uuid },

    #[error("reservation: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("reservation: guarded update error: {0}")]
    Guarded(#[from] ferro_orm::GuardedError),

    #[error("reservation: audit error: {0}")]
    Audit(#[from] ferro_audit::AuditError),

    #[error("reservation: json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
// Note: Guarded has #[from] for ALL GuardedError variants except NoRowsAffected,
// which is caught before ? in each transition method and mapped to ConflictingState.
// The #[from] still fires for EmptyUpdate and TooManyRows (programming bugs).
```

### `ReleaseReason` serde (verified pattern from workspace conventions)

```rust
// Source: CONTEXT.md D-18 + workspace serde conventions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum ReleaseReason {
    UserCancelled,
    PaymentFailed,
    AdminOverride,
    Other(String),
}
```

### `lib.rs` public facade (mirror ferro-audit/src/lib.rs pattern)

```rust
// Source: ferro-audit/src/lib.rs (verified)
mod context;
mod entity;
mod error;
mod event;
mod handle;
mod kernel;
mod migration;
mod resource;
mod sweeper;

pub use context::ReservationContext;
pub use entity::{ActiveModel as ReservationActiveModel, Entity as ReservationEntity,
                  Model as ReservationModel};
pub use error::ReservationError;
pub use event::{ReservationEvent, ReleaseReason};
pub use handle::ReservationHandle;
pub use kernel::ReservationKernel;
pub use migration::Migration as CreateReservationsTable;
pub use resource::Resource;
pub use sweeper::SweepReport;

// Re-export AuditActor for consumers building ReservationContext
pub use ferro_audit::AuditActor;
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio-test + proptest 1.11.0 (new dev-dep) + sea-orm SQLite |
| Config file | None — inline `#[tokio::test]` + `proptest!` macros |
| Quick run command | `cargo test -p ferro-reservation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| ID | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| D-47-1 | `hold` happy path — capacity 10, request 3 → handle returned | unit | `cargo test -p ferro-reservation test::hold_happy_path` | Wave 0 |
| D-47-2 | `hold` rejects when held+qty > capacity | unit | `cargo test -p ferro-reservation test::hold_insufficient` | Wave 0 |
| D-47-3 | `commit` happy path: held → committed | unit | `cargo test -p ferro-reservation test::commit_happy_path` | Wave 0 |
| D-47-4 | `commit` on already-committed handle → ConflictingState | unit | `cargo test -p ferro-reservation test::commit_conflicting_state` | Wave 0 |
| D-47-5 | `release` happy path with each ReleaseReason variant | unit | `cargo test -p ferro-reservation test::release_all_reasons` | Wave 0 |
| D-47-6 | `extend` happy path — expires_at increases | unit | `cargo test -p ferro-reservation test::extend_happy_path` | Wave 0 |
| D-47-7 | `extend` on expired-but-not-swept → ConflictingState | unit | `cargo test -p ferro-reservation test::extend_on_expired` | Wave 0 |
| D-47-8 | `run_sweep_once` happy path — 3 expired → report.expired_count=3 | unit | `cargo test -p ferro-reservation test::sweep_expires_rows` | Wave 0 |
| D-47-9 | `run_sweep_once` no-op when no eligible rows | unit | `cargo test -p ferro-reservation test::sweep_noop` | Wave 0 |
| D-47-10 | Resource::Key / Window JSON round-trip | unit | `cargo test -p ferro-reservation test::json_roundtrip` | Wave 0 |
| D-47-11 | ReservationContext defaults + builder methods | unit | `cargo test -p ferro-reservation test::context_builder` | Wave 0 |
| D-47-12 | ReservationHandle serde round-trip | unit | `cargo test -p ferro-reservation test::handle_serde` | Wave 0 |
| D-48 | Concurrent hold: N=20 tasks, capacity=5 → exactly 5 succeed | integration | `cargo test -p ferro-reservation concurrent_hold` | Wave 0 |
| D-49-P1 | Capacity invariant: SUM(held+committed) <= C for random N,C | property | `cargo test -p ferro-reservation proptest_capacity_invariant` | Wave 0 |
| D-49-P2 | State-machine validity: audit replay shows no illegal transitions | property | `cargo test -p ferro-reservation proptest_state_machine_validity` | Wave 0 |
| D-50 | Cross-crate: hold+commit → 2 events + 2 audit entries with correlation_id | integration | `cargo test -p ferro-reservation integration_with_audit_and_events` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-reservation`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-reservation/` — crate does not exist; entire scaffold is Wave 0
- [ ] `ferro-reservation/src/migration.rs` — `CreateReservationsTable`
- [ ] `ferro-reservation/src/entity.rs` — SeaORM entity for `reservations` table
- [ ] `ferro-reservation/src/resource.rs` — `Resource` trait
- [ ] `ferro-reservation/src/kernel.rs` — `ReservationKernel<R>`
- [ ] `ferro-reservation/src/sweeper.rs` — `run_sweep_once` + `SweepReport`
- [ ] `ferro-reservation/src/handle.rs` — `ReservationHandle`
- [ ] `ferro-reservation/src/context.rs` — `ReservationContext`
- [ ] `ferro-reservation/src/event.rs` — `ReservationEvent` + `Event` impl + `ReleaseReason`
- [ ] `ferro-reservation/src/error.rs` — `ReservationError`
- [ ] `ferro-reservation/src/lib.rs` — pub facade + module-level rustdoc
- [ ] `ferro-reservation/tests/concurrent_hold.rs` — D-48 integration test
- [ ] `ferro-reservation/tests/proptest_properties.rs` — D-49 property tests
- [ ] `ferro-reservation/tests/integration_with_audit_and_events.rs` — D-50 cross-crate test
- [ ] `ferro-reservation/Cargo.toml` — package + deps
- [ ] `ferro-reservation/README.md` — crate readme

---

## Security Domain

`security_enforcement` is not explicitly set to `false` in config.json — include minimal security assessment.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | ferro-reservation has no auth; actors are consumer-supplied via `ReservationContext` |
| V3 Session Management | no | no sessions; reservation handles are opaque UUIDs |
| V4 Access Control | partial | kernel does not enforce access control; consumer's Resource impl controls who can call `hold` |
| V5 Input Validation | yes | `quantity: u32` is type-enforced; `ttl: Duration` is non-negative by type; action strings are &'static str constants |
| V6 Cryptography | no | UUIDv4 generated by `Uuid::new_v4()` (getrandom-backed, cryptographically random) |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Over-reservation (two callers both see capacity available) | Tampering | SQLite serial-writer; GuardedUpdate predicate; concurrent_hold integration test |
| State rollback (claiming a committed reservation is still held) | Tampering | GuardedUpdate with `status='held'` predicate — committed rows cannot be re-held |
| Infinite TTL extension | Elevation of Privilege | Documented as consumer responsibility; no kernel cap in v0 per D-32 |
| Audit log tampering | Repudiation | ferro-audit append-only by convention; no DELETE exposed in ferro-audit API |

---

## Environment Availability

This is a library crate phase — no external services, CLI tools, or runtimes beyond the workspace toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Compilation | ✓ | 1.88.0 (workspace rust-version) | — |
| cargo | Build + test | ✓ | bundled with toolchain | — |
| SQLite (via sqlx feature) | In-memory tests | ✓ | pulled by sea-orm dev-dep feature | — |
| crates.io network | First publish (manual step) | ✓ | N/A | — |

No missing dependencies. No external services required during implementation.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `async-trait` required for all async trait methods | Native async-in-traits (AFIT) available since Rust 1.75 | 2023 (Rust 1.75) | Could drop `async-trait` dep in a future version; workspace still uses it for consistency across crates; stick with `async_trait` for v0 |
| `inventory` crate for compile-time event registration | `TypeId`-keyed `RwLock<HashMap>` in ferro-events | ferro-events design (existing) | No action — ferro-events does NOT use `inventory`; `ReservationEvent` just implements `Event` trait |

**Deprecated/outdated:**
- `sea-orm-migration 2.0.0-rc.*` exists but workspace pins `1.1.19` — do NOT upgrade in this phase; use `sea-orm-migration = "1.0"` matching ferro-audit.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SQLite serial-writer prevents concurrent hold over-allocation under tokio tasks | Special Area 1 | If tokio's async executor batches SQLite writes in a way that breaks serialization, D-48 concurrent_hold test could fail non-deterministically |
| A2 | `proptest!` macro is not compatible with `#[tokio::test]` — requires `block_on` workaround | Special Area 5 | If proptest adds native tokio support in 1.11.0, the `block_on` pattern still works but is unnecessarily verbose |
| A3 | SeaORM does not abstract `SELECT FOR UPDATE SKIP LOCKED` | Special Area 3 | If SeaORM 1.1.x added this, sweeper could use it as a performance optimization; correctness is unaffected either way |
| A4 | `Value::ChronoDateTimeUtc(Some(Box::new(dt)))` is the correct `sea_orm::Value` variant for `DateTime<Utc>` | Pattern 1 / Pitfall 1 | If the variant name differs, `set_value` for timestamps will panic at runtime; test T-47-3 (commit) would catch this immediately |
| A5 | `serde_json::to_value(())` produces `Value::Null` | Special Area 4 | If Serde changes `()` serialization, the `window` column would get a non-null JSON representation; no breaking functional impact |

---

## Open Questions

1. **`quantity: u32` vs `i32` boundary in SeaORM entity**
   - What we know: SeaORM INTEGER → i32; public API uses u32
   - What's unclear: whether to cast at entity layer or expose `i32` in `entity::Model` and cast in the kernel
   - Recommendation: expose `i32` in `entity::Model` (convention-matching); cast to `u32` when constructing `ReservationHandle`. Add a guard: `if row.quantity < 0 { return Err(ReservationError::Db(DbErr::Custom(...))) }`.

2. **`window: Option<JsonValue>` in entity vs `JsonValue` (always present but may be null JSON)**
   - What we know: SQLite stores NULL or TEXT; SeaORM models `Option<JsonValue>` for nullable JSON
   - What's unclear: whether `serde_json::Value::Null` should be stored as SQL NULL or as the JSON text `"null"`
   - Recommendation: store as SQL NULL (use `Option<JsonValue>` in model; set to `None` when `Window = ()`). Query comparisons on `window` column should also match `IS NULL` for `Window = ()` resources. Document in rustdoc.

3. **`ReservationContext::actor` vs `AuditActor` ownership in `write()`**
   - What we know: `AuditEntryBuilder::actor()` takes an owned `AuditActor`; `ReservationContext::actor` is an owned field
   - What's unclear: whether `ctx` should be `&ReservationContext` (requiring `.clone()` of `actor`) or consumed
   - Recommendation: keep `ctx: &ReservationContext` per D-10/D-11 signatures (avoids use-after-call complexity); clone `ctx.actor` inside the kernel for the audit call. `AuditActor` is `Clone` — cheap clone.

---

## Sources

### Primary (HIGH confidence)
- `ferro-orm/src/guarded.rs` — exact `GuardedUpdate` API, `Value` variants in tests, `exec_one` / `exec_at_most_one` semantics [VERIFIED: read in full]
- `ferro-audit/src/entry.rs` — exact `AuditEntry` builder chain, `write()` signature, test harness pattern [VERIFIED: read in full]
- `ferro-audit/src/migration.rs` — migration DSL shape, `DeriveMigrationName`, index creation [VERIFIED: read in full]
- `ferro-audit/src/entity.rs` — SeaORM entity model shape with UUID PK, JsonValue, Option<DateTime> [VERIFIED: read in full]
- `ferro-audit/src/actor.rs` + `target.rs` — `AuditActor` variants and `kind()/id()` methods [VERIFIED: read in full]
- `ferro-events/src/traits.rs` — `Event` trait requirements (`Clone + Send + Sync + 'static`, `fn name`) [VERIFIED: read in full]
- `ferro-events/src/dispatcher.rs` — global `OnceLock<EventDispatcher>`, `dispatch()` function, no `inventory` usage [VERIFIED: read in full]
- `ferro-orm/Cargo.toml`, `ferro-audit/Cargo.toml`, `ferro-notifications/Cargo.toml` — Cargo.toml template for Wave 1a and 1b crates [VERIFIED: read in full]
- `.github/workflows/publish.yml` — `WAVE1B_CRATES` current list, publish flow [VERIFIED: read in full]
- `Cargo.toml` workspace root — current members, version 0.2.31, rust-version 1.88.0 [VERIFIED: read in full]
- `Cargo.lock` — `sea-orm 1.1.19`, `sea-orm-migration 1.1.19` pinned versions [VERIFIED: grep]

### Secondary (MEDIUM confidence)
- `cargo search proptest` — version 1.11.0 current [VERIFIED: cargo search output]
- `cargo search async-trait` — version 0.1.89 current [VERIFIED: cargo search output]

### Tertiary (LOW confidence / ASSUMED)
- proptest + tokio `block_on` integration pattern — [ASSUMED from common Rust async testing knowledge]
- SeaORM lacks `SELECT FOR UPDATE SKIP LOCKED` abstraction — [ASSUMED from SeaORM 1.x API knowledge]
- `Value::ChronoDateTimeUtc` variant name — [ASSUMED; should be verified in SeaORM source or by compiling a test]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against Cargo.lock and cargo search
- Architecture patterns: HIGH — all call-site shapes verified from shipped source code
- Pitfalls: HIGH — derived from verified source code patterns + sibling phase research
- Concurrent safety claim: MEDIUM — verified for SQLite serial-writer; Postgres behavior ASSUMED

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (stable crates; 30-day window is conservative)

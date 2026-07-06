# Phase 154: ferro-reservation — Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults applied to every gray area)
**Milestone:** v11.11 Resource Reservation & Live Read-Model Primitives
**Driver:** gestiscilo-it inventory monitoring field test (and v6.3 Online Checkout slot-hold)
**Killer feature (milestone):** Race-free reservations as a first-class framework primitive. Phase 154 is the *kernel itself* — Phases 152 (ferro-orm) and 153 (ferro-audit) shipped its preconditions; this phase composes them into the typed hold/commit/release/expire surface.

<domain>
## Phase Boundary

Create a new `ferro-reservation` crate inside the ferro workspace that ships a **generic, domain-neutral resource reservation kernel** with TTL expiry and event broadcast.

The crate exposes:

- `Resource` trait — consumer-implemented capacity model (`Key`, `Window`, async `capacity`, async `held`)
- `ReservationKernel<R: Resource>` — orchestrator with `hold`, `commit`, `release`, `extend`, `run_sweep_once`
- `ReservationHandle` — opaque token carrying the persisted row's id + snapshot fields
- `ReservationContext` — per-call metadata bundle (audit actor, correlation id, optional tenant) — keeps the kernel API tight while threading audit/correlation cleanly
- `ReservationEvent` — `Held | Committed | Released { reason } | Expired` — emitted via `ferro-events` on every state transition
- `ReservationError` — single `thiserror` enum, `"reservation: …"` display prefix
- `CreateReservationsTable` — SeaORM migration consumers register in their own `Migrator`

The crate ships ONE foundational primitive that consumer apps depend on (gestiscilo-it v6.3 Online Checkout, v6.7 Inventory Monitoring; any future ticketing / rate-limiting / queue-admission app). It does NOT prescribe domain semantics — `Resource::Key` and `Resource::Window` are consumer-defined; the kernel knows nothing about inventory, products, slots, or seats.

**In scope:** crate scaffold, `Resource` trait, `ReservationKernel<R>` orchestrator, `ReservationHandle`, `ReservationContext`, `ReservationEvent` (via ferro-events), state-transition correctness through `ferro_orm::GuardedUpdate`, automatic `ferro-audit` emission on every state change, sweeper primitive (`run_sweep_once`), SeaORM migration (`reservations` table + 2 indexes), in-memory SQLite tests, property-based tests (`proptest` — milestone budget), integration test with ferro-orm + ferro-audit + ferro-events end-to-end, rustdoc, user-facing doc page, workspace version bump 0.2.31 → 0.2.32, auto-publish in Wave 1b.

**Out of scope (deferred):** `ferro-queue` runtime dep (consumers wire `run_sweep_once` into their own queue / cron / interval), distributed locks (Redis / Raft / etcd), Postgres CI integration tests, MCP tools to introspect reservations from an agent, websocket broadcast of reservation state to clients (Phase 155 `ferro-projection` covers live read-model fanout; ferro-reservation only emits the domain events), reservation grouping / multi-resource holds in one transaction, `try_hold` variant that returns immediately on insufficient capacity without retry, retention / archival of old reservations (consumer-driven via SQL).
</domain>

<decisions>
## Implementation Decisions

### Crate placement & scope

- **D-01:** Ship as a new top-level workspace crate at `ferro-reservation/` — mirrors Phase 152's `ferro-orm/` and Phase 153's `ferro-audit/` placement. The roadmap explicitly names `ferro-reservation`; downstream apps will import it as `use ferro_reservation::{ReservationKernel, Resource};`. Adding it inside `framework` would force every consumer to depend on the full framework crate for a primitive that is independently useful for any capacity-constrained domain.
- **D-02:** Crate is thin and additive at v0. It owns ONE table (`reservations`), one orchestrator type, one trait, and the matching error/event/context types. It does NOT subsume inventory semantics, ticket types, slot windows, or any domain model — those are consumer responsibilities. `ferro-reservation` is the *kernel*, not the application.
- **D-03:** Has internal ferro-* runtime deps: `ferro-orm` (for `GuardedUpdate` — state-transition correctness mechanism), `ferro-events` (for emitting `ReservationEvent`), `ferro-audit` (for automatic state-change logging — D-28 makes this unconditional). All three are Wave 1a leaves and already published. ferro-reservation is therefore a **Wave 1b** crate. No new top-level deps beyond what those three crates already pull in (sea-orm 1.0, thiserror 2, serde, serde_json, uuid, chrono, tokio, tracing).
- **D-04:** Wave 1b publish (depends on Wave 1a crates only). Add `ferro-reservation` to `WAVE1B_CRATES` in `.github/workflows/publish.yml` alongside `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`. New-crate-first-publish bootstrap from local terminal (CI token has publish-update only — see `project_ferro_publish_token_scoping.md`).

### Resource trait

- **D-05:** `Resource` is consumer-implemented:
  ```rust
  #[async_trait::async_trait]
  pub trait Resource: Send + Sync + 'static {
      type Key: Hash + Eq + Clone + Send + Sync + Serialize + DeserializeOwned;
      type Window: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned;
      // Window = () for non-windowed resources (atomic counters, simple capacity).

      const KIND: &'static str;     // dotted namespace: "inventory.unit", "checkout.slot"

      async fn capacity<C: ConnectionTrait>(
          &self, conn: &C, key: &Self::Key, window: &Self::Window,
      ) -> Result<u32, ReservationError>;

      async fn held<C: ConnectionTrait>(
          &self, conn: &C, key: &Self::Key, window: &Self::Window,
      ) -> Result<u32, ReservationError>;
  }
  ```
- **D-06:** `Resource::capacity` and `Resource::held` are generic over `<C: ConnectionTrait>` to match Phase 152/153 patterns — consumer can call them inside their own transactions. This is a deliberate deviation from the design doc's `&DatabaseConnection` signature, for consistency with `GuardedUpdate::exec_one` and `AuditEntry::write` (both already `<C: ConnectionTrait>`).
- **D-07:** `Resource::held` returns "sum of all reservations that occupy capacity" — typically the SUM of `quantity` over rows where `status IN ('held', 'committed')` for the same `(resource_kind, resource_key, window)`. Documented as a convention; the trait does not enforce it. The consumer's implementation decides whether to fold in non-kernel reservations (legacy inventory, externally-held quantities, etc.).
- **D-08:** `Resource::KIND` is a `&'static str` constant (not a method). One resource impl serves one kind; the constant doubles as the value persisted to `resource_kind` and makes the kind grep-able. Dotted-namespace convention (`"inventory.unit"`, `"checkout.slot"`, `"api.quota"`) mirrors `ferro-audit`'s `action` / `target.kind` convention (Phase 153 D-08, D-19).

### Kernel API & connection model

- **D-09:** `ReservationKernel<R: Resource>` is constructed with a `DatabaseConnection` for the sweeper path (which has no caller-supplied connection) and the resource impl:
  ```rust
  pub struct ReservationKernel<R: Resource> {
      db: DatabaseConnection,
      resource: R,
  }

  impl<R: Resource> ReservationKernel<R> {
      pub fn new(db: DatabaseConnection, resource: R) -> Self;
  }
  ```
  Per-call methods accept an explicit `&C: ConnectionTrait` so consumers can run them inside their own transactions. The owned `db` is used only by `run_sweep_once` and for read paths that have no natural caller-supplied connection.
- **D-10:** `hold` signature:
  ```rust
  pub async fn hold<C: ConnectionTrait>(
      &self,
      conn: &C,
      key: R::Key,
      window: R::Window,
      quantity: u32,
      ttl: Duration,
      ctx: &ReservationContext,
  ) -> Result<ReservationHandle, ReservationError>;
  ```
  Sequence:
  1. Call `R::capacity(&conn, &key, &window)`
  2. Call `R::held(&conn, &key, &window)`
  3. If `held + quantity > capacity` → `Err(ReservationError::Insufficient { requested, available, capacity })`
  4. INSERT one `reservations` row with `status = 'held'`, `expires_at = now() + ttl`, snapshot the inputs
  5. Emit `ReservationEvent::Held { … }` via ferro-events
  6. Write `AuditEntry::record("reservation.held")…write(&conn).await` with the ctx's actor
  7. Return `ReservationHandle` snapshot
- **D-11:** `commit` / `release` / `extend` signatures:
  ```rust
  pub async fn commit<C: ConnectionTrait>(
      &self, conn: &C, handle: ReservationHandle, ctx: &ReservationContext,
  ) -> Result<(), ReservationError>;

  pub async fn release<C: ConnectionTrait>(
      &self, conn: &C, handle: ReservationHandle, reason: ReleaseReason, ctx: &ReservationContext,
  ) -> Result<(), ReservationError>;

  pub async fn extend<C: ConnectionTrait>(
      &self, conn: &C, handle: ReservationHandle, by: Duration, ctx: &ReservationContext,
  ) -> Result<(), ReservationError>;
  ```
  `handle` is taken by value to enforce use-once at the type level. Re-using a committed/released handle is a compile-time error.
- **D-12:** Every state-transition method goes through `ferro_orm::GuardedUpdate`:
  ```rust
  GuardedUpdate::new(reservations::Entity)
      .filter(reservations::Column::Id.eq(handle.id))
      .filter(reservations::Column::Status.eq("held"))
      .set_value(reservations::Column::Status, "committed".into())
      .set_value(reservations::Column::CommittedAt, Utc::now().into())
      .exec_one(&conn)
      .await?;
  ```
  `NoRowsAffected` from the guarded update → `ReservationError::ConflictingState { id: handle.id, expected: "held" }` (the row was already committed/released/expired by a concurrent caller or the sweeper). The kernel does NOT re-query to find the actual state — that would be a race window; the error names the expected state and the caller can introspect via the row id if needed.
- **D-13:** `extend` only succeeds on `held` status with an unexpired `expires_at` (guarded predicate `status = 'held' AND expires_at > now()`). Extending an already-expired-but-not-yet-swept reservation is a `ConflictingState` error. Documented as the v0 semantic: TTL extension is a "still-fresh" operation; bringing an expired hold back from the dead is not supported (would race against the sweeper).
- **D-14:** No `try_hold` variant. The capacity check is part of `hold`'s standard sequence; an `Insufficient` error IS the no-capacity signal. A non-blocking variant adds API surface for a marginal benefit; defer to v0.x.
- **D-15:** Kernel is `Clone + Send + Sync` (the underlying `DatabaseConnection` is). Consumers wrap it in `Arc` if they need cheap sharing across tasks; the kernel itself does not enforce a singleton.

### State machine, status, concurrency

- **D-16:** Four distinct statuses persisted as `VARCHAR`: `"held" | "committed" | "released" | "expired"`. Stored as strings (not an enum column) for cross-dialect simplicity and so the same migration runs on SQLite + Postgres. SeaORM `ActiveEnum`-style typed column considered and rejected: stringly-typed JSON-style columns are easier for downstream MCP introspection (`db_schema` reads them as plain strings).
- **D-17:** Allowed transitions (all enforced by `GuardedUpdate` predicates, never application code):
  - `held` → `committed` (via `commit`)
  - `held` → `released` (via `release`)
  - `held` → `expired` (via sweeper)
  - Terminal states (`committed`, `released`, `expired`) have NO outgoing transitions. Any attempt is a `ConflictingState` error.
- **D-18:** `ReleaseReason` is a typed enum recorded in the audit log + emitted in `ReservationEvent::Released`:
  ```rust
  pub enum ReleaseReason {
      UserCancelled,
      PaymentFailed,
      AdminOverride,
      Other(String),    // free-form for app-specific reasons
  }
  ```
  Serde-derived with `#[serde(rename_all = "snake_case")]` and `#[serde(tag = "reason")]`. Documented but consumer-free at compile time.
- **D-19:** Concurrency correctness claim — *the kernel's only correctness mechanism is the per-statement atomicity of the underlying `GuardedUpdate`*. There is no application-level locking, no SELECT-then-UPDATE, no retry loop. SQLite serial-writer guarantees + Postgres `READ COMMITTED` are sufficient for every state transition. The capacity check in `hold` is the one read-then-write window; correctness there is guaranteed by the INSERT failing on the unique constraint (D-39) if a concurrent insert + capacity check race produced an over-allocation. The integration test (D-50) proves this end-to-end.
- **D-20:** No deadlocks possible by construction — every state transition is a single `UPDATE … WHERE id = ?`. No row-level locks held across awaits, no multi-row transactions, no FK cycles.

### Sweeper

- **D-21:** Sweeper API is a single method on the kernel:
  ```rust
  pub async fn run_sweep_once(&self) -> Result<SweepReport, ReservationError>;

  pub struct SweepReport {
      pub expired_count: u32,
      pub scanned_at: DateTime<Utc>,
  }
  ```
  Implementation: `SELECT id, … FROM reservations WHERE status = 'held' AND expires_at < now() LIMIT 500`; for each, run a `GuardedUpdate` `held → expired` transition; on success, emit `ReservationEvent::Expired` and `AuditEntry::record("reservation.expired")`. The 500-row batch cap prevents one slow sweep from holding the DB connection for too long; consumer schedules subsequent sweeps if backlog persists.
- **D-22:** **No runtime `ferro-queue` dependency.** Consumers schedule `run_sweep_once` themselves — either via a `ferro_queue::Job` impl they own, a cron task, or a plain `tokio::time::interval` loop. The rustdoc shows the three idiomatic patterns. Rationale: keeps `ferro-reservation` independent of queue runtime choice (a future ferro app might use a different scheduler); reduces dep graph; one less Wave-coupling concern.
- **D-23:** Sweeper uses `AuditActor::System` for the audit entry (no caller-supplied context). The audit `action` is `"reservation.expired"` and the `target` is `AuditTarget::new("reservation", id.to_string())`. Sweep emits one audit entry + one event per expired reservation.
- **D-24:** Sweeper is idempotent under concurrent execution. If two sweeper tasks race on the same expired row, only one wins the guarded `held → expired` transition; the other gets `NoRowsAffected` and skips that row silently (not surfaced as an error — concurrent sweepers are a normal deployment shape, not a bug).

### Event emission (ferro-events)

- **D-25:** `ReservationEvent` enum, serde-derived with `#[serde(rename_all = "snake_case", tag = "kind")]`:
  ```rust
  pub enum ReservationEvent {
      Held { id: Uuid, resource_kind: String, resource_key: JsonValue, window: Option<JsonValue>, quantity: u32, expires_at: DateTime<Utc> },
      Committed { id: Uuid, resource_kind: String, resource_key: JsonValue },
      Released { id: Uuid, resource_kind: String, resource_key: JsonValue, reason: ReleaseReason },
      Expired { id: Uuid, resource_kind: String, resource_key: JsonValue },
  }
  ```
  Implements `ferro_events::Event` (the trait existing consumers already implement). Payload is JSON for `resource_key` and `window` because the kernel is generic over `R::Key` / `R::Window` and ferro-events' fanout is JSON-shaped — at the event-bus boundary the typed key becomes opaque JSON; subscribers re-deserialize if they need to. Documented in rustdoc.
- **D-26:** Events are emitted via `ferro_events::dispatch(event)` AFTER the guarded `UPDATE`/`INSERT` succeeds. If event dispatch fails (event bus disconnected, listener panic), the state transition is already committed — kernel returns `Ok(())` and logs the dispatch error at `tracing::warn!`. Rationale: an event-bus failure must not roll back a successful reservation; consumers can replay missed events from the audit log (which never depends on event dispatch).
- **D-27:** No event filtering / subscription primitives in this crate. Consumers attach `ferro_events::Listener` impls themselves. The crate's contract is "I emit `ReservationEvent` on every state change"; everything else is consumer wiring.

### Audit emission (ferro-audit)

- **D-28:** **Audit emission is unconditional** on every successful state transition. Every successful `hold` / `commit` / `release` / sweeper-driven `expire` writes one `AuditEntry` with `action = "reservation.{held|committed|released|expired}"`, `target = AuditTarget::new("reservation", id)`, `before` / `after` capturing the status change + quantity. Rationale: ferro-reservation is marketed as race-free WITH historical evidence; making audit opt-in undermines that promise.
- **D-29:** `ReservationContext` is the per-call audit metadata bundle:
  ```rust
  pub struct ReservationContext {
      pub actor: AuditActor,
      pub correlation_id: Option<Uuid>,
      pub tenant_id: Option<String>,
      pub reason: Option<String>,
  }

  impl ReservationContext {
      pub fn system() -> Self;                    // AuditActor::System
      pub fn user(user_id: impl Into<String>) -> Self;
      pub fn job(name: impl Into<String>) -> Self;
      pub fn anonymous() -> Self;
      pub fn with_correlation(self, id: Uuid) -> Self;
      pub fn with_tenant(self, t: impl Into<String>) -> Self;
      pub fn with_reason(self, r: impl Into<String>) -> Self;
  }
  ```
  Bundling these into one struct keeps `hold` / `commit` / `release` signatures stable as audit metadata grows (e.g., future `request_id` support). Defaults to `system()` for sweeper-internal calls.
- **D-30:** Audit-write failure rolls back the in-memory bookkeeping but the DB row is already updated by `GuardedUpdate`. The kernel returns `ReservationError::Audit(AuditError)` so the caller sees the failure; the state transition itself is committed. Documented as "the audit and state are consistent at the DB level — if audit fails, the state change still happened; the consumer's monitoring should alarm on `ReservationError::Audit`." This matches Phase 153's append-only-audit philosophy: audit never blocks state changes; failed audit is operational visibility, not a correctness guarantee.

### TTL & `extend`

- **D-31:** `ttl: Duration` is a `std::time::Duration`. Persisted as `expires_at: DateTime<Utc>` computed at hold time (`Utc::now() + ttl`). Sub-second precision is preserved (chrono's `DateTime<Utc>` carries nanoseconds; SQLite/Postgres timestamp columns hold microseconds).
- **D-32:** `extend(handle, by: Duration)` adds `by` to the persisted `expires_at`. Guarded transition: `status = 'held' AND expires_at > now()`. Multiple extends compound; no upper cap on extension count or duration in v0. Document the operational risk: "a held reservation can be extended indefinitely; consumers wanting a hard TTL ceiling enforce it at the call site."
- **D-33:** No "auto-extend on use" semantics. Reservations are explicit handles; calling `commit` doesn't auto-extend. Consumers who want renewable holds (e.g., a checkout cart kept alive by user activity) call `extend` from a heartbeat endpoint.

### ReservationHandle

- **D-34:** `ReservationHandle` carries a full snapshot of the persisted row at hold time:
  ```rust
  pub struct ReservationHandle {
      pub id: Uuid,
      pub resource_kind: String,
      pub resource_key: JsonValue,
      pub window: Option<JsonValue>,
      pub quantity: u32,
      pub held_at: DateTime<Utc>,
      pub expires_at: DateTime<Utc>,
      pub tenant_id: Option<String>,
  }
  ```
  Serde-derived (`Serialize + Deserialize`) so callers can embed it in a Stripe payment intent's `metadata`, a queued-job payload, or any other side-channel. The `id` is the only field used as a primary key in subsequent calls; the rest is reference data.
- **D-35:** Handle does NOT include `correlation_id` or audit context — those are per-call (set in `ReservationContext`), not per-reservation. A reservation can have a different actor when committed (e.g., Stripe webhook system actor) than when held (user actor); the audit log captures both, the handle does not.

### Multi-tenancy

- **D-36:** `tenant_id: Option<String>` on the `reservations` row, set from `ReservationContext::with_tenant(...)` if the caller provides it. Stringly-typed for the same reason Phase 153 D-13 chose stringly-typed — ferro has no first-class tenant primitive (search confirmed only consumer-specific `tenant_id` usage in `ferro-cli`'s `make_stripe` template). Forward-compatible if ferro grows a typed tenant later.
- **D-37:** Kernel does NOT scope `Resource::capacity` / `Resource::held` queries by tenant automatically. The consumer's `Resource` implementation includes tenant filtering itself (typically by adding `tenant_id` to `Resource::Key`). This keeps the trait generic — multi-tenancy is a `Key` concern, not a kernel concern.

### Schema & migration

- **D-38:** Ship a SeaORM migration as a public re-export so consumers register it explicitly:
  ```rust
  pub use migration::Migration as CreateReservationsTable;
  ```
  Same pattern as Phase 153 D-18. Consumer's `Migrator`:
  ```rust
  vec![
      Box::new(ferro_audit::CreateAuditLogTable),
      Box::new(ferro_reservation::CreateReservationsTable),
      // ... app migrations
  ]
  ```
- **D-39:** Schema columns (matches `INVENTORY-PRIMITIVES.md` §`ferro-reservation` with column-name fixes):
  ```
  reservations
  ├── id              UUID PRIMARY KEY
  ├── resource_kind   VARCHAR NOT NULL                    -- "inventory.unit", "checkout.slot"
  ├── resource_key    JSON NOT NULL                       -- serialized Resource::Key
  ├── window          JSON NULL                           -- serialized Resource::Window; NULL when Window = ()
  ├── quantity        INTEGER NOT NULL                    -- u32 stored as INTEGER (SQLite) / INTEGER (Postgres)
  ├── status          VARCHAR NOT NULL                    -- "held" | "committed" | "released" | "expired"
  ├── expires_at      TIMESTAMP NOT NULL                  -- set at hold; mutated by extend
  ├── held_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  ├── committed_at    TIMESTAMP NULL                      -- set on commit
  ├── released_at     TIMESTAMP NULL                      -- set on release
  ├── release_reason  VARCHAR NULL                        -- serialized ReleaseReason::tag on release
  ├── tenant_id       VARCHAR NULL
  ```
- **D-40:** Indexes (matches design + sweeper access pattern):
  - `idx_reservations_kind_key_window_status` on `(resource_kind, resource_key, window, status)` — primary `Resource::held` lookup path
  - `idx_reservations_status_expires` on `(status, expires_at)` — sweeper scan path
  - PRIMARY KEY on `id` covers `commit` / `release` / `extend` row-by-id lookups
- **D-41:** `id` is `Uuid` (not auto-increment), generated client-side at `hold()` time. UUIDv4. Lets the kernel return the handle WITH its id without a re-fetch round-trip (matches Phase 153 D-21).
- **D-42:** `held_at` is set by the DB (`CURRENT_TIMESTAMP` default) for the same clock-skew reason as Phase 153 D-22. `committed_at` / `released_at` are set by the application (chrono `Utc::now()`) inside the `GuardedUpdate` `set_value` chain — the row is being conditionally updated, so application clock is fine and avoids needing dialect-specific `CURRENT_TIMESTAMP` expression handling in SeaORM.

### Error model

- **D-43:** `ReservationError` is a `thiserror`-derived enum, one error per crate, panics nowhere:
  ```rust
  pub enum ReservationError {
      #[error("reservation: insufficient capacity (requested {requested}, available {available} of {capacity})")]
      Insufficient { requested: u32, available: u32, capacity: u32 },

      #[error("reservation: id={id} not in expected state '{expected}' (already committed/released/expired or never existed)")]
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
  ```
  Display prefix is `"reservation: …"` for grep-friendliness across the workspace (matches `"guarded: …"`, `"audit: …"`, `"config: …"`).
- **D-44:** `Insufficient { capacity }` includes the resource's `capacity` value at the time of the failed check — useful for telemetry and surfacing to UI ("3 units left, you asked for 5").
- **D-45:** `ConflictingState::expected` is `&'static str` (not an enum) to keep error-construction cheap. The kernel passes `"held"` for all state transitions. Documented in rustdoc.
- **D-46:** `From<GuardedError>` for `ReservationError::Guarded` lets the kernel use the `?` operator on guarded calls without an explicit `map_err`. The `NoRowsAffected` case is mapped explicitly inside each method to `ConflictingState` BEFORE the `?` — because the kernel knows the predicate's intent and the consumer wants the semantic "the row was not in `held`", not the raw "guarded returned no rows".

### Testing

- **D-47:** Unit tests live next to the code (`#[cfg(test)] mod tests`) in `ferro-reservation/src/`. Cover:
  1. `hold` happy path: capacity 10, request 3 → handle returned, row persisted with `status = 'held'`, `quantity = 3`, `expires_at` ≈ now + ttl.
  2. `hold` rejects when `held + requested > capacity` → `ReservationError::Insufficient { requested, available, capacity }`.
  3. `commit` happy path: `held` → `committed`, `committed_at` set.
  4. `commit` on already-committed handle → `ConflictingState { expected: "held" }`.
  5. `release` happy path with each `ReleaseReason` variant.
  6. `extend` happy path: `expires_at` increases by the requested delta.
  7. `extend` on expired-but-not-swept row → `ConflictingState`.
  8. `run_sweep_once` happy path: 3 rows with `expires_at < now()` → all transition to `expired`, report.expired_count = 3.
  9. `run_sweep_once` no-op when no rows are eligible.
  10. `Resource::Key` / `Resource::Window` serialization round-trip via JSON column.
  11. `ReservationContext` defaults + builder methods.
  12. `ReservationHandle` serde round-trip.
- **D-48:** Integration test (`tests/concurrent_hold.rs`) — **the property-test that anchors v11.11's correctness claim**: spin up N=20 tokio tasks all attempting `hold(capacity=5, quantity=1)`; assert exactly 5 succeed with `Ok(_)`, the other 15 fail with `Insufficient`, and the persisted row count for `status = 'held'` is 5.
- **D-49:** Property-based tests via **`proptest`** (Phase 153 D-32 documented the milestone budget lands here). Add `proptest = "1"` as dev-dep. Two key properties from `INVENTORY-PRIMITIVES.md` §`Testing strategy`:
  - **Property 1:** For any random interleaving of N concurrent `hold` calls against capacity C (N, C ∈ [1, 20]), `SUM(quantity WHERE status IN ('held','committed'))` ≤ C.
  - **Property 2:** For any sequence of `hold → commit | release | (let expire)` operations, the persisted state never violates the state-machine transitions (`committed` only reachable from `held`, etc.). Implemented by replaying the audit log and asserting the transitions are valid.
- **D-50:** Cross-crate integration test (`tests/integration_with_audit_and_events.rs`) — proves the three-crate composition end-to-end: hold + commit a reservation, then assert (a) two `ReservationEvent` instances dispatched (`Held`, `Committed`), (b) two `AuditEntry` rows persisted with matching `correlation_id`, (c) `AuditEntry::reconstruct_state(history)` reproduces the final state. This is the showcase test that justifies the milestone.
- **D-51:** Postgres integration tests deferred (same call as Phase 152 D-19 and Phase 153 D-33). SQLite serial-writer + property tests under in-memory SQLite are sufficient to validate the kernel's correctness claim. Risk accepted; documented.
- **D-52:** Test harness: in-memory SQLite, re-derive the harness inline (do not depend on `framework`). The migration under test is `CreateReservationsTable` + `CreateAuditLogTable` (for integration tests) registered in a test-only `Migrator`.

### Documentation

- **D-53:** Module-level rustdoc on `lib.rs` with the inventory-decrement example from `INVENTORY-PRIMITIVES.md` §`ferro-reservation`, rewritten to use the final v0 API. Lead with the *why* (capacity-constrained apps all hand-roll the same buggy `read → check → write`; this is the typed replacement), then show the one-call API. Include the four-status state diagram in ASCII. Document the audit-emission guarantee and the event-bus best-effort semantics.
- **D-54:** New user-facing doc page `docs/src/database/reservations.md` covering: the resource/window abstraction, defining a `Resource` impl for a domain object, kernel construction, the four lifecycle methods, TTL + sweeper, event subscription pattern, audit log inspection, the three sweeper-scheduling idioms (ferro-queue job, cron, tokio interval), common patterns (slot hold during checkout, ticket reservations, API rate-limit buckets), the consistency model (per-statement atomicity), and the operational footgun (audit failure does NOT roll back state).
- **D-55:** ferro-mcp introspection: no new MCP tools in this phase. `application_info` will auto-include `ferro-reservation` in `installed_crates`; `db_schema` will pick up the `reservations` table; `generation_context` / `code_templates` will pick up the rustdoc automatically. A future `reservation_check_capacity` MCP tool (read-only, "what's the available capacity for this resource right now?") is plausible in v0.x once a real agent use case surfaces.

### Release

- **D-56:** Workspace `[workspace.package] version` bumps one patch (from `0.2.31` to `0.2.32`) when Phase 154 verifies. Standard ferro release process; matches the cadence established by Phase 152 (→ 0.2.30) and Phase 153 (→ 0.2.31).
- **D-57:** Add `ferro-reservation` to **Wave 1b** of `.github/workflows/publish.yml` (D-04). New-crate bootstrap from local terminal — same operational reality as Phase 152 / Phase 153.
- **D-58:** CHANGELOG entry under `ferro-reservation` (new section, placed at the top per Phase 152 D-25 convention) summarising: new crate, `Resource` trait for capacity-constrained domains, `ReservationKernel<R>` with `hold` / `commit` / `release` / `extend`, TTL with `run_sweep_once` sweeper primitive, `ReservationEvent` via ferro-events, automatic audit emission via ferro-audit, `ReservationContext` bundle for per-call audit metadata, `ReservationError` umbrella enum, race-free state transitions through `ferro_orm::GuardedUpdate`.

### Folded scope from todos

No pending todos matched Phase 154 (`gsd-tools todo match-phase 154` returned zero matches at gather time).

### Claude's Discretion

Within the boundaries set above, the planner/executor decides:

- Internal module layout of `ferro-reservation/src/` (likely `lib.rs` + `kernel.rs` + `resource.rs` + `handle.rs` + `context.rs` + `event.rs` + `error.rs` + `migration.rs` + `entity.rs` + `sweeper.rs`, but the planner is free to consolidate where the public surface is unchanged)
- Whether to expose the SeaORM `Entity` / `Model` / `ActiveModel` types as a public re-export for consumers who want native SeaORM queries (recommended; matches Phase 153)
- Exact wording of `tracing::warn!` diagnostics on event-dispatch failure / audit failure
- Whether `SweepReport` is exposed as a public type or only via the `Result` shape (recommended: public type — consumers want to log it)
- Exact `proptest` strategy shape (the properties are locked in D-49; the generator construction is open)
- Test file naming inside `ferro-reservation/tests/`
- Whether to ship a `ReservationKernel::available_capacity(&conn, &key, &window) -> Result<u32, _>` read-only helper (it's just `capacity - held`; consumers can call those methods directly, but exposing the convenience is cheap)

### Deferred (NOT in this phase)

- **`try_hold` non-blocking variant** — v0 ships only `hold` returning `Insufficient` on no-capacity. Retry loops are caller territory; a non-blocking try variant is a marginal API addition deferred to v0.x.
- **Runtime `ferro-queue` integration / `ReservationSweeperJob` ready-to-register Job impl** — would push runtime deps; cleaner to ship sweeper-as-primitive and let consumers wire scheduling. Documentation shows the three idiomatic patterns.
- **`ReservationKernel::cancel_all_for(key, window, ...)`** — bulk-release helper for end-of-show / event-cancel scenarios. Plausible v0.x; no current consumer asks for it.
- **Reservation grouping (multi-resource hold in one atomic call)** — e.g., "hold one of A AND one of B together or neither." Distinct primitive (saga / two-phase commit territory); ferro-reservation v0 holds single-resource handles only.
- **Distributed locks (Redis / Raft / etcd)** — explicitly out per `INVENTORY-PRIMITIVES.md` §`Out of scope`. Single-region SQLite/Postgres is sufficient for the targeted consumers.
- **Postgres CI integration tests** — would require docker-Postgres in CI for one crate; disproportionate. SQLite serial-writer + the property tests cover correctness.
- **MCP tool: `reservation_check_capacity`** — interesting v0.x addition; Phase 154 is the substrate.
- **WebSocket broadcast of reservation state to clients** — Phase 155 `ferro-projection` is the canonical home for live-read-model fanout. ferro-reservation only emits domain events.
- **Reservation archival / retention** — `prune_committed_older_than(cutoff)` helper. Consumer-driven via raw SQL or a future v0.x addition.
- **`ferro::prelude` / framework re-export** — same call as Phase 152 / Phase 153; consumers import `ferro-reservation` directly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design source of truth

- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-reservation` — original spec (trait shape, kernel API, error variants, schema, sweeper, events). Authoritative.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Cross-crate relationships` — confirms ferro-reservation depends on ferro-orm + ferro-events + ferro-audit; not depended on by anything in v11.11.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Consumer integration shape` — the two illustrative `Resource` impls (gestiscilo InventoryUnit, ticketing Seat, rate-limiter ApiQuota). Use these as the design's own integration smoke tests when writing rustdoc.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Testing strategy` — per-crate test scope + cross-crate integration + property-based tests. Phase 154 carries the milestone's property-test budget.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Migration / rollout` — confirms ferro-reservation ships after Phase 152 + 153 (both shipped), additive, no breaking changes.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Out of scope` — explicit non-goals: no distributed locks, no inventory-specific schema in ferro, no UI components, no multi-region replication.

### Project conventions

- `CLAUDE.md` §`Architecture Principles` — project-agnostic crates rule (no hardcoded app identity, no consumer-specific types in the public API). ferro-reservation must not bind to inventory units, products, slots, tickets, or any consumer-specific id shape.
- `CLAUDE.md` §`Testing & Linting` — exact fmt + clippy + test commands required pre-commit. Applies identically to ferro-reservation.
- `CLAUDE.md` §`Workspace Structure` — ferro-reservation is added to this table during execution.
- `CLAUDE.md` §`Form Field Rules` — N/A for this phase (no form rendering).
- `.planning/PROJECT.md` — vision anchors; the projection/intent abstraction is the killer feature this milestone unblocks (via reservations + live read-models).
- `.planning/STATE.md` — current workspace version (`0.2.31` post-153), next version is `0.2.32` after Phase 154 verifies.

### Sibling phase context (must read before planning)

- `.planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-CONTEXT.md` — Phase 152 is the **structural twin AND the runtime dep** for crate scaffolding, Wave-publish placement (1a / 1b template), error-naming convention (`"reservation: …"` prefix mirrors `"guarded: …"`), `<C: ConnectionTrait>` generic style, testing harness choice, doc-page placement (`docs/src/database/`), CHANGELOG shape. The `GuardedUpdate` builder from this phase is the entire correctness mechanism of ferro-reservation's state transitions.
- `.planning/phases/153-ferro-audit-crate-structured-before-after-audit-log-with-rep/153-CONTEXT.md` — Phase 153 is the **second structural twin AND the runtime dep** for the audit-emission contract (D-28), the migration-as-public-re-export pattern (D-38 mirrors 153 D-18), `tenant_id: Option<String>` stringly-typed convention (D-36 mirrors 153 D-13), `AuditActor` shape consumed by `ReservationContext::user/system/job/anonymous` constructors. `AuditEntry::record(...).write(...)` is the API ferro-reservation calls inside every state transition.

### Patterns to mirror (template ferro-* crates)

- `ferro-orm/Cargo.toml` — closest Wave-1a sibling Cargo.toml shape; ferro-reservation Cargo.toml is the Wave-1b equivalent (same metadata fields, adds ferro-orm / ferro-events / ferro-audit to `[dependencies]`).
- `ferro-orm/src/lib.rs` — module-level rustdoc tone for a v0 single-purpose crate.
- `ferro-audit/Cargo.toml` — sea-orm + sea-orm-migration + serde + uuid + chrono + tracing dep set; ferro-reservation inherits the same database-adjacent deps plus async-trait (for `Resource`).
- `ferro-audit/src/migration.rs` — SeaORM migration shape; `CreateReservationsTable` mirrors `CreateAuditLogTable`.
- `ferro-audit/src/entity.rs` — SeaORM entity definition with UUID PK + JSON columns + nullable timestamps; `reservations::Entity` mirrors the same pattern.
- `ferro-events/src/lib.rs` / `ferro-events/src/traits.rs` — `Event` trait + dispatcher pattern. `ReservationEvent` implements `Event`.
- `ferro-notifications/Cargo.toml` — closest Wave-1b sibling; cross-reference for the Wave-1b Cargo.toml shape under `[dependencies]` (internal ferro-* + sea-orm + external deps stacked together).
- `.github/workflows/publish.yml` — Wave 1b crate list (`WAVE1B_CRATES`); ferro-reservation is added here.
- `framework/src/database/testing.rs` — in-memory SQLite testing harness reference; ferro-reservation re-derives the harness inline.

### Cross-phase coordination

- Phase 152 (shipped): `ferro-orm::GuardedUpdate` is the entire state-transition correctness mechanism. Every status change in ferro-reservation goes through it. No alternative path.
- Phase 153 (shipped): `ferro-audit::AuditEntry::record(...).write(...)` is the audit-emission call inside every state transition (D-28).
- Phase 155 (ferro-projection): independent of Phase 154 in code but typically deployed alongside. A consumer's `ReservationProjection` subscribes to `ReservationEvent` and maintains a live read-model. Phase 154 does NOT include any projection-specific affordances.

### Conventions repository (operator memory)

- `feedback_ci_clippy_command_match.md` — match CI's exact clippy command (`--all --all-targets -- -D warnings`) in pre-push checks.
- `feedback_validate_scope_premises.md` — ferro-reservation does not currently exist as a crate; this premise was verified before writing this CONTEXT (`ls ferro-reservation` → not found; `grep -rl "ferro-reservation\|ferro_reservation" --include="*.rs" --include="*.toml"` returned only references in this design doc and roadmap).
- `project_ferro_publish_token_scoping.md` — CI publish token has publish-update only; new-crate bootstrap requires personal `publish-new`-scoped token from a local terminal.
- `feedback_macbook_thermal_ferro_builds.md` — pace long auto-advance chains; Phase 154's plan / execute sequence will compile a fresh crate plus run property tests — moderate thermal cost expected. Pause if thermal stress is signalled.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets

- **`ferro_orm::GuardedUpdate`** (Phase 152, shipped at `0.2.30+`) — the entire state-transition correctness mechanism. Reservations call `GuardedUpdate::new(entity).filter(id_eq).filter(status_eq("held")).set_value(...).exec_one(&conn)` for every transition.
- **`ferro_audit::AuditEntry::record(action).…write(&conn)`** (Phase 153, shipped at `0.2.31`) — the audit-emission call inside every state transition. The builder API takes a `ConnectionTrait` so the audit write can join the caller's transaction.
- **`ferro_audit::AuditActor`** (Phase 153) — `User | System | Job | ApiClient | Anonymous` — directly reused inside `ReservationContext::actor`. Constructors mirror: `ReservationContext::user(...)` wraps `AuditActor::User(...)`, etc.
- **`ferro_events::dispatch(event)`** (existing Wave 1a) — fires `ReservationEvent` to subscribers. ferro-events is `async-trait`-based; `ReservationEvent` implements `Event`.
- **SeaORM 1.0 + sea-orm-migration 1.0** are workspace deps (already in `ferro-audit` Cargo.toml — copy the dep set verbatim).
- **`thiserror` 2, `serde`, `serde_json`, `uuid` (v4 + serde), `chrono` (serde), `tracing`** are workspace deps; ferro-reservation adds them as direct deps with the same versions.
- **`async-trait` 0.1** is a workspace dep (used by `ferro-events`); ferro-reservation adds it as a direct dep for the `Resource` trait's async methods.
- **`proptest` 1** — NEW dev-dep for property tests (D-49). No prior use in the workspace; safe to add.
- **In-memory SQLite testing pattern** — `framework/src/database/testing.rs` reference; ferro-audit and ferro-orm already re-derive it inline. ferro-reservation does the same.
- **No existing reservation code in the workspace** — `grep -rl "ReservationKernel\|reservation_kind\|ReservationEvent\|ferro_reservation\|ferro-reservation"` returned only references in this design doc, roadmap, and STATE.md. Greenfield.

### Established patterns

- **One Error enum per crate** (`thiserror` derive) — convention across `ferro-orm`, `ferro-audit`, `ferro-wallet`, `ferro-stripe`, `ferro-events`. `ReservationError` follows.
- **Display prefix on error enum** — `"reservation: …"` mirrors `"guarded: …"`, `"audit: …"`, `"config: …"`. Cross-workspace grep-friendly.
- **Builder pattern: `with_*` taking `mut self` → `Self`** — `ReservationContext` builder methods (`with_correlation`, `with_tenant`, `with_reason`) follow this shape.
- **Generic over `ConnectionTrait`** — `GuardedUpdate::exec_*`, `AuditEntry::write`, ferro-audit query helpers all accept `<C: ConnectionTrait>`. Kernel methods + `Resource::capacity` / `Resource::held` follow suit.
- **`#[serde(rename_all = "snake_case")]`** on enums — applies to `ReservationEvent`, `ReleaseReason`. Tag-on-`kind` for the event enum.
- **Wave 1b Cargo.toml metadata fields** — `description`, `keywords`, `categories = ["database", "asynchronous"]`, `repository`, `readme = "README.md"`, `homepage = "https://ferro-rs.dev"`. Copy from `ferro-audit/Cargo.toml`; adjust keywords / categories for reservation semantics.
- **SeaORM migration as public re-export** — `pub use migration::Migration as CreateReservationsTable;` mirrors Phase 153 D-18.
- **Static `inventory`-based event registration** — `ferro-events` uses `inventory` for compile-time event-listener registration. ferro-reservation does NOT register listeners; consumers do.

### Integration points

- **Workspace `Cargo.toml`** — add `"ferro-reservation"` to `[workspace.members]`.
- **`.github/workflows/publish.yml`** — add `ferro-reservation` to `WAVE1B_CRATES` alongside `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`.
- **Workspace version bump** — `[workspace.package] version = "0.2.32"`.
- **`framework/src/lib.rs`** — DO NOT add an automatic re-export of `ferro_reservation`. Consumers depend on `ferro-reservation` directly. Same call as Phase 152 / 153.
- **`README.md` (workspace root)** — add ferro-reservation to the workspace crates table (mirror Phase 152 / 153).
- **`CLAUDE.md` "Workspace Structure" table** — add a row for ferro-reservation so downstream agents see it immediately.
- **ferro-mcp `application_info` / `installed_crates`** — picks up ferro-reservation automatically once it's a workspace member; no MCP code changes expected.
- **`docs/SUMMARY.md` / nav** — add `reservations.md` to the `Database` section (mirrors how `atomic-updates.md` (152) and `audit-log.md` (153) were added).

### Constraints surfaced by the scout

- ferro-reservation is **a new top-level crate** — Phase 154 is the bootstrap. First publish requires manual personal-token bootstrap from local terminal (CI token is publish-update only) — same operational reality as Phase 151 / 152 / 153.
- The framework has **no first-class tenant primitive** today — `tenant_id` only appears in `ferro-cli`'s `make_stripe` template and as a stringly-typed column in ferro-audit. `ReservationContext::tenant_id: Option<String>` is correctly stringly-typed (D-36) and stays forward-compatible.
- The framework has **no first-class correlation/request-id primitive** today — same constraint as Phase 153 D-25; `ReservationContext::correlation_id: Option<Uuid>` is caller-supplied.
- **Wave 1b serializes after Wave 1a** in `publish.yml` — Phase 154's publish runs after Phase 152 / 153 versions are visible on crates.io. Bumping the workspace version triggers all crates' versions to step together; the dependency on `ferro-orm = "0.2.32"` / `ferro-audit = "0.2.32"` resolves naturally because they all bump in lockstep.
- **No existing `proptest` precedent in the workspace** — adding it as a dev-dep is fine but slightly visible; document in the rustdoc that the milestone's property-test budget lives in this crate.

</code_context>

<specifics>
## Specific Ideas

- The canonical sample from the design doc, rewritten to the v0 builder API for the rustdoc top example:
  ```rust
  use ferro_reservation::{ReservationKernel, ReservationContext, Resource, ReleaseReason};
  use std::time::Duration;

  // Consumer-defined Resource impl
  struct InventoryUnitResource { /* db reference, business rules */ }

  #[async_trait::async_trait]
  impl Resource for InventoryUnitResource {
      type Key = (TenantId, ProductId);
      type Window = BookingWindow;
      const KIND: &'static str = "inventory.unit";

      async fn capacity<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, _w: &Self::Window) -> Result<u32, _> { /* ... */ }
      async fn held<C: ConnectionTrait>(&self, conn: &C, key: &Self::Key, w: &Self::Window) -> Result<u32, _> { /* ... */ }
  }

  // Application setup
  let kernel = ReservationKernel::new(db.clone(), InventoryUnitResource::new(/* ... */));

  // Online-checkout: hold a slot during payment
  let ctx = ReservationContext::user(user_id.to_string()).with_correlation(request_id);
  let handle = kernel.hold(&conn, key, window, /*qty=*/1, Duration::from_secs(15 * 60), &ctx).await?;

  // ... process Stripe payment ...
  match stripe_result {
      Ok(_) => kernel.commit(&conn, handle, &ctx).await?,
      Err(_) => kernel.release(&conn, handle, ReleaseReason::PaymentFailed, &ctx).await?,
  }
  ```
- The error-naming style across the workspace (`"guarded: …"`, `"audit: …"`, `"config: …"`) — `ReservationError` follows the same `"reservation: …"` Display prefix.
- The framing in the rustdoc: lead with *why* (capacity-constrained apps all hand-roll the same buggy `read → check → write` pattern; this typed kernel makes it race-free by construction WITH audit history). Show the four-status state diagram in ASCII (`held → committed | released | expired`). Then the operational footguns: (1) event dispatch failure is logged but does NOT roll back state; (2) audit failure surfaces as `Audit` error but the DB row is already updated; (3) `extend` indefinite — no upper cap.
- Dotted-namespace convention for `Resource::KIND` mirrors ferro-audit's `action` / `target.kind` convention: `"inventory.unit"`, `"checkout.slot"`, `"api.quota"`. Documented as a convention; not enforced at compile time.
- The mental model the rustdoc opens with: "ferro-reservation is the *capacity* primitive. ferro-events says *something happened*. ferro-audit says *here is the evidence forever*. ferro-orm::GuardedUpdate says *only one writer wins*. ferro-reservation says *the resource is reserved, with a deadline, race-free, with audit and broadcast — pick a side from the trio at the right layer*."
- The state diagram for the rustdoc top:
  ```
                hold()                 commit()
       ──────────────▶ held ────────────────────▶ committed
                       │
                       │ release(reason)
                       ▼
                  released
                       ▲
                       │ run_sweep_once()
                       │
       ──────────────▶ held ─── ttl ────────────▶ expired
  ```
- The sweeper-scheduling idioms documented (one of these patterns lands in the user-facing doc):
  ```rust
  // (1) ferro-queue Job
  // (2) tokio::time::interval task
  // (3) cron-driven CLI runner (`ferro reservation:sweep` could ship in v0.x)
  ```

</specifics>

<deferred>
## Deferred Ideas

- **`try_hold` non-blocking variant** — v0.x.
- **Bulk `cancel_all_for(key, window, reason, ctx)`** — v0.x.
- **Reservation grouping (multi-resource atomic hold)** — distinct primitive; saga-shaped; not v0.
- **Distributed locks (Redis / Raft / etcd)** — explicitly out per design doc §`Out of scope`.
- **Postgres CI integration tests** — would require docker-Postgres in CI for one crate; deferred until v11.11 wraps and we can decide pragmatically.
- **MCP tool: `reservation_check_capacity`** — read-only "available capacity right now?" introspection for agents; v0.x.
- **WebSocket broadcast of reservation state to clients** — Phase 155 `ferro-projection` is the canonical home.
- **Reservation archival / retention (`prune_committed_older_than`)** — consumer-driven via raw SQL today; v0.x helper plausible.
- **Runtime `ferro-queue` integration / `ReservationSweeperJob` ready-to-register Job impl** — would push runtime deps; v0.x or framework-level addition.
- **`ferro reservation:sweep` CLI subcommand** — convenient cron entrypoint; ferro-cli scope, not ferro-reservation.
- **`ferro::prelude` / framework re-export of `ReservationKernel`** — same call as Phase 152 / 153; consumers import `ferro-reservation` directly.
- **Per-call audit suppression flag** — currently audit emission is unconditional (D-28); a future `ReservationContext::without_audit()` option is plausible if a real consumer hits the friction.
- **Capacity-aware retry** — caller territory in v0; the kernel returns `Insufficient` and the consumer decides whether to retry, queue, or surface.

### Reviewed Todos (not folded)

No todos matched this phase (`gsd-tools todo match-phase 154` returned zero matches).

</deferred>

---

*Phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl*
*Context gathered: 2026-05-13*
*Mode: --auto (single-pass, recommended defaults applied to every gray area)*

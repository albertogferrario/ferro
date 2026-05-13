# Phase 155: ferro-projection — Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults applied to every gray area)
**Milestone:** v11.11 Resource Reservation & Live Read-Model Primitives (final phase)
**Driver:** gestiscilo-it v6.7 Inventory Monitoring (Magazzino operator live dashboard) — plus the broader pattern: any app emitting domain events that wants a materialized read-model with WebSocket fanout.
**Killer feature (milestone):** Race-free reservations as a first-class framework primitive. Phase 155 is the live-read-model half of the milestone — Phase 154's reservation kernel + ferro-events + ferro-broadcast become a one-line wiring: dispatch a domain event, the projection updates, the WebSocket delta lands in the operator's browser. v0 ships the substrate; the killer feature is the *composability* with the existing event bus and broadcaster.

<domain>
## Phase Boundary

Create a new `ferro-projection` (singular) crate inside the ferro workspace that ships a **generic, domain-neutral live read-model runtime** driven by domain events with persisted snapshots and WebSocket-broadcast deltas.

The crate exposes:

- `Projection` trait — consumer-implemented read-model shape (`Event: ferro_events::Event`, `State`, `Delta`, `const NAME`, `key()`, `apply()`)
- `ProjectionKey` — opaque stringly-typed identifier (newtype around `String`) — consumer derives from the event
- `ProjectionRuntime<P: Projection>` — orchestrator with `new`, `register` (auto-wires into `global_dispatcher`), `read`, `apply_event` (manual entry point), `rebuild`
- `ProjectionError` — single `thiserror` enum, `"projection: …"` display prefix
- `CreateProjectionSnapshotsTable` — SeaORM migration consumers register in their own `Migrator`

The crate ships ONE foundational primitive: take any `ferro_events::Event`, fold it into a per-key materialized state row, persist it, fan a delta to a `projection.{name}.{key}` WebSocket channel via `ferro-broadcast`. It does NOT prescribe domain semantics — `Projection::Event`, `State`, `Delta`, and `key()` are consumer-defined; the runtime knows nothing about inventory, dashboards, slots, or any consumer schema.

**In scope:** crate scaffold, `Projection` trait, `ProjectionKey` newtype, `ProjectionRuntime<P>` orchestrator (auto-register listener path + manual `apply_event` path), per-key in-process serialization (DashMap of per-key Mutexes), snapshot persistence on every apply (write-through), `ProjectionError`, SeaORM migration (`projection_snapshots` table + PK on `(projection_name, key)`), `rebuild` from caller-supplied event iterator, broadcast delta on every apply (channel `projection.{name}.{key}`, event name from `P::broadcast_event_name()` defaulting to `"delta"`), in-memory SQLite tests, property-based tests (`proptest` — apply determinism + replay equivalence), integration test with `ferro-events` + `ferro-broadcast` end-to-end, rustdoc with explicit disambiguation from the existing `ferro-projections` (plural) crate, user-facing doc page `docs/src/features/live-read-models.md`, workspace version bump `0.2.32 → 0.2.33`, auto-publish in Wave 1b.

**Out of scope (deferred):** in-crate persistent event log (`projection_events` table) — v0 consumers supply the event stream to `rebuild` from their own source (audit log, queue logs, recovery file); cross-instance coordination (multi-node singleton, distributed actor) — v0 is in-process per-key serialized only; checkpoint-based snapshot interval (write-every-N-events) — v0 writes every apply, `snapshot_interval()` is a forward-compat hook; auth on the broadcast channel — caller wires their own `ChannelAuthorizer` via `ferro-broadcast`'s existing primitive; MCP introspection tools for projections — `db_schema` + `application_info` pick up the crate automatically; ferro-queue runtime dep for delayed/queued projection updates — consumers wire that themselves; deep-merge semantics in `apply` — pure function, no library opinion on how state evolves; multi-projection-per-event fanout coordination — register multiple runtimes, each holds its own listener; UI components for projection state — apps render their own UI, projection's contract is "state + delta stream"; Postgres CI integration tests — same call as Phases 152 / 153 / 154; tenant-scoped automatic filtering — consumers bake tenancy into `ProjectionKey`.
</domain>

<decisions>
## Implementation Decisions

### Crate placement, naming, scope

- **D-01:** Ship as a new top-level workspace crate at `ferro-projection/` (singular) — mirrors Phases 152 (`ferro-orm/`), 153 (`ferro-audit/`), 154 (`ferro-reservation/`). The roadmap, design doc (`INVENTORY-PRIMITIVES.md` §`ferro-projection`), and phase directory (`155-ferro-projection-crate-...`) all consistently use the singular form. The new crate is independently useful for any event-sourced read-model and must not require pulling the full `framework` crate.
- **D-02:** **Naming disambiguation is load-bearing.** The workspace already ships a `ferro-projections` (plural) crate from v9.0 — that crate is the *Service Projection* abstraction (`ServiceDef` → `IntentGraph` → `JsonUiRenderer`). Phase 155's `ferro-projection` (singular) is an entirely orthogonal abstraction (live read-model from event stream). The module-level rustdoc in `lib.rs` **MUST** lead with a one-paragraph "Not to be confused with" callout that names `ferro-projections` (plural), explains the orthogonality, and points the reader at the right crate for each use case. The user-facing doc page (`docs/src/features/live-read-models.md`) is titled "Live Read-Models" — not "Projections" — to keep the public surface free of singular/plural ambiguity in nav menus and search results. Same convention applies to the workspace README crate-table entry and the `CLAUDE.md` workspace-structure row: "live read-models from events" is the framing; "projection" is the implementation term.
- **D-03:** Crate is thin and additive at v0. It owns ONE table (`projection_snapshots`), one trait, one orchestrator type, one key newtype, and the matching error type. It does NOT subsume event sourcing infrastructure, CQRS write models, application-level state stores, persistent event logs, or any general-purpose actor system. `ferro-projection` v0 is the *live read-model* primitive, nothing more. The "event log" mentioned in the design doc lives in the consumer's existing storage (audit log, queue, recovery files); `rebuild` accepts an `IntoIterator<Item = P::Event>` — consumers feed it from wherever their events were recorded.
- **D-04:** Has internal ferro-* runtime deps: `ferro-events` (Wave 1a — `Event` / `Listener` traits + `global_dispatcher`) and `ferro-broadcast` (Wave 1a — `Broadcaster` + `Broadcast::channel().event().data().send()` builder). NO dep on `ferro-orm` (snapshots are full row replaces by composite PK; no guarded predicate needed; SeaORM upsert is the primitive). NO dep on `ferro-audit` (projections are derived state, not state-changing operations; the consumer's underlying events / `apply` already carry the audit story). External deps: `sea-orm` (1.0), `sea-orm-migration` (1.0), `async-trait` (0.1), `thiserror` (2), `serde` + `serde_json`, `uuid` (v4 + serde), `chrono` (serde), `tracing` (0.1), `dashmap` (6 — per-key Mutex registry), `tokio` (sync + rt features). All workspace versions; no new top-level deps beyond what ferro-events + ferro-broadcast + ferro-audit / ferro-reservation already pull in.
- **D-05:** Wave 1b publish (depends on Wave 1a crates only). Add `ferro-projection` to `WAVE1B_CRATES` in `.github/workflows/publish.yml` alongside `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`, `ferro-reservation`. New-crate-first-publish bootstrap from local terminal (CI token has publish-update only — see `project_ferro_publish_token_scoping.md`).

### Projection trait

- **D-06:** `Projection` is consumer-implemented and tied to the existing `ferro_events::Event` trait — NOT a new ferro-projection-specific `DomainEvent` trait (the design doc names `DomainEvent` but the codebase already has `ferro_events::Event` with the right bounds: `Clone + Send + Sync + 'static`). Reusing it avoids fragmenting the framework's event taxonomy.
  ```rust
  #[async_trait::async_trait]
  pub trait Projection: Send + Sync + 'static {
      type Event: ferro_events::Event + Serialize + DeserializeOwned;
      type State: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;
      type Delta: Serialize + Clone + Send + Sync + 'static;

      /// Dotted-namespace identifier, e.g. "inventory.dashboard", "checkout.cart".
      /// Persisted to `projection_snapshots.projection_name`. MUST be unique across all
      /// `Projection` impls in a single application.
      const NAME: &'static str;

      /// Derive the per-row key from the event. The runtime serializes apply per key.
      fn key(&self, event: &Self::Event) -> ProjectionKey;

      /// Fold the event into the running state and return the delta to broadcast.
      /// Pure function — must NOT perform IO or block. The runtime calls this inside
      /// a per-key Mutex; long-running work blocks ALL events for that key.
      fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta;

      /// Reserved for v0.x event-log-backed snapshot mode. v0 ignores this — every
      /// apply persists. Default returns 100 so v0.x flips on this trait method
      /// without breaking consumers who never overrode it.
      fn snapshot_interval(&self) -> u32 { 100 }

      /// Event name used for the broadcast frame. Defaults to "delta". Consumers can
      /// override if their frontend dispatches on the event name (e.g., "dashboard_updated").
      fn broadcast_event_name(&self) -> &'static str { "delta" }
  }
  ```
- **D-07:** `State: Default` is required so a fresh key (no persisted snapshot) initializes from `P::State::default()`. This sidesteps an `Option<State>` API on first-apply and matches the natural semantic of "empty dashboard, no items yet". Documented as a hard convention: every projection state must have a sensible default. If a consumer's state model has no sensible default, they implement `Default` to return an empty/zero variant and treat the first event as an initializer.
- **D-08:** `apply` is **synchronous** (not async). Justifications: (a) the runtime calls `apply` inside a per-key Mutex — async would let the lock cross await boundaries and serialize unrelated work; (b) `apply` is a pure state fold by design, with no IO; (c) async traits cost a `Box::pin` per call which is wasteful for a hot fold path; (d) the design doc's pseudocode shows a sync `apply`. Consumers wanting async work (HTTP fetches, additional DB queries) do that BEFORE `dispatch`-ing the event or AFTER receiving the broadcast delta, not inside `apply`.
- **D-09:** `Projection::Event: Serialize + DeserializeOwned` (beyond the `ferro_events::Event` requirement). Rationale: `rebuild` takes an iterator of events that may come from JSON-serialized storage (audit log entries, queue payloads); deserializing them is the consumer's job. The extra bounds are cheap (most ferro events are already `serde`-derived) and unlock the rebuild path without per-consumer wrappers.
- **D-10:** `apply` returns `P::Delta` — the runtime broadcasts this. The delta is the consumer's choice: full state, JSON Patch ([RFC 6902](https://www.rfc-editor.org/rfc/rfc6902)), a minimal struct describing the change, anything `Serialize`. v0 is opinion-free on delta shape; it's a `Serialize + Clone` consumer payload. The convention documented in the rustdoc: small delta payloads are friendlier to high-frequency event streams; full-state deltas are simpler when frequency is low. Consumers pick.

### ProjectionKey

- **D-11:** `ProjectionKey` is a stringly-typed newtype, NOT a generic associated type or a typed enum. Rationale: matches ferro-audit's `AuditTarget::id: String` (Phase 153 D-07), Phase 154's `Resource::Key` JSON-shaped opacity at the event-bus boundary, and the broader project-agnostic crate principle (CLAUDE.md §`Architecture Principles`). A typed `Key` per projection would force the runtime to carry a generic key parameter through the broadcast channel name, the DB column, and the per-key Mutex map — adding noise for no benefit.
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct ProjectionKey(String);

  impl ProjectionKey {
      pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
      pub fn as_str(&self) -> &str { &self.0 }
  }

  impl fmt::Display for ProjectionKey { ... }
  impl From<String> for ProjectionKey { ... }
  impl From<&str> for ProjectionKey { ... }
  ```
- **D-12:** Consumer is responsible for stringifying any compound key. Convention: dotted-or-colon namespace, e.g. `"inventory.warehouse-a"`, `"checkout.user-42.cart"`, `"tenant-7:dashboard.summary"`. Documented as a convention; not enforced at compile time. Multi-tenancy lives inside the key string (D-37 of Phase 154's pattern, mirrored here) — the runtime does not auto-scope by tenant.

### ProjectionRuntime API

- **D-13:** `ProjectionRuntime<P: Projection>` owns the database connection, the broadcaster handle, the projection impl, and the per-key Mutex registry:
  ```rust
  pub struct ProjectionRuntime<P: Projection> {
      db: DatabaseConnection,
      broadcaster: Arc<Broadcaster>,
      projection: P,
      locks: DashMap<String, Arc<Mutex<()>>>,
  }

  impl<P: Projection> ProjectionRuntime<P> {
      pub fn new(db: DatabaseConnection, broadcaster: Arc<Broadcaster>, projection: P) -> Self;
  }
  ```
  Per-call methods accept either an event or a key; the `db` is owned (not `&mut`) so the runtime can be shared via `Arc` across listeners and tokio tasks. The broadcaster is `Arc<Broadcaster>` because `ferro_broadcast::Broadcast::new` requires that shape.
- **D-14:** **Two entry points** to applying an event:
  1. **`register(self: Arc<Self>)`** — wires a `ProjectionListener<P>` into the global event dispatcher (`ferro_events::global_dispatcher()`). Default path; one-line wiring; the killer feature. After `register`, every `P::Event::dispatch().await` flows through the projection automatically.
  2. **`async fn apply_event(&self, event: &P::Event) -> Result<(), ProjectionError>`** — manual entry point for tests, replay scripts, custom dispatchers, or consumers who don't want global-listener registration. The auto-registered listener (D-15) calls this method.

  Both paths share the same per-key serialization, snapshot persistence, and broadcast logic. Documented as "pick one or both; mixing is safe".
- **D-15:** `register` constructs a `ProjectionListener<P>` holding `Arc<ProjectionRuntime<P>>` and calls `global_dispatcher().listen(listener)`:
  ```rust
  struct ProjectionListener<P: Projection> {
      runtime: Arc<ProjectionRuntime<P>>,
  }

  #[async_trait::async_trait]
  impl<P: Projection> Listener<P::Event> for ProjectionListener<P> {
      async fn handle(&self, event: &P::Event) -> Result<(), ferro_events::Error> {
          self.runtime.apply_event(event).await
              .map_err(|e| ferro_events::Error::listener_failed(
                  std::any::type_name::<ProjectionListener<P>>(),
                  e.to_string(),
              ))
      }
  }
  ```
  The runtime owns the cycle through `Arc<Self>`; `register` consumes one `Arc::clone` for the listener and is idempotent across multiple calls (callers might inadvertently double-register; the dispatcher tolerates this — same as Laravel's listen API — and runs each registration once per event). Document the duplicate-registration tolerance.
- **D-16:** `async fn read(&self, key: &ProjectionKey) -> Result<Option<P::State>, ProjectionError>` — reads the persisted snapshot from `projection_snapshots`. Returns `None` if no row exists for `(P::NAME, key)`. Subscribers fetch the initial snapshot via this method, then apply deltas client-side. No locking — `read` is the public read path and may run concurrently with `apply_event`; readers see whichever snapshot was committed last (no torn reads — JSON column is atomic at the DB level).
- **D-17:** `async fn rebuild<I>(&self, key: &ProjectionKey, events: I) -> Result<P::State, ProjectionError>` where `I: IntoIterator<Item = P::Event>` — discards the persisted snapshot for `key`, folds the supplied event sequence through `P::State::default()` via `apply`, persists the final state, broadcasts ONE delta carrying the full final state (special-case event name `"rebuild"` instead of the default), returns the rebuilt state. Used for schema changes or after audit-detected divergence. Acquires the same per-key Mutex as `apply_event` (so rebuild serializes against in-flight applies). Consumer supplies the event stream from their own source (audit log replay, queue logs, recovery file). The crate does NOT prescribe where the events come from — `rebuild` is the integration point.
- **D-18:** Runtime is `Clone + Send + Sync` only when the underlying `DatabaseConnection` and `Arc<Broadcaster>` are (both already are). Consumers wrap the runtime in `Arc` if they need cheap sharing across tasks (the auto-registered listener already requires `Arc<Self>`).

### Apply algorithm (single source of correctness)

- **D-19:** `apply_event` executes the following sequence inside the per-key Mutex:
  1. **Compute the key** via `self.projection.key(event)` (cheap, sync).
  2. **Acquire the per-key Mutex** from `self.locks.entry(key.0.clone()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()`, then `.lock().await`. Held across the snapshot read, apply, write, and broadcast — strictly per-key serialization.
  3. **Load the snapshot** from `projection_snapshots` via `Entity::find_by_id((P::NAME, &key.0))`. If absent, start from `P::State::default()`.
  4. **Apply the event**: `let delta = self.projection.apply(&mut state, event);` (sync, fast).
  5. **Persist the new state** via SeaORM upsert on `(projection_name, key)`. `state` is serialized to `serde_json::Value`; `version` increments by 1; `updated_at = Utc::now()`. This is a single SQL statement (`INSERT … ON CONFLICT (projection_name, key) DO UPDATE …` for Postgres / `INSERT … ON CONFLICT(projection_name, key) DO UPDATE …` for SQLite — both via SeaORM's `on_conflict` builder).
  6. **Broadcast the delta** via `Broadcast::new(self.broadcaster.clone()).channel(channel_name).event(P::broadcast_event_name()).data(delta).send().await`. Channel name: `format!("projection.{}.{}", P::NAME, key.as_str())`.
  7. **Release the Mutex** (RAII on drop) before returning.
- **D-20:** **Per-key in-process serialization is the entire correctness mechanism.** No optimistic concurrency control (OCC), no distributed lock, no transactional read-then-write. The Mutex guarantees that for any given key, the load/apply/persist/broadcast sequence is uninterrupted. Concurrent events on DIFFERENT keys run in parallel (DashMap shards). v0 explicitly does NOT support cross-instance coordination — multi-instance deployments need to elect a single projection-runner node (or accept eventual consistency at the snapshot row, last-writer-wins). Documented as the v0 constraint; v0.x adds an optimistic-concurrency `version` column check that fails apply if the row's version has advanced beyond what was loaded, letting multi-instance deployments retry or quarantine.
- **D-21:** **Broadcast failure does NOT roll back state.** If `Broadcast::send` returns `Err`, the snapshot row is already persisted; the runtime logs `tracing::warn!(error=%e, channel, "projection broadcast failed; snapshot persisted")` and surfaces `ProjectionError::Broadcast(message)`. Rationale: subscribers can always reconcile by re-reading the snapshot; losing a single delta does not corrupt the projection. This mirrors Phase 154 D-26 (event-bus failure does not roll back reservations).
- **D-22:** **Database failure DOES surface as an error.** If the snapshot upsert fails (DB down, constraint violation, JSON serialization), `apply_event` returns `ProjectionError::Db` and skips the broadcast. The consumer's listener-failure path (via `register`) maps this to `ferro_events::Error::listener_failed`, which the global dispatcher treats per its normal listener-error semantic. Document that consumers wanting to never block a downstream listener register the projection with a separate dispatcher or a `dispatch_async` path — out-of-scope for v0.

### Schema & migration

- **D-23:** Ship a SeaORM migration as a public re-export so consumers register it explicitly in their `Migrator`:
  ```rust
  pub use migration::Migration as CreateProjectionSnapshotsTable;
  ```
  Mirrors Phase 153 D-18 / Phase 154 D-38. Consumer's migrator:
  ```rust
  vec![
      Box::new(ferro_audit::CreateAuditLogTable),
      Box::new(ferro_reservation::CreateReservationsTable),
      Box::new(ferro_projection::CreateProjectionSnapshotsTable),
      // ... app migrations
  ]
  ```
- **D-24:** Schema columns:
  ```
  projection_snapshots
  ├── projection_name VARCHAR NOT NULL              -- P::NAME ("inventory.dashboard")
  ├── key             VARCHAR NOT NULL              -- ProjectionKey.as_str()
  ├── state           JSON NOT NULL                 -- serialized P::State
  ├── version         BIGINT NOT NULL               -- monotonic counter, +1 per apply; reset on rebuild
  ├── updated_at      TIMESTAMP NOT NULL            -- app-set Utc::now() inside the upsert
  ├── PRIMARY KEY (projection_name, key)            -- composite PK; covers the only lookup path
  ```
  No additional secondary indexes in v0. Every access path (read, upsert, rebuild) hits the composite PK directly.
- **D-25:** `version` is forward-compat scaffolding for v0.x optimistic concurrency (D-20). v0 increments it on every apply but does not use it as a guarded predicate — the per-key Mutex is sufficient. A v0.x release can add `WHERE version = $loaded_version` to the upsert without a schema change.
- **D-26:** `state` is JSON (not BLOB / TEXT). SQLite stores JSON as TEXT under the hood but the SeaORM column type is `Json`; Postgres uses native `JSONB`. Matches Phase 153 D-19 / Phase 154 D-39 — JSON columns are the workspace convention for serialized payloads.
- **D-27:** Migration sets `updated_at` default to `CURRENT_TIMESTAMP` on the column definition but the application explicitly sets `updated_at = Utc::now()` inside the upsert (D-19 step 5) — same rationale as Phase 154 D-42: application time on every update avoids dialect-specific `CURRENT_TIMESTAMP` SeaORM expression handling.

### Error model

- **D-28:** `ProjectionError` is a `thiserror`-derived enum, one error per crate, panics nowhere:
  ```rust
  pub enum ProjectionError {
      #[error("projection: db error: {0}")]
      Db(#[from] sea_orm::DbErr),

      #[error("projection: json error: {0}")]
      Json(#[from] serde_json::Error),

      #[error("projection: broadcast error: {0}")]
      Broadcast(String),                          // ferro_broadcast::Error mapped via to_string()

      #[error("projection: events error: {0}")]
      Events(String),                             // ferro_events::Error mapped via to_string()

      #[error("projection: state not found for {name}/{key}")]
      StateNotFound { name: &'static str, key: String },
  }
  ```
  Display prefix `"projection: …"` for grep-friendliness across the workspace (matches `"guarded: …"`, `"audit: …"`, `"reservation: …"`).
- **D-29:** `ProjectionError::Broadcast` and `ProjectionError::Events` are `String`-payload variants (not `#[from]`) because the source crates' `Error` enums are not `Send + Sync + 'static` in a way that composes cleanly through `thiserror::From` — same pragmatic choice Phase 149 made for `Error::Broadcast(String)` in `ferro-notifications`. `From<ferro_broadcast::Error>` is implemented by hand to call `to_string()`.
- **D-30:** `StateNotFound` is reserved for an explicit `read_required` helper if a consumer wants `Result<State, _>` rather than `Result<Option<State>, _>`. Not in v0 surface; the variant exists so v0.x can ship `read_required` without a breaking enum change.

### Concurrency & in-process serialization

- **D-31:** Per-key Mutex registry uses `DashMap<String, Arc<Mutex<()>>>` keyed on `ProjectionKey.as_str()`. The entry is created lazily on first event for that key. The runtime never removes entries (memory cost: one `Arc<Mutex>` per ever-seen key — bounded by the consumer's key cardinality). Documented as the v0 constraint; v0.x can add a key-eviction policy if a real consumer hits memory pressure.
- **D-32:** Two `apply_event` calls on the SAME key serialize. Two `apply_event` calls on DIFFERENT keys parallelize (DashMap allows concurrent access; per-key Mutexes are independent). The integration test (D-48) proves this end-to-end with 20 tokio tasks across 5 keys.
- **D-33:** `read` does NOT take the per-key Mutex. Concurrent `read` + `apply_event` is safe: the SQL JSON column upsert is atomic at the DB level; the reader sees either the pre-upsert or post-upsert state. There is no half-written snapshot because the upsert is one statement.
- **D-34:** **Single-instance assumption is explicit.** The rustdoc, the user-facing doc page, and the `lib.rs` module comment all state: "v0 ferro-projection assumes a single application instance owns each projection's listener. Multi-instance deployments must elect a single projection-runner node or accept last-writer-wins behavior on concurrent applies to the same key from different nodes." This is the load-bearing operational caveat for v0.

### Listener registration

- **D-35:** `register(self: Arc<Self>)` consumes one `Arc::clone` to build a `ProjectionListener<P>` and registers it via `ferro_events::global_dispatcher().listen(listener)`. The listener is stored type-erased in the dispatcher's internal map (keyed by `TypeId::of::<P::Event>()`). Multiple distinct projections that share an event type all receive each dispatch.
- **D-36:** Calling `register` twice on the same `Arc<ProjectionRuntime<P>>` registers two listeners — both will fire on each dispatch. This matches the existing `EventDispatcher::listen` semantic (no deduplication on listener identity — same as Laravel's `Event::listen`). Documented in rustdoc with a "register once at app startup" recommendation. The runtime does not maintain an `is_registered` flag because the dispatcher is global and the runtime is `Send`-able across construction sites; a flag would lie under realistic usage patterns.
- **D-37:** **No `unregister` API in v0.** ferro-events' dispatcher doesn't expose per-listener removal (only `forget::<E>()` which removes ALL listeners for an event type). v0 ferro-projection lives within that constraint — projections register at startup and live for the app's lifetime. A v0.x `unregister` would require coordination with ferro-events.

### Broadcast contract

- **D-38:** Broadcast channel name: `format!("projection.{}.{}", P::NAME, key.as_str())`. Single-dot separator after the literal `"projection"` prefix, then the projection name, then the key. Example: `"projection.inventory.dashboard.warehouse-a"`. Documented in the rustdoc with the expected client-side subscription pattern. Public channel (no `private-` / `presence-` prefix); consumers wanting private channels override the projection-side channel naming via a future `Projection::channel_for(&key)` method (deferred to v0.x — not in surface today).
- **D-39:** Broadcast event name from `Projection::broadcast_event_name()`, defaulting to `"delta"`. Override returns `&'static str`. Common overrides documented in rustdoc: `"dashboard_updated"`, `"cart_changed"`, `"row_patched"`. The fixed default keeps a consumer's frontend code simple in the common case (subscribe → listen for `"delta"`).
- **D-40:** Broadcast payload is the JSON-serialized `P::Delta`. The runtime does NOT wrap the delta in an envelope (no `{ type, version, state, delta }` outer object) — the frontend receives raw `Delta` JSON on the channel/event tuple. Consumers wanting an envelope make `P::Delta` a struct that carries one. Keeps ferro-projection out of payload-shape opinions.
- **D-41:** `rebuild` broadcasts a single frame on the channel with event name `"rebuild"` (overriding the configured `broadcast_event_name`). Payload is the final `P::State` serialized as JSON. Subscribers reset their local state on `"rebuild"` events. Documented as the rebuild client-side contract.

### Rebuild semantics

- **D-42:** `rebuild(key, events)` discards the existing snapshot row entirely (DELETE then INSERT, or DELETE then upsert-with-default-then-fold — equivalent under the per-key Mutex). The `version` resets to `events.len() as i64` (or zero if the iterator is empty). Caller-supplied event order is the canonical order — `rebuild` does not sort.
- **D-43:** Calling `rebuild` with an empty iterator deletes the snapshot row (state becomes `P::State::default()` on next `read` → `None`). Documented in rustdoc as the "wipe this key" affordance. Returns `P::State::default()`.
- **D-44:** `rebuild` is NOT transactional across the delete + folded-upsert. v0 takes the per-key Mutex, does DELETE, then folds-and-upserts. If the process crashes mid-rebuild, the snapshot is gone but the per-key state in memory is also gone — a subsequent `apply_event` re-initializes from `Default`. Documented as the v0 crash semantic; v0.x can add a single-transaction rebuild if a consumer hits the gap.

### Testing

- **D-45:** Unit tests live next to the code (`#[cfg(test)] mod tests`) in `ferro-projection/src/`. Cover:
  1. `ProjectionKey::new` + `as_str` + `Display` round-trip + Serde round-trip.
  2. `ProjectionError` Display strings start with `"projection: …"`.
  3. Trait method defaults: `snapshot_interval()` returns 100, `broadcast_event_name()` returns `"delta"`.
  4. Runtime construction: `ProjectionRuntime::new(db, broadcaster, projection)` returns owned runtime; `Arc<Runtime>` is `Send + Sync`.
  5. `apply_event` happy path: empty snapshot table → apply one event → snapshot row exists with `version = 1`, `state` matches expected, broadcast was called once with the expected channel + event + data.
  6. `apply_event` second call on the same key folds onto the loaded state (not `Default`); `version = 2`.
  7. `apply_event` on a new key initializes from `Default`.
  8. `read` returns `None` for an absent key; `Some(state)` after `apply_event`.
  9. `rebuild` with three events produces the same final state as three sequential `apply_event` calls.
  10. `rebuild` with empty iterator deletes the row; subsequent `read` returns `None`.
- **D-46:** Integration test (`tests/event_bus_integration.rs`) — proves the auto-register path end-to-end:
  - Build a minimal `Projection` impl that counts events per key.
  - Build a `Broadcaster` and tap into a `BroadcastMessage` receiver (use the existing ferro-broadcast testing pattern — search the existing `ferro-broadcast/src/broadcaster.rs` test module for the closest pattern; if none, derive a minimal collector via the `Broadcaster::client_count` / channel inspection surface, or hand-roll a `Client` mock).
  - `Arc::new(Runtime::new(db, broadcaster, projection)).register()`.
  - Dispatch 5 events via `Event::dispatch().await`.
  - Assert: 5 snapshot upserts persisted, 5 broadcast frames captured on the expected channel.
- **D-47:** Cross-crate showcase test (`tests/projection_over_reservation_events.rs`) — composes ferro-projection with Phase 154's `ReservationEvent`:
  - Define a tiny `ReservationCountProjection` that maintains per-`resource_kind` held / committed / released counts driven by `ReservationEvent::{Held, Committed, Released}`.
  - Use the in-memory SQLite testing harness, wire `reservations` + `projection_snapshots` migrations.
  - Hold 3 reservations, commit 1, release 1 — assert the projection state and the broadcast frames match.
  - This is the milestone-completing demonstration: reservations emit events, projections fold them, deltas land on broadcast channels — the full v11.11 chain in 60 lines.
- **D-48:** Concurrency integration test (`tests/concurrent_apply.rs`) — proves D-31 / D-32 / D-33 end-to-end:
  - Spawn 20 tokio tasks across 5 distinct keys (4 tasks per key), each calling `apply_event` on a counter-event.
  - Assert: each key's final count is exactly 4; total snapshot rows = 5; no panics, no errors.
  - Verifies per-key serialization (no lost increments) AND cross-key parallelism (no global serialization).
- **D-49:** Property-based tests via `proptest` (dev-dep already added in Phase 154 D-49):
  - **Property 1 (apply determinism):** For any random sequence of events `[E0, E1, …, EN]` on a single key, applying them sequentially yields the same final state as folding them through `apply` with no DB/broadcast involvement. Implemented as a pure-fold reference and an integration-fold comparison.
  - **Property 2 (replay equivalence):** For any random sequence of events on a single key, calling `apply_event` for each yields the same final persisted state as a single `rebuild(key, events)` call. Confirms the rebuild path is a faithful replay.
  - **Property 3 (cross-key independence):** For any random partitioning of events across N keys (N ∈ [1, 10]), the per-key final states are identical regardless of the interleaving order between keys. Confirms per-key isolation.
- **D-50:** Test harness: in-memory SQLite, re-derive the harness inline (do not depend on `framework`). Each test sets up `Migrator` registering `CreateProjectionSnapshotsTable` (plus the integration test's `CreateReservationsTable` for D-47). `Broadcaster` is constructed fresh per test; tests that need to capture broadcasts wire a lightweight collector. Postgres CI integration tests deferred (same call as Phase 152 D-19 / Phase 153 D-33 / Phase 154 D-51).

### Documentation

- **D-51:** Module-level rustdoc on `lib.rs` opens with the disambiguation paragraph (D-02): "**Not to be confused with `ferro-projections` (plural).** That crate is the Service Projection abstraction (ServiceDef → IntentGraph → JsonUiRenderer). This crate (`ferro-projection`, singular) is the live read-model runtime that subscribes to domain events, maintains a materialized state, and broadcasts deltas. The two abstractions are orthogonal — most apps will use both for different reasons." Then leads with the *why* (every app emitting events that wants a dashboard ends up hand-rolling load-apply-persist-broadcast; this is the typed kernel), then shows the one-call API (Projection impl + `Arc::new(Runtime::new(...)).register()`). Includes the per-key serialization diagram and the broadcast-channel naming convention. Documents the three operational footguns: (1) broadcast failure does NOT roll back state, (2) single-instance assumption, (3) `register` is not idempotent on `Arc` identity.
- **D-52:** New user-facing doc page `docs/src/features/live-read-models.md` (title: "Live Read-Models"). Placement: under `features/` (sibling of `features/projections.md` which covers the v9.0 Service Projection abstraction). Page-level note clarifies the singular/plural distinction once and links to the other page. Content: what a live read-model is, when you want one, defining a `Projection`, wiring the runtime, the broadcast channel contract, the rebuild affordance, the operational footguns, a worked example (reservation-count dashboard composing ferro-reservation + ferro-projection — same shape as the D-47 showcase test, expanded into a tutorial).
- **D-53:** ferro-mcp introspection: no new MCP tools in this phase. `application_info` auto-includes `ferro-projection` in `installed_crates`; `db_schema` picks up the `projection_snapshots` table; `generation_context` / `code_templates` pick up the rustdoc automatically. A future `list_projections` MCP tool (read-only, "what projections are wired in this app?") is plausible in v0.x once the agent use case surfaces — likely needs a registration registry that v0 does not maintain (D-37's constraint).

### Release

- **D-54:** Workspace `[workspace.package] version` bumps one patch (from `0.2.32` to `0.2.33`) when Phase 155 verifies. Standard ferro release process; matches the cadence Phases 152 (→ 0.2.30) / 153 (→ 0.2.31) / 154 (→ 0.2.32) established.
- **D-55:** Add `ferro-projection` to **Wave 1b** of `.github/workflows/publish.yml` (D-05). New-crate bootstrap from local terminal — same operational reality as Phases 151 / 152 / 153 / 154.
- **D-56:** CHANGELOG entry under `ferro-projection` (new section, placed at the top per Phase 152 D-25 convention) summarising: new crate, `Projection` trait for live read-models from domain events, `ProjectionRuntime<P>` with auto-register + manual `apply_event` paths, per-key in-process serialization, snapshot persistence with version counter, delta broadcast on `projection.{name}.{key}` channels, `rebuild` for replay from a caller-supplied event stream, explicit disambiguation from `ferro-projections` (plural). Also notes the milestone-completion line: "v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm GuardedUpdate, ferro-audit, ferro-reservation, ferro-projection now all shipped."

### Folded scope from todos

No pending todos matched Phase 155 (`gsd-tools todo match-phase 155` returned zero matches at gather time).

### Claude's Discretion

Within the boundaries set above, the planner/executor decides:

- Internal module layout of `ferro-projection/src/` (likely `lib.rs` + `projection.rs` (trait) + `key.rs` + `runtime.rs` + `listener.rs` + `entity.rs` + `migration.rs` + `error.rs`, but the planner is free to consolidate where the public surface is unchanged)
- Whether to expose the SeaORM `Entity` / `Model` / `ActiveModel` types as a public re-export for consumers wanting native SeaORM queries against `projection_snapshots` (recommended; matches Phase 153 / 154)
- Exact `tracing::warn!` / `tracing::error!` wording on broadcast / DB / events failures
- Exact `proptest` strategy shape (the three properties are locked in D-49; the generator construction is open)
- Test file names within `ferro-projection/tests/`
- Whether to ship a `ProjectionRuntime::is_registered` introspection method (recommended NO; D-36's note that the flag would lie under realistic usage stands)
- Whether `ProjectionListener<P>` is a public type (recommended NO — it's an implementation detail of `register`; consumers don't construct one directly)
- Whether to ship a `ProjectionRuntime::read_required(key) -> Result<P::State, ProjectionError>` helper alongside `read` (recommended YES — uses `StateNotFound` variant from D-30; small surface, clear ergonomic win)
- Exact rustdoc prose & code-block formatting

### Deferred (NOT in this phase)

- **Persistent in-crate event log (`projection_events` table)** — v0 stays snapshot-only; `rebuild` accepts a caller-supplied iterator. A `projection_events` table that ferro-projection writes to on every apply is a v0.x or v1.x addition, blocking on a concrete consumer use case for replay-from-event-log.
- **Checkpoint-based snapshot interval (write-every-N-events)** — v0 writes every apply; `snapshot_interval()` exists as a forward-compat hook. v0.x flips this on once the event-log table lands.
- **Optimistic concurrency control on the version column** — v0 relies on per-key Mutex; v0.x adds `WHERE version = $loaded_version` to the upsert without schema change.
- **Cross-instance projection coordination** — v0 is single-instance; multi-node deployments elect a singleton runner or accept LWW. Cross-instance coordination (Redis lock, distributed actor, kafka-style partitioned consumer) is its own framework feature, out-of-scope for v11.11.
- **Listener unregistration** — gated on ferro-events shipping a per-listener removal API. v0 lives within the registered-for-life constraint.
- **`Projection::channel_for(&key)` override** — for private / presence channels. v0 uses the public `projection.{name}.{key}` convention.
- **Tenant-scoped automatic filtering** — consumers bake tenancy into the `ProjectionKey` string (same call as Phase 153 / 154's tenant treatment).
- **Postgres CI integration tests** — same call as Phases 152 / 153 / 154.
- **MCP tool: `list_projections`** — needs a projection registry the runtime doesn't currently maintain; v0.x.
- **`ferro::prelude` / framework re-export of `ProjectionRuntime`** — same call as Phases 152 / 153 / 154; consumers import `ferro-projection` directly.
- **`audit_log!`-style macro façade for `Projection` definitions** — the trait is small enough; a macro adds complexity for marginal ergonomic gain.
- **Deep-merge / patch-based `apply` semantics** — `apply` is pure-function consumer territory; ferro-projection does not opine.
- **UI components for projection state rendering** — apps render their own UI; projection's contract is "state + delta stream".
- **Multi-projection-per-event coordination primitives** — multiple `ProjectionRuntime::register` calls already work; no extra orchestration needed in v0.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design source of truth

- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-projection` — original spec (trait shape, snapshot strategy, broadcast contract, rebuild). Authoritative.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Cross-crate relationships` — confirms ferro-projection depends on ferro-events + ferro-broadcast + sea-orm; independent of Phases 152 / 153 / 154 in code but typically deployed alongside.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Testing strategy` — per-crate apply-determinism + snapshot/replay equivalence + property-based tests. Phase 155 carries the milestone's projection-side property-test budget.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Migration / rollout` — confirms ferro-projection ships in parallel with ferro-reservation, additive, no breaking changes.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Out of scope` — explicit non-goals: no UI components, no cross-instance replication.

### Project conventions

- `CLAUDE.md` §`Architecture Principles` — project-agnostic crates rule (no hardcoded app identity, no consumer-specific types in the public API). `ferro-projection` must not bind to inventory, dashboards, slots, tickets, or any consumer-specific id shape.
- `CLAUDE.md` §`Testing & Linting` — exact fmt + clippy + test commands required pre-commit. Applies identically to ferro-projection.
- `CLAUDE.md` §`Workspace Structure` — ferro-projection is added to this table during execution.
- `CLAUDE.md` §`Vision Anchors` — "core abstraction is projection / intent (`ferro-projections`, plural)". The plural's prime real-estate in the framework's vocabulary makes the singular's naming ambiguity load-bearing; D-02 / D-51 / D-52 spend explicit surface area on disambiguation.
- `.planning/PROJECT.md` — vision anchors; the projection/intent abstraction is the killer feature this milestone unblocks (via reservations + live read-models). ferro-projection (singular) is the live-read-model side of that promise; ferro-projections (plural) is the projection/intent side. Both are pillars; the framework deliberately ships both with similar names to signal a shared philosophical lineage.
- `.planning/STATE.md` — current workspace version (`0.2.32` post-154), next version is `0.2.33` after Phase 155 verifies.

### Sibling phase context (must read before planning)

- `.planning/phases/154-ferro-reservation-crate-generic-hold-commit-release-with-ttl/154-CONTEXT.md` — Phase 154 is the **structural twin** for Wave 1b crate scaffolding, error-naming convention (`"projection: …"` mirrors `"reservation: …"`), Cargo.toml metadata shape, migration-as-public-re-export pattern (D-23 mirrors 154 D-38), publish.yml Wave 1b placement, CHANGELOG shape, doc-page placement under `docs/src/features/`. `ReservationEvent` (Phase 154 D-25) is the cross-crate showcase event in D-47.
- `.planning/phases/153-ferro-audit-crate-structured-before-after-audit-log-with-rep/153-CONTEXT.md` — Phase 153 is the precedent for the migration-as-public-re-export pattern + JSON column convention + stringly-typed identifier rationale (D-11 mirrors 153 D-13 / D-08). ferro-audit and ferro-projection are sibling primitives the consumer composes (events emitted, events audited, events folded into projections).
- `.planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-CONTEXT.md` — Phase 152 is the precedent for the crate-scaffolding shape, Wave 1a/1b distinction, and the broader "single-purpose primitive crate" pattern. ferro-projection does NOT depend on ferro-orm (D-04 rationale), but the surface shape is the same.

### Patterns to mirror (template ferro-* crates)

- `ferro-reservation/Cargo.toml` — closest Wave-1b sibling Cargo.toml shape; ferro-projection's Cargo.toml mirrors it (same metadata fields, replaces ferro-orm + ferro-events + ferro-audit deps with ferro-events + ferro-broadcast).
- `ferro-reservation/src/lib.rs` — module-level rustdoc tone for a v0 single-purpose Wave-1b crate; ferro-projection's lib.rs follows the same shape with the added disambiguation paragraph (D-02 / D-51).
- `ferro-audit/Cargo.toml` — second reference for database-adjacent crate Cargo.toml shape (sea-orm + sea-orm-migration + serde + uuid + chrono + tracing).
- `ferro-audit/src/migration.rs` — SeaORM migration shape; `CreateProjectionSnapshotsTable` mirrors `CreateAuditLogTable`. Composite PK on `(projection_name, key)` is the v0 difference from ferro-audit's single-column UUID PK.
- `ferro-audit/src/entity.rs` — SeaORM entity definition with JSON columns + nullable timestamps; `projection_snapshots::Entity` mirrors the JSON-column pattern.
- `ferro-events/src/dispatcher.rs` — `EventDispatcher::listen<E, L>` is the listener registration API (D-15); `global_dispatcher()` returns the singleton instance.
- `ferro-events/src/traits.rs` — `Event` and `Listener<E>` traits — `Projection::Event: Event` (D-06), `ProjectionListener<P>: Listener<P::Event>` (D-15).
- `ferro-events/src/error.rs` — `Error::listener_failed(listener, message)` constructor — used by `ProjectionListener::handle` to map `ProjectionError` (D-15).
- `ferro-broadcast/src/broadcast.rs` — `Broadcast::new(broadcaster).channel(name).event(event_name).data(payload).send().await` is the broadcast builder API used by D-19 step 6.
- `ferro-broadcast/src/broadcaster.rs` — `Broadcaster::new()` + `Arc<Broadcaster>` is the construction shape; ferro-projection's `Runtime::new` accepts this.
- `ferro-notifications/src/error.rs` — precedent for `String`-payload variants when the source `Error` doesn't compose cleanly through `thiserror::From` (D-29 cites this).
- `.github/workflows/publish.yml` — Wave 1b crate list (`WAVE1B_CRATES`); ferro-projection is added alongside `ferro-reservation`.
- `framework/src/database/testing.rs` — in-memory SQLite testing harness reference; ferro-projection re-derives the harness inline (no `framework` dep).

### Cross-phase coordination

- **Phase 152 (shipped, `ferro-orm 0.2.32`)**: NOT a dep — snapshots are full row upserts, no guarded predicate needed (D-04).
- **Phase 153 (shipped, `ferro-audit 0.2.32`)**: NOT a dep — projections are derived state, not state-changing. Consumers compose ferro-audit and ferro-projection in their own application code if they want both.
- **Phase 154 (shipped, `ferro-reservation 0.2.32`)**: NOT a dep — but `ReservationEvent` is the canonical event type used in the cross-crate showcase test (D-47) and the user-facing doc page (D-52). The integration test demonstrates the full v11.11 chain end-to-end.
- **ferro-events (existing Wave 1a)**: dispatcher + `Event` / `Listener` traits — the load-bearing dep. `Projection::Event: ferro_events::Event` (D-06); `ProjectionListener<P>` implements `ferro_events::Listener<P::Event>` (D-15).
- **ferro-broadcast (existing Wave 1a)**: `Broadcaster` + `Broadcast` builder — the other load-bearing dep. Used in D-19 step 6 for delta fanout.

### Conventions repository (operator memory)

- `feedback_ci_clippy_command_match.md` — match CI's exact clippy command (`--all --all-targets -- -D warnings`) in pre-push checks.
- `feedback_validate_scope_premises.md` — `ferro-projection` does not currently exist as a crate; verified at gather time via `ls ferro-projection` (not found) and `grep -rl "ferro_projection\|ferro-projection\b" --include="*.rs" --include="*.toml"` (only references in this design doc, roadmap, and Phase 154 deferred section). Greenfield.
- `project_ferro_publish_token_scoping.md` — CI publish token has publish-update only; new-crate bootstrap requires personal `publish-new`-scoped token from a local terminal.
- `feedback_macbook_thermal_ferro_builds.md` — pace long auto-advance chains; Phase 155's plan / execute sequence compiles a fresh crate plus runs property tests — moderate thermal cost expected. Pause if thermal stress is signalled.
- `feedback_killer_feature_framing.md` — ferro-projection is the *composability* killer feature of v11.11: events + reservations + broadcaster + audit all wire into a live dashboard in one Arc::new + register call. Frame the rustdoc and doc page around that.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets

- **`ferro_events::Event`** (Wave 1a, shipped) — `Clone + Send + Sync + 'static` bound + `name() -> &'static str`. Projection's `Event` associated type bounds onto this trait (D-06). Ergonomic `.dispatch().await` extension method already in place — consumers don't change dispatch sites to flow through projections; `register` is the only wiring.
- **`ferro_events::Listener<E>`** (Wave 1a, shipped) — `async fn handle(&self, event: &E) -> Result<(), Error>`. `ProjectionListener<P>` implements this trait (D-15).
- **`ferro_events::global_dispatcher()`** (Wave 1a, shipped) — returns `&'static EventDispatcher`; `listen<E, L>` registers a listener at runtime. Idempotent at the API level but each call adds an entry — D-36 documents this for consumers.
- **`ferro_events::Error::listener_failed(listener, message)`** (Wave 1a, shipped) — constructor used by `ProjectionListener::handle` to map `ProjectionError` (D-15).
- **`ferro_broadcast::Broadcaster::new()`** (Wave 1a, shipped) — caller constructs; `Arc<Broadcaster>` is the runtime-injected shape.
- **`ferro_broadcast::Broadcast::new(broadcaster).channel(name).event(event_name).data(payload).send().await`** (Wave 1a, shipped) — fluent broadcast API used by D-19 step 6.
- **SeaORM 1.0 + sea-orm-migration 1.0** are workspace deps (already in `ferro-audit` / `ferro-reservation` Cargo.tomls — copy the relevant subset).
- **`thiserror` 2, `serde`, `serde_json`, `uuid` (v4 + serde), `chrono` (serde), `tracing`** are workspace deps; ferro-projection adds them as direct deps with the same versions.
- **`async-trait` 0.1** is a workspace dep (used by ferro-events, ferro-reservation); ferro-projection adds it as a direct dep for the `Projection` trait's async methods (the `rebuild` method specifically; `apply` is sync per D-08).
- **`dashmap` 6** is a workspace dep (used by ferro-broadcast, ferro-stripe). ferro-projection uses it for the per-key Mutex registry (D-31).
- **`tokio` 1** with `sync` + `rt` features — `tokio::sync::Mutex` for the per-key lock (D-19 step 2).
- **`proptest` 1** — dev-dep already added in Phase 154 (D-49 cites). ferro-projection reuses it for the three properties in D-49.
- **In-memory SQLite testing pattern** — `framework/src/database/testing.rs` reference; ferro-audit / ferro-reservation already re-derive it inline. ferro-projection does the same.
- **No existing live-read-model code in the workspace** — `grep -rl "ProjectionRuntime\|projection_snapshots\|ferro_projection\|ferro-projection\b" --include="*.rs" --include="*.toml" --include="*.md"` returned only references in this design doc, roadmap, Phase 154's deferred section, and STATE.md. Greenfield.

### Established patterns

- **One Error enum per crate** (`thiserror` derive) — convention across `ferro-orm`, `ferro-audit`, `ferro-reservation`, `ferro-wallet`, `ferro-stripe`, `ferro-events`, `ferro-notifications`. `ProjectionError` follows.
- **Display prefix on error enum** — `"projection: …"` mirrors `"guarded: …"`, `"audit: …"`, `"reservation: …"`, `"config: …"`. Cross-workspace grep-friendly.
- **Stringly-typed payload variants in error enums** — `ProjectionError::Broadcast(String)` and `ProjectionError::Events(String)` follow Phase 149's `Error::Broadcast(String)` precedent (D-29).
- **Builder pattern: `with_*` taking `mut self` → `Self`** — not used in `ProjectionRuntime` (the constructor takes three args directly), but the `ferro_broadcast::Broadcast::channel().event().data().send()` chain inside the apply algorithm uses this shape.
- **Generic over `ConnectionTrait`** — Phase 152 / 153 / 154 use this on per-call methods. ferro-projection v0 does NOT — the runtime owns the `DatabaseConnection` and runs single-statement upserts directly. Documented deviation: the apply path is per-key serialized via in-process Mutex (D-20), so caller-supplied transactions don't add correctness; v0.x can revisit if a real consumer needs to compose apply with their own transaction.
- **`#[serde(rename_all = "snake_case")]`** on enums — applies to nothing in ferro-projection v0 (no enums in the public surface; `ProjectionError` is internal). `ProjectionKey` is a tuple struct, not an enum.
- **Wave 1b Cargo.toml metadata fields** — `description`, `keywords`, `categories = ["database", "asynchronous", "web-programming"]`, `repository`, `readme = "README.md"`, `homepage = "https://ferro-rs.dev"`. Copy from `ferro-reservation/Cargo.toml`; adjust keywords / categories for projection semantics (`keywords = ["projection", "read-model", "events", "broadcast", "ferro"]`).
- **SeaORM migration as public re-export** — `pub use migration::Migration as CreateProjectionSnapshotsTable;` mirrors Phase 153 D-18 / Phase 154 D-38.
- **Public `Entity` re-export for SeaORM-native queries** — recommended (Claude's Discretion); matches Phase 153 / 154.
- **Composite primary key in SeaORM** — `(projection_name, key)` mirrors how multi-column PKs are declared in SeaORM (e.g. junction tables; check `framework/migrations/` for the closest precedent if one exists).

### Integration points

- **Workspace `Cargo.toml`** — add `"ferro-projection"` to `[workspace.members]`.
- **`.github/workflows/publish.yml`** — add `ferro-projection` to `WAVE1B_CRATES` alongside `ferro-ai`, `ferro-projections`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`, `ferro-reservation`.
- **Workspace version bump** — `[workspace.package] version = "0.2.33"`.
- **`framework/src/lib.rs`** — DO NOT add an automatic re-export of `ferro_projection`. Consumers depend on `ferro-projection` directly. Same call as Phases 152 / 153 / 154.
- **`README.md` (workspace root)** — add ferro-projection to the workspace crates table. CRITICAL: the table row description must explicitly say "live read-model from domain events with delta broadcast (not the same as `ferro-projections` plural — see disambiguation)" so the table itself communicates the distinction at a glance.
- **`CLAUDE.md` "Workspace Structure" table** — add a row for ferro-projection with the same disambiguation framing as the README.
- **ferro-mcp `application_info` / `installed_crates`** — picks up ferro-projection automatically once it's a workspace member; no MCP code changes expected.
- **`docs/SUMMARY.md` / nav** — add `live-read-models.md` to the `Features` section, sibling of `projections.md`. The nav text is "Live Read-Models" (NOT "Projections"); the on-page header reiterates the disambiguation.

### Constraints surfaced by the scout

- ferro-projection is **a new top-level crate** — Phase 155 is the bootstrap. First publish requires manual personal-token bootstrap from local terminal (CI token is publish-update only) — same operational reality as Phases 151 / 152 / 153 / 154.
- **The framework has TWO crates with confusingly similar names.** `ferro-projections` (plural, v9.0, shipped) covers the Service Projection abstraction (data → IntentGraph → JsonUiRenderer); `ferro-projection` (singular, Phase 155) covers live read-models from domain events. This is the single biggest authoring risk for Phase 155. Mitigation across the phase: D-02 (rustdoc disambiguation), D-51 (lib.rs lead paragraph), D-52 (doc-page title "Live Read-Models"), README + CLAUDE.md row descriptions, CHANGELOG framing. The naming itself is locked because the design doc, roadmap, and directory all consistently use the singular form — renaming would force a roadmap edit + spec edit + directory rename, and the substance (live read-models) genuinely is a projection of events. The disambiguation lives in documentation, not in renames.
- **No precedent for runtime listener registration on `global_dispatcher()` in the workspace.** ferro-events tests register listeners via `EventDispatcher::on(closure)` rather than `listen<E, L>(struct_listener)` — Phase 155 will be the first user of the struct-listener path. The plan should include a smoke-test plan task that verifies the listener fires after `register` (D-46's integration test is the canonical version).
- **No precedent for capturing `Broadcaster` output in tests** in the existing codebase. Check `ferro-broadcast/src/broadcaster.rs` test module first; if no helper exists, the plan needs a small `BroadcastCapture` mock (D-46) — a tokio mpsc channel that mocks the broadcaster's send path. Document the helper in the test code so future projection tests reuse it.
- **`dashmap` 6 is already a workspace dep** (used by ferro-broadcast, ferro-stripe) — no new top-level dep cost.
- **Sea-orm composite primary keys** need a custom `PrimaryKeyTrait` impl in the entity — the SeaORM book has the pattern; the planner / executor will follow it. Document the column order: `(projection_name, key)` — first-column lookups still hit the PK index.

</code_context>

<specifics>
## Specific Ideas

- The canonical sample from the design doc, rewritten to the v0 API for the rustdoc top example:
  ```rust
  use ferro_projection::{Projection, ProjectionKey, ProjectionRuntime};
  use ferro_events::{Event, async_trait};
  use ferro_broadcast::Broadcaster;
  use serde::{Serialize, Deserialize};
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
      totals: std::collections::HashMap<String, i64>,  // sku → quantity
  }

  #[derive(Clone, Serialize)]
  struct WarehouseDelta { sku: String, new_total: i64 }

  // Consumer projection impl
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

  // Application setup (one-line wiring)
  let runtime = Arc::new(ProjectionRuntime::new(db.clone(), broadcaster.clone(), WarehouseProjection));
  runtime.clone().register();

  // Anywhere in the app:
  InventoryAdjusted { warehouse: "a".into(), sku: "sku-1".into(), delta: 5 }
      .dispatch()
      .await?;

  // Frontend subscribes to `projection.inventory.dashboard.a` and receives:
  // event "delta", data { "sku": "sku-1", "new_total": 5 }
  ```
- The error-naming style across the workspace (`"guarded: …"`, `"audit: …"`, `"reservation: …"`, `"config: …"`) — `ProjectionError` follows the same `"projection: …"` Display prefix.
- The framing in the rustdoc: lead with the disambiguation paragraph (D-51), then *why* (every app emitting events that wants a live dashboard hand-rolls the same load → apply → persist → broadcast cycle; this is the typed kernel), then the one-line wiring example. Show the per-key serialization diagram and the broadcast-channel naming convention. Document the three operational footguns: (1) broadcast failure does NOT roll back state, (2) single-instance assumption, (3) `register` is not idempotent on `Arc` identity (calling twice = two listeners = two applies per event).
- The mental model the rustdoc opens with: "ferro-projection is the *live-read-model* primitive: events come in, state comes out, deltas land on a WebSocket channel. ferro-projections (plural) is the *projection/intent* primitive: data shape comes in, rendered UI comes out. They share a name because they share a philosophical lineage — both turn a description into a continuously updated artifact — but they are orthogonal abstractions implemented in separate crates."
- The per-key serialization diagram for the rustdoc top:
  ```
   Event::dispatch() ─┐
                      │
        ProjectionListener<P> ──┐
                                │
                                ▼
   ┌── per-key Mutex (DashMap<String, Arc<Mutex<()>>>) ──┐
   │                                                     │
   │   1. load snapshot from projection_snapshots        │
   │   2. apply(&mut state, &event) → Delta              │
   │   3. upsert snapshot (state, version+1)             │
   │   4. broadcast on projection.{name}.{key}            │
   │                                                     │
   └─────────────────────────────────────────────────────┘
                                │
                                ▼
                  WebSocket clients receive the delta
  ```
- Dotted-namespace convention for `Projection::NAME` and the broadcast channel mirrors ferro-audit's `action` and ferro-reservation's `Resource::KIND`: `"inventory.dashboard"`, `"checkout.cart"`, `"orders.recent"`. Documented as a convention; not enforced at compile time.
- The CHANGELOG framing line for the milestone completion: "v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm GuardedUpdate (Phase 152), ferro-audit (Phase 153), ferro-reservation (Phase 154), ferro-projection (Phase 155) now all shipped. Apps with capacity constraints + live dashboards can compose these four primitives instead of hand-rolling them."
- README.md / CLAUDE.md row text for ferro-projection (locking the disambiguation in the table itself):
  ```
  | `ferro-projection` | Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas. **Not the same as `ferro-projections` (plural)** — see crate docs for the distinction. | `src/lib.rs` |
  ```

</specifics>

<deferred>
## Deferred Ideas

- **Persistent in-crate event log (`projection_events` table)** — v0.x. v0 ships snapshot-only; `rebuild` accepts caller-supplied event iterator.
- **Checkpoint-based snapshot interval (write-every-N-events)** — v0.x; the `snapshot_interval()` method already exists as a forward-compat hook.
- **Optimistic concurrency control on the version column** — v0.x; the column is already in the schema.
- **Cross-instance projection coordination** — its own framework feature, not v11.11 scope.
- **`Projection::channel_for(&key)` override for private/presence channels** — v0.x.
- **Tenant-scoped automatic filtering** — consumers bake tenancy into `ProjectionKey`; same call as Phases 153 / 154.
- **Postgres CI integration tests** — same call as Phases 152 / 153 / 154.
- **MCP tool: `list_projections`** — needs a registry; v0.x.
- **`unregister` API on `ProjectionRuntime`** — gated on ferro-events shipping per-listener removal.
- **Macro façade for `Projection` definitions** — trait is small enough.
- **Deep-merge / patch semantics in `apply`** — consumer territory.
- **UI components for projection state rendering** — apps own their UI.
- **Multi-projection-per-event orchestration primitives** — multiple `register` calls already work.
- **`ferro reservation:sweep`-style CLI subcommand for triggering rebuilds** — ferro-cli scope, not ferro-projection.
- **`ferro::prelude` / framework re-export** — same call as Phases 152 / 153 / 154.

### Reviewed Todos (not folded)

No todos matched this phase (`gsd-tools todo match-phase 155` returned zero matches).

</deferred>

---

*Phase: 155-ferro-projection-crate-live-read-model-from-domain-events-wi*
*Context gathered: 2026-05-14*
*Mode: --auto (single-pass, recommended defaults applied to every gray area)*

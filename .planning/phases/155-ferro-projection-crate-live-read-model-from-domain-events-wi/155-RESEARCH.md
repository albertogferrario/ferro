# Phase 155: ferro-projection — Research

**Researched:** 2026-05-14
**Domain:** Wave 1b workspace crate scaffolding · SeaORM 1.x composite PK + upsert · ferro-events listener registration · ferro-broadcast capture · per-key in-process serialization
**Confidence:** HIGH (all locked decisions in CONTEXT.md are implementable from verified workspace precedent + SeaORM 1.x verified syntax)

## Summary

Phase 155 ships `ferro-projection` (singular), a Wave 1b crate that subscribes to `ferro-events`, folds events into per-key persisted snapshots, and broadcasts deltas via `ferro-broadcast`. Every locked decision in `155-CONTEXT.md` (D-01..D-56) has a clean implementation path from the existing workspace pattern. The structural twin is **Phase 154 (ferro-reservation)**: same Wave 1b shape, same migration-as-public-re-export pattern, same first-publish-from-local-terminal operational reality.

Three implementation concerns need surfacing for the planner:

1. **SeaORM 1.x composite primary key** — declared by applying `#[sea_orm(primary_key)]` to multiple columns. No workspace precedent yet (all current `ferro-*` entities use single-column UUID PKs). The macro auto-generates `PrimaryKey` enum + tuple `ValueType`. Verified against SeaORM 1.1.x docs.
2. **`OnConflict::columns([..]).update_columns([..])`** is the cross-dialect upsert syntax (Postgres + SQLite both supported natively via sea-query). The planner needs to use the `columns` plural form (not chained `.column()` calls) for the composite conflict target.
3. **Broadcast capture in tests has zero precedent in the workspace.** The `Broadcaster` is concrete (not a trait) and `Broadcast::send` calls `Broadcaster::broadcast` which sends a `ServerMessage::Event(BroadcastMessage)` to each subscribed client's `mpsc::Sender<ServerMessage>`. Capture is straightforward: in tests, construct a real `Broadcaster`, `add_client(socket_id, sender)` with a test mpsc, `subscribe(socket_id, channel, None, None)`, then drain the receiver. No mock needed — the production code path already supports this. A small `BroadcastCapture` helper consolidates the boilerplate.

**Primary recommendation:** 7 plans, mirroring Phase 154's arc exactly. Plans 01 → 02 are sequential (Wave 1 → 2 — workspace registration is needed before `cargo build -p` can succeed). Plans 03 → 06 sequence on shared crate state. Plan 07 is release + manual first-publish checkpoint.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Event subscription + folding | `ferro-projection` runtime | `ferro-events` (dispatch) | Projection is the fold; ferro-events is the dispatch. |
| Per-key serialization | `ferro-projection` (in-process Mutex) | — | DashMap + `tokio::sync::Mutex` is in-crate; no external locking. |
| Snapshot persistence | `ferro-projection` (sea-orm upsert) | `sea-orm` (1.x) | One row per `(projection_name, key)` — ferro-projection owns the table. |
| Delta broadcast | `ferro-broadcast` (Broadcaster) | `ferro-projection` (channel naming convention) | Broadcaster is the transport; ferro-projection owns the channel-name format. |
| Rebuild from event iterator | `ferro-projection` | Consumer (supplies events) | Crate is opinion-free on event storage. |
| Consumer event type | Consumer code | `ferro-events::Event` (bound) | `Projection::Event: ferro_events::Event + Serialize + DeserializeOwned`. |
| Migration registration | Consumer's `Migrator` | `ferro-projection` (`CreateProjectionSnapshotsTable` re-export) | Same pattern as Phase 153 D-18 / Phase 154 D-38. |

## Project Constraints (from CLAUDE.md)

The planner MUST honour these — copied verbatim from CLAUDE.md / project memory:

- **Project-agnostic crate.** No hardcoded app name, brand, URL, tenant id. Consumer-specific concepts (inventory, dashboards, slots) MUST NOT appear in the public API. Generic types only.
- **Pre-commit gate (CI-exact):** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` — every plan ends with this.
- **One `Error` enum per crate** via `thiserror::Error` derive. `ProjectionError` display strings start with `"projection: …"` (grep-friendly across workspace alongside `"guarded: …"`, `"audit: …"`, `"reservation: …"`).
- **Builder pattern:** `with_*` methods take `mut self`, return `Self` (consuming). Not directly used in `ProjectionRuntime` (constructor takes three args), but `Broadcast::channel().event().data().send()` chain uses this shape inside `apply_event`.
- **No co-author lines in commits.** No "Generated with Claude" attribution.
- **Documentation:** every public symbol gets a `///` doc comment. `docs/src/features/live-read-models.md` is the user-facing page (D-52).
- **MCP introspection:** no new MCP tools — `application_info`, `db_schema`, `generation_context` pick up the crate automatically once it's in `[workspace.members]`.
- **Vision anchor:** the killer feature of v11.11 is the **composability** of ferro-events + ferro-broadcast + ferro-reservation + ferro-projection in a single `Arc::new + register` call. Frame the rustdoc and doc page around that demonstration, not the trait API in isolation.

---

## §User Constraints (locked decisions)

Source: `155-CONTEXT.md`. The 56 D-decisions are the source of truth. The planner MUST NOT relitigate any locked decision; the research below assumes them.

**Decision groups (decisions ranges in CONTEXT.md):**

| Group | Range | Locked outcome |
|-------|-------|----------------|
| Crate placement, naming, scope | D-01..D-05 | New top-level `ferro-projection/`, Wave 1b, deps `ferro-events` + `ferro-broadcast`, disambiguation from `ferro-projections` (plural) is load-bearing |
| `Projection` trait | D-06..D-10 | `Event: ferro_events::Event + Serialize + DeserializeOwned`, sync `apply`, returns `P::Delta`; const `NAME` |
| `ProjectionKey` | D-11..D-12 | Stringly-typed newtype, dotted-or-colon namespace convention |
| Runtime API | D-13..D-18 | `new`, `register`, `read`, `apply_event`, `rebuild`; runtime owns DB + broadcaster |
| Apply algorithm | D-19..D-22 | Per-key Mutex → load → apply → upsert → broadcast. Broadcast failure does NOT roll back. DB failure DOES. |
| Schema & migration | D-23..D-27 | Composite PK `(projection_name, key)`, JSON state, BIGINT version, app-set `updated_at` |
| Error model | D-28..D-30 | `thiserror`, `"projection: …"` prefix, `Db`/`Json`/`Broadcast`/`Events`/`StateNotFound` variants |
| Concurrency | D-31..D-34 | `DashMap<String, Arc<Mutex<()>>>`, no eviction in v0, single-instance assumption explicit |
| Listener registration | D-35..D-37 | `register(self: Arc<Self>)` consumes one clone, no `unregister` in v0 |
| Broadcast contract | D-38..D-41 | Channel `projection.{name}.{key}`, default event `"delta"`, raw delta payload, rebuild emits `"rebuild"` event |
| Rebuild semantics | D-42..D-44 | DELETE then fold; empty iterator wipes the row; not transactional |
| Testing | D-45..D-50 | 10 unit cases + 3 integration tests + 3 proptest properties |
| Documentation | D-51..D-53 | Module-rustdoc disambiguation lead, `docs/src/features/live-read-models.md`, no new MCP tools |
| Release | D-54..D-56 | Version bump 0.2.32 → 0.2.33, Wave 1b publish, CHANGELOG entry with milestone-completion line |

**Claude's Discretion** (from CONTEXT.md): internal module layout, public re-export of SeaORM `Entity`, tracing wording, proptest generator shape, test file names, `is_registered` method (recommended NO), `ProjectionListener<P>` publicity (recommended NO), `read_required` helper (recommended YES).

**Deferred** (out of scope): in-crate event log, snapshot interval, OCC on version, cross-instance coordination, `unregister`, `channel_for`, tenant-scoped filtering, Postgres CI, MCP `list_projections`, prelude re-export, macro façade, deep-merge semantics, UI components, multi-projection orchestration.

---

## §Approach

The implementation arc mirrors Phase 154 exactly. The crate ships in 7 plans across 5 waves:

```
Wave 1 (sequential, scaffolding)
├── Plan 01: scaffold crate (Cargo.toml, lib.rs rustdoc, stub modules, ProjectionError body)
│
Wave 2 (sequential, registration)
├── Plan 02: register in workspace (root Cargo.toml members, version bump 0.2.32 → 0.2.33,
│            publish.yml WAVE1B_CRATES, CLAUDE.md table, README.md table)
│
Wave 3 (sequential, schema)
├── Plan 03: SeaORM migration + entity (CreateProjectionSnapshotsTable + Model
│            with composite PK + public Entity re-export)
│
Wave 4 (sequential, leaf types)
├── Plan 04: Projection trait + ProjectionKey newtype (D-06..D-12)
│            including the trait method defaults (snapshot_interval, broadcast_event_name)
│
Wave 5 (sequential, runtime)
├── Plan 05: ProjectionRuntime body (new + read + apply_event + register +
│            ProjectionListener<P>) + 10 D-45 unit tests
│
Wave 6 (sequential, integration tests)
├── Plan 06: rebuild body + 3 integration tests (D-46/47/48) + 3 proptest properties (D-49)
│
Wave 7 (sequential, release)
└── Plan 07: docs/src/features/live-read-models.md + CHANGELOG + manual first-publish
```

**Why this is sequential, not parallel:**

- Plans 01 → 02: cargo cannot build `ferro-projection` until it appears in workspace members. Phase 153 SUMMARY documents this exact lesson — scaffolding alone fails `cargo build -p` until registration lands.
- Plans 02 → 03: migration tests need the workspace dep graph populated.
- Plans 03 → 04 → 05: the trait references the entity (`Projection::State` is serialized into the `state` column), the runtime references both, so each plan compiles on top of the previous.
- Plans 05 → 06: integration tests link against the runtime symbols.
- Plans 06 → 07: docs and CHANGELOG sit on a complete, tested crate.

**Why 7 plans and not 6:** plan 06 already carries 3 integration tests + 3 proptests + the `rebuild` body. Folding plan 07's release work (user doc + CHANGELOG + manual publish) into 06 would create an unwieldy plan. Phase 154 made the same call (separate release plan).

---

## §Validation Architecture (Nyquist Dimension 8)

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built into Rust 2021, `tokio = { features = ["full"] }`) |
| Config file | none — uses `Cargo.toml [dev-dependencies]` |
| Quick run command | `cargo test -p ferro-projection --lib` |
| Full suite command | `cargo test -p ferro-projection --all-features` |
| Property-based testing | `proptest` 1.x (already a workspace dev-dep — Phase 154 D-49 added) |

### Test Inputs / Fixtures

| Fixture | Scope | Source | Use Case |
|---------|-------|--------|----------|
| `TestProjection` (counter projection) | unit + integration | in-test definition | D-45 unit tests + D-46 event-bus integration |
| `ReservationCountProjection` | cross-crate showcase | in-test definition (folds `ReservationEvent`) | D-47 (milestone showcase) |
| In-memory SQLite | every test | `Database::connect("sqlite::memory:")` (precedent: ferro-audit/tests, ferro-reservation/tests) | All persistence tests |
| `BroadcastCapture` helper | all broadcast assertions | new in-test helper (D-46 surfaces the need) | Drain `mpsc::Receiver<ServerMessage>` |
| Event sequence generator | proptest | `proptest::collection::vec(strategy_for_event(), 0..50)` | D-49 properties 1/2/3 |
| Concurrent task harness | integration | `tokio::spawn` × N + `JoinSet` collect | D-48 concurrent_apply |

### Coverage Matrix — Decisions → Test Cases

| Decision | Test Name | Test File | Test Type |
|----------|-----------|-----------|-----------|
| D-06 (trait shape) | compile-time (trait impl for `TestProjection`) | `src/projection.rs` doc-test or test mod | smoke |
| D-07 (`State: Default` for first-apply) | `apply_event_on_new_key_initializes_from_default` | `src/runtime.rs` (unit) | unit |
| D-08 (sync `apply`) | compile-time (no `.await` inside `apply`) | `src/projection.rs` doc-test | smoke |
| D-09 (Event Serialize/DeserializeOwned) | compile-time bound | `src/projection.rs` | smoke |
| D-10 (delta is consumer-shaped) | `apply_returns_consumer_delta` | `src/runtime.rs` | unit |
| D-11 (`ProjectionKey` newtype) | `projection_key_roundtrip` (D-45 #1) | `src/key.rs` | unit |
| D-12 (dotted-namespace convention) | doc-comment only; no test (convention, not enforcement) | — | — |
| D-13 (Runtime owns db + broadcaster + locks) | `runtime_construction_is_send_sync` (D-45 #4) | `src/runtime.rs` | unit |
| D-14 (two entry points) | `register_wires_listener` (D-46) AND `manual_apply_event` (D-45 #5) | integration + unit | unit + integration |
| D-15 (ProjectionListener registration) | `register_listener_fires_on_dispatch` | `tests/event_bus_integration.rs` (D-46) | integration |
| D-16 (read returns Option) | `read_returns_none_for_absent_key` (D-45 #8a) | `src/runtime.rs` | unit |
| D-17 (rebuild signature + behaviour) | `rebuild_three_events_equivalent_to_three_applies` (D-45 #9) | `src/runtime.rs` | unit |
| D-18 (Arc<Runtime> Send+Sync) | `arc_runtime_is_send_sync` (D-45 #4) | `src/runtime.rs` | unit |
| D-19 (apply algorithm 7-step) | `apply_event_happy_path_persists_and_broadcasts` (D-45 #5) | `src/runtime.rs` | unit |
| D-20 (per-key serialization) | `concurrent_same_key_serializes` | `tests/concurrent_apply.rs` (D-48) | integration |
| D-21 (broadcast failure does NOT roll back) | `broadcast_failure_keeps_snapshot` | `src/runtime.rs` (uses a mock broadcaster that errors) | unit |
| D-22 (db failure DOES surface) | `db_error_surfaces_as_projection_error_db` | `src/runtime.rs` | unit |
| D-23 (migration as public re-export) | `CreateProjectionSnapshotsTable` import succeeds | `src/migration.rs` test mod | unit |
| D-24 (schema columns) | `migration_creates_table_with_expected_columns` | `src/migration.rs` test mod | unit |
| D-25 (version counter +1 per apply) | `version_increments_on_each_apply` (D-45 #6) | `src/runtime.rs` | unit |
| D-26 (JSON state column) | `state_round_trips_through_json_column` | `src/entity.rs` test mod | unit |
| D-27 (app-set updated_at) | `updated_at_is_app_set_on_upsert` | `src/runtime.rs` | unit |
| D-28 (ProjectionError variants) | `projection_error_display_strings_start_with_projection_prefix` (D-45 #2) | `src/error.rs` | unit |
| D-29 (`String`-payload broadcast/events) | `from_broadcast_error_preserves_message` | `src/error.rs` | unit |
| D-31 (per-key Mutex registry) | `cross_key_parallelizes` | `tests/concurrent_apply.rs` (D-48) | integration |
| D-32 (concurrent same-key vs cross-key) | `concurrent_apply_20_tasks_5_keys` (D-48) | `tests/concurrent_apply.rs` | integration |
| D-33 (read does not lock) | `concurrent_read_during_apply` | `tests/concurrent_apply.rs` | integration |
| D-34 (single-instance assumption) | rustdoc only — covered by `cargo doc` lint | — | doc |
| D-35 (register wires global dispatcher) | covered by D-46 | `tests/event_bus_integration.rs` | integration |
| D-36 (register is not idempotent on Arc identity) | `register_twice_fires_listener_twice` | `tests/event_bus_integration.rs` | integration |
| D-37 (no unregister API) | absence of method — covered by `cargo doc` | — | doc |
| D-38 (channel name format) | `broadcast_channel_name_format` | `src/runtime.rs` (capture broadcaster) | unit |
| D-39 (default event name `"delta"`) | `default_event_name_is_delta` (D-45 #3) | `src/projection.rs` | unit |
| D-40 (raw delta payload — no envelope) | `broadcast_payload_is_raw_delta` | `src/runtime.rs` | unit |
| D-41 (rebuild emits `"rebuild"` event) | `rebuild_broadcasts_rebuild_event` | `src/runtime.rs` | unit |
| D-42 (rebuild DELETEs then folds) | `rebuild_after_existing_state_resets_version` | `src/runtime.rs` | unit |
| D-43 (empty rebuild wipes the row) | `rebuild_empty_iterator_deletes_row` (D-45 #10) | `src/runtime.rs` | unit |
| D-44 (rebuild not transactional — v0 crash semantic) | rustdoc only | — | doc |
| D-45 (unit cases #1..#10) | listed inline above | mostly `src/runtime.rs` + `src/key.rs` + `src/error.rs` + `src/projection.rs` | unit |
| D-46 (event-bus auto-register integration) | `auto_register_fires_listener_5_times` | `tests/event_bus_integration.rs` | integration |
| D-47 (cross-crate reservation showcase) | `reservation_count_projection_over_reservation_events` | `tests/projection_over_reservation_events.rs` | integration |
| D-48 (concurrent apply harness) | `concurrent_apply_20_tasks_across_5_keys` | `tests/concurrent_apply.rs` | integration |
| D-49 #1 (apply determinism) | `proptest_apply_determinism` | `tests/property_invariants.rs` | proptest |
| D-49 #2 (replay equivalence) | `proptest_rebuild_equals_sequential_applies` | `tests/property_invariants.rs` | proptest |
| D-49 #3 (cross-key independence) | `proptest_cross_key_isolation` | `tests/property_invariants.rs` | proptest |
| D-50 (in-memory SQLite harness re-derived inline) | each test sets up its own `Database::connect("sqlite::memory:")` | every test file | — |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-projection --lib`
- **Per plan merge:** `cargo test -p ferro-projection --all-features`
- **Phase gate:** full workspace `cargo test --all-features` + the CI-exact clippy command (no warnings)

### Wave 0 Gaps

- [ ] `ferro-projection/tests/event_bus_integration.rs` — covers D-46
- [ ] `ferro-projection/tests/projection_over_reservation_events.rs` — covers D-47 (depends on `ferro-reservation` dev-dep)
- [ ] `ferro-projection/tests/concurrent_apply.rs` — covers D-48
- [ ] `ferro-projection/tests/property_invariants.rs` — covers D-49 (3 properties)
- [ ] `ferro-projection/tests/common/mod.rs` — `BroadcastCapture` helper (drain `mpsc::Receiver<ServerMessage>` into a `Vec<BroadcastMessage>` with filter by channel name). Documented inline; future projection tests reuse it.

### Validation criteria — how the planner knows verification is sufficient

Each plan's verification step asserts:

1. **`cargo build -p ferro-projection`** — code compiles (catches missing imports, trait bound mismatches).
2. **`cargo clippy --all --all-targets -- -D warnings`** — CI-exact lint command (no per-plan deviation per `feedback_ci_clippy_command_match`).
3. **`cargo test -p ferro-projection --all-features`** — every test passes, including property tests.
4. **`cargo doc --no-deps -p ferro-projection`** — no broken doc-links (catches stale `[`ferro-events`]` references after refactors).
5. **`cargo test --workspace --all-features`** — no other crate broke. Critical at plan 02 (workspace registration can fail if Cargo.lock isn't updated).

When all five pass, the plan is verifiable.

---

## §Technical Concerns

### 1. SeaORM 1.x composite primary key declaration

**Verified pattern** (SeaORM 1.1.x docs, junction-table example):

```rust
use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projection_snapshots")]
pub struct Model {
    /// Projection logical name, e.g. "inventory.dashboard". Half of the
    /// composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub projection_name: String,

    /// Per-row key inside the projection, e.g. "warehouse-a". Other half
    /// of the composite primary key (D-24).
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,

    /// Serialized P::State (D-26 — JSON column).
    pub state: JsonValue,

    /// Monotonic counter (D-25); +1 per apply, reset on rebuild.
    pub version: i64,

    /// App-set Utc::now() inside the upsert (D-27).
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

Notes verified against SeaORM 1.1.x docs:

- **Apply `#[sea_orm(primary_key)]` to each column** that forms the composite key. `auto_increment = false` is recommended explicitly because the macro defaults to `auto_increment = true` on single-column int PKs; with composite PKs the macro auto-derives `false` but explicit is safer.
- The macro generates a derived `PrimaryKey` enum (e.g., `enum PrimaryKey { ProjectionName, Key }`) and a tuple `ValueType = (String, String)`.
- `Entity::find_by_id((name.to_string(), key.0.clone()))` is the lookup form — accepts the tuple.
- Maximum supported arity is 12; we use 2.
- No workspace precedent — all existing `ferro-*` entities use single-column UUID PKs (`ferro-audit/src/entity.rs:19`, `ferro-reservation/src/entity.rs:17`, `framework/src/session/driver/database.rs:199`). The smoke test in plan 03 (`migration_creates_table_with_composite_pk`) is the proving artefact.

**Source:** `[CITED: https://www.sea-ql.org/SeaORM/docs/1.1.x/generate-entity/entity-structure/]` — composite primary key pattern (verified via WebFetch 2026-05-14).

### 2. SeaORM `OnConflict` for cross-dialect upsert

**Verified pattern** (sea-query 0.32 `OnConflict` docs):

```rust
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, EntityTrait};

let am = ActiveModel {
    projection_name: ActiveValue::Set(P::NAME.to_string()),
    key: ActiveValue::Set(key.as_str().to_string()),
    state: ActiveValue::Set(serde_json::to_value(&new_state)?),
    version: ActiveValue::Set(new_version),
    updated_at: ActiveValue::Set(Utc::now().naive_utc()),
};

Entity::insert(am)
    .on_conflict(
        OnConflict::columns([Column::ProjectionName, Column::Key])
            .update_columns([Column::State, Column::Version, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(&self.db)
    .await?;
```

Notes:

- Use **`OnConflict::columns([..])`** (plural) for the composite conflict target — NOT chained `.column().column()` calls. The plural form is the documented multi-column API.
- `update_columns([..])` takes a slice; updates each column from `excluded.*` (Postgres/SQLite) or the implicit "new" row (MySQL).
- **Cross-dialect:** SQLite + Postgres both generate native `INSERT … ON CONFLICT (cols) DO UPDATE SET col = excluded.col`. MySQL polyfills via `ON DUPLICATE KEY UPDATE`. ferro-projection v0 tests against in-memory SQLite (D-50); Postgres CI is deferred (same call as ferro-audit / ferro-reservation).
- **Empty-update edge case:** SeaORM returns `DbErr::RecordNotInserted` when no rows are affected. For projection snapshots we always update something (state changes every apply by definition), so this should not fire. If it ever does, append `.do_nothing()` or treat `RecordNotInserted` as a no-op.

**Source:** `[CITED: https://docs.rs/sea-query/0.32/sea_query/query/struct.OnConflict.html]` — `OnConflict::columns([..]).update_columns([..])` (verified via WebFetch 2026-05-14).

### 3. `tokio::sync::Mutex` (NOT `parking_lot::Mutex`)

**The lock is held across `.await` points** (DB upsert + broadcast send). `parking_lot::Mutex` is sync — its guard does NOT implement `Send`, so it cannot cross an `.await`. `tokio::sync::Mutex` is async-aware, returns `MutexGuard` that is `Send`, and is the right choice.

```rust
// CORRECT (D-19 step 2 — holds lock across await):
let lock_arc = self.locks
    .entry(key.0.clone())
    .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
    .clone();  // <-- CLONE the Arc BEFORE the RefMut drops (see concern #4)
let _guard = lock_arc.lock().await;
// load snapshot, apply, upsert, broadcast — all .await — guard is held
```

**Cost note:** `tokio::sync::Mutex` is slightly slower than `parking_lot::Mutex` for uncontended locks. For projection apply (DB+broadcast = ~ms scale), the cost is invisible. Document this choice in the runtime module rustdoc.

**Source:** `[CITED: https://docs.rs/tokio/1/tokio/sync/struct.Mutex.html]` — "When to use Tokio's Mutex" (matches our use case exactly: lock held across `.await`).

### 4. `DashMap::entry().or_insert_with()` — releasing the shard lock

**Trap:** `DashMap::entry(k).or_insert_with(f)` returns a `RefMut` that holds the **shard write lock** for as long as the `RefMut` is alive. If you then call `.lock().await` on the per-key `Arc<Mutex<()>>` while the `RefMut` is still in scope, you hold the DashMap shard lock across the per-key Mutex acquisition — which can cause cross-key contention through the shard.

**Correct pattern:**

```rust
// Clone the Arc INSIDE a short scope so the RefMut drops before we await.
let lock_arc: Arc<tokio::sync::Mutex<()>> = {
    self.locks
        .entry(key.0.clone())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()  // RefMut deref → Arc<Mutex<()>>; .clone() returns owned Arc
};  // RefMut drops here, shard unlocked
let _guard = lock_arc.lock().await;  // safe: shard not held during the await
```

This is the same pattern `ferro-broadcast/src/broadcaster.rs` uses (see line 271: `drop(channel); // Release DashMap guard before await`). Document the pattern in the runtime module with a code-comment for future reviewers.

**Source:** `[CITED: https://docs.rs/dashmap/6/dashmap/struct.DashMap.html#method.entry]` — "RefMut holds shard write lock for its lifetime" (verified via DashMap 6 docs).

### 5. `Broadcaster` capture in tests — zero precedent, but trivial

The `Broadcaster` is **concrete** (not a trait) — see `ferro-broadcast/src/broadcaster.rs:36`. It exposes:

- `add_client(socket_id: String, sender: mpsc::Sender<ServerMessage>)` — register a test client (already public, used in `broadcaster.rs` test mod).
- `subscribe(socket_id, channel_name, auth, member_info)` — subscribe the test client to the channel.
- The `Broadcast::new(broadcaster).channel(name).event(event_name).data(payload).send()` chain calls `Broadcaster::broadcast(channel, event, data)`, which pushes a `ServerMessage::Event(BroadcastMessage)` to each subscribed client's `mpsc::Sender<ServerMessage>`.

**Test-side capture:**

```rust
// ferro-projection/tests/common/mod.rs (NEW — shared helper)

use ferro_broadcast::{Broadcaster, BroadcastMessage, ServerMessage};
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
    /// Construct a fresh broadcaster + mock client subscribed to `channel`.
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

    /// Drain all currently-buffered broadcast messages.
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

This uses the **production code path** (real `Broadcaster`, real `mpsc::Sender`, real `subscribe` semantics) — no mock, no trait, no test-only fork of `ferro-broadcast`. The boilerplate lives in `tests/common/mod.rs`, reused by D-46 / D-47 / D-48 / runtime unit tests asserting on broadcast frames.

**Note:** unit tests inside `src/runtime.rs` cannot use `tests/common/mod.rs` (Rust doesn't share `tests/common` with library tests). For unit tests, inline a smaller version of the helper inside a `#[cfg(test)] mod tests` block.

### 6. SeaORM `Json` column for `serde_json::Value` — cross-dialect

`ColumnDef::new(...).json().not_null()` in the migration plus `pub state: JsonValue` in the entity is the established pattern (`ferro-audit/src/entity.rs:46`, `ferro-reservation/src/entity.rs:24`). Verified working:

- **SQLite:** stores as `TEXT` under the hood; SeaORM serializes/deserializes via `serde_json`.
- **Postgres:** stores as native `JSON` (NOT `JSONB`). For `JSONB`, the migration would need `.custom("JSONB")`. ferro-projection v0 follows the workspace convention (`.json()`); Postgres CI is deferred so dialect-specific tuning is not in scope.

### 7. `version: i64` column — `BigInteger` (not `Integer`)

D-24 says `BIGINT`. In the migration: `ColumnDef::new(Column::Version).big_integer().not_null()`. In the entity: `pub version: i64`. Rationale: SeaORM maps `big_integer` → `i64` on both SQLite and Postgres. `Integer` would cap at `i32` (~2.1B). A high-frequency projection (1 event/sec for 70 years ≈ 2.2B events on one key) would overflow `i32`. `i64` is virtually unbounded for any realistic projection lifetime.

### 8. `ProjectionListener<P>` registration with the global dispatcher — verify the bound

Critical verification against `ferro-events`:

- `EventDispatcher::listen<E, L>` signature (`ferro-events/src/dispatcher.rs:73-79`):
  ```rust
  pub fn listen<E, L>(&self, listener: L)
  where E: Event, L: Listener<E>,
  ```
- `Listener<E>` requires `Send + Sync + 'static` (`ferro-events/src/traits.rs:132`).
- `Event` requires `Clone + Send + Sync + 'static` (`ferro-events/src/traits.rs:34`).
- `Projection::Event` per D-06 is `ferro_events::Event + Serialize + DeserializeOwned`. ✅ Satisfies `E: Event`.
- The listener storage path clones the event into the handler closure (`dispatcher.rs:92`: `let event = event.clone();`). This means **`P::Event` MUST be `Clone`** — which it already is via `ferro_events::Event: Clone`. ✅
- `ProjectionListener<P>` per D-15:
  ```rust
  struct ProjectionListener<P: Projection> {
      runtime: Arc<ProjectionRuntime<P>>,
  }
  ```
  Needs: `Send + Sync + 'static`. `Arc<T>` is `Send + Sync` if `T` is `Send + Sync`. `ProjectionRuntime<P>` is `Send + Sync` (the `DatabaseConnection` and `Arc<Broadcaster>` are; `DashMap` is; `P` is `Send + Sync + 'static` per the trait). ✅
- The async-trait `impl Listener<P::Event> for ProjectionListener<P>` needs `async fn handle` returning `Result<(), ferro_events::Error>`. The body wraps `ProjectionError` via `to_string()` + `Error::listener_failed(type_name::<Self>(), msg)`. ✅ (D-15's body is correct).

**Type erasure path:** `EventDispatcher` stores listeners in `RwLock<HashMap<TypeId, Vec<ListenerEntry>>>` keyed by `TypeId::of::<P::Event>()`. Each `ListenerEntry.handler: Box<dyn Any + Send + Sync>` is downcast back to `ListenerFn<E>` on dispatch (`dispatcher.rs:165`: `entry.handler.downcast_ref::<ListenerFn<E>>().cloned()`). The `TypeId` based dispatch means **all `ProjectionListener<P>` instances sharing the same `P::Event` are stored in the same bucket** — multiple registered projections sharing an event type all fire. ✅ Matches D-35.

**Smoke test:** the D-46 integration test is the canonical end-to-end proof. Build it first as Phase 155's listener-registration ground truth.

### 9. `Projection::Event: Serialize + DeserializeOwned` — verify against rebuild iterator

D-09 / D-17: `rebuild` accepts `IntoIterator<Item = P::Event>`. The iterator does NOT itself require `Serialize`/`DeserializeOwned` — the events are already deserialized by the caller. The bound exists so the consumer's event-storage path (audit log, queue payloads) can deserialize back to `P::Event` before handing it to `rebuild`. The trait bound is therefore **not load-bearing for `rebuild` itself**, but it documents the convention that projection events must be round-trippable through JSON for replay-from-storage use cases.

This is fine — the rustdoc for `Projection::Event` should explain this. No code change needed.

---

## §Cargo.toml derivation

Mirroring `ferro-reservation/Cargo.toml` exactly, swapping the runtime deps. Verified against the workspace's existing dep versions (no `[workspace.dependencies]` block in root Cargo.toml — each crate declares its own with matching versions across the workspace).

```toml
[package]
name = "ferro-projection"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas (not the same as ferro-projections plural)"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["projection", "read-model", "events", "broadcast", "ferro"]
categories = ["database", "asynchronous", "web-programming"]
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
dashmap = "6"
tokio = { version = "1", features = ["sync", "rt"] }
ferro-events    = { path = "../ferro-events",    version = "0.2" }
ferro-broadcast = { path = "../ferro-broadcast", version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
ferro-reservation = { path = "../ferro-reservation", version = "0.2" }
```

**Categories:** `["database", "asynchronous", "web-programming"]` — adds `web-programming` over Phase 154's two-category set because projections fan to WebSockets (the broadcast leg).

**Keywords:** `["projection", "read-model", "events", "broadcast", "ferro"]` — explicit "read-model" alongside "projection" to make crates.io search surface the singular crate for the right query (mitigating naming-clash confusion).

**Description:** explicit "(not the same as ferro-projections plural)" — the crates.io listing carries the disambiguation. Specifics §"README.md / CLAUDE.md row text" already locks this disambiguation in the workspace docs.

**Why `ferro-reservation` is a dev-dep:** D-47 cross-crate showcase test needs `ReservationEvent`. Not a runtime dep — ferro-projection v0 is event-agnostic.

**Why `tokio` features are split (deps vs dev-deps):** the runtime needs `sync` (for `tokio::sync::Mutex`) and `rt` (for `Arc<Runtime>` to be `Send` across spawns). Tests need `full` for `tokio::test`, `spawn`, `JoinSet`, etc. Matches ferro-reservation precedent.

---

## §Risks & Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|------------|--------|------------|
| R1 | **Naming-clash confusion with `ferro-projections` (plural)** | HIGH | HIGH | D-02/D-51/D-52 lock the disambiguation strategy: rustdoc lead paragraph in `lib.rs`, doc-page title "Live Read-Models" (not "Projections"), README/CLAUDE.md row text explicitly calls out the distinction, crates.io description carries it too. The plan-checker should grep the final RESEARCH.md / lib.rs / docs for the disambiguation paragraph as a pre-commit assertion. |
| R2 | **Broadcast capture in tests has zero workspace precedent** | MEDIUM | MEDIUM | Concern #5 above: ship `BroadcastCapture` helper in `tests/common/mod.rs` using the production code path (real `Broadcaster` + real `mpsc`). Document it inline so future projection tests reuse it. Verified `Broadcaster::add_client` + `subscribe` API surfaces are already public. |
| R3 | **SeaORM 1.x composite PK syntax has no workspace precedent** | MEDIUM | MEDIUM | Concern #1 above: verified pattern (apply `#[sea_orm(primary_key)]` to each column) is documented and trivial. The Plan 03 migration smoke test (`migration_creates_table_with_composite_pk`) proves it before runtime code lands. |
| R4 | **Listener type-erasure through `RwLock<HashMap<TypeId, …>>`** | LOW | MEDIUM | Concern #8 above: traced through `ferro-events/src/dispatcher.rs` — `TypeId::of::<P::Event>()` keying works correctly for distinct projections sharing an event type. D-46 integration test is the canonical proof; build it early in Plan 06 so this is verified end-to-end. |
| R5 | **Cross-dialect upsert (SQLite + Postgres)** | LOW | LOW | Concern #2 above: `OnConflict::columns([..]).update_columns([..])` is dialect-agnostic at the sea-query layer; in-memory SQLite covers the integration test budget; Postgres CI is deferred (same call as Phases 152 / 153 / 154 — sequentially shipped without Postgres CI). |
| R6 | **DashMap shard-lock held across await** | LOW | HIGH (perf, hard to diagnose) | Concern #4 above: clone the `Arc<Mutex<()>>` inside a narrow scope so the `RefMut` drops before `.await`. Mirror the existing `ferro-broadcast` pattern (`drop(channel)` at line 271). Code-comment the pattern in the runtime module for future reviewers. |
| R7 | **First-publish bootstrap requires personal token (cannot automate)** | KNOWN | HIGH (manual step) | Plan 07 documents this as a `user_setup` blocker — same operational reality as Phases 151 / 152 / 153 / 154. The phase cannot complete without the operator running `cargo publish -p ferro-projection` from a local terminal with a `publish-new`-scoped token. CI takes over for subsequent versions. |
| R8 | **`broadcast.{name}.{key}` channel name collision with another app system** | LOW | MEDIUM | D-38 locks the channel format with a `projection.` prefix. The literal prefix is unique to ferro-projection. Document in rustdoc that the prefix is reserved. If a consumer wants private channels, they wait for v0.x `Projection::channel_for(&key)` (Deferred). |

---

## §Open Questions

CONTEXT.md is comprehensive. Two micro-questions surfaced during research; both are answerable by Claude's discretion within CONTEXT.md's bounds:

1. **Should `BroadcastCapture` be exposed as a public `pub mod testing` from `ferro-projection` itself**, so downstream consumer apps testing their own projections can reuse it? **Recommendation: NO for v0.** The helper lives in `tests/common/mod.rs` (test-only, not part of the public API surface). v0.x can promote it to a `#[cfg(any(test, feature = "testing"))] pub mod testing` if a real consumer asks. Adding a public testing module now bakes the test-only API into the semver contract before any consumer needs it.

2. **Should the `version` BIGINT column be exposed on the public read API** (e.g., `read_with_version() -> Option<(P::State, i64)>`)? **Recommendation: NO for v0.** D-25 marks `version` as forward-compat scaffolding for v0.x optimistic concurrency. v0 does not need to expose it on `read`; doing so would create a public surface that v0.x has to maintain even if the OCC design changes. Add `read_with_version` (or `read_required_with_version`) in v0.x when the OCC story lands.

Both are forward-compat decisions, not blockers for Phase 155.

---

## §Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` + Rust toolchain | Build | ✓ | 1.88.0 (workspace `rust-version`) | — |
| `sea-orm` 1.0 | Migration + entity | ✓ | already in `ferro-audit` / `ferro-reservation` deps | — |
| `sea-orm-migration` 1.0 | Migration | ✓ | already in workspace | — |
| `ferro-events` (path dep) | Runtime listener registration | ✓ | path: `../ferro-events`, version 0.2 | — |
| `ferro-broadcast` (path dep) | Delta broadcast | ✓ | path: `../ferro-broadcast`, version 0.2 | — |
| `ferro-reservation` (dev-dep) | D-47 cross-crate showcase | ✓ | path: `../ferro-reservation`, version 0.2 (shipped Phase 154) | — |
| `tokio` 1 | Async runtime | ✓ | workspace dep | — |
| `dashmap` 6 | Per-key Mutex registry | ✓ | already in `ferro-broadcast` / `ferro-stripe` | — |
| `proptest` 1 | D-49 property tests | ✓ | dev-dep added Phase 154 | — |
| `serde` 1 + `serde_json` 1 | JSON state column | ✓ | workspace deps | — |
| `chrono` 0.4 | `updated_at` timestamp | ✓ | workspace dep | — |
| `uuid` 1 | Not actually used in v0 schema — projection_snapshots uses composite (name, key), no UUID PK | ✓ | retained in deps for consistency with `ferro-audit`/`ferro-reservation` Cargo.toml shape, but consider dropping if unused | drop the dep |
| crates.io | First publish | manual | personal `publish-new` token required | none — manual checkpoint |

**Missing dependencies with no fallback:** none. All runtime deps are in the workspace.

**Missing dependencies with fallback:** none.

**Operational blocker (known, unavoidable):** first publish to crates.io requires a `publish-new`-scoped token from the operator's local terminal. CI's `CARGO_REGISTRY_TOKEN` is `publish-update`-only. This is documented as Plan 07's `user_setup` block (same as Phases 151 / 152 / 153 / 154).

**Dep consideration — `uuid`:** Phase 155's `projection_snapshots` table has no UUID column (composite PK is `(VARCHAR, VARCHAR)`). The planner can drop `uuid` from `[dependencies]` if they want a minimally-scoped Cargo.toml. Recommendation: **keep it** for symmetry with `ferro-audit` / `ferro-reservation` and to avoid a downstream surprise if v0.x needs UUIDs (e.g., for an event-log table).

---

## §Plan-Skeleton Recommendation

7 plans, mirroring Phase 154's 7-plan arc exactly.

### Plan-01 — Scaffold the crate (Wave 1, depends_on: [])

**Files created:**
- `ferro-projection/Cargo.toml` (mirror `ferro-reservation/Cargo.toml` shape; deps per §Cargo.toml derivation above)
- `ferro-projection/README.md` (one paragraph: crate purpose + disambiguation pointer to crate docs)
- `ferro-projection/src/lib.rs` (module rustdoc with D-51 disambiguation lead paragraph + per-key diagram + three footguns + canonical example skeleton; module declarations only — no bodies)
- `ferro-projection/src/error.rs` (full `ProjectionError` enum per D-28..D-30, including `From<ferro_broadcast::Error>` + `From<ferro_events::Error>` hand-impls)
- `ferro-projection/src/key.rs` (stub — full body lands in Plan 04)
- `ferro-projection/src/projection.rs` (stub trait declaration — full body lands in Plan 04)
- `ferro-projection/src/runtime.rs` (stub — full body lands in Plan 05)
- `ferro-projection/src/listener.rs` (stub — full body lands in Plan 05)
- `ferro-projection/src/entity.rs` (stub — full body lands in Plan 03)
- `ferro-projection/src/migration.rs` (stub — full body lands in Plan 03)

**Load-bearing acceptance:**
- `cargo build -p ferro-projection` **fails** at this point (crate not in workspace yet) — that's expected; Plan 02 fixes it.
- `ProjectionError` body is complete and unit-tested (D-45 #2 — display strings).
- `lib.rs` rustdoc disambiguation paragraph is present and grep-matchable: `grep -q "Not to be confused with .ferro-projections. (plural)" ferro-projection/src/lib.rs`.

### Plan-02 — Register in workspace (Wave 2, depends_on: [01])

**Files modified:**
- `Cargo.toml` (root): add `"ferro-projection"` to `[workspace.members]`; bump `[workspace.package].version = "0.2.33"` (from `"0.2.32"`)
- `.github/workflows/publish.yml`: add `ferro-projection` to `WAVE1B_CRATES` line (alongside `ferro-reservation`)
- `CLAUDE.md`: insert row for `ferro-projection` in the Workspace Structure table per Specifics §"README.md / CLAUDE.md row text"
- `README.md` (workspace root): insert matching row in the crates table

**Load-bearing acceptance:**
- `cargo build -p ferro-projection` now succeeds (workspace member resolved).
- `cargo test --workspace --all-features` still passes (no other crate broken).
- `grep -q "ferro-projection" .github/workflows/publish.yml` confirms Wave 1b registration.
- `grep -q "Not the same as .ferro-projections. (plural)" README.md CLAUDE.md` confirms the disambiguation row text.

### Plan-03 — Migration + entity (Wave 3, depends_on: [02])

**Files modified:**
- `ferro-projection/src/migration.rs` — full `CreateProjectionSnapshotsTable` body per Concern #1+#7 (composite PK on `(projection_name, key)`, JSON state, BIGINT version, app-set `updated_at`; no secondary indexes per D-24). Migration name `m20260514_000001_create_projection_snapshots_table`.
- `ferro-projection/src/entity.rs` — full `Model` + `Relation` + `ActiveModelBehavior` per Concern #1 (composite primary key, `JsonValue` for state, `i64` for version)
- `ferro-projection/src/lib.rs`: `pub use migration::Migration as CreateProjectionSnapshotsTable;` + `pub use entity::{ActiveModel as ProjectionSnapshotActiveModel, Entity as ProjectionSnapshotEntity, Model as ProjectionSnapshotModel};`

**Tests:**
- `migration_creates_table_with_composite_pk_and_columns` in `src/migration.rs` test mod (mirrors `ferro-audit/src/migration.rs:104-176` shape).
- `state_round_trips_through_json_column` in `src/entity.rs` test mod (mirrors `ferro-reservation/src/entity.rs:86-131` shape).

**Load-bearing acceptance:**
- Composite PK declaration compiles (proves Concern #1 syntax against SeaORM 1.x).
- Migration's `up()` and `down()` both work on in-memory SQLite.
- Entity round-trips through `ActiveModel::insert` + `Entity::find_by_id((name, key))`.

### Plan-04 — Leaf types: Projection trait + ProjectionKey (Wave 4, depends_on: [03])

**Files modified:**
- `ferro-projection/src/key.rs` — full `ProjectionKey` body per D-11 (newtype, `new`/`as_str`/`Display`/`From<String>`/`From<&str>`, serde derive)
- `ferro-projection/src/projection.rs` — full `Projection` trait per D-06..D-10, including default `snapshot_interval()` and `broadcast_event_name()` impls
- `ferro-projection/src/lib.rs`: `pub use key::ProjectionKey;` + `pub use projection::Projection;`

**Tests (D-45 #1, #3):**
- `projection_key_roundtrip` (D-45 #1)
- `projection_key_display`
- `projection_key_serde_roundtrip`
- `default_event_name_is_delta` (D-45 #3)
- `default_snapshot_interval_is_100`

**Load-bearing acceptance:**
- A `struct TestProjection;` impl compiles inside the test mod, validating the trait surface end-to-end (Event/State/Delta/key/apply).
- The rustdoc disambiguation paragraph reappears in the module-level docs of `projection.rs` (final-pass D-51).

### Plan-05 — Runtime body + listener + register + 10 D-45 unit tests (Wave 5, depends_on: [04])

**Files modified:**
- `ferro-projection/src/runtime.rs` — full `ProjectionRuntime<P>` body per D-13 + `new` + `read` + `apply_event` + `register` (D-14..D-22). Implements the 7-step apply algorithm per D-19, the DashMap shard-lock release pattern per Concern #4, the `tokio::sync::Mutex` choice per Concern #3, the `OnConflict::columns([..]).update_columns([..])` upsert per Concern #2.
- `ferro-projection/src/listener.rs` — `ProjectionListener<P>` body per D-15 (NOT `pub` — implementation detail per CONTEXT.md Claude's Discretion); `register` impl on `Arc<Runtime>` clones the Arc into the listener and calls `global_dispatcher().listen(listener)`.
- `ferro-projection/src/lib.rs`: `pub use runtime::ProjectionRuntime;` (and optionally `read_required` helper per CONTEXT.md Discretion — recommended YES, uses `StateNotFound`)
- `ferro-projection/src/runtime.rs` test mod — all 10 D-45 unit tests.

**Tests (D-45 #4..#10 + D-21 + D-22):**
1. `apply_event_happy_path_persists_and_broadcasts` (#5)
2. `apply_event_second_call_folds_onto_loaded_state` (#6)
3. `apply_event_on_new_key_initializes_from_default` (#7)
4. `read_returns_none_for_absent_key_then_some_after_apply` (#8)
5. `rebuild_three_events_equivalent_to_three_applies` (#9 — even though full rebuild body lands in Plan 06, a minimal version may be needed here; alternative: defer to Plan 06)
6. `rebuild_empty_iterator_deletes_row` (#10 — same caveat)
7. `runtime_construction_is_send_sync` (#4)
8. `broadcast_failure_does_not_rollback_snapshot` (D-21)
9. `db_error_surfaces_as_projection_error_db` (D-22)
10. `version_increments_on_each_apply` (D-25)

**Load-bearing acceptance:**
- Per-key Mutex acquisition pattern in `apply_event` source uses the clone-from-RefMut pattern (Concern #4) — `grep -q "\.clone()" ferro-projection/src/runtime.rs` near the DashMap entry; the planner specifies this exactly.
- `cargo clippy --all --all-targets -- -D warnings` passes — proves no `parking_lot::Mutex` accidentally introduced (Concern #3).
- All 10 D-45 unit tests green.
- **Recommendation:** if `rebuild` body is required for tests #5 and #10, fold its body into Plan 05 instead of Plan 06. Otherwise defer those two tests to Plan 06.

### Plan-06 — Rebuild body + integration tests + proptests (Wave 6, depends_on: [05])

**Files modified:**
- `ferro-projection/src/runtime.rs`: add `rebuild` method per D-17 / D-41 / D-42 / D-43 (DELETE then fold + broadcast `"rebuild"` event with full state).
- `ferro-projection/tests/common/mod.rs` (NEW): `BroadcastCapture` helper per Concern #5.
- `ferro-projection/tests/event_bus_integration.rs` (NEW): D-46 — auto-register path end-to-end with 5 dispatched events.
- `ferro-projection/tests/projection_over_reservation_events.rs` (NEW): D-47 — `ReservationCountProjection` driven by Phase 154's `ReservationEvent`. The milestone-completing showcase.
- `ferro-projection/tests/concurrent_apply.rs` (NEW): D-48 — 20 tasks × 5 keys = 4 events per key, asserts per-key serialization + cross-key parallelism.
- `ferro-projection/tests/property_invariants.rs` (NEW): D-49 — 3 properties.

**Tests added:**
- `auto_register_fires_listener_5_times` + `register_twice_fires_listener_twice` (D-36) (`event_bus_integration.rs`)
- `reservation_count_projection_over_reservation_events` (`projection_over_reservation_events.rs`)
- `concurrent_apply_20_tasks_across_5_keys` + `concurrent_read_during_apply` (`concurrent_apply.rs`)
- `proptest_apply_determinism` + `proptest_rebuild_equals_sequential_applies` + `proptest_cross_key_isolation` (`property_invariants.rs`)

**Load-bearing acceptance:**
- D-47 cross-crate showcase test exists and asserts the **full** v11.11 chain: a `ReservationEvent::Held` dispatch flows through the projection runtime, persists a snapshot, broadcasts a delta on `projection.reservation.count.<resource_kind>`. This is the milestone-completing demonstration.
- All 3 proptests pass with default config (256 cases) — no flakiness in CI.
- `cargo test -p ferro-projection --all-features` runs the full integration + proptest suite green.

### Plan-07 — Release: docs + CHANGELOG + manual first-publish (Wave 7, depends_on: [06])

**Files modified:**
- `docs/src/features/live-read-models.md` (NEW): page title "Live Read-Models". Content outline:
  1. Opening paragraph: explicit disambiguation from `ferro-projections` (plural), link to features/projections.md.
  2. What a live read-model is (kernel framing: load → apply → persist → broadcast).
  3. The hand-rolled-pattern anti-pattern (every app emitting events that wants a dashboard hand-rolls this).
  4. Defining a `Projection` (trait surface).
  5. Wiring the runtime (`Arc::new(Runtime::new(...)).register()` one-liner).
  6. The broadcast channel contract (`projection.{name}.{key}` + event name + raw delta payload).
  7. The rebuild affordance.
  8. The three operational footguns (broadcast failure no rollback; single-instance assumption; register-twice-fires-twice).
  9. Worked example: reservation-count dashboard composing ferro-reservation + ferro-projection (expanded from D-47).
- `docs/src/SUMMARY.md`: add `- [Live Read-Models](features/live-read-models.md)` under the Features section, sibling of `Projections` (the plural one). Nav text is "Live Read-Models" NOT "Projections".
- `CHANGELOG.md`: NEW top-level `## ferro-projection` section ABOVE the existing `## ferro-reservation` section. `### [0.2.33] — 2026-05-14` initial-release entry summarising the full public surface per D-56. Include the milestone-completion line: "v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm GuardedUpdate, ferro-audit, ferro-reservation, ferro-projection now all shipped."

**Pre-release gate (run ALL, zero warnings):**
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo build --workspace
cargo test --all-features
cargo doc --no-deps -p ferro-projection
```

**Manual checkpoint (operator runs from local terminal):**
```bash
# Verify the name is available:
# open https://crates.io/search?q=ferro-projection
# Then:
CARGO_REGISTRY_TOKEN=<personal publish-new token> \
cargo publish -p ferro-projection
```

After bootstrap, subsequent versions auto-publish via the existing GH Actions Wave 1b loop on master push.

**Load-bearing acceptance:**
- `docs/src/features/live-read-models.md` exists with title "Live Read-Models" and the disambiguation paragraph appears in the opening section.
- `docs/src/SUMMARY.md` nav text is exactly `Live Read-Models` (not `Projections`).
- `CHANGELOG.md` has the `## ferro-projection` section ABOVE `## ferro-reservation`, with milestone-completion line.
- Pre-release gate is green.
- `ferro-projection` appears on crates.io after the manual bootstrap.

---

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | SeaORM 1.x auto-derives `auto_increment = false` for composite PKs; explicit annotation on each column is the only requirement | Concern #1 | LOW — if explicit `auto_increment = false` is needed and missing, `cargo build` fails loudly with a clear error; the planner adds the explicit annotation in Plan 03 |
| A2 | `OnConflict::columns([..]).update_columns([..])` produces SQLite-compatible `ON CONFLICT(cols) DO UPDATE SET col = excluded.col` | Concern #2 | LOW — verified against sea-query 0.32 docs; if it doesn't, Plan 03's smoke test catches it before runtime code lands |
| A3 | The workspace continues using **no `[workspace.dependencies]` block** (each crate declares its own dep versions) | §Cargo.toml derivation | LOW — verified by `grep` of root Cargo.toml; if the planner introduces a workspace deps block in Phase 155 to centralize sea-orm version, that's an orthogonal refactor outside Phase 155 scope |
| A4 | `tokio::sync::Mutex` is in `tokio = { features = ["sync"] }` | Concern #3 + Cargo.toml derivation | NONE — verified against tokio 1.x docs |
| A5 | `DashMap::entry(k).or_insert_with(f)` holds the shard write lock for as long as the returned `RefMut` is alive | Concern #4 | LOW — verified against `ferro-broadcast/src/broadcaster.rs:271` (`drop(channel); // Release DashMap guard before await`) which uses the same pattern |
| A6 | `Broadcaster::add_client` + `Broadcaster::subscribe` are public and stable | Concern #5 | NONE — verified at `ferro-broadcast/src/broadcaster.rs:77` (`pub fn add_client`) and `:102` (`pub async fn subscribe`) |
| A7 | `ferro_events::EventDispatcher::listen<E, L>` accepts `L: Listener<E>` where `L: Send + Sync + 'static`, sufficient for `ProjectionListener<P>` (which wraps `Arc<ProjectionRuntime<P>>`) | Concern #8 | NONE — verified at `ferro-events/src/dispatcher.rs:73` against `traits.rs:132` |

**No `[ASSUMED]` claims remain at HIGH risk.** All technical concerns are either verified against workspace code or against citable SeaORM / sea-query / tokio / dashmap / ferro-broadcast / ferro-events documentation.

---

## Sources

### Primary (HIGH confidence — workspace code, directly read)
- `ferro-reservation/Cargo.toml` — Wave 1b Cargo.toml template
- `ferro-reservation/src/lib.rs` — module rustdoc tone + structure
- `ferro-audit/Cargo.toml` — secondary Cargo.toml reference (database-only deps)
- `ferro-audit/src/lib.rs` — public re-export shape
- `ferro-audit/src/migration.rs` — SeaORM migration shape, `Expr::current_timestamp()` default, sqlite_master test pattern
- `ferro-audit/src/entity.rs` — SeaORM entity with JSON column + nullable timestamps
- `ferro-orm/Cargo.toml` + `ferro-orm/src/lib.rs` — Wave 1a Cargo.toml reference; surgical `sea_orm` re-export pattern
- `ferro-reservation/src/entity.rs` + `ferro-reservation/src/migration.rs` — single-column UUID PK reference (contrast to composite PK Phase 155 needs)
- `ferro-events/src/lib.rs` + `ferro-events/src/dispatcher.rs` + `ferro-events/src/traits.rs` + `ferro-events/src/error.rs` — listener registration API, `Event: Clone + Send + Sync + 'static`, `Listener<E>: Send + Sync + 'static`, `Error::listener_failed` constructor
- `ferro-broadcast/src/lib.rs` + `ferro-broadcast/src/broadcast.rs` + `ferro-broadcast/src/broadcaster.rs` — `Broadcast::new(broadcaster).channel().event().data().send()` builder, `Broadcaster::add_client` + `subscribe` public API, DashMap shard-lock-drop-before-await pattern
- `framework/src/session/driver/database.rs:199` — `#[sea_orm(primary_key, auto_increment = false)]` pattern reference
- `Cargo.toml` (root) — workspace members + version 0.2.32
- `.github/workflows/publish.yml` — `WAVE1B_CRATES` line (currently includes `ferro-reservation`)
- `155-CONTEXT.md` — D-01..D-56 locked decisions (source of truth)
- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-projection` — original design source
- `.planning/STATE.md` — current workspace version 0.2.32, next version 0.2.33

### Secondary (HIGH-MEDIUM confidence — official documentation, verified via WebFetch)
- `https://www.sea-ql.org/SeaORM/docs/1.1.x/generate-entity/entity-structure/` — composite primary key declaration pattern (multiple `#[sea_orm(primary_key)]` annotations, junction-table example)
- `https://docs.rs/sea-query/0.32/sea_query/query/struct.OnConflict.html` — `OnConflict::columns([..]).update_columns([..])` multi-column conflict target

### Tertiary (assumptions verified against multiple sources)
- tokio 1.x `tokio::sync::Mutex` semantics (Send guard, lock-across-await) — knowledge + workspace evidence (`ferro-broadcast` uses the same async-Mutex pattern in its broadcaster spawn handler)
- dashmap 6 `RefMut` shard-lock semantics — workspace evidence (`ferro-broadcast/src/broadcaster.rs:271` explicitly drops the guard before await)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dep is already used by ferro-audit / ferro-reservation / ferro-broadcast in the workspace; versions verified against existing Cargo.tomls
- Architecture: HIGH — apply algorithm (D-19), per-key Mutex (D-31), listener registration (D-15) all trace through verified workspace code
- Pitfalls: HIGH — DashMap shard-lock pattern verified in ferro-broadcast source; tokio::Mutex vs parking_lot reasoning verified; composite PK / on_conflict syntax verified against SeaORM 1.1.x docs
- Validation Architecture: HIGH — coverage matrix maps every D-decision to at least one test case; fixtures and harness pattern mirror ferro-reservation/tests exactly

**Research date:** 2026-05-14
**Valid until:** 2026-06-13 (30 days — sea-orm 1.x is stable, ferro internal APIs are stable in v0.2.x)

## RESEARCH COMPLETE

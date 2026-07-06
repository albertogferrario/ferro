---
phase: 155
plan: 05
subsystem: ferro-projection
tags: [projection, runtime, listener, sea-orm, broadcast, dashmap, tokio-mutex]
dependency_graph:
  requires: [155-04]
  provides: [ProjectionRuntime-full-body, ProjectionListener-impl, apply-event-7-step, register-wiring]
  affects: [ferro-projection/src/runtime.rs, ferro-projection/src/listener.rs]
tech_stack:
  added: []
  patterns:
    - shard-lock-drop-before-await (DashMap RefMut scoped before tokio Mutex .await)
    - OnConflict composite-PK upsert (sea-query plural columns() form)
    - struct-listener path for ferro_events::Listener<E> (first in-workspace user)
key_files:
  created: []
  modified:
    - ferro-projection/src/runtime.rs
    - ferro-projection/src/listener.rs
decisions:
  - tokio::sync::Mutex (not parking_lot) for per-key locks — held across .await points
  - DashMap RefMut scoped in narrow block, drops before Mutex .await (mirrors broadcaster.rs:271)
  - Broadcast failure does NOT roll back persisted snapshot — tracing::warn + ProjectionError::Broadcast
  - read() does NOT take per-key Mutex — concurrent read+apply_event safe at DB level (D-33)
  - ProjectionListener<P> stays pub(crate) — implementation detail of register()
metrics:
  duration_minutes: 15
  completed: "2026-05-13T23:45:38Z"
  tasks_completed: 3
  files_modified: 2
---

# Phase 155 Plan 05: ProjectionRuntime<P> Body + ProjectionListener<P> Impl Summary

Central plan of Phase 155. The `ProjectionRuntime<P>` body lands: `read`, `read_required`, `apply_event` (7-step D-19 algorithm), and `register` (killer-feature one-line wiring). The `ProjectionListener<P>` body lands: first in-workspace struct-listener path for `ferro_events::Listener<E>`.

## One-liner

Full `ProjectionRuntime<P>` body with 7-step per-key upsert+broadcast apply algorithm, `ProjectionListener<P>` event adapter, and `register()` one-line global-dispatcher wiring.

## Files Overwritten

### `ferro-projection/src/runtime.rs`

Replaced the plan-01 stub (35 lines, `new` only) with the full body (450 lines):

- `read(&self, key: &ProjectionKey) -> Result<Option<P::State>, ProjectionError>` — composite-PK lookup via `Entity::find_by_id((P::NAME, key.0))`, deserializes JSON state. Does NOT acquire the per-key Mutex (D-33).
- `read_required(&self, key) -> Result<P::State, ProjectionError>` — wraps `read`, returns `ProjectionError::StateNotFound { name: P::NAME, key }` if absent (D-30).
- `apply_event(&self, event: &P::Event) -> Result<(), ProjectionError>` — 7-step D-19 algorithm (see below).
- `register(self: Arc<Self>)` — constructs `ProjectionListener { runtime: self.clone() }` and calls `ferro_events::global_dispatcher().listen::<P::Event, _>(listener)`.

### `ferro-projection/src/listener.rs`

Replaced the plan-01 stub (21 lines, struct only) with the full impl (27 lines):

- `pub(crate) struct ProjectionListener<P: Projection>` with `pub(crate) runtime: Arc<ProjectionRuntime<P>>`
- `#[async_trait::async_trait] impl<P: Projection> ferro_events::Listener<P::Event> for ProjectionListener<P>`
- `handle` delegates to `self.runtime.apply_event(event).await` and maps `ProjectionError` to `ferro_events::Error::listener_failed(type_name::<Self>(), e.to_string())`

## 7-Step Apply Algorithm (D-19)

```rust
// Step 1: compute key
let key = self.projection.key(event);

// Step 2: shard-lock-drop-before-await (mirrors broadcaster.rs:271)
let lock_arc: Arc<tokio::sync::Mutex<()>> = {
    self.locks
        .entry(key.0.clone())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}; // <-- RefMut drops here, shard unlocked
let _guard = lock_arc.lock().await;

// Step 3: load snapshot (or Default)
// Step 4: apply (sync, inside Mutex)
let delta = self.projection.apply(&mut state, event);

// Step 5: upsert via composite-PK OnConflict
Entity::insert(am)
    .on_conflict(
        OnConflict::columns([Column::ProjectionName, Column::Key])
            .update_columns([Column::State, Column::Version, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(&self.db).await?;

// Step 6: broadcast (failure does NOT roll back — D-21)
// Step 7: Mutex released (RAII on drop of _guard)
```

## Shard-Lock-Drop-Before-Await Pattern Confirmed

The DashMap `RefMut` is scoped inside `{ ... }` and drops at the closing brace BEFORE the `lock_arc.lock().await` call. This releases the DashMap shard lock before the Mutex acquisition's `.await` point — preventing cross-key concurrency from serializing through the shard.

Pattern source: `ferro-broadcast/src/broadcaster.rs:271` (`drop(channel); // Release DashMap guard before await`).

## SeaORM OnConflict Composite-PK Upsert

```rust
OnConflict::columns([Column::ProjectionName, Column::Key])
    .update_columns([Column::State, Column::Version, Column::UpdatedAt])
    .to_owned()
```

`columns()` plural form for composite conflict target — not chained `.column().column()`. No workspace precedent before this plan; proven against SQLite in-memory in the test suite (Risk R4 closed).

## 10 Unit Tests Added

| Test Name | Coverage |
|---|---|
| `new_returns_owned_runtime_arc_is_send_sync` | D-45 #4: runtime construction, Send+Sync bounds |
| `apply_event_initial_writes_version_1` | D-45 #5: first apply, version=1 |
| `apply_event_second_call_folds_and_bumps_version` | D-45 #6: fold accumulation, version=2 |
| `apply_event_new_key_initializes_from_default` | D-45 #7: per-key Default init |
| `read_returns_none_for_absent_key` | D-45 #8: read None path |
| `read_returns_some_after_apply` | D-45 #8: read Some path |
| `read_required_returns_state_not_found_for_absent` | D-30: StateNotFound variant |
| `version_increments_per_apply_same_key` | 5 sequential applies → version=5 |
| `updated_at_advances_per_apply` | timestamp advances between applies |
| `cross_key_apply_does_not_share_lock` | 3 keys → 3 distinct DashMap entries (D-32 smoke) |

## Cumulative Gate Result

```
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- 13 prior tests (plans 1-4: error, migration, entity, key, projection)
- 10 new runtime tests

`cargo clippy -p ferro-projection --all-targets -- -D warnings` exits 0.
`cargo doc -p ferro-projection --no-deps` exits 0.

## Risk Closures

- **R4 (OnConflict no-precedent risk):** Closed. The composite-PK upsert test suite proves the `OnConflict::columns([..])` path works against SQLite.
- **R5 (first-user-of-listener-trait risk):** Closed at construction path. `ProjectionListener<P>` implements `ferro_events::Listener<P::Event>` and compiles clean. Plan 06's `event_bus_integration.rs` integration test proves the dispatch path end-to-end.

## First In-Workspace Struct-Listener

`ProjectionListener<P>` is the first `struct ... impl Listener<E> for ...` in the workspace. All prior ferro-events usage in tests uses the closure `EventDispatcher::on(closure)` API. Phase 155 is the first consumer of the struct-listener path (RESEARCH.md §Technical Concerns #8).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused imports (`self`, `ActiveModelTrait`)**
- **Found during:** Task 3 (clippy gate)
- **Issue:** `use crate::entity::{self, ...}` imported the module alias `self` which was unused; `ActiveModelTrait` imported but not called directly (SeaORM uses it implicitly via the `EntityTrait::insert` path)
- **Fix:** Removed `self` and `ActiveModelTrait` from the import list
- **Files modified:** `ferro-projection/src/runtime.rs`
- **Commit:** included in task 2 commit (7c081b34)

## What Plan 06 Adds

- `rebuild` method body (D-17, D-42-D-44)
- `event_bus_integration.rs` integration test (D-46): full `register() → dispatch() → snapshot persisted` chain
- `projection_over_reservation_events.rs` milestone showcase (D-47)
- `concurrent_apply.rs` concurrency test (D-48): 20 concurrent tasks across 5 keys
- `property_invariants.rs` proptest suite (D-49): determinism, replay equivalence, cross-key independence

## Self-Check: PASSED

- runtime.rs: FOUND
- listener.rs: FOUND
- SUMMARY.md: FOUND
- Commit f6c799fe (listener): FOUND
- Commit 7c081b34 (runtime): FOUND

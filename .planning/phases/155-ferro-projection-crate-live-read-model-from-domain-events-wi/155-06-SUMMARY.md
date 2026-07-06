---
phase: 155
plan: "06"
subsystem: ferro-projection
tags: [projection, live-read-model, integration-tests, proptest, rebuild, broadcast]
dependency_graph:
  requires: [155-05]
  provides: [rebuild-method, broadcast-capture-helper, event-bus-integration, reservation-showcase, concurrent-apply, proptest-properties]
  affects: [ferro-projection]
tech_stack:
  added: []
  patterns: [rebuild-delete-fold-insert, broadcast-capture-via-real-broadcaster, proptest-with-tokio-block-on, per-key-serialization-proof, cross-crate-composition-showcase]
key_files:
  created:
    - ferro-projection/tests/common/mod.rs
    - ferro-projection/tests/event_bus_integration.rs
    - ferro-projection/tests/projection_over_reservation_events.rs
    - ferro-projection/tests/concurrent_apply.rs
    - ferro-projection/tests/proptest_properties.rs
  modified:
    - ferro-projection/src/runtime.rs
decisions:
  - rebuild uses DELETE + fold + INSERT (not an upsert) — delete_by_id before folding matches D-42 wipe-then-rebuild semantics
  - BroadcastCapture uses real Broadcaster::new + add_client + subscribe — no mocks, locked by RESEARCH.md §Technical Concerns #5
  - concurrent_apply holds a cloned DatabaseConnection before Arc wrapping runtime — db field is pub(crate); conn.clone() is cheap (Arc-backed)
  - clippy uninlined_format_args fixed in concurrent_apply.rs and proptest_properties.rs (deviation Rule 1)
metrics:
  completed: "2026-05-14"
  tasks: 7
  files_modified: 6
---

# Phase 155 Plan 06: rebuild body + integration tests + proptest Summary

rebuild<I> method on ProjectionRuntime, BroadcastCapture test helper, and four integration tests closing all load-bearing risks: event-bus auto-register (D-46), cross-crate reservation showcase (D-47), concurrent per-key serialization (D-48), and three proptest replay-correctness properties (D-49).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | rebuild<I> method + 2 unit tests | 9bf80184 | ferro-projection/src/runtime.rs |
| 2 | BroadcastCapture helper | 60ba905d | ferro-projection/tests/common/mod.rs |
| 3 | event_bus_integration.rs (D-46) | 682cf49c | ferro-projection/tests/event_bus_integration.rs |
| 4 | projection_over_reservation_events.rs (D-47) | 8c3fe57f | ferro-projection/tests/projection_over_reservation_events.rs |
| 5 | concurrent_apply.rs (D-48) | 5afbcd49 | ferro-projection/tests/concurrent_apply.rs |
| 6 | proptest_properties.rs (D-49) | 8e67b117 | ferro-projection/tests/proptest_properties.rs |
| 7 | Full gate: clippy + doc fix | f7c965cd | ferro-projection/tests/concurrent_apply.rs, proptest_properties.rs |

## Key Deliverables

### rebuild<I> method (D-17, D-41, D-42, D-43, D-44)

Signature:
```rust
pub async fn rebuild<I>(
    &self,
    key: &ProjectionKey,
    events: I,
) -> Result<P::State, ProjectionError>
where
    I: IntoIterator<Item = P::Event>
```

Behavior:
- Acquires the same per-key Mutex as `apply_event` (serializes against in-flight applies)
- `Entity::delete_by_id((P::NAME, key))` — DELETE before fold (D-42)
- Folds event iterator through `P::State::default()` via `P::apply`
- Empty iterator: returns `Default`, no INSERT, no broadcast (D-43)
- Non-empty: `Entity::insert(am)` with `version = events.len() as i64`
- Broadcasts ONE `"rebuild"` frame (overrides `P::broadcast_event_name`) with full final state (D-41)
- Not transactional (D-44); documented in rustdoc

Unit tests added:
- `rebuild_three_events_equals_three_sequential_applies` (D-45 #9): Path A (3 applies) == Path B (rebuild with same events), both total=15
- `rebuild_empty_deletes_row_and_returns_default` (D-45 #10): seed value, rebuild with empty Vec, row wiped, returns Default

### BroadcastCapture test helper

Uses production code path: `Broadcaster::new()` + `add_client(socket_id, mpsc::Sender)` + `subscribe(socket_id, channel, None, None)`. `drain()` filters `ServerMessage::Event(_)` variants. No mocks.

### D-47 Milestone-Completing Showcase Test

`reservation_events_fold_into_per_resource_kind_counters` in `tests/projection_over_reservation_events.rs` demonstrates the v11.11 four-primitive composition: `ReservationCountProjection` folds `ReservationEvent::{Held, Committed, Released}` into per-`resource_kind` counters; 3 Held + 1 Committed + 1 Released on `"inventory.unit"` produces `state.held=3, committed=1, released=1` with ≥5 broadcast frames on `projection.reservations.counters.inventory.unit`. A maintainer reading this single file understands the full ferro-orm + ferro-audit + ferro-reservation + ferro-projection composition.

## Cumulative Test Count

```
cargo test -p ferro-projection
  lib:            25 passed  (23 from plan 05 + 2 rebuild from this plan)
  concurrent_apply:  1 passed  (D-48)
  event_bus_integration: 1 passed (D-46)
  projection_over_reservation_events: 1 passed (D-47)
  proptest_properties: 3 passed (D-49)
  doc-tests:      1 passed (+ 2 ignored)
  TOTAL:          31 non-doc tests passing
```

## Load-Bearing Risk Closure

| Risk | Test | Status |
|------|------|--------|
| R3 (composite-PK no-precedent) | plan 03 smoke tests | Closed (plan 03) |
| R4 (OnConflict no-precedent) | plan 05 apply_event tests | Closed (plan 05) |
| R5 (first-user-of-listener-trait) | event_bus_integration.rs (D-46) | **Closed (this plan)** |
| Cross-key concurrency claim | concurrent_apply.rs (D-48) | **Closed (this plan)** |

## Gate Status

- `cargo test -p ferro-projection` — 31 passing
- `cargo clippy -p ferro-projection --all-targets -- -D warnings` — 0 warnings
- `cargo doc -p ferro-projection --no-deps` — clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy uninlined_format_args in test files**
- **Found during:** Task 7 (full gate)
- **Issue:** `format!("key-{}", key_idx)` and `format!("k-{}", k)` in concurrent_apply.rs and proptest_properties.rs triggered `-D warnings`
- **Fix:** Inlined to `format!("key-{key_idx}")` and `format!("k-{k}")`
- **Files modified:** ferro-projection/tests/concurrent_apply.rs, ferro-projection/tests/proptest_properties.rs
- **Commit:** f7c965cd

## Known Stubs

None — all projection state is live data driven by applied events. No placeholders.

## Threat Flags

None — test files only; no new network endpoints or auth paths.

## Next Plan

Plan 07: user-facing docs (`docs/src/features/live-read-models.md`), CHANGELOG entry with milestone-completion narrative, workspace-wide gate (`cargo test --all-features`), and manual first-publish bootstrap for `ferro-projection` to crates.io.

---
phase: 246-result-read-model-snapshot
plan: "01"
subsystem: ferro-projection
tags: [projection, snapshot, offload, persistence, sea-orm]
dependency_graph:
  requires: []
  provides:
    - ferro_projection::snapshot_write
    - ferro_projection::snapshot_read
  affects:
    - ferro-projection public API
tech_stack:
  added: []
  patterns:
    - SeaORM OnConflict upsert on composite PK (projection_name, key)
    - Free functions over existing entity (no new table, no new migration)
    - sqlite::memory: + TestMigrator inline unit test pattern
key_files:
  created:
    - ferro-projection/src/direct.rs
  modified:
    - ferro-projection/src/lib.rs
decisions:
  - "version fixed at 1 on snapshot_write; Column::Version omitted from update_columns (D-02: no meaningful version for one-shot results)"
  - "Column stays crate-internal; only the two free functions are public (direct.rs imports entity::{Column, Entity} privately, matching runtime.rs)"
  - "snapshot_read returns Ok(None) for absent keys — no fabricated pending state (D-08)"
metrics:
  duration: 256s
  completed: "2026-08-13T21:35:27Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 246 Plan 01: Direct Snapshot Write/Read API — Summary

**One-liner:** Two free functions (`snapshot_write`, `snapshot_read`) over the existing `projection_snapshots` entity, providing a direct write/read path decoupled from the event-fold `Projection` trait using the same SeaORM upsert idiom as `apply_event`.

## What Was Built

`ferro-projection/src/direct.rs` adds `snapshot_write` and `snapshot_read` as `pub async fn` free functions. Both operate over the existing `projection_snapshots` entity (composite PK `(projection_name, key)`, `state: JsonValue`, `version: i64`, `updated_at`) with no new table or migration (D-02).

### Public Signatures

```rust
pub async fn snapshot_write(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
    state: JsonValue,
) -> Result<(), ProjectionError>

pub async fn snapshot_read(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
) -> Result<Option<JsonValue>, ProjectionError>
```

### Key Design Points

- **`version = 1` (fixed):** `ActiveValue::Set(1_i64)` on every `snapshot_write`. The `OnConflict` update clause includes only `[Column::State, Column::UpdatedAt]` — `Column::Version` is deliberately omitted. A repeat write to the same `(name, key)` overwrites `state` and `updated_at` without changing `version`. For one-shot offload results, version tracking adds no value and a read-modify-write increment would add concurrency risk (anti-pattern noted in RESEARCH §Pattern 1).

- **`Column` stays private:** `direct.rs` imports `crate::entity::{ActiveModel, Column, Entity}` as crate-internal items (same pattern as `runtime.rs`). Only the two free functions are re-exported from `lib.rs`. The internal SeaORM column type does not leak through the public API; the framework facade in Plan 02 will compose `snapshot_write` rather than touching `Column` directly.

- **`snapshot_read` returns `Ok(None)` for absent keys:** No fabricated pending row, no error. Callers (and downstream Plan 02 `read_result`) interpret `None` as "not yet done" (D-08).

### `lib.rs` Additions

```rust
mod direct;
// ...
pub use direct::{snapshot_read, snapshot_write};
```

`entity::Column` is not added to the re-export block.

## Test Results

Three unit tests in `#[cfg(test)] mod tests` inside `direct.rs`, using the `TestMigrator` + `sqlite::memory:` pattern from `runtime.rs`:

| Test | Behavior | Result |
|------|----------|--------|
| `direct_snapshot_round_trip` | write → read returns the same `JsonValue` | green |
| `direct_snapshot_overwrite` | two writes same key → read returns second value, no error | green |
| `snapshot_read_returns_none_for_absent` | read on a never-written key returns `Ok(None)` | green |

Full crate test suite: `cargo test -p ferro-projection` — **31 tests, all passed**. Existing tests unaffected.

Linting: `cargo fmt --all -- --check` and `cargo clippy -p ferro-projection --all-targets -- -D warnings` both clean.

## Deviations from Plan

None — plan executed exactly as written. The only minor adaptation was that `rustfmt` preferred single-line `.await.expect(...)` chains for short `snapshot_read` calls in the test module; applied immediately before commit.

## Commits

| Hash | Message |
|------|---------|
| `2d7b3420` | feat(246-01): add direct snapshot write/read API to ferro-projection |

## Known Stubs

None. The two functions are fully implemented and tested.

## Threat Flags

None beyond the plan's threat model. The `direct.rs` functions use parameterized SeaORM queries (`Entity::insert` / `find_by_id`) — no string-concatenated SQL (T-246-tamper mitigated). The unbounded growth note (T-246-DoS) is already in the plan as accepted/deferred.

## Self-Check: PASSED

- `ferro-projection/src/direct.rs` — FOUND
- `ferro-projection/src/lib.rs` (mod direct + pub use) — FOUND
- Commit `2d7b3420` — verified via `git rev-parse --short HEAD`
- All three unit tests green (evidence: `cargo test -p ferro-projection` output above)

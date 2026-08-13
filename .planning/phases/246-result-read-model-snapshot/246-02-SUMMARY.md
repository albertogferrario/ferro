---
phase: 246-result-read-model-snapshot
plan: "02"
subsystem: framework
tags: [offload, projection, snapshot, facade, envelope, serde]
dependency_graph:
  requires:
    - ferro_projection::snapshot_write (246-01)
    - ferro_projection::snapshot_read (246-01)
    - ferro_queue::OffloadSerializable (245)
  provides:
    - ferro::offload::persist_result
    - ferro::offload::persist_error
    - ferro::offload::read_result
    - ferro::offload::OffloadResult<T>
    - ferro::offload::OFFLOAD_PROJECTION_NAME
  affects:
    - framework public API (::ferro::offload::* path)
    - framework/Cargo.toml (new ferro-projection dep)
tech_stack:
  added: []
  patterns:
    - serde internally-tagged enum (status/value|error envelope)
    - Compose direct API — no raw SeaORM in framework facade
    - sqlite::memory: + TestMigrator inline unit test pattern (from ferro-projection)
key_files:
  created:
    - framework/src/offload.rs
  modified:
    - framework/Cargo.toml
    - framework/src/lib.rs
decisions:
  - "persist_result/persist_error compose ferro_projection::snapshot_write — no SeaORM column types in framework's public surface"
  - "Error type is ferro_projection::ProjectionError (From<serde_json::Error> already exists); no new OffloadError type in framework"
  - "pub mod offload; is top-level sibling of pub mod queue, making ::ferro::offload::* the correct macro-emission paths (D-11)"
  - "ferro-projection added as always-on (non-optional) dep in framework/Cargo.toml (D-11, RESEARCH RQ6)"
metrics:
  duration: 261s
  completed: "2026-08-13T21:42:16Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 246 Plan 02: Offload Result Facade — Summary

**One-liner:** `framework/src/offload.rs` composes `ferro_projection::snapshot_write/read` into `persist_result`/`persist_error`/`read_result` + `OffloadResult<T>` envelope, making `::ferro::offload::*` the macro-facing result persistence surface.

## What Was Built

### `framework/src/offload.rs` (new, 235 lines)

Four public items:

```rust
pub const OFFLOAD_PROJECTION_NAME: &str = "offload.result";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OffloadResult<T> {
    Completed { value: T },
    Failed { error: String },
}

pub async fn persist_result<T: OffloadSerializable>(
    handle_key: &str,
    value: &T,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError>

pub async fn persist_error(
    handle_key: &str,
    error: &str,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError>

pub async fn read_result<T: OffloadSerializable>(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, ProjectionError>
```

### Key Design Points

- **Composes Plan 01 — no SeaORM leak:** `persist_result` and `persist_error` build a `serde_json::json!` envelope and call `ferro_projection::snapshot_write`. No `OnConflict`, no `Entity::insert`, no `Column::*` reference in the facade. Verified by `! grep -q "OnConflict" framework/src/offload.rs`.

- **Error type reuse:** `ProjectionError` already has `#[from] serde_json::Error` (via the `Json` variant). No new error type in `framework`. The `serde_json::to_value(value).map_err(ProjectionError::from)?` call in `persist_result` uses this conversion directly.

- **`()` round-trip verified (Pitfall 3):** `serde_json::to_value(&())` yields `Value::Null`. The stored envelope is `{"status":"completed","value":null}`. `serde_json::from_value::<OffloadResult<()>>(…)` with the internally-tagged serde representation deserializes correctly back to `Completed { value: () }`. The `offload_result_unit_output` test asserts this.

- **Non-fatal contract (T-246-05):** Both `persist_result` and `persist_error` return `Result<(), ProjectionError>` rather than panicking. Documented in the module doc: callers (Plan 04 worker hook) must `tracing::warn!` on error and continue — never fail the job.

### `framework/Cargo.toml`

```toml
ferro-projection = { path = "../ferro-projection", version = "0.3" }
```

Added immediately after the existing `ferro-projections` (plural, optional) line. Always-on (no feature gate) because `ferro-queue` is already always-on and the offload result path is part of the core queue substrate (D-11, RESEARCH RQ6). No transitive cycle: `ferro-projection` depends only on `ferro-events` + `ferro-broadcast`, both already in `framework`'s graph.

### `framework/src/lib.rs`

```rust
/// Offload result persistence and retrieval helpers.
pub mod offload;
```

Added as a top-level module declaration (line 226), before `pub mod queue` (line 229). Resolves `::ferro::offload::persist_result`, `::ferro::offload::persist_error`, `::ferro::offload::read_result`, and `::ferro::offload::OffloadResult` — exactly the paths Plan 04's macro and worker hook will emit (D-11).

## Test Results

Four unit tests in `#[cfg(test)] mod tests` inside `framework/src/offload.rs`, using the `TestMigrator` + `sqlite::memory:` pattern from `ferro-projection/src/direct.rs`:

| Test | Behavior | Result |
|------|----------|--------|
| `offload_result_completed_round_trip` | `persist_result("k1", &"hello", db)` → `read_result::<String>("k1", db)` == `Completed { value: "hello" }` | green |
| `offload_result_failed_round_trip` | `persist_error("k2", "boom", db)` → `read_result::<String>("k2", db)` == `Failed { error: "boom" }` | green |
| `offload_result_absent_is_none` | `read_result::<String>("nope", db)` == `None` | green |
| `offload_result_unit_output` | `persist_result("k3", &(), db)` → `read_result::<()>("k3", db)` == `Completed { value: () }` | green |

`cargo test -p ferro-rs offload_result` — **4 tests, all passed**.
`cargo fmt --all -- --check` — clean.
`cargo clippy -p ferro-rs --all-targets -- -D warnings` — clean.

## Note for Plan 04 (worker hook)

The Plan 04 worker hook must:
1. Call `::ferro::offload::persist_result(key, &value, db)` after a successful `handle()` and `tracing::warn!` on `Err` — do NOT return `Err` from the hook (non-fatal contract).
2. Call `::ferro::offload::persist_error(key, err_msg, db)` when retries are exhausted (`handle_failure` path) and `tracing::warn!` on `Err`.
3. Obtain `db` from `::ferro::queue::Queue::connection()` (`&'static DatabaseConnection`, D-12).

## Deviations from Plan

None — plan executed exactly as written. The only adaptation was applying `cargo fmt` to collapse short `.await?` chains, as rustfmt preferred single-line forms for those two call sites.

## Known Stubs

None. All four public items are fully implemented and tested.

## Threat Flags

None beyond the plan's threat model. The facade uses parameterized `snapshot_write` (no string-concatenated SQL). T-246-02 (error string disclosure) is accepted and documented in the module-level doc comment.

## Commits

| Hash | Message |
|------|---------|
| `a7fea55b` | chore(246-02): add always-on ferro-projection dep to framework |
| `8987adc6` | feat(246-02): add offload result facade (offload.rs + lib.rs wiring) |

## Self-Check: PASSED

- `framework/src/offload.rs` — FOUND (235 lines)
- `framework/src/lib.rs` contains `pub mod offload;` at line 226 (top-level) — FOUND
- `framework/Cargo.toml` contains non-optional `ferro-projection` dep — FOUND
- Commit `a7fea55b` — recorded
- Commit `8987adc6` — recorded
- All four unit tests green (evidence: `cargo test -p ferro-rs offload_result` output above)
- `OFFLOAD_PROJECTION_NAME = "offload.result"` — FOUND
- No `OnConflict` in offload.rs — VERIFIED
- `cargo clippy -p ferro-rs --all-targets -- -D warnings` — clean

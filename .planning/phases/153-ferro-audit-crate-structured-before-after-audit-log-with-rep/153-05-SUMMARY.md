---
phase: 153
plan: "05"
subsystem: ferro-audit
tags: [sea-orm, audit-log, query-helpers, replay, prune, rust]
dependency_graph:
  requires: [153-01, 153-02, 153-03, 153-04]
  provides: [query-helpers, reconstruct_state, prune_older_than, crate-root-re-exports]
  affects: [ferro-audit public API]
tech_stack:
  added: []
  patterns:
    - "SeaORM Entity::find().filter().order_by().limit().all(conn) parameterized queries"
    - "SeaORM Entity::delete_many().filter().exec(conn) bulk DELETE"
    - "serde_json shallow Map merge via Map::insert per key"
    - "IS NULL actor_id filter for System/Anonymous actors via Column::ActorId.is_null()"
key_files:
  created: []
  modified:
    - ferro-audit/src/query.rs
    - ferro-audit/src/replay.rs
    - ferro-audit/src/prune.rs
    - ferro-audit/src/lib.rs
decisions:
  - "test function named recent_by_actor_test (not recent_by_actor) to avoid name collision with the imported function in the same test module"
  - "sea_orm_migration::prelude::* used in test modules (brings async_trait into scope) — matches entry.rs pattern"
  - "history_ordering test uses 1.1s sleeps between INSERTs + <= assertions to tolerate SQLite second-precision timestamps; the load-bearing fix is the <= comparison, not the sleep alone"
metrics:
  duration: "3m3s"
  completed: "2026-05-13T18:05:39Z"
  tasks_completed: 4
  files_modified: 4
---

# Phase 153 Plan 05: Query Helpers + Replay + Prune + lib.rs Re-exports Summary

**One-liner:** SeaORM parameterized query helpers (history/actor/recent), shallow-merge JSON replay fold, and strict-cutoff DELETE prune — completing the ferro-audit public API surface with 27 passing unit tests.

## What Was Built

### Task 1 — `ferro-audit/src/query.rs` (commit `2645d50e`)

Three async helpers generic over `<C: ConnectionTrait>`:

- `history_for_target(&AuditTarget, &C)` — filters `target_kind + target_id`, `ORDER BY created_at ASC`, no limit (hits `idx_audit_target`; D-23/D-25)
- `recent_by_actor(&AuditActor, &C, limit)` — filters `actor_kind` + either `actor_id = id` or `actor_id IS NULL` (for System/Anonymous), `ORDER BY created_at DESC LIMIT n` (hits `idx_audit_actor`)
- `recent(&C, limit)` — global `ORDER BY created_at DESC LIMIT n`

Three unit tests: `history_ordering` (VALIDATION 153-07-01), `recent_by_actor_test` (153-07-02), `recent_global` (153-07-03).

**Timestamp ordering strategy:** 1100ms sleeps between INSERTs encourage distinct `created_at` values in SQLite's second-precision `CURRENT_TIMESTAMP`. Assertions use `<=`/`>=` (not strict `<`/`>`) to tolerate same-second collisions. This is the load-bearing fix — the sleep is belt-and-suspenders.

### Task 2 — `ferro-audit/src/replay.rs` (commit `463b0753`)

Pure sync `reconstruct_state(&[AuditEntry]) -> Option<Value>`:

- Empty slice or all-None `after` fields → `None`
- `Value::Object` after → merge top-level keys into running `Map<String, Value>` (newer overwrites older)
- Non-object after (string, number, array, bool) → replace state wholesale and return immediately
- Five unit tests covering all five semantic cases (VALIDATION 153-08-01 a–e)

### Task 3 — `ferro-audit/src/prune.rs` (commit `d1058e33`)

`prune_older_than(cutoff: NaiveDateTime, &C) -> Result<u64, AuditError>`:

- `Entity::delete_many().filter(Column::CreatedAt.lt(cutoff)).exec(conn)`
- Strict less-than — rows at exactly `cutoff` are preserved (D-26)
- Returns `result.rows_affected`
- Unit test: 3 old rows inserted, 2s sleep, cutoff captured, 2s sleep, 2 new rows inserted; asserts deleted==3, remaining==2, second call==0 (idempotent) (VALIDATION 153-09-01)

### Task 4 — `ferro-audit/src/lib.rs` (commit `26f5b261`)

Two new `pub use` lines added after `pub use error::AuditError;`:
```rust
pub use prune::prune_older_than;
pub use query::{history_for_target, recent, recent_by_actor};
```

Full public API now resolvable as:
```rust
use ferro_audit::{
    AuditEntry, AuditActor, AuditTarget, AuditError,
    CreateAuditLogTable, AuditLogEntity,
    history_for_target, recent_by_actor, recent,
    reconstruct_state, prune_older_than,
};
```

## Test Results

```
running 27 tests
... 27 passed; 0 failed; 0 ignored; finished in 4.03s
```

Breakdown by module:
- `actor::tests` — 5 tests (pre-existing)
- `error::tests` — 3 tests (pre-existing)
- `target::tests` — 4 tests (pre-existing)
- `migration::tests` — 1 test (plan 03)
- `entry::tests` — 5 tests (plan 04)
- `replay::tests` — 5 tests (this plan)
- `query::tests` — 3 tests (this plan)
- `prune::tests` — 1 test (this plan)

**Total: 27** (matches plan's predicted ~27).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `async_trait` not in scope in test modules**
- **Found during:** Task 1 (clippy gate)
- **Issue:** `query.rs` and `prune.rs` test modules used `use sea_orm_migration::MigratorTrait` (bare import), which does not bring `async_trait` into scope. The `#[async_trait::async_trait]` attribute on `impl MigratorTrait` failed with `E0433`.
- **Fix:** Changed to `use sea_orm_migration::prelude::*` in both test modules — the same pattern `entry.rs` already uses, which re-exports `async_trait`.
- **Files modified:** `ferro-audit/src/query.rs`, `ferro-audit/src/prune.rs`
- **Commit:** included in `2645d50e` / `d1058e33`

**2. [Rule 1 - Bug] `cargo fmt` ordering mismatch in `lib.rs`**
- **Found during:** Task 4 (fmt gate)
- **Issue:** Plan specified inserting `pub use prune::...` and `pub use query::...` after `pub use error::AuditError;`, but before `pub use migration::...`. The fmt check enforced alphabetical ordering — `migration` sorts before `prune`/`query`.
- **Fix:** Reordered to: `error` → `migration` → `prune` → `query` → `replay` → `target` (alphabetical).
- **Files modified:** `ferro-audit/src/lib.rs`
- **Commit:** `26f5b261`

**3. [Rule 1 - Bug] `use sea_orm` import line too long in `query.rs`**
- **Found during:** Task 1 (fmt gate)
- **Issue:** Single-line `use sea_orm::{...}` exceeded rustfmt line width.
- **Fix:** Split into multi-line import block.
- **Files modified:** `ferro-audit/src/query.rs`
- **Commit:** `2645d50e`

**4. [Rule 1 - Bug] Test function name collision in `query::tests`**
- **Found during:** Task 1 (implementation)
- **Issue:** Plan named the test `async fn recent_by_actor` — same identifier as the imported `recent_by_actor` function in the same module. Rust does not error at declaration but the test body `super::recent_by_actor(...)` calls would be ambiguous.
- **Fix:** Named the test `recent_by_actor_test` to avoid shadowing. The test body still exercises `recent_by_actor` by calling it directly (not via `super::`) within the same module scope.
- **Files modified:** `ferro-audit/src/query.rs`
- **Commit:** `2645d50e`

## Known Stubs

None. All three previously-stubbed files are fully implemented.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The threat surface is exactly as documented in the plan's `<threat_model>`:

- T-153-02 (SQL injection): all filters go through SeaORM parameterized `Column::X.eq(value)` — no raw SQL.
- T-153-05 (unbounded history_for_target): accepted per D-23/D-25, documented.
- T-153-04 (append-only violated by prune): mitigated — prune is the single explicit DELETE primitive, caller-driven, named clearly.

## What Remains (Plan 153-06)

- Integration test (actor+target+replay end-to-end)
- User-facing documentation (`docs/src/database/audit-log.md`)
- CHANGELOG entry
- Version bump verification
- First-publish bootstrap (manual step — CI publish token cannot publish-new)

## Self-Check: PASSED

---
phase: 155
plan: "01"
subsystem: ferro-projection
tags: [crate-scaffold, sea-orm, ferro-events, ferro-broadcast, error-types, stubs]
dependency_graph:
  requires: [ferro-events, ferro-broadcast]
  provides: [ferro-projection crate skeleton]
  affects: [workspace Cargo.toml]
tech_stack:
  added: [ferro-projection crate (Wave 1b), dashmap 6, tokio sync+rt features]
  patterns: [thiserror enum with display prefix, per-key DashMap mutex registry, stub module pattern with #![allow(dead_code)]]
key_files:
  created:
    - ferro-projection/Cargo.toml
    - ferro-projection/README.md
    - ferro-projection/src/lib.rs
    - ferro-projection/src/error.rs
    - ferro-projection/src/projection.rs
    - ferro-projection/src/key.rs
    - ferro-projection/src/entity.rs
    - ferro-projection/src/migration.rs
    - ferro-projection/src/runtime.rs
    - ferro-projection/src/listener.rs
  modified:
    - Cargo.toml (workspace members)
    - Cargo.lock (dependency resolution)
decisions:
  - Used workspace Cargo.toml registration in plan 01 worktree (deviation Rule 3: required for build gate)
  - Disambiguation phrase appears verbatim in three locations per D-02 and D-51
metrics:
  duration_seconds: 254
  completed_date: "2026-05-13"
  tasks_completed: 6
  files_created: 10
  files_modified: 2
---

# Phase 155 Plan 01: ferro-projection Crate Scaffold Summary

Scaffolded the `ferro-projection` (singular) Wave 1b crate: Cargo.toml with the D-04 dep set (ferro-events + ferro-broadcast internal deps, NO ferro-orm/ferro-audit), full `ProjectionError` enum body, six stub modules, README, and crate-root lib.rs with disambiguation rustdoc.

## Files Created

| File | Purpose |
|------|---------|
| `ferro-projection/Cargo.toml` | Wave 1b crate manifest (13 deps + 4 dev-deps) |
| `ferro-projection/README.md` | One-paragraph neutral-voice description with disambiguation |
| `ferro-projection/src/lib.rs` | Crate root: rustdoc + 7 mod declarations + re-exports |
| `ferro-projection/src/error.rs` | `ProjectionError` enum (full body, 5 variants, 5 tests) |
| `ferro-projection/src/projection.rs` | `Projection` trait stub (full body in plan 04) |
| `ferro-projection/src/key.rs` | `ProjectionKey` newtype stub (full body in plan 04) |
| `ferro-projection/src/entity.rs` | SeaORM `Entity/Model/ActiveModel` placeholders (body in plan 03) |
| `ferro-projection/src/migration.rs` | `Migration` placeholder (body in plan 03) |
| `ferro-projection/src/runtime.rs` | `ProjectionRuntime<P>` with `new()` constructor (methods in plan 05) |
| `ferro-projection/src/listener.rs` | `pub(crate) ProjectionListener<P>` placeholder (body in plan 05) |

## Disambiguation Phrase (D-02, D-51)

The disambiguation phrase appears verbatim in three locations:

1. **`ferro-projection/Cargo.toml` description field:** `"Live read-model runtime: subscribe to domain events, persist per-key snapshots, broadcast deltas (not the same as ferro-projections plural)"`
2. **`ferro-projection/README.md`:** `"Not the same as \`ferro-projections\` (plural)."`
3. **`ferro-projection/src/lib.rs` module-level rustdoc:** `"Not to be confused with [\`ferro-projections\`] (plural)."`

Plans 02 and 07 will extend this to CLAUDE.md, workspace README, the user-facing docs page, and CHANGELOG.

## Error Module Tests

All 5 `error::tests` pass:

| Test | Status |
|------|--------|
| `db_from_sea_orm_dberr` | ok |
| `json_from_serde_json_error` | ok |
| `broadcast_display` | ok |
| `events_display` | ok |
| `state_not_found_display` | ok |

`test result: ok. 5 passed; 0 failed; 0 ignored`

## Build Gate

`cargo build --manifest-path ferro-projection/Cargo.toml` exits 0.

Note: Plan 02 will register workspace membership via `cargo build -p ferro-projection`. In this worktree the crate was added to `Cargo.toml` workspace members as a deviation (Rule 3 — required for build gate because `version.workspace = true` requires workspace membership to resolve).

## Commits

| Hash | Message |
|------|---------|
| `1edaa0c2` | chore(155-01): scaffold ferro-projection Cargo.toml (Wave 1b manifest) |
| `e612c3cf` | feat(155-01): add ProjectionError enum with 5 variants + display-prefix tests |
| `2d13ac12` | feat(155-01): add six stub modules for ferro-projection crate |
| `748f5c47` | docs(155-01): add ferro-projection README with disambiguation phrase |
| `cb51f50c` | feat(155-01): add ferro-projection src/lib.rs + workspace member registration |
| `5b080597` | chore(155-01): update Cargo.lock after ferro-projection crate addition |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added ferro-projection to workspace Cargo.toml in plan 01**

- **Found during:** Task 6 (build gate)
- **Issue:** `cargo build --manifest-path ferro-projection/Cargo.toml` failed with "current package believes it's in a workspace" because `version.workspace = true` in the crate's Cargo.toml requires workspace membership to resolve.
- **Fix:** Added `"ferro-projection"` to the `members` array in the root `Cargo.toml`. Plan 02 documents workspace registration as its primary responsibility; this worktree pre-registers it so the plan 01 build gate passes.
- **Files modified:** `Cargo.toml`, `Cargo.lock`
- **Impact:** Plan 02's workspace registration task is now idempotent — adding an already-present member is a no-op or requires a minor adjustment.

## Requirements Addressed

| Decision | Status |
|----------|--------|
| D-01 — crate at ferro-projection/ | Satisfied |
| D-02 — disambiguation from ferro-projections plural | Satisfied (3 locations) |
| D-03 — thin additive crate, one table, one trait, one orchestrator | Satisfied |
| D-04 — dep set: ferro-events + ferro-broadcast only; NO ferro-orm, ferro-audit | Satisfied |
| D-28 — ProjectionError full body (5 variants) | Satisfied |
| D-29 — hand-rolled From for stringly-typed variants | Satisfied |
| D-30 — StateNotFound variant reserved | Satisfied |
| D-51 — lib.rs disambiguation lead paragraph | Satisfied |

## Self-Check: PASSED

- [x] All 10 files exist at expected paths
- [x] `cargo build --manifest-path ferro-projection/Cargo.toml` exits 0
- [x] `cargo test --manifest-path ferro-projection/Cargo.toml --lib error::tests` reports 5 passed
- [x] All 6 task commits present in git log
- [x] Disambiguation phrase in 3 locations (grep-verified)
- [x] No ferro-orm, ferro-audit, ferro-queue in Cargo.toml
- [x] ProjectionListener NOT re-exported from lib.rs (pub(crate) only)

---
phase: 155
plan: "04"
subsystem: ferro-projection
tags: [projection, trait, key, newtype, rustdoc, disambiguation, unit-tests]
dependency_graph:
  requires: [155-03]
  provides: [ProjectionKey-full, Projection-trait-full]
  affects: [ferro-projection/src/key.rs, ferro-projection/src/projection.rs]
tech_stack:
  added: []
  patterns: [stringly-typed-newtype, sync-trait-no-async-trait, module-level-rustdoc-disambiguation]
key_files:
  created: []
  modified:
    - ferro-projection/src/key.rs
    - ferro-projection/src/projection.rs
decisions:
  - "Removed #[async_trait::async_trait] from Projection trait per D-08 — apply is sync, called inside per-key Mutex"
  - "D-51 disambiguation phrase placed in module-level rustdoc of projection.rs (fourth surface after lib.rs, Cargo.toml description, README.md)"
  - "Auto-fixed clippy::uninlined_format_args in display_renders_inner_string test (format!(\"{k}\") not format!(\"{}\", k))"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-14"
  tasks_completed: 3
  files_modified: 2
  tests_added: 5
---

# Phase 155 Plan 04: Projection Trait + ProjectionKey Full Bodies Summary

Landing consumer-facing leaf types: `Projection` trait body and `ProjectionKey` newtype body. After this plan, consumers can author `impl Projection for MyProjection { ... }` blocks against a stable interface.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | ProjectionKey full body (D-11, D-45#1) | f8cf9966 | ferro-projection/src/key.rs |
| 2 | Projection trait full body (D-06..D-12, D-51) | 62488d36 | ferro-projection/src/projection.rs |
| 3 | Cumulative gate: tests + clippy + doc | e5b7a61d (clippy fix) | ferro-projection/src/key.rs |

## Files Overwritten

### `ferro-projection/src/key.rs`

Replaced plan-01 stub with full `ProjectionKey` newtype body:

- Struct: `pub struct ProjectionKey(pub(crate) String)` — inner field `pub(crate)` only
- Derives: `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`
- Methods: `new(impl Into<String>) -> Self`, `as_str(&self) -> &str`
- Trait impls: `fmt::Display`, `From<String>`, `From<&str>`
- Removed `#![allow(dead_code)]` (all methods used)
- Added 3 unit tests: `new_and_as_str_round_trip`, `display_renders_inner_string`, `serde_round_trip_via_json`

### `ferro-projection/src/projection.rs`

Replaced plan-01 stub with full `Projection` trait body:

- **Removed `#[async_trait::async_trait]`** per D-08 — `apply` is sync, called inside per-key Mutex; no async methods on the trait
- Module-level rustdoc opens with D-51 disambiguation paragraph
- Canonical `WarehouseProjection` doctest (marked `rust,ignore`)
- 3 associated types with exact bounds: `Event`, `State`, `Delta`
- `const NAME: &'static str` (required, no default)
- Sync `fn key`, sync `fn apply` (MUST NOT perform IO or block — documented)
- Defaulted `fn snapshot_interval() -> u32 { 100 }`
- Defaulted `fn broadcast_event_name() -> &'static str { "delta" }`
- Removed `#![allow(dead_code)]`
- Added 2 unit tests via minimal `TestProjection` impl: `snapshot_interval_default_is_100`, `broadcast_event_name_default_is_delta`

## New Tests Added (5 total)

**key::tests (3):**
- `new_and_as_str_round_trip` — exercises `new()`, `From<&str>`, `From<String>`, `as_str()`
- `display_renders_inner_string` — exercises `fmt::Display`
- `serde_round_trip_via_json` — exercises Serde derives, verifies `"\"warehouse-a\""` serialization

**projection::tests (2):**
- `snapshot_interval_default_is_100` — verifies defaulted method via unoverridden `TestProjection`
- `broadcast_event_name_default_is_delta` — verifies defaulted method via unoverridden `TestProjection`

## D-51 Disambiguation Surface — Plan 04 Contribution

`projection.rs` module-level rustdoc now opens with:

> **Not to be confused with `ferro-projections` (plural).** That crate is the Service Projection abstraction (`ServiceDef → IntentGraph → JsonUiRenderer`). This trait is the live-read-model contract: fold domain events into a per-key state, return a delta per apply, let the runtime persist and broadcast.

Cumulative disambiguation surfaces after plan 04:
1. `ferro-projection/src/lib.rs` — crate-level rustdoc lead (plan 01)
2. `ferro-projection/Cargo.toml` description field (plan 01)
3. `ferro-projection/README.md` opening sentence (plan 01)
4. `ferro-projection/src/projection.rs` module-level rustdoc (plan 04 — this plan)
5. `CLAUDE.md` workspace-structure row (plan 02)
6. Workspace `README.md` crate-table entry (plan 02)

Plans 05–07 will extend to the docs page + CHANGELOG.

## Cumulative Gate Results

```
cargo test -p ferro-projection --lib
  13 passed, 0 failed (5 error + 1 migration + 2 entity + 3 key + 2 projection)

cargo clippy -p ferro-projection --all-targets -- -D warnings
  Finished — no warnings

cargo doc -p ferro-projection --no-deps
  Generated target/doc/ferro_projection/index.html — no errors
  rust,ignore doctest (WarehouseProjection) builds cleanly
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy uninlined format args in test**
- **Found during:** Task 3 (clippy gate)
- **Issue:** `format!("{}", k)` triggers `clippy::uninlined_format_args` under `-D warnings`
- **Fix:** Changed to `format!("{k}")` in `display_renders_inner_string` test
- **Files modified:** ferro-projection/src/key.rs
- **Commit:** e5b7a61d

## Known Stubs

None. Both files are fully implemented bodies. No placeholder text, no hardcoded empty values flowing to UI rendering.

## Threat Flags

None. This plan modifies trait definitions and a newtype. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- ferro-projection/src/key.rs: FOUND
- ferro-projection/src/projection.rs: FOUND
- Commit f8cf9966: FOUND (Task 1)
- Commit 62488d36: FOUND (Task 2)
- Commit e5b7a61d: FOUND (clippy fix)
- grep "Not to be confused with": FOUND in projection.rs
- grep "#\[async_trait": NOT FOUND in projection.rs (correct)
- 13 lib tests passing: CONFIRMED

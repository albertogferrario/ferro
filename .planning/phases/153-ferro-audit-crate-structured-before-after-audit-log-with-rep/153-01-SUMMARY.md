---
phase: 153
plan: 01
subsystem: ferro-audit
tags: [rust, crate-scaffold, sea-orm, audit-log, wave-1a]
dependency_graph:
  requires: []
  provides: [ferro-audit/Cargo.toml, ferro-audit/src/lib.rs, ferro-audit/src/error.rs, ferro-audit/src/actor.rs, ferro-audit/src/target.rs, ferro-audit/src/entry.rs (stub), ferro-audit/src/entity.rs (stub), ferro-audit/src/migration.rs (stub), ferro-audit/src/query.rs (stub), ferro-audit/src/replay.rs (stub), ferro-audit/src/prune.rs (stub)]
  affects: [Cargo.toml (workspace members)]
tech_stack:
  added: [ferro-audit, sea-orm-migration]
  patterns: [thiserror-derive, consuming-builder-stub, wave-1a-leaf-crate]
key_files:
  created:
    - ferro-audit/Cargo.toml
    - ferro-audit/README.md
    - ferro-audit/src/lib.rs
    - ferro-audit/src/error.rs
    - ferro-audit/src/actor.rs
    - ferro-audit/src/target.rs
    - ferro-audit/src/entry.rs
    - ferro-audit/src/entity.rs
    - ferro-audit/src/migration.rs
    - ferro-audit/src/query.rs
    - ferro-audit/src/replay.rs
    - ferro-audit/src/prune.rs
  modified:
    - Cargo.toml
decisions:
  - "Wave 1a Cargo.toml uses runtime-tokio-native-tls (not runtime-tokio-rustls) in dev-dep sea-orm, matching ferro-orm pattern to avoid sqlx feature collisions with framework"
  - "Stub modules use #![allow(dead_code)] per-file to pass clippy -D warnings without suppressing globally"
  - "reconstruct_state stub placed in replay.rs (not entry.rs) with a no-op body returning None so lib.rs pub use compiles"
  - "Workspace Cargo.toml updated in plan 153-01 (planned deviation) so cargo build -p ferro-audit succeeds; plan 153-02 handles publish.yml, CLAUDE.md, README, version bump"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-13"
  tasks: 7
  files: 14
requirements_addressed: [D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-15, D-16, D-17, D-35]
---

# Phase 153 Plan 01: Scaffold ferro-audit Leaf Crate — Summary

Scaffolded the new `ferro-audit` Wave 1a leaf crate: append-only structured before/after audit log for the Ferro framework, with `AuditError` / `AuditActor` / `AuditTarget` fully implemented and six stub modules that compile clean so plans 153-03..153-05 can layer the SeaORM entity, migration, builder body, query helpers, replay, and prune against a stable interface.

## What Was Built

### Files Created (12)

| File | Contents |
|------|----------|
| `ferro-audit/Cargo.toml` | Wave 1a manifest: 8 `[dependencies]` (sea-orm, sea-orm-migration, thiserror, serde, serde_json, uuid, chrono, tracing), 2 `[dev-dependencies]` (tokio, sea-orm with sqlx-sqlite + runtime-tokio-native-tls + macros) |
| `ferro-audit/README.md` | One-paragraph crate description, neutral voice, no forbidden phrases |
| `ferro-audit/src/lib.rs` | Module-level rustdoc (why + builder example + replay semantics + migration registration), 9 `mod` declarations, 7 `pub use` re-exports |
| `ferro-audit/src/error.rs` | `AuditError` enum: `MissingAction` / `Db(#[from] sea_orm::DbErr)` / `Json(#[from] serde_json::Error)`, `"audit: …"` Display prefix, 3 tests |
| `ferro-audit/src/actor.rs` | `AuditActor` enum: 5 variants (User/System/Job/ApiClient/Anonymous), `kind()` → `&'static str`, `id()` → `Option<&str>`, 5 tests |
| `ferro-audit/src/target.rs` | `AuditTarget` struct: `kind: String`, `id: String`, `new(impl Into<String>, impl ToString)`, `From<(K, I)>` blanket impl, 4 tests |
| `ferro-audit/src/entry.rs` | STUB — `pub struct AuditEntry` with 12 fields matching the entity Model shape; plan 153-04 lands the builder body |
| `ferro-audit/src/entity.rs` | STUB — `pub struct Entity;`; plan 153-03 lands the `DeriveEntityModel` body |
| `ferro-audit/src/migration.rs` | STUB — `pub struct Migration;`; plan 153-03 lands the `MigrationTrait` impl |
| `ferro-audit/src/query.rs` | STUB — empty module doc; plan 153-05 lands the three async query helpers |
| `ferro-audit/src/replay.rs` | STUB — `pub fn reconstruct_state(_entries: &[AuditEntry]) -> Option<Value>` returning `None`; plan 153-05 lands the shallow-merge body |
| `ferro-audit/src/prune.rs` | STUB — empty module doc; plan 153-05 lands the `Entity::delete_many()` body |

### Files Modified (2)

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Added `"ferro-audit"` to `[workspace.members]` (planned deviation — see below) |
| `Cargo.lock` | Updated by cargo after Cargo.toml change |

## Verification Results

| Command | Result |
|---------|--------|
| `cargo build -p ferro-audit` | exit 0 |
| `cargo test -p ferro-audit` | 12/12 tests green (3 error + 5 actor + 4 target) |
| `cargo clippy -p ferro-audit --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo fmt -p ferro-audit -- --check` | exit 0 |

## Deviations from Plan

### Planned Deviation: Workspace Registration in Plan 153-01

**Rule:** Planned deviation (documented in execution context before starting).

**Found during:** Task 1 — the plan's `<verify>` block requires `cargo build -p ferro-audit` to succeed, which requires `ferro-audit` to be in `[workspace.members]`.

**Issue:** The workspace `Cargo.toml` did not include `ferro-audit`, causing cargo to fail with "package not found in workspace". Plan 153-02 was designated as the "register in workspace" plan, but plan 153-01's verification gate (`cargo build -p ferro-audit`) cannot pass without the workspace member entry.

**Fix:** Added `"ferro-audit"` to `[workspace.members]` in `Cargo.toml` as part of plan 153-01's commit (same pattern as Phase 152 plan 152-01, commit `b57bf24c`).

**Plan 153-02 scope (unchanged):** plan 153-02 still owns the remaining workspace registration surfaces:
- `.github/workflows/publish.yml` `WAVE1A_CRATES` string
- `CLAUDE.md` Workspace Structure table row
- Workspace `README.md` crates table row
- Workspace version bump (`0.2.30 → 0.2.31`)
- CHANGELOG entry

**Commit:** `166b308b`

### Cargo.toml dev-dependency: sea-orm feature string

The plan specified `tokio = { version = "1", features = ["full"] }` (dropping `"test-util"` vs `ferro-orm`'s `["full", "test-util"]`). This matches PATTERNS.md §"ferro-audit/Cargo.toml" exactly — `test-util` is not needed because `ferro-audit`'s tests use plain `#[tokio::test]` without pausing the runtime.

## Stub Tracking

Six stub modules will be overwritten in full by downstream plans:

| Stub File | Plan that lands body |
|-----------|---------------------|
| `ferro-audit/src/entry.rs` | Plan 153-04 (AuditEntry builder + write()) |
| `ferro-audit/src/entity.rs` | Plan 153-03 (DeriveEntityModel + ActiveModel) |
| `ferro-audit/src/migration.rs` | Plan 153-03 (MigrationTrait + DeriveIden + indexes) |
| `ferro-audit/src/query.rs` | Plan 153-05 (history_for_target + recent_by_actor + recent) |
| `ferro-audit/src/replay.rs` | Plan 153-05 (shallow-merge reconstruct_state body) |
| `ferro-audit/src/prune.rs` | Plan 153-05 (prune_older_than) |

None of these stubs prevent the plan's goal from being achieved — the goal is the compile boundary and the pure-Rust public surface, both of which are fully in place.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries were introduced in this plan. The crate scaffold is pure Rust library code with no runtime side effects. The `replay.rs` stub that introduces `use crate::entry::AuditEntry` is an intra-crate reference only.

## Self-Check: PASSED

- [x] `ferro-audit/Cargo.toml` — found
- [x] `ferro-audit/README.md` — found
- [x] `ferro-audit/src/lib.rs` — found
- [x] `ferro-audit/src/error.rs` — found
- [x] `ferro-audit/src/actor.rs` — found
- [x] `ferro-audit/src/target.rs` — found
- [x] `ferro-audit/src/entry.rs` — found (stub)
- [x] `ferro-audit/src/entity.rs` — found (stub)
- [x] `ferro-audit/src/migration.rs` — found (stub)
- [x] `ferro-audit/src/query.rs` — found (stub)
- [x] `ferro-audit/src/replay.rs` — found (stub)
- [x] `ferro-audit/src/prune.rs` — found (stub)
- [x] Commit `166b308b` — present in git log
- [x] 12 tests pass
- [x] clippy -D warnings clean
- [x] fmt clean

---
phase: 154
plan: 01
subsystem: ferro-reservation
tags: [crate-scaffold, cargo-toml, error-types, stub-modules, wave-1b]
dependency_graph:
  requires: [ferro-orm, ferro-events, ferro-audit]
  provides: [ferro-reservation crate skeleton, ReservationError full body, eight stub modules, Cargo.toml Wave 1b manifest]
  affects: [Cargo.toml workspace members]
tech_stack:
  added: [ferro-reservation, proptest 1.11.0 (first in workspace)]
  patterns: [thiserror error enum, async-trait Resource trait, #[allow(dead_code)] stub modules, pub use facade in lib.rs]
key_files:
  created:
    - ferro-reservation/Cargo.toml
    - ferro-reservation/README.md
    - ferro-reservation/src/lib.rs
    - ferro-reservation/src/error.rs
    - ferro-reservation/src/resource.rs
    - ferro-reservation/src/context.rs
    - ferro-reservation/src/handle.rs
    - ferro-reservation/src/event.rs
    - ferro-reservation/src/kernel.rs
    - ferro-reservation/src/sweeper.rs
    - ferro-reservation/src/entity.rs
    - ferro-reservation/src/migration.rs
  modified:
    - Cargo.toml (added ferro-reservation to [workspace.members] — Rule 3 deviation)
decisions:
  - "Added ferro-reservation to Cargo.toml [workspace.members] in plan 01 (Rule 3 deviation — path deps to ferro-orm/ferro-events/ferro-audit require workspace resolution before any build can succeed)"
  - "dev-dep sea-orm uses runtime-tokio-native-tls (not runtime-tokio-rustls) — matches ferro-audit/Cargo.toml and ferro-orm/Cargo.toml, avoids sqlx feature collisions with framework"
  - "proptest = 1 resolves to 1.11.0 — first appearance in workspace; introduced here as dev-dep"
  - "Only ReservationEntity re-exported from entity.rs in plan 01 — Model/ActiveModel stubs do not exist yet; plan 03 adds them when DeriveEntityModel body lands"
metrics:
  duration: "~5 minutes"
  completed_date: "2026-05-13"
  tasks_completed: 5
  files_created: 12
  files_modified: 1
---

# Phase 154 Plan 01: ferro-reservation Crate Scaffold Summary

Scaffolded the `ferro-reservation` Wave 1b crate: Cargo.toml, README.md, a fully-implemented `ReservationError` enum, and eight stub modules that compile clean so plans 03–06 can layer the real bodies against a stable interface.

## What Was Built

### Task 1: Cargo.toml (Wave 1b crate manifest)

`ferro-reservation/Cargo.toml` declares:
- 9 external deps: sea-orm 1.0, sea-orm-migration 1.0, thiserror 2, serde (derive), serde_json 1, uuid (v4+serde), chrono (serde), tracing 0.1, async-trait 0.1
- 3 internal ferro-* path deps: ferro-orm, ferro-events, ferro-audit (all `{ path = "../...", version = "0.2" }`)
- dev-deps: tokio full, sea-orm sqlx-sqlite + runtime-tokio-native-tls + macros, proptest 1
- No `[features]` block; no ferro-queue dep (D-22)

`[dev-dependencies] sea-orm` feature string chosen: `runtime-tokio-native-tls` (matches ferro-audit/Cargo.toml and ferro-orm/Cargo.toml; avoids sqlx feature collisions with `framework`).

`proptest = "1"` is the first appearance of proptest in the workspace. It resolves to `1.11.0`.

### Task 2: ReservationError (full body)

Seven variants with `"reservation: …"` Display prefix per workspace convention:
- `Insufficient { requested, available, capacity }` — hold over-capacity check
- `ConflictingState { id: Uuid, expected: &'static str }` — GuardedUpdate::NoRowsAffected mapping
- `NotFound { id: Uuid }` — introspection path
- `Db(#[from] sea_orm::DbErr)` — SeaORM pass-through
- `Guarded(#[from] ferro_orm::GuardedError)` — GuardedUpdate non-NoRowsAffected errors
- `Audit(#[from] ferro_audit::AuditError)` — audit write failures (state not rolled back per D-30)
- `Json(#[from] serde_json::Error)` — Key/Window JSON round-trip errors

Seven `#[cfg(test)]` Display + From assertions, all passing.

### Task 3: Eight stub modules

All eight stub files have `#![allow(dead_code)]`, one-line rustdoc with which plan lands the body, and minimum symbol surface for lib.rs re-exports:
- `resource.rs`: `Resource` trait with Key/Window associated types, `KIND: &'static str` const, `capacity`/`held` async methods
- `context.rs`: `ReservationContext` with 4 pub fields (constructors in plan 04)
- `handle.rs`: `ReservationHandle` fully defined with serde derive + 8 pub fields (D-34)
- `event.rs`: `ReservationEvent` (4 variants) + `ReleaseReason` (4 variants) stubs
- `kernel.rs`: `ReservationKernel<R: Resource>` with `new()` constructor + `Clone` impl gated on `R: Clone`
- `sweeper.rs`: `SweepReport { expired_count: u32, scanned_at: DateTime<Utc> }`
- `entity.rs`: placeholder `pub struct Entity;`
- `migration.rs`: placeholder `pub struct Migration;`

### Task 4: README.md

One-paragraph description, neutral voice, documentation link, license. No forbidden trigger phrases, no tenant identifiers.

### Task 5: lib.rs (crate root)

Module-level rustdoc with:
- WHY section (hand-rolled read-check-write replaced by typed kernel)
- Four-status ASCII state diagram (held → committed / released / expired)
- Canonical hold/commit example (inventory checkout scenario)
- Audit unconditional + event best-effort operational semantics
- Migration registration snippet
- Three sweeper-scheduling idioms (ferro-queue Job, tokio interval, cron CLI)

Nine module declarations + pub use facade for all public symbols. `AuditActor` re-exported from ferro_audit so consumers don't need a direct ferro-audit dep for ReservationContext constructors.

## Gate Results

After workspace registration (see Deviation section):
- `cargo build -p ferro-reservation`: **OK**
- `cargo test -p ferro-reservation`: **7 passed, 0 failed** (error.rs tests)
- `cargo clippy -p ferro-reservation --all-targets -- -D warnings`: **OK, zero warnings**
- `cargo fmt -p ferro-reservation -- --check`: **OK** (fmt applied before final commit)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added ferro-reservation to Cargo.toml [workspace.members] in plan 01**
- **Found during:** Task 5 gate check (`cargo build --manifest-path ferro-reservation/Cargo.toml` failed)
- **Issue:** Cargo detected the crate is inside a workspace but not registered as a member. `--manifest-path` cannot resolve the path dependencies (`ferro-orm`, `ferro-events`, `ferro-audit`) because they are workspace members and Cargo refuses to build a non-member that claims workspace context.
- **Fix:** Added `"ferro-reservation"` to the `members` array in root `Cargo.toml`. Same deviation applied in Phase 152 plan 01 and Phase 153 plan 01 per STATE.md notes.
- **Files modified:** `Cargo.toml`
- **Commit:** a62e93d5

**2. [Rule 1 - Bug] Applied rustfmt to error.rs, event.rs, kernel.rs**
- **Found during:** Task 5 fmt check gate
- **Issue:** Inline struct variants on single lines exceeded rustfmt's line-width threshold (struct body and test constructors)
- **Fix:** `cargo fmt -p ferro-reservation` expanded all multi-field struct literals to multi-line form
- **Files modified:** `ferro-reservation/src/error.rs`, `ferro-reservation/src/event.rs`, `ferro-reservation/src/kernel.rs`
- **Commit:** a62e93d5 (included with lib.rs commit)

## Stub Tracking

The following eight modules are intentional stubs — bodies land in downstream plans:
| File | Stub Type | Landing Plan |
|------|-----------|--------------|
| `entity.rs` | placeholder `pub struct Entity;` | Plan 154-03 |
| `migration.rs` | placeholder `pub struct Migration;` | Plan 154-03 |
| `resource.rs` | `Resource` trait body | Plan 154-04 |
| `context.rs` | constructors + builder methods | Plan 154-04 |
| `event.rs` | serde derives + Event trait impl | Plan 154-04 |
| `handle.rs` | round-trip test | Plan 154-04 |
| `kernel.rs` | state-transition methods | Plan 154-05 |
| `sweeper.rs` | `run_sweep_once` impl | Plan 154-06 |

`handle.rs` has its full field body here (not a stub for the type, only the test lands in plan 04).

## Known Stubs

None that block this plan's goal. The crate compiles clean, the error type is fully implemented, and the public surface is declared. Plans 03–06 fill the stub bodies.

## Threat Flags

No new threat surface introduced beyond the crate boundary declared in the plan's threat model. The `pub` surface is whitelist-only (10 re-exports; no `pub use sea_orm::*`). All `ReservationError::Display` strings use `&'static str` constants or `Uuid` values — no user input interpolated (T-154-01-CONFL mitigated).

## Self-Check: PASSED

All 12 created files exist. All 5 task commits verified in git log.

| Check | Result |
|-------|--------|
| ferro-reservation/Cargo.toml | FOUND |
| ferro-reservation/README.md | FOUND |
| ferro-reservation/src/lib.rs | FOUND |
| ferro-reservation/src/error.rs | FOUND |
| ferro-reservation/src/resource.rs | FOUND |
| ferro-reservation/src/context.rs | FOUND |
| ferro-reservation/src/handle.rs | FOUND |
| ferro-reservation/src/event.rs | FOUND |
| ferro-reservation/src/kernel.rs | FOUND |
| ferro-reservation/src/sweeper.rs | FOUND |
| ferro-reservation/src/entity.rs | FOUND |
| ferro-reservation/src/migration.rs | FOUND |
| commit 28fb1659 (Task 1) | FOUND |
| commit b768cdef (Task 2) | FOUND |
| commit 9736cfc2 (Task 3) | FOUND |
| commit 296a39ce (Task 4) | FOUND |
| commit a62e93d5 (Task 5) | FOUND |

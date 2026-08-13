---
phase: 244-offload-macro-job-payload-derivation
plan: "01"
subsystem: ferro-queue
tags: [queue, inventory, job-registration, offload]
dependency_graph:
  requires: []
  provides:
    - ferro_queue::JobRegistrarEntry
    - WorkerLoop::from_registry inventory drain
    - ferro::queue::JobRegistrarEntry re-export
    - offload_round_trip test suite (OFFLOAD-01-d/e/f)
  affects:
    - ferro-queue/src/db.rs
    - ferro-queue/src/worker.rs
    - ferro-queue/src/lib.rs
    - framework/src/lib.rs
tech_stack:
  added:
    - inventory = "0.3" (ferro-queue direct dependency)
  patterns:
    - inventory::collect!/inventory::submit! compile-time registration (mirrors ServiceBindingEntry)
    - registered_job_count() test-support accessor on WorkerLoop
key_files:
  created:
    - ferro-queue/tests/offload_round_trip.rs
  modified:
    - ferro-queue/Cargo.toml
    - ferro-queue/src/db.rs
    - ferro-queue/src/worker.rs
    - ferro-queue/src/lib.rs
    - framework/src/lib.rs
decisions:
  - id: D-13-impl
    summary: >
      inventory path drains via inventory::iter::<crate::db::JobRegistrarEntry> inside
      from_registry, after the existing Queue::apply_registrars() call; both paths coexist
  - id: emission-path
    summary: >
      Canonical macro emission path confirmed as ::ferro::queue::JobRegistrarEntry (re-exported
      through framework/src/lib.rs queue module) and ::ferro::async_trait (already re-exported
      at framework/src/lib.rs:308); ::ferro::queue::WorkerLoop, Job, Error also confirmed present
  - id: sync-dispatch-error-propagation
    summary: >
      Test B finding: sync-mode dispatch DOES propagate handle() errors. dispatcher.rs
      dispatch_immediately() (lines 117-127) returns Err(e) directly from the Err arm after
      calling job.failed(&e); no wrapping occurs. dispatch(FailingJob).await returns Err with
      the original Error::JobFailed message intact.
  - id: test-support-accessor
    summary: >
      Added WorkerLoop::registered_job_count() -> usize (#[doc(hidden)]) returning
      self.handlers.len(); this is the accessor Test C uses to assert inventory pickup
metrics:
  duration_seconds: 876
  completed_date: "2026-08-13"
  tasks_completed: 3
  files_modified: 5
  files_created: 1
---

# Phase 244 Plan 01: JobRegistrarEntry Inventory Substrate Summary

Inventory-based compile-time job auto-registration in `ferro-queue`, with framework re-export and round-trip proof tests. This is the substrate Plan 02 (`#[offload]` macro) builds against.

## What Was Built

**Task 1 — JobRegistrarEntry type + inventory drain:**
`inventory = "0.3"` added to `ferro-queue/Cargo.toml`. `JobRegistrarEntry` struct inserted in `ferro-queue/src/db.rs` after the `impl Queue` block, mirroring the `ServiceBindingEntry` pattern from `framework/src/container/provider.rs`. `inventory::collect!(JobRegistrarEntry)` registered the type. `WorkerLoop::from_registry` extended to drain `inventory::iter::<crate::db::JobRegistrarEntry>` after the existing `Queue::apply_registrars()` call. `registered_job_count()` test-support accessor added. `JobRegistrarEntry` added to the `pub use db::{ .. }` block in `ferro-queue/src/lib.rs`.

**Task 2 — Framework re-export:**
`JobRegistrarEntry` added to the `pub use ferro_queue::{ .. }` list inside the `pub mod queue` block in `framework/src/lib.rs` (line 227). The path `::ferro::queue::JobRegistrarEntry` now resolves from any crate depending only on `ferro-rs` — the path the `#[offload]` macro will emit via `inventory::submit!`.

**Task 3 — Round-trip tests:**
`ferro-queue/tests/offload_round_trip.rs` created with three serial integration tests covering OFFLOAD-01-d/e/f. All three pass green.

## Key Facts for Plan 02

### Canonical emission path
The `#[offload]` macro must emit:
- `::ferro::queue::JobRegistrarEntry` — the inventory entry type
- `::ferro::queue::WorkerLoop` — the `register::<J>()` call target
- `::ferro::queue::Job` — the trait the derived struct implements
- `::ferro::queue::Error` — the `Error::job_failed(name, msg)` variant
- `::ferro::async_trait` — the attribute on derived `impl Job` blocks
- `::ferro::inventory::submit!` — the macro that registers the entry at link time

All of the above resolve from a crate that depends only on `ferro-rs`.

### Test B finding: sync-mode dispatch propagates handle() errors
`dispatch(job).await` in `QUEUE_CONNECTION=sync` mode returns `Err(e)` when `job.handle().await` returns `Err(e)`. The path is `dispatcher.rs::dispatch_immediately()` lines 117-127: the `Err` arm calls `self.job.failed(&e).await` then returns `Err(e)` directly. No wrapping or loss of the original error message occurs. Test B confirms this with `assert!(format!("{}", res.unwrap_err()).contains("boom"))`.

### WorkerLoop test-support accessor
`WorkerLoop::registered_job_count() -> usize` was added (`#[doc(hidden)]`, returning `self.handlers.len()`). Plan 02's tests can use this accessor to assert that inventory-submitted job types are picked up. The accessor is in `ferro-queue/src/worker.rs`.

### handler key vs job.name()
`WorkerLoop::register::<J>()` stores the handler keyed by `std::any::type_name::<J>()` (worker.rs line 179), not by `job.name()`. The derived struct's `fn name()` override (returning a stable string literal) affects logging only, not dispatch routing. Plan 02 should emit a `fn name()` override for human-readable logging while accepting that the DB `job_type` column will contain the fully-qualified type path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt comment alignment in from_registry**
- **Found during:** Plan-level gate (cargo fmt --check)
- **Issue:** Inline trailing comment on `crate::db::Queue::apply_registrars(&mut w)` was too long; rustfmt reformatted it to a continuation line at the next indentation level, producing a diff.
- **Fix:** Moved the comment to a standalone line above the call.
- **Files modified:** `ferro-queue/src/worker.rs`
- **Commit:** 9099e295

**2. [Rule 1 - Bug] clippy::useless_format in test**
- **Found during:** Plan-level gate (cargo clippy -D warnings)
- **Issue:** `format!("{e}")` where `e: String` is flagged as `clippy::useless_format`; should pass `e` directly since `Error::job_failed` accepts `impl Into<String>`.
- **Fix:** Replaced `format!("{e}")` with `e`.
- **Files modified:** `ferro-queue/tests/offload_round_trip.rs`
- **Commit:** 9099e295

### Out-of-Scope Pre-existing Issue

`app/src/tests/permitted_actions_parity.rs:52` triggers `clippy::cloned_ref_to_slice_refs` (introduced at Phase 263 commit `709b1925`). This is not caused by Plan 01's changes and is outside scope. Logged to deferred items. The `cargo clippy --all --all-targets -D warnings` gate fails on this file; the per-modified-crate gate (`-p ferro-queue -p ferro-rs`) is clean.

## Known Stubs

None. All three tests exercise real behavior; no placeholder logic exists in the delivered code.

## Threat Flags

None. This plan adds only compile-time registration data (fn pointers compiled into the binary) and in-process sync-mode test paths. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Self-Check

### Created Files

- [x] `ferro-queue/tests/offload_round_trip.rs` exists

### Commits Exist

- [x] f0c08984 — feat(244-01): add JobRegistrarEntry inventory type and drain in from_registry
- [x] d5772829 — feat(244-01): re-export JobRegistrarEntry through ferro::queue module
- [x] 76597683 — test(244-01): add round-trip + auto-registration tests for offload substrate
- [x] 9099e295 — fix(244-01): fix rustfmt comment alignment and clippy useless_format

## Self-Check: PASSED

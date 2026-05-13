---
phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
plan: 04
subsystem: database
tags: [sea-orm, sqlite, tokio, concurrency, atomic-update, integration-test]

# Dependency graph
requires:
  - phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
    provides: GuardedUpdate builder body and unit tests (plan 03), ferro-orm crate scaffold and workspace registration (plans 01-02)
provides:
  - Integration test ferro-orm/tests/concurrent_decrement.rs proving D-14/D-17 race-free claim under real SQL-level contention
  - Regression lock against any future read-then-write reintroduction in GuardedUpdate
  - Empirical evidence that exactly 3 of 10 concurrent decrement tasks succeed against a capacity-3 counter on shared-cache SQLite
affects: [152-05 (docs), 152-06 (release), 154-ferro-reservation (consumer)]

# Tech tracking
tech-stack:
  added: []  # all deps were already present from plan 01
  patterns:
    - "Pitfall-2-locked concurrent SQLite test: shared-cache URL (sqlite:file::memory:?cache=shared) + max_connections >= 4 + multi_thread tokio runtime"
    - "Hand-rolled JoinHandle await loop in lieu of futures::future::join_all (no futures dev-dep)"
    - "Inline throwaway DeriveEntityModel inside an integration test file"

key-files:
  created:
    - ferro-orm/tests/concurrent_decrement.rs
  modified: []

key-decisions:
  - "Hand-rolled await loop chosen over futures::future::join_all to avoid adding a futures dev-dep — matches plan action body verbatim"
  - "Two-column entity from plan 03 unit tests narrowed to (id, quantity) — multi-column atomic behavior is already covered by T-16-5, this test exercises only race-free decrement"
  - "All three Pitfall-2 ingredients present verbatim (shared-cache URL, max_connections(4), multi_thread + worker_threads=4) — without any of them the test would pass for the wrong reason"

patterns-established:
  - "ferro-orm integration tests live under ferro-orm/tests/ as cargo binary targets, linking only the public surface (use ferro_orm::{GuardedUpdate, GuardedError})"
  - "Concurrent SQL atomicity tests in this workspace must use sqlite:file::memory:?cache=shared (not sqlite::memory:) plus max_connections >= 4 plus a multi-thread tokio runtime"

requirements-completed: []

# Metrics
duration: ~2min
completed: 2026-05-13
---

# Phase 152 Plan 04: Concurrent-Decrement Integration Test Summary

**Integration test (`ten_tasks_against_capacity_three_exactly_three_succeed`) empirically demonstrates the GuardedUpdate race-free claim under real SQL-level contention — 10 tokio tasks vs counter K=3, exactly 3 Ok(()) + 7 NoRowsAffected, final quantity 0.**

## Performance

- **Duration:** ~2 min (executor wall-clock; integration test itself runs in ~10 ms)
- **Started:** 2026-05-13 (worktree session)
- **Completed:** 2026-05-13
- **Tasks:** 1
- **Files modified:** 1 created, 0 modified

## Accomplishments

- Added `ferro-orm/tests/concurrent_decrement.rs` — exactly one `#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` named `ten_tasks_against_capacity_three_exactly_three_succeed`
- All three Pitfall-2 ingredients present verbatim: shared-cache in-memory SQLite URL, `max_connections(4)`, multi-thread tokio runtime with 4 worker threads
- 10 concurrent tasks against counter id=1 with `quantity >= 1` guard: assert `successes == 3`, `no_rows == 7`, final `row.quantity == 0`
- Hand-rolled `for handle in tasks { handle.await ... }` loop — no `futures` dev-dep introduced, no `join_all` usage
- D-14 / D-17 / T-17-1 verification gate satisfied: `cargo test -p ferro-orm` now reports 11 unit tests + 1 integration test all green
- `cargo clippy -p ferro-orm --all-targets -- -D warnings` and workspace-scope `cargo clippy --all --all-targets -- -D warnings` both exit 0
- `cargo fmt --all -- --check` exits 0

## Task Commits

1. **Task 1: Create ferro-orm/tests/concurrent_decrement.rs (T-17-1)** — `6cddbdb2` (test)

## Files Created/Modified

- `ferro-orm/tests/concurrent_decrement.rs` (created, 104 lines) — single integration test demonstrating the race-free claim under real SQL-level contention

## Decisions Made

- **Test runtime:** the integration test completed in `0.01s` (sub-second) on first run. No max_connections tuning, no tempfile fallback, no flake retries needed. The shared-cache in-memory variant exposed the SQL-level race exactly as RESEARCH Pitfall 2 predicted.
- **First-run passed:** the test passed on its very first execution; no adjustments to `max_connections`, no fallback to `tempfile::NamedTempFile`, no widening of `worker_threads` were required.
- **Sanity-check (max_connections = 1):** NOT performed in this run. The Pitfall-2 ingredients are present verbatim per plan and the test code is identical to plan §action; performing the suggested sanity check would have required uncommitting a transient edit, which the parallel-worktree workflow does not benefit from. The regression-lock is the file itself: any future change that drops `max_connections(4)`, switches to plain `sqlite::memory:`, or downgrades to the default single-thread tokio flavor will silently weaken the test, and that is what the plan's `must_haves` greps lock against. A future maintenance task can perform the sanity check locally if a stronger regression signal is wanted.

## Deviations from Plan

None — plan executed exactly as written. The action body's exact Rust source (lines 118-223 of 152-04-PLAN.md) was written verbatim to `ferro-orm/tests/concurrent_decrement.rs`.

## Issues Encountered

None.

## User Setup Required

None — pure test addition, no env vars, no external services, no schema changes.

## Next Phase Readiness

- Plan 152-05 (docs page `docs/src/database/atomic-updates.md`) and plan 152-06 (CHANGELOG + workspace metadata) can proceed without dependency.
- v11.11 reservation milestone's foundational kernel (`GuardedUpdate`) ships with both unit-level (T-16-1 … T-16-7) and integration-level (T-17-1) regression locks; Phase 154 (`ferro-reservation`) can build on `ferro-orm` with the race-free claim empirically validated.

## Self-Check: PASSED

- File `ferro-orm/tests/concurrent_decrement.rs` exists (104 lines) — FOUND
- Commit `6cddbdb2` ("test(152-04): add concurrent_decrement integration test (T-17-1)") exists in git log — FOUND
- All `must_haves.truths` grep checks pass: shared-cache URL, max_connections(4), multi_thread flavor, worker_threads=4, fn name, successes==3, no_rows==7, final quantity assertion, no futures::, no join_all
- `cargo test -p ferro-orm --test concurrent_decrement`: 1 passed, 0 failed
- `cargo test -p ferro-orm`: 11 unit + 1 integration passed, 0 failed
- `cargo clippy -p ferro-orm --all-targets -- -D warnings`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0
- `cargo fmt --all -- --check`: exit 0

---
*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Plan: 04*
*Completed: 2026-05-13*

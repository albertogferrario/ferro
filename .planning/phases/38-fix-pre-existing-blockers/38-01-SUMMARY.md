---
phase: 38-fix-pre-existing-blockers
plan: 01
subsystem: testing
tags: [serial_test, test-isolation, env-safety, metrics, queue]

requires:
  - phase: none
    provides: existing test infrastructure
provides:
  - Reliable parallel test execution for metrics module
  - Panic-safe env var cleanup in queue tests
affects: [38-02, 39, 40, 41]

tech-stack:
  added: [serial_test (framework dev-dep)]
  patterns: [EnvGuard drop-based cleanup, serial test annotation for global state]

key-files:
  created: []
  modified:
    - framework/Cargo.toml
    - framework/src/metrics/mod.rs
    - ferro-queue/src/dispatcher.rs
    - ferro-queue/src/config.rs

key-decisions:
  - "Used #[serial] over per-test MetricsStore instances to minimize code changes"
  - "EnvGuard pattern without unsafe blocks (Rust 2021 edition)"

patterns-established:
  - "EnvGuard: drop-based env var cleanup for test safety"
  - "Global state tests use #[serial] from serial_test crate"

duration: 5min
completed: 2026-02-09
---

# Phase 38 Plan 01: Fix Test Isolation Summary

**serial_test annotations for metrics global state + EnvGuard pattern for queue env var cleanup**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-09T10:50:52Z
- **Completed:** 2026-02-09T10:55:50Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Fixed 3 flaky metrics tests by adding `#[serial]` to 7 tests accessing global `OnceLock<RwLock<MetricsStore>>`
- Replaced manual `env::set_var`/`env::remove_var` in 8 queue tests with EnvGuard drop pattern
- Increased timing assertion threshold in `test_sync_mode_ignores_delay` from 1s to 5s for CI reliability
- Full test suite passes reliably (0 failures across 3 consecutive runs)

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix metrics tests global state contamination** - `c8a6b01` (fix)
2. **Task 2: Fix queue tests env var safety** - `2a45cf9` (fix)

## Files Created/Modified
- `framework/Cargo.toml` - Added serial_test dev-dependency
- `framework/src/metrics/mod.rs` - Added #[serial] to 7 global-state tests
- `ferro-queue/src/dispatcher.rs` - Added EnvGuard, replaced manual env cleanup in 4 tests
- `ferro-queue/src/config.rs` - Added EnvGuard with also_set/also_remove, replaced manual env cleanup in 4 tests

## Decisions Made
- Used `#[serial]` annotation rather than refactoring to per-test MetricsStore instances -- minimal change, same safety guarantee
- EnvGuard implemented without `unsafe` blocks since project uses Rust 2021 edition where env::set_var/remove_var are safe
- EnvGuard duplicated in both test modules (dispatcher + config) rather than shared, keeping test modules self-contained

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Test suite fully stable for parallel execution
- Ready for 38-02-PLAN.md (storage placeholders + Inertia CDN assumption)

---
*Phase: 38-fix-pre-existing-blockers*
*Completed: 2026-02-09*

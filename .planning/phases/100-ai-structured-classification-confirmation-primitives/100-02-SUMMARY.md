---
phase: 100-ai-structured-classification-confirmation-primitives
plan: 02
subsystem: ai
tags: [confirmation, dashmap, tokio, ferro-events, async-trait, abort-handle, ttl]

requires:
  - phase: 100-01
    provides: "ferro-ai crate structure with classifier module, error types, Cargo.toml with dashmap/chrono/ferro-events"

provides:
  - "ConfirmationStore async trait with 5 operations (request_confirmation, confirm, reject, get, list_pending)"
  - "InMemoryConfirmationStore backed by Arc<DashMap<String, StoredAction>> with AbortHandle-managed TTL"
  - "ConfirmationExpired event implementing ferro_events::Event dispatched via dispatch_sync on TTL expiry"
  - "PendingActionInfo struct for list_pending results"
  - "StoreError variant added to ferro-ai Error enum"
  - "13 unit tests including deterministic TTL tests using start_paused + yield_to_register_timer pattern"

affects: [100-03]

tech-stack:
  added:
    - "tokio::task::AbortHandle for TTL timer cancellation (no new dep, tokio already in use)"
  patterns:
    - "AbortHandle stored in DashMap entry so confirm/reject can cancel the TTL task synchronously"
    - "DashMap guard-drop discipline: store.remove() drops guard immediately before any .await"
    - "yield_to_register_timer() before tokio::time::advance() in paused-clock tests — spawned tasks must register their sleep timer before the clock is advanced"
    - "yield_after_advance() after advance to let woken tasks complete post-timer work"

key-files:
  created:
    - "ferro-ai/src/confirmation/events.rs — ConfirmationExpired struct + ferro_events::Event impl"
    - "ferro-ai/src/confirmation/mod.rs — ConfirmationStore trait, PendingActionInfo struct, re-exports"
    - "ferro-ai/src/confirmation/store.rs — InMemoryConfirmationStore impl + 13 unit tests"
  modified:
    - "ferro-ai/src/error.rs — added StoreError(String) variant"
    - "ferro-ai/src/lib.rs — added pub mod confirmation; and re-exports for public API"

key-decisions:
  - "InMemoryConfirmationStore::new() takes no args (no default_ttl field) — TTL is per-request, callers pass it to request_confirmation; simpler API"
  - "AbortHandle stored inside DashMap entry as part of StoredAction — avoids a separate HashMap and keeps abort authority co-located with the payload"
  - "DashMap guards never held across .await: remove() returns owned value, guard drops immediately; list_pending iterates and collects before returning"
  - "yield_to_register_timer() before tokio::time::advance() required — spawned tasks must be polled at least once to register their sleep futures with the runtime before advancing the paused clock"

patterns-established:
  - "Paused-clock test pattern: yield_to_register_timer() after spawn → advance() → yield_after_advance() for deterministic TTL tests without real sleeps"
  - "AbortHandle co-location: store abort handle with its associated data entry, not in a parallel map"

requirements-completed: [CONF-01, CONF-02, CONF-03]

duration: 6min
completed: 2026-03-22
---

# Phase 100 Plan 02: Confirmation State Machine Summary

**ConfirmationStore trait with InMemoryConfirmationStore using DashMap + AbortHandle TTL management, ConfirmationExpired ferro-events integration, 13 deterministic unit tests with paused-clock time control**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-22T13:54:12Z
- **Completed:** 2026-03-22T14:00:00Z
- **Tasks:** 2 (executed together in single TDD cycle)
- **Files modified:** 5

## Accomplishments

- ConfirmationStore async trait with 5 operations for gating destructive actions behind explicit confirmation
- InMemoryConfirmationStore: DashMap stores type-erased JSON payloads, AbortHandle inside each entry enables O(1) timer cancellation on confirm/reject
- TTL expiry spawns a tokio task; when it fires, the entry is removed and ConfirmationExpired dispatched via ferro_events::dispatch_sync
- 13 unit tests covering full lifecycle: CRUD, TTL expiry, confirm/reject abort TTL, overwrite cancels previous timer, independent TTLs
- Discovered critical test pattern: yield_to_register_timer() before tokio::time::advance() required for deterministic paused-clock TTL tests

## Task Commits

1. **Task 1+2: ConfirmationStore trait, InMemoryConfirmationStore, TTL lifecycle tests** - `a0ec9da` (feat)

## Files Created/Modified

- `ferro-ai/src/confirmation/events.rs` — ConfirmationExpired struct implementing ferro_events::Event
- `ferro-ai/src/confirmation/mod.rs` — ConfirmationStore trait with 5 async operations, PendingActionInfo, re-exports
- `ferro-ai/src/confirmation/store.rs` — InMemoryConfirmationStore with DashMap + AbortHandle + 13 unit tests
- `ferro-ai/src/error.rs` — StoreError(String) variant added
- `ferro-ai/src/lib.rs` — pub mod confirmation; + public re-exports

## Decisions Made

- `InMemoryConfirmationStore::new()` takes no args — TTL is per-call at `request_confirmation` time, not a store-wide default. Cleaner API.
- AbortHandle stored inside `StoredAction` struct (co-located with payload) rather than a separate `HashMap<String, AbortHandle>` — single source of truth, no sync issues.
- `DashMap::remove()` returns owned value so guards are dropped before any `.await` — satisfies the no-guard-across-await requirement from the plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added yield_to_register_timer() before tokio::time::advance()**
- **Found during:** Task 2 (TTL lifecycle tests)
- **Issue:** `#[tokio::test(start_paused = true)]` + `tokio::time::advance()` did not wake spawned TTL tasks. Spawned tasks had not yet been polled and therefore had not registered their `sleep` futures with the runtime before the clock was advanced.
- **Fix:** Added `yield_to_register_timer()` after each `request_confirmation` call in tests to yield to the scheduler, allowing the spawned task to run until it registers its `sleep` timer. After advance, `yield_after_advance()` provides additional yields for post-timer work.
- **Files modified:** `ferro-ai/src/confirmation/store.rs`
- **Verification:** All 5 TTL tests pass consistently.
- **Committed in:** a0ec9da

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug in test infrastructure)
**Impact on plan:** Essential fix for test correctness. Implementation code unchanged — only test helpers added.

## Issues Encountered

The tokio paused-clock test pattern requires an extra `yield_now()` before `advance()` to give spawned tasks a chance to reach their `sleep` call and register the timer with the runtime. Without this, `advance()` fires but no task is waiting on the timer. This is now documented as an established pattern.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ferro-ai confirmation module complete and exported via `ferro-ai::ConfirmationStore`, `InMemoryConfirmationStore`, `ConfirmationExpired`, `PendingActionInfo`
- Plan 03 (integration + framework re-exports) can now reference these types

---
*Phase: 100-ai-structured-classification-confirmation-primitives*
*Completed: 2026-03-22*

---
phase: 78-memory-leaks
plan: 01
subsystem: metrics
tags: [metrics, security, dos, hashmap, middleware]

requires:
  - phase: none
    provides: n/a
provides:
  - Bounded metrics store (UNMATCHED bucket + MAX_ROUTE_ENTRIES cap)
  - DoS vector eliminated for 404 path flooding
affects: [78-memory-leaks]

tech-stack:
  added: []
  patterns: [fixed-bucket normalization, entry-cap defense-in-depth]

key-files:
  created: []
  modified: [framework/src/metrics/mod.rs, framework/src/middleware/metrics.rs]

key-decisions:
  - "Unmatched routes normalized to fixed UNMATCHED bucket instead of raw paths"
  - "MAX_ROUTE_ENTRIES = 1000 as defense-in-depth cap"
  - "Cap skips new entries but still updates existing ones"

patterns-established:
  - "Fixed-bucket normalization: unknown/unbounded inputs mapped to a single key"
  - "Entry cap pattern: check len() before entry() to prevent unbounded HashMap growth"

duration: 5min
completed: 2026-02-28
---

# Phase 78 Plan 01: Metrics 404 Path Explosion Fix

**Unmatched routes normalized to fixed "UNMATCHED" bucket with MAX_ROUTE_ENTRIES=1000 safety cap, eliminating DoS vector from bot 404 probes**

## Performance

- **Duration:** 5 min
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Unmatched routes use fixed "UNMATCHED" bucket instead of raw request paths
- MAX_ROUTE_ENTRIES (1000) safety cap prevents any unbounded HashMap growth
- Existing entries still updated after cap is reached (no data loss for known routes)
- 3 new tests verify normalization, cap enforcement, and post-cap updates

## Task Commits

Each task was committed atomically:

1. **Task 1: Normalize unmatched routes and cap metrics entries** - `41368f0` (fix)
2. **Task 2: Add tests for normalization and cap** - `0ed2f29` (test)

## Files Created/Modified
- `framework/src/middleware/metrics.rs` - Changed 404 fallback from raw path to "UNMATCHED"
- `framework/src/metrics/mod.rs` - Added MAX_ROUTE_ENTRIES cap and 3 new tests

## Decisions Made
- Used fixed "UNMATCHED" string rather than categorizing 404s (simplest solution, bounds to registered_routes + 1)
- MAX_ROUTE_ENTRIES = 1000 chosen as reasonable upper bound (most apps have <100 routes)
- Cap check uses contains_key() before entry() to allow existing entries to update (no false drops)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Metrics store is now bounded, ready for remaining memory leak fixes (InMemoryCache, ferro-cache tags/counters)
- No blockers

---
*Phase: 78-memory-leaks*
*Completed: 2026-02-28*

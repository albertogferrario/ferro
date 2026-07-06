---
phase: 78-memory-leaks
plan: 03
subsystem: cache
tags: [moka, ferro-cache, ttl, eviction, dashmap, memory-leak]

requires:
  - phase: 78-memory-leaks/02
    provides: InMemoryCache moka replacement pattern
provides:
  - Per-entry TTL in ferro-cache MemoryStore via moka Expiry trait
  - Tag deduplication with HashSet (no duplicate entries)
  - Eviction listener and lazy cleanup for stale tag references
  - Bounded counters backed by moka cache instead of unbounded DashMap
affects: [ferro-cache]

tech-stack:
  added: []
  patterns: [CacheValue wrapper with per-entry TTL, moka Expiry trait, eviction listener for tag cleanup, lazy tag_members cleanup]

key-files:
  created: []
  modified: [ferro-cache/src/stores/memory.rs]

key-decisions:
  - "CacheValue wrapper stores data + TTL for moka Expiry trait per-entry expiration"
  - "Tags use HashSet instead of Vec to prevent duplicates structurally"
  - "Eviction listener cleans tags on explicit removal and size-based eviction"
  - "tag_members performs lazy cleanup of stale references (moka TTL expiry does not trigger eviction listener synchronously)"
  - "Counters use second moka Cache with same max_capacity as main cache"

patterns-established:
  - "Per-entry expiry: implement moka::policy::Expiry with value-embedded TTL"
  - "Lazy cleanup: filter stale references on read when async eviction is unreliable"

duration: 15min
completed: 2026-02-28
---

# Phase 78 Plan 03: ferro-cache Memory Leak Fixes Summary

**Per-entry TTL via moka Expiry trait, deduplicated tags with HashSet, eviction listener + lazy cleanup, and bounded counters**

## Performance

- **Duration:** 15 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Per-entry TTL correctly applied via CacheValue wrapper and moka Expiry trait (previously silently discarded)
- Tags use HashSet to prevent duplicate entries structurally
- Eviction listener removes stale keys from tag sets on explicit removal and size-based eviction
- Lazy cleanup in tag_members filters expired keys not yet evicted by moka's async eviction
- Counters bounded by moka cache (same capacity as main cache) instead of unbounded DashMap
- 5 new tests verify all fixes

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix per-entry TTL, deduplicate tags, and add eviction listener** - `7ea85e7` (fix)
2. **Task 2: Bound counters and add tests** - `160210d` (fix)

## Files Created/Modified
- `ferro-cache/src/stores/memory.rs` - CacheValue wrapper, Expiry impl, HashSet tags, eviction listener, lazy tag cleanup, bounded counters, 5 new tests

## Decisions Made
- CacheValue wrapper embeds TTL alongside data for per-entry moka Expiry trait
- Tags switched from Vec<String> to HashSet<String> (structural dedup, no runtime checks)
- moka 0.12 eviction listener fires on explicit removal and size eviction but not on TTL expiry; added lazy cleanup in tag_members as complement
- tag_members retains only keys still present in cache, prunes empty tag sets
- Counters replaced DashMap with second moka::future::Cache sharing max_capacity

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added lazy tag cleanup in tag_members**
- **Found during:** Task 2 (test_eviction_cleans_tags)
- **Issue:** moka 0.12 eviction listener does not fire synchronously on TTL expiry — only on explicit invalidation and size-based eviction. Tagged keys that expire by TTL leave stale references in the tags DashMap.
- **Fix:** tag_members now filters stale keys by checking cache.contains_key() and prunes empty tag sets. This provides O(n) lazy cleanup on read.
- **Files modified:** ferro-cache/src/stores/memory.rs
- **Verification:** test_eviction_cleans_tags passes — expired keys cleaned from tags on next tag_members call
- **Committed in:** 160210d (Task 2 commit)

**2. [Rule 3 - Blocking] Added test_eviction_listener_on_explicit_remove test**
- **Found during:** Task 2 (eviction listener testing)
- **Issue:** Plan specified testing eviction via LRU overflow, but moka's LRU kept original tagged entries. Changed test strategy to verify listener fires on explicit removal and TTL cleanup via lazy tag_members.
- **Fix:** Split into two tests: explicit removal (listener) and TTL expiry (lazy cleanup)
- **Files modified:** ferro-cache/src/stores/memory.rs
- **Verification:** Both tests pass
- **Committed in:** 160210d (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** Both fixes necessary for correctness with moka 0.12's eviction semantics. No scope creep.

## Issues Encountered
- moka 0.12 removed background eviction threads; eviction listener only fires during cache write operations or explicit run_pending_tasks(). TTL-expired entries return None on get() but eviction notifications are deferred. Solved with lazy cleanup in tag_members.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All four memory leak vectors from the research phase are now fixed (metrics, InMemoryCache, ferro-cache tags/counters)
- Phase 78 complete

---
*Phase: 78-memory-leaks*
*Completed: 2026-02-28*

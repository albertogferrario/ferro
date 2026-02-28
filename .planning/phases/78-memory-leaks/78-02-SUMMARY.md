---
phase: 78-memory-leaks
plan: 02
subsystem: cache
tags: [moka, cache, memory, ttl, eviction, lru]

requires:
  - phase: 78-memory-leaks
    provides: Research identifying InMemoryCache as unbounded leak vector
provides:
  - InMemoryCache backed by moka::sync::Cache with bounded capacity and per-entry TTL
  - with_capacity() constructor for custom sizing
affects: [ferro-cache, framework-cache]

tech-stack:
  added: [moka 0.12 (sync feature) in framework crate]
  patterns: [Expiry trait for per-entry TTL, CacheValue wrapper for TTL metadata]

key-files:
  created: []
  modified: [framework/Cargo.toml, framework/src/cache/memory.rs]

key-decisions:
  - "moka::sync::Cache over moka::future::Cache — InMemoryCache CacheStore uses async_trait but operations are synchronous; sync avoids unnecessary runtime overhead"
  - "CacheValue wraps value + optional TTL so Expiry trait reads TTL per entry"
  - "Default capacity 10,000 entries matching ferro-cache MemoryStore convention"
  - "expire_after_read returns existing duration_until_expiry (no TTL refresh on read)"

patterns-established:
  - "CacheTtlExpiry pattern: Expiry trait implementation reads TTL from value metadata"
  - "with_capacity() constructor for bounded cache instances"

duration: 12min
completed: 2026-02-28
---

# Phase 78 Plan 02: InMemoryCache Moka Replacement Summary

**Replaced RwLock<HashMap> InMemoryCache with moka::sync::Cache providing bounded capacity (10,000 default), per-entry TTL via Expiry trait, and proactive eviction**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Eliminated unbounded HashMap growth in InMemoryCache with moka LRU eviction
- Per-entry TTL via custom Expiry trait implementation (expire_after_create, expire_after_update)
- Proactive expired entry filtering (no more lazy-only TTL checks)
- Lock-free concurrent reads replacing RwLock contention
- 3 new tests verifying capacity bounds, CRUD operations, and TTL eviction
- All 3 existing tests pass unchanged (CacheStore API preserved)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add moka dependency and replace InMemoryCache** - `30f579a` (feat)
2. **Task 2: Update and extend tests** - `1b2dcfa` (test)

## Files Created/Modified
- `framework/Cargo.toml` - Added moka 0.12 with sync feature
- `framework/src/cache/memory.rs` - Replaced entire implementation: CacheValue, CacheTtlExpiry, moka::sync::Cache backend, 3 new tests

## Decisions Made
- Used moka::sync::Cache (not future::Cache) since CacheStore async_trait wraps synchronous operations
- Default capacity 10,000 matches ferro-cache MemoryStore convention
- expire_after_read preserves existing TTL (no refresh on read, matching previous behavior)
- CacheValue struct embeds optional TTL so Expiry trait can read it per entry

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- InMemoryCache memory leak eliminated
- ferro-cache tag and counter fixes are separate plans in the same phase

---
*Phase: 78-memory-leaks*
*Completed: 2026-02-28*

---
phase: 42-api-resources-advanced
plan: 02
subsystem: api
tags: [resources, hashmap, batch-loading, collection, relationships]

# Dependency graph
requires:
  - phase: 41-api-resources-basics
    provides: Resource trait, ResourceMap builder
provides:
  - when_loaded / when_loaded_many methods on ResourceMap for batch-loaded relationships
  - Resource::collection() convenience method for mapping slices
affects: [42-api-resources-advanced, api-handlers]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "HashMap-based conditional field inclusion for batch-loaded relationships"
    - "Default trait method for collection mapping"

key-files:
  created: []
  modified:
    - framework/src/http/resources/resource_map.rs
    - framework/src/http/resources/resource.rs

key-decisions:
  - "None - followed plan as specified"

patterns-established:
  - "when_loaded pattern: HashMap lookup with transform closure, omits field on miss"
  - "collection() as default trait method avoiding boilerplate map/collect"

# Metrics
duration: 4min
completed: 2026-02-10
---

# Phase 42 Plan 02: when_loaded/when_loaded_many + Resource::collection() Summary

**HashMap-based conditional field inclusion for batch-loaded relationships, plus collection mapping convenience method**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-10T05:12:10Z
- **Completed:** 2026-02-10T05:16:18Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `when_loaded` for belongs_to/has_one batch-loaded relationship data via HashMap lookup
- Added `when_loaded_many` for has_many batch-loaded relationship data via HashMap<K, Vec<M>> lookup
- Added `Resource::collection()` default method eliminating iter/map/collect boilerplate
- 7 new tests covering all edge cases (present, missing, empty vec, combined, collection)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add when_loaded and when_loaded_many to ResourceMap** - `bbb810e` (feat)
2. **Task 2: Add Resource::collection() convenience method** - `330fd67` (feat)

## Files Created/Modified

- `framework/src/http/resources/resource_map.rs` - Added when_loaded, when_loaded_many methods + 6 unit tests
- `framework/src/http/resources/resource.rs` - Added collection() default trait method + 1 unit test

## Decisions Made

None - followed plan as specified.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- when_loaded/when_loaded_many ready for use in handlers with batch-loaded data
- Resource::collection() available for all Resource implementors
- Ready for 42-03-PLAN.md

---
*Phase: 42-api-resources-advanced*
*Completed: 2026-02-10*

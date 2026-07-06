---
phase: 38-fix-pre-existing-blockers
plan: 02
subsystem: infra
tags: [storage, s3, inertia, error-handling, tailwind]

# Dependency graph
requires:
  - phase: none
    provides: existing ferro-storage and ferro-inertia crates
provides:
  - S3 driver returns proper errors instead of panicking
  - Inertia dev template without external CDN dependency
affects: [39-core-authentication, 44-real-time-improvements]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "NotImplemented error variant for unfinished driver stubs"

key-files:
  created: []
  modified:
    - ferro-storage/src/error.rs
    - ferro-storage/src/drivers/s3.rs
    - ferro-storage/src/facade.rs
    - ferro-inertia/src/response.rs

key-decisions:
  - "S3 facade returns S3Driver instance that errors on use, rather than panicking at initialization"

patterns-established:
  - "Use Error::NotImplemented for placeholder driver methods"

# Metrics
duration: 4min
completed: 2026-02-09
---

# Phase 38 Plan 02: Fix Storage Placeholders + Inertia CDN Assumption Summary

**S3 driver returns NotImplemented errors instead of panicking, Inertia dev template no longer hardcodes Tailwind CDN**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-09T10:50:51Z
- **Completed:** 2026-02-09T10:54:38Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Replaced 14 `todo!()` macros in S3Driver with proper `Error::NotImplemented` returns
- Replaced `unimplemented!()` in facade `create_driver` with working S3Driver instantiation
- Added `NotImplemented` error variant and convenience constructor to ferro-storage Error enum
- Removed hardcoded Tailwind CDN `<script>` from Inertia development template

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace S3 driver todo!() with proper errors** - `94c73c1` (fix)
2. **Task 2: Remove hardcoded Tailwind CDN from Inertia dev template** - `3a7830e` (fix)

## Files Created/Modified
- `ferro-storage/src/error.rs` - Added NotImplemented variant and constructor
- `ferro-storage/src/drivers/s3.rs` - Replaced 14 todo!() with Error::NotImplemented
- `ferro-storage/src/facade.rs` - Replaced unimplemented!() with S3Driver instantiation
- `ferro-inertia/src/response.rs` - Removed hardcoded Tailwind CDN script tag

## Decisions Made
- S3 facade creates an S3Driver that returns errors on every method call, rather than panicking at driver initialization time. This allows the application to start even with S3 configured, deferring errors to actual usage.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ferro-storage is now safe to use with the `s3` feature enabled without risk of panics
- Inertia dev template works without external CDN access
- Phase 38 plan 01 (test isolation) still pending execution

---
*Phase: 38-fix-pre-existing-blockers*
*Completed: 2026-02-09*

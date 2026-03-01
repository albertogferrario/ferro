---
phase: 91-framework-integration
plan: 01
subsystem: api
tags: [ferro-projections, feature-gate, re-exports, error-conversion]

requires:
  - phase: 90-renderer-trait-json-ui-renderer
    provides: ferro-projections crate with all public types (ServiceDef, Intent, Renderer, etc.)
provides:
  - Feature-gated re-export of all 22 ferro-projections types via `ferro::` namespace
  - Error conversion chain: ferro_projections::Error -> FrameworkError -> HttpResponse
  - `?` operator support for projection errors in handler functions
affects: [91-framework-integration, app]

tech-stack:
  added: []
  patterns: [feature-gated optional dependency with re-exports, error conversion behind cfg gate]

key-files:
  created: []
  modified: [framework/Cargo.toml, framework/src/lib.rs, framework/src/error.rs, framework/src/http/response.rs]

key-decisions:
  - "ProjectionsError and ProjectionsWarning aliases avoid name collisions with existing Error/Warning re-exports"
  - "Projection errors map to FrameworkError::Internal (500 status) since projection failures are internal logic errors"
  - "Both FrameworkError and HttpResponse From impls needed for ? operator in handlers returning Response"

patterns-established:
  - "Feature-gated error conversion: #[cfg(feature)] on From impl blocks for optional crate errors"

duration: 8min
completed: 2026-03-01
---

# Phase 91 Plan 01: Framework Integration Summary

**Feature-gated re-export of 22 ferro-projections types and error conversion chain behind `projections` feature flag**

## Performance

- **Duration:** 8 min
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added `projections` feature gate in framework Cargo.toml with optional ferro-projections dependency
- Re-exported all 22 public types from ferro-projections behind `#[cfg(feature = "projections")]`
- Implemented From<ferro_projections::Error> for both FrameworkError and HttpResponse
- Added test verifying error conversion produces 500 status with message propagation

## Task Commits

Each task was committed atomically:

1. **Task 1: Add projections feature gate and re-exports** - `b6c8961` (feat)
2. **Task 2: Add error conversion for ferro_projections::Error** - `16872cb` (feat)

## Files Created/Modified
- `framework/Cargo.toml` - Added `projections` feature and `ferro-projections` optional dependency
- `framework/src/lib.rs` - Re-export block for all 22 ferro-projections public types
- `framework/src/error.rs` - From<ferro_projections::Error> for FrameworkError + test
- `framework/src/http/response.rs` - From<ferro_projections::Error> for HttpResponse

## Decisions Made
- Used `Error as ProjectionsError` and `Warning as ProjectionsWarning` aliases following existing `Error as EventError`, `Error as QueueError` pattern
- Mapped projection errors to `FrameworkError::Internal` since projection failures are internal logic errors (not user-facing validation)
- Added both FrameworkError and HttpResponse From impls because Response = Result<HttpResponse, HttpResponse> requires the HttpResponse conversion for `?` to work

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 22 ferro-projections types accessible via `ferro::` namespace with `--features projections`
- Error conversion chain complete for handler ergonomics
- Ready for Phase 91 Plan 02 (CLI command) and Plan 03 (MCP tools)

---
*Phase: 91-framework-integration*
*Completed: 2026-03-01*

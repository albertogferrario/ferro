---
phase: 45-dx-polish
plan: 01
subsystem: api
tags: [error-handling, dx, json-response, hints]

# Dependency graph
requires:
  - phase: 38
    provides: FrameworkError variants (ModelNotFound, ParamParse, Unauthorized)
provides:
  - FrameworkError::hint() method with actionable guidance per variant
  - JSON error responses with "hint" field for developer guidance
  - Consistent "message" key in all error JSON responses
affects: [46-mcp-cli-updates, docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Error hint pattern: hint() returns Option<String> per variant"
    - "JSON error response always uses 'message' key (not 'error')"

key-files:
  created: []
  modified:
    - framework/src/error.rs
    - framework/src/http/response.rs

key-decisions:
  - "Always include hints (no production env check) since errors are developer-facing APIs"
  - "Normalize JSON key from 'error' to 'message' for consistency across all variants"

patterns-established:
  - "FrameworkError hint pattern: match on variant, return actionable fix guidance"

# Metrics
duration: 6min
completed: 2026-02-10
---

# Phase 45 Plan 01: Actionable Error Hints Summary

**Added hint() method to FrameworkError with actionable fix guidance per variant, included in JSON error responses**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-10T07:02:41Z
- **Completed:** 2026-02-10T07:08:19Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `hint()` method to `FrameworkError` returning `Option<String>` with developer-facing fix guidance
- Six variants now provide hints: ServiceNotFound, ParamError, ModelNotFound, ParamParse, Database, Unauthorized
- Four variants intentionally have no hints: Internal, Domain, ValidationError, Validation
- Updated `From<FrameworkError> for HttpResponse` to include `"hint"` key in JSON when present
- Normalized JSON error responses to use `"message"` key consistently (was `"error"` for some variants)
- Added 10 tests covering hint presence/absence and correct HTTP status codes

## Task Commits

Each task was committed atomically:

1. **Task 1: Add hint field and enhance FrameworkError messages** - `c2ec124` (feat)
2. **Task 2: Verify error response format includes hints** - `2291bfa` (test)

## Files Created/Modified
- `framework/src/error.rs` - Added `hint()` method and test module with 10 tests
- `framework/src/http/response.rs` - Updated `From<FrameworkError>` impl to include hint in JSON, normalized key names

## Decisions Made
- Always include hints in responses (no production env check) - hints are on developer-facing error APIs, not user-facing endpoints. The plan acknowledged this tradeoff.
- Normalized JSON error key from `"error"` to `"message"` for consistency - all variants now use `"message"` as the primary key.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Inconsistent JSON key names across error variants**
- **Found during:** Task 1 (updating From<FrameworkError> impl)
- **Issue:** Some variants used `"error"` key while Unauthorized and Validation used `"message"` key, creating inconsistent API surface
- **Fix:** Standardized all variants to use `"message"` key
- **Files modified:** framework/src/http/response.rs
- **Verification:** All tests pass with consistent key names
- **Committed in:** c2ec124 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Consistency fix necessary for coherent error API. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Error hint infrastructure complete and tested
- Ready for additional DX polish plans (CLI completeness, documentation catch-up)
- The hint pattern can be extended if new FrameworkError variants are added

---
*Phase: 45-dx-polish*
*Completed: 2026-02-10*

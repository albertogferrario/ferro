---
phase: 81-consumer-dx-polish
plan: 02
subsystem: api
tags: [mcp, openapi, validation, error-handling, ferro-api-mcp]

# Dependency graph
requires:
  - phase: 79-openapi-bridge
    provides: ferro-api-mcp crate with service.rs, http.rs, error.rs
provides:
  - Pre-flight input validation for MCP tool arguments
  - Categorized API error messages with actionable suggestions
  - Structured tool error formatting for agent self-correction
affects: [81-consumer-dx-polish, 82-docs]

# Tech tracking
tech-stack:
  added: []
  patterns: [validate_args pre-flight check, categorized reqwest errors, HTTP status suggestion mapping]

key-files:
  created: []
  modified: [ferro-api-mcp/src/service.rs, ferro-api-mcp/src/http.rs, ferro-api-mcp/src/main.rs]

key-decisions:
  - "validate_args as pure function for testability (no side effects, returns Vec<String>)"
  - "Validation runs before HTTP call to avoid unnecessary network requests"
  - "HTTP status suggestions appended to body text (not separate field) to keep Error enum stable"
  - "url_str captured before Url is moved into request builder"

patterns-established:
  - "Pre-flight validation: validate tool args against input_schema before execute()"
  - "Categorized error formatting: Connection error / API returned HTTP {status} prefixes"

# Metrics
duration: 12min
completed: 2026-02-28
---

# Phase 81-02: Input Validation & Error DX Summary

**Pre-flight argument validation and categorized API error messages with actionable suggestions for agent self-correction**

## Performance

- **Duration:** 12 min
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- validate_args checks required fields and type correctness before API calls, preventing unnecessary HTTP requests
- 8 unit tests cover validation: missing fields, wrong types, valid args, unknown fields, empty required, all type variants, number-accepts-integers, json_type_name
- Connection errors hint "is the server running?", timeouts hint "may be slow or overloaded"
- HTTP status codes 401/403/404/422/429/5xx produce actionable suggestions
- Tool error responses use structured prefixes ("API returned HTTP {status}", "Connection error:") for agent parsing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add input validation before API calls** - `a08d869` (feat)
2. **Task 2: Improve API error responses with actionable suggestions** - `b94f623` (feat)

## Files Created/Modified
- `ferro-api-mcp/src/service.rs` - validate_args, json_type_name, validation wiring in tool handler, structured error formatting, 8 unit tests
- `ferro-api-mcp/src/http.rs` - Categorized reqwest errors, HTTP status suggestions
- `ferro-api-mcp/src/main.rs` - Fixed pre-existing clippy warning (useless format!)

## Decisions Made
- validate_args as pure function returning Vec<String> for testability
- Validation errors returned as CallToolResult::error before HTTP call
- HTTP status suggestions appended to body text to keep Error enum stable
- url_str captured before Url is moved into request builder (ownership)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed pre-existing clippy warning in main.rs**
- **Found during:** Task 1 (clippy verification)
- **Issue:** `format!("literal string")` flagged as useless format
- **Fix:** Changed to `"literal string".to_string()`
- **Files modified:** ferro-api-mcp/src/main.rs
- **Verification:** clippy passes with -D warnings
- **Committed in:** a08d869 (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed pre-existing formatting issues in main.rs**
- **Found during:** Task 1 (fmt verification)
- **Issue:** Two formatting deviations in main.rs
- **Fix:** Ran cargo fmt
- **Files modified:** ferro-api-mcp/src/main.rs
- **Verification:** cargo fmt --check passes
- **Committed in:** a08d869 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Pre-existing issues fixed to pass CI checks. No scope creep.

## Issues Encountered
- Rust ownership: url moved into request builder before url_str was captured. Fixed by reordering the extraction.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Input validation and error DX complete
- Ready for remaining Phase 81 plans (wave 2)

---
*Phase: 81-consumer-dx-polish*
*Completed: 2026-02-28*

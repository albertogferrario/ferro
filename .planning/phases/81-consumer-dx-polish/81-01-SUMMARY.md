---
phase: 81-consumer-dx-polish
plan: 01
subsystem: api
tags: [mcp, openapi, cli, diagnostics, ferro-api-mcp]

requires:
  - phase: 79-consumer-mcp
    provides: ferro-api-mcp crate with CLI, spec parser, MCP server

provides:
  - --dry-run CLI flag for spec validation without starting MCP server
  - Startup diagnostics (version, API name, base URL, tool count)
  - API connectivity check via HEAD request
  - Categorized spec fetch errors (connection, timeout, DNS, HTTP status, JSON validity)

affects: [81-consumer-dx-polish]

tech-stack:
  added: []
  patterns: [categorized-error-messages, startup-diagnostics, dry-run-validation]

key-files:
  created: []
  modified: [ferro-api-mcp/src/main.rs, ferro-api-mcp/src/spec.rs]

key-decisions:
  - "categorize_reqwest_error as helper for spec.rs error classification"
  - "JSON validity check in fetch_spec before returning body"
  - "format_spec_fetch_error in main.rs for presentation-layer error formatting"

patterns-established:
  - "Categorized errors: inspect error type to produce actionable messages"
  - "Startup diagnostics: version/API/URL/count summary before server start"

duration: 8min
completed: 2026-02-28
---

# Phase 81 Plan 01: Startup Diagnostics Summary

**--dry-run CLI flag and categorized spec fetch errors for ferro-api-mcp connection troubleshooting**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added --dry-run CLI flag that validates spec and prints tool summary without starting MCP server
- Added startup summary to stderr (version, API name, base URL, tool count)
- Added best-effort HEAD connectivity check to API base URL
- Categorized spec fetch errors in spec.rs: connection refused, timeout, decode, non-2xx, non-JSON
- Added format_spec_fetch_error in main.rs for presentation-layer error formatting
- Zero-operation specs now produce a warning

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --dry-run flag and startup connectivity check** - `a08d869` (feat) — included in prior 81-02 commit by previous agent
2. **Task 2: Improve spec fetch error categorization** - `fa7c7b3` (feat)

## Files Created/Modified
- `ferro-api-mcp/src/main.rs` - --dry-run flag, startup summary, connectivity check, format_spec_fetch_error
- `ferro-api-mcp/src/spec.rs` - categorize_reqwest_error, HTTP status check, JSON validity check in fetch_spec

## Decisions Made
- `categorize_reqwest_error` inspects `is_connect()`, `is_timeout()`, `is_decode()` for specific messages
- HTTP status checked before body read (non-2xx returns error immediately)
- JSON validity checked with `serde_json::from_str::<Value>` before returning body
- `format_spec_fetch_error` in main.rs handles presentation-layer categorization (DNS, connection refused, timeout) via string matching on error messages

## Deviations from Plan

Task 1 was already implemented by a prior agent in commit `a08d869` (tagged as 81-02). The work was found already present in HEAD. Task 2 was implemented fresh as specified.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 81-01 diagnostics complete
- Ready for subsequent 81-xx plans

---
*Phase: 81-consumer-dx-polish*
*Completed: 2026-02-28*

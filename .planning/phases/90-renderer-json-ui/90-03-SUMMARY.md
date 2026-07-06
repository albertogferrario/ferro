---
phase: 90-renderer-json-ui
plan: 03
subsystem: ui
tags: [renderer, json-ui, process, analyze, track, integration-tests, pipeline]

# Dependency graph
requires:
  - phase: 90-renderer-json-ui
    plan: 02
    provides: JsonUiRenderer with Browse/Focus/Collect/Summarize layouts
provides:
  - All 7 intent layouts fully implemented in JsonUiRenderer (no remaining todo!() stubs)
  - Full pipeline integration tests: ServiceDef -> derive_intents -> render -> valid JSON-UI
  - Doctest documenting JsonUiRenderer public API usage
affects: [91-mcp-integration, 92-cli-scaffold, 93-validation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Process layout: Card+Badge state display with guard Alerts and transition action Buttons"
    - "Analyze layout: summary Card + sortable Table with all readable fields, no Pagination"
    - "Track layout: Table with DateTime system fields visible, sorted desc, with Pagination"
    - "Process Input mode: Form + transition buttons combined"

key-files:
  created: []
  modified:
    - ferro-projections/src/render/json_ui.rs

key-decisions:
  - "Process falls back to Focus when no state machine defined"
  - "Analyze acknowledges JSON-UI has no chart components; sortable Table is the analytical view"
  - "Track includes DateTime system fields (CreatedAt/UpdatedAt) unlike other intents"
  - "Process Input mode combines Collect form with transition action buttons"

patterns-established:
  - "Intent-specific helper functions (is_datetime_field, is_numeric_field) keep filter logic declarative"
  - "Pipeline integration tests use derive_intents() output directly, not hand-crafted IntentScores"

# Metrics
duration: 12min
completed: 2026-03-01
---

# Phase 90 Plan 03: Process/Analyze/Track Layouts + Integration Tests

**All 7 intents fully implemented with 309 total tests validating the complete ServiceDef-to-JSON-UI pipeline**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Implemented Process layout: Card+Badge state display, guard Alert, transition action Buttons; falls back to Focus without state machine
- Implemented Analyze layout: summary Card with stat placeholders for numeric fields, sortable Table with all readable fields including DateTime, no Pagination
- Implemented Track layout: Table with DateTime system fields visible, Status columns, sorted desc, with Pagination
- 5 full pipeline integration tests (product catalog, user profile, survey form, order workflow, activity log)
- Edge case coverage: empty ServiceDef, system-fields-only, all-Sensitive fields
- All 7 intents + Custom render without error on universal service fixture
- Doctest on JsonUiRenderer documenting basic usage

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Process, Analyze, and Track intent layouts** - `c198414` (feat)
2. **Task 2: Full pipeline integration tests and doctest** - `b279ae2` (test)

## Files Created/Modified
- `ferro-projections/src/render/json_ui.rs` - Process/Analyze/Track implementations + integration tests + doctest

## Decisions Made
- Process falls back to Focus layout when no state machine is defined (a Process service without state machine is just a detail view)
- Analyze uses sortable Table as the analytical view since JSON-UI has no chart components
- Track includes DateTime system fields (CreatedAt, UpdatedAt) unlike other intents where they are hidden
- Process Input mode combines Collect form with transition action buttons for editing while progressing state
- Summary stats in Analyze use DescriptionList with data_path bindings as structural placeholders (computation happens at framework layer)

## Deviations from Plan
None - plan executed exactly as written

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 7 intent layouts fully implemented in JsonUiRenderer
- 301 unit tests + 8 doctests = 309 total in ferro-projections
- No remaining todo!() stubs in the render module
- Ready for Phase 91+ MCP integration and CLI scaffold work

---
*Phase: 90-renderer-json-ui*
*Completed: 2026-03-01*

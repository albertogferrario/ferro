---
phase: 25-data-binding
plan: 02
subsystem: ui
tags: [json-ui, data-binding, serde, view-integration]

# Dependency graph
requires:
  - phase: 25-data-binding (plan 01)
    provides: resolve_path and resolve_path_string functions, data_path on form fields
provides:
  - data field on JsonUiView for embedded view data
  - render_json() explicit-vs-embedded data resolution
  - resolve_path and resolve_path_string available via ferro_rs re-exports
affects: [26-action-system, 28-html-renderer]

# Tech tracking
tech-stack:
  added: []
  patterns: [embedded view data with explicit override, skip_serializing_if is_null]

key-files:
  created: []
  modified: [ferro-json-ui/src/view.rs, framework/src/json_ui/mod.rs, framework/src/lib.rs]

key-decisions:
  - "data field placed after title, before components in JsonUiView"
  - "render_json uses explicit data when non-null, falls back to view.data"

patterns-established:
  - "View data embedding: JsonUiView.data carries data alongside components"
  - "Data precedence: explicit render parameter > embedded view data"

# Metrics
duration: 3min
completed: 2026-02-09
---

# Phase 25 Plan 02: View Data Integration Summary

**Embedded data field on JsonUiView with builder method, render_json explicit-vs-embedded data resolution, and framework re-exports for resolve_path/resolve_path_string**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T07:12:54Z
- **Completed:** 2026-02-09T07:16:08Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `data` field to `JsonUiView` with `skip_serializing_if is_null` serde behavior
- Builder `.data()` method for ergonomic view construction with embedded data
- `render_json()` prefers explicit data parameter, falls back to view's embedded data when explicit is null
- `resolve_path` and `resolve_path_string` re-exported from `ferro_rs` public API
- 6 new tests: 4 for view data serialization/round-trip, 2 for render_json data precedence

## Task Commits

Each task was committed atomically:

1. **Task 1: Add data field to JsonUiView and update render API** - `fef4064` (feat)
2. **Task 2: Update framework re-exports and run workspace validation** - `41caebb` (feat)

## Files Created/Modified
- `ferro-json-ui/src/view.rs` - Added data field, builder method, and 4 serialization tests
- `framework/src/json_ui/mod.rs` - Updated render_json() with data precedence logic, added 2 tests
- `framework/src/lib.rs` - Added resolve_path and resolve_path_string to ferro_json_ui re-exports

## Decisions Made
- Placed data field after title and before components to maintain logical field ordering in the struct
- render_json() uses explicit data when non-null, falls back to view.data -- explicit parameter represents "live" handler data while embedded is for self-contained views

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 25 (Data Binding) complete -- resolve_path, resolve_path_string, data_path on form fields, and data on views all implemented
- Ready for Phase 26 (Action System) which maps declared actions to Ferro handlers

---
*Phase: 25-data-binding*
*Completed: 2026-02-09*

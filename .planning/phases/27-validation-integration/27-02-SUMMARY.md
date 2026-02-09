---
phase: 27-validation-integration
plan: 02
subsystem: ui
tags: [json-ui, validation, render-pipeline, hashmap, convenience-methods]

requires:
  - phase: 27-validation-integration-01
    provides: resolve_errors() and resolve_errors_all() tree walkers, JsonUiView.errors field
  - phase: 26-action-system
    provides: clone-resolve-serialize pattern in JsonUi render pipeline

provides:
  - JsonUi::render_with_errors() for HTML with validation errors
  - JsonUi::render_json_with_errors() for JSON with validation errors
  - JsonUi::render_validation_error() accepting framework ValidationError directly
  - JsonUi::render_json_validation_error() JSON variant
  - resolve_errors and resolve_errors_all re-exported from framework

affects: [28-html-renderer, 29-layout-system]

tech-stack:
  added: []
  patterns:
    - "resolve_with_errors combines action + error resolution in single clone"
    - "ValidationError convenience methods delegate to HashMap variants"

key-files:
  created: []
  modified:
    - framework/src/json_ui/mod.rs
    - framework/src/lib.rs

key-decisions:
  - "resolve_with_errors sets view.errors = Some(errors.clone()) alongside field-level resolution"
  - "render_validation_error delegates via .all() to render_with_errors — single indirection"

patterns-established:
  - "Error render methods mirror standard render methods with additional errors parameter"

duration: 4min
completed: 2026-02-09
---

# Phase 27 Plan 02: Framework Render Integration and Re-exports Summary

**JsonUi::render_with_errors() and render_validation_error() convenience methods wire validation errors into the JSON-UI render pipeline, with resolve_errors/resolve_errors_all re-exported from framework**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-09T07:51:42Z
- **Completed:** 2026-02-09T07:55:26Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added resolve_with_errors() private helper combining action resolution and error population
- Added render_with_errors() and render_json_with_errors() accepting HashMap errors
- Added render_validation_error() and render_json_validation_error() accepting framework ValidationError directly
- Re-exported resolve_errors and resolve_errors_all from framework for direct use
- 5 new tests covering field population, JSON response, empty map, framework type, and action preservation

## Task Commits

Each task was committed atomically:

1. **Task 1: Add render_with_errors methods to JsonUi** - `678e612` (feat)
2. **Task 2: Re-export resolve_errors from framework** - `bdc91f8` (feat)

## Files Created/Modified
- `framework/src/json_ui/mod.rs` - Added resolve_with_errors, render_with_errors, render_json_with_errors, render_validation_error, render_json_validation_error, 5 tests
- `framework/src/lib.rs` - Added resolve_errors, resolve_errors_all to ferro_json_ui re-exports

## Decisions Made
- resolve_with_errors sets view.errors = Some(errors.clone()) alongside field-level resolution for dual consumption (component-level + view-level)
- render_validation_error delegates via .all() to render_with_errors for single indirection

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 27 (Validation Integration) complete
- Handlers can use JsonUi::render_validation_error() for one-call error attachment
- Ready for Phase 28 (HTML Renderer)

---
*Phase: 27-validation-integration*
*Completed: 2026-02-09*

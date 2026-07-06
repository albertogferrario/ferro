---
phase: 27-validation-integration
plan: 01
subsystem: ui
tags: [json-ui, validation, tree-walker, serde, hashmap]

requires:
  - phase: 26-action-system
    provides: resolve_actions pattern and component tree walker

provides:
  - resolve_errors() tree walker for first-error-per-field
  - resolve_errors_all() tree walker for concatenated errors
  - JsonUiView.errors field for validation error passthrough

affects: [28-html-renderer, 27-02-framework-integration]

tech-stack:
  added: []
  patterns:
    - "resolve_errors mirrors resolve_actions tree walk pattern"
    - "set_field_error helper with explicit-error-priority rule"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/view.rs
    - ferro-json-ui/src/resolve.rs
    - ferro-json-ui/src/lib.rs

key-decisions:
  - "Explicit component errors take priority over validation map errors"
  - "resolve_errors_all joins messages with '. ' separator"

patterns-established:
  - "Form field error resolution: match by field name, skip if error already set"

duration: 3min
completed: 2026-02-09
---

# Phase 27 Plan 01: Error Resolver and View Errors Field Summary

**resolve_errors() and resolve_errors_all() tree walkers populate form field error messages from validation error map, with explicit-error-priority rule**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T07:47:20Z
- **Completed:** 2026-02-09T07:50:00Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments
- Added `errors: Option<HashMap<String, Vec<String>>>` to `JsonUiView` with `skip_serializing_if`
- Implemented `resolve_errors()` to set first error message per field on Input, Select, Checkbox, Switch
- Implemented `resolve_errors_all()` to concatenate all error messages per field
- Tree walker recurses into Card, Form, Modal, Tabs children (mirrors resolve_actions pattern)
- Explicit component errors are never overwritten by validation map

## Task Commits

Each task was committed atomically:

1. **Task 1: Add errors field to JsonUiView and resolve_errors tree walker** - `04513f9` (feat)

## Files Created/Modified
- `ferro-json-ui/src/view.rs` - Added errors field, builder method, 3 tests
- `ferro-json-ui/src/resolve.rs` - Added resolve_errors, resolve_errors_all, set_field_error helper, resolve_errors_node walker, 10 tests
- `ferro-json-ui/src/lib.rs` - Added resolve_errors, resolve_errors_all to public re-exports

## Decisions Made
- Explicit component errors take priority over validation map errors (do-not-overwrite rule)
- resolve_errors_all joins multiple messages with ". " separator (readable concatenation)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- resolve_errors and resolve_errors_all available in ferro-json-ui public API
- Ready for 27-02 (framework render integration and re-exports)

---
*Phase: 27-validation-integration*
*Completed: 2026-02-09*

---
phase: 26-action-system
plan: 01
subsystem: ui
tags: [json-ui, action-resolver, tree-walker, serde]

requires:
  - phase: 25-data-binding
    provides: JsonUiView with data field and component tree

provides:
  - Action.url field for resolved URLs
  - resolve_actions() tree-walking function
  - resolve_actions_strict() with error collection

affects: [27-validation-integration, 28-html-renderer]

tech-stack:
  added: []
  patterns:
    - "Callback-based action resolution (decoupled from route registry)"
    - "Recursive component tree walking for action mutation"

key-files:
  created:
    - ferro-json-ui/src/resolve.rs
  modified:
    - ferro-json-ui/src/action.rs
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/view.rs
    - ferro-json-ui/src/lib.rs

key-decisions:
  - "url field added directly to Action struct (Option<String>) rather than separate ResolvedAction type"
  - "Resolver uses Fn(&str) -> Option<String> callback to keep ferro-json-ui decoupled from framework"

patterns-established:
  - "Callback-based resolution: ferro-json-ui provides tree walker, framework provides concrete resolver"

duration: 4min
completed: 2026-02-09
---

# Phase 26 Plan 01: Action URL Field and Resolver Summary

**Added url field to Action and built recursive tree-walking action resolver with callback-based handler-to-URL mapping**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-09T07:27:59Z
- **Completed:** 2026-02-09T07:31:54Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Action struct extended with `url: Option<String>` for resolved URLs (omitted from JSON when None)
- `resolve_actions()` walks entire component tree recursively, resolving handler names to URLs via callback
- `resolve_actions_strict()` variant collects unresolvable handler names and returns them as errors
- All component nesting handled: Card children/footer, Form action/fields, Modal children/footer, Tabs tab children, Table row_actions
- 17 new tests (3 action url tests + 9 resolver tests + 5 existing tests updated)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add url field to Action and update serialization** - `9515c05` (feat)
2. **Task 2: Build resolve_actions tree walker** - `f25050f` (feat)

## Files Created/Modified
- `ferro-json-ui/src/resolve.rs` - New module: resolve_actions, resolve_actions_strict, recursive tree walker
- `ferro-json-ui/src/action.rs` - Added `url: Option<String>` field to Action struct, new serialization tests
- `ferro-json-ui/src/component.rs` - Updated Action construction sites with url: None
- `ferro-json-ui/src/view.rs` - Updated Action construction site with url: None
- `ferro-json-ui/src/lib.rs` - Added resolve module declaration and public exports

## Decisions Made
- Added `url` directly to `Action` struct as `Option<String>` rather than creating a separate `ResolvedAction` type. Simpler, works for both HTML and JSON output.
- Resolver uses `Fn(&str) -> Option<String>` callback pattern so ferro-json-ui has zero dependency on the framework's route registry.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Action resolution infrastructure complete, ready for Plan 02 (framework bridge integration)
- resolve_actions and resolve_actions_strict exported and tested
- Plan 02 will connect the resolver to Ferro's `route()` function in the render pipeline

---
*Phase: 26-action-system*
*Completed: 2026-02-09*

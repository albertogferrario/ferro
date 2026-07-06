---
phase: 04-convention-over-configuration
plan: 01
subsystem: routing
tags: [resource-routing, rest, macros, declarative-api]

# Dependency graph
requires:
  - phase: 03-validation-syntax-streamlining
    provides: ValidateRules derive macro, validation infrastructure
provides:
  - resource! macro for RESTful route generation
  - Auto-generated route names from path prefix
  - Support for partial resource routes via only: parameter
  - Middleware chaining support for resources
affects: [routing, api-design, controllers]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "resource! macro pattern for 7 RESTful routes from single declaration"
    - "Convention-based route naming (path.action format)"

key-files:
  created: []
  modified:
    - framework/src/routing/macros.rs
    - framework/src/routing/mod.rs
    - framework/src/lib.rs
    - app/src/controllers/user.rs
    - app/src/routes.rs
    - cancer-macros/src/redirect.rs

key-decisions:
  - "Use declarative macro_rules! instead of proc macro for resource routing"
  - "Module path captured with $($controller:ident)::+ pattern for proper expansion"
  - "Extended redirect! macro validation to recognize resource! patterns"

patterns-established:
  - "resource! macro: Convention over configuration for RESTful routes"
  - "Route naming: path prefix becomes route name prefix (users.index, users.show, etc.)"

# Metrics
duration: 25 min
completed: 2026-01-15
---

# Phase 4 Plan 1: Resource Macro Summary

**Declarative resource! macro for generating 7 RESTful routes from single controller module reference with auto-named routes**

## Performance

- **Duration:** 25 min
- **Started:** 2026-01-15T04:55:00Z
- **Completed:** 2026-01-15T05:20:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Created `resource!` macro that generates all 7 RESTful routes from single declaration
- Reduced user routes from 5 lines to 1 line in sample app
- Auto-generated route names following convention (users.index, users.show, etc.)
- Added `only:` parameter support for generating subset of routes
- Extended redirect! macro to recognize resource! patterns for compile-time validation

## Task Commits

Each task was committed atomically:

1. **Task 1: Create resource! macro for RESTful routes** - `4f05a51` (feat)
2. **Task 2: Add missing controller methods to user.rs** - `a604d5a` (feat)
3. **Task 3: Update sample app routes to use resource!** - `e19d958` (feat)

## Files Created/Modified

- `framework/src/routing/macros.rs` - ResourceDef, ResourceAction, ResourceRoute structs and resource! macro
- `framework/src/routing/mod.rs` - Export new resource routing types
- `framework/src/lib.rs` - Export __box_handler, ResourceAction, ResourceDef, ResourceRoute
- `app/src/controllers/user.rs` - Added create, edit, update, destroy handlers for full RESTful coverage
- `app/src/routes.rs` - Replaced group!/get!/post! with single resource! macro call
- `cancer-macros/src/redirect.rs` - Extended route name extraction to parse resource! patterns

## Decisions Made

1. **Declarative macro over proc macro** - Used macro_rules! like existing routing macros for consistency. Proc macro would allow more flexibility but breaks pattern.

2. **Module path token capture** - Used `$($controller:ident)::+` pattern to properly capture and expand module paths like `controllers::user` with `::action` suffix.

3. **Extended redirect! validation** - Updated the redirect macro's route name extraction to recognize resource! patterns and generate expected route names, maintaining compile-time validation for redirects.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed macro token expansion for module paths**
- **Found during:** Task 3 (Update sample app routes)
- **Issue:** Initial `$controller:path` pattern didn't allow appending `::action` suffix
- **Fix:** Changed to `$($controller:ident)::+` pattern which properly captures module paths
- **Files modified:** framework/src/routing/macros.rs
- **Verification:** `cargo check --package app` passes
- **Committed in:** e19d958 (Task 3 commit)

**2. [Rule 3 - Blocking] Extended redirect! macro route validation**
- **Found during:** Task 3 (Update sample app routes)
- **Issue:** redirect! macro validates route names at compile time but couldn't find resource! generated routes
- **Fix:** Updated extract_route_names() to parse resource! patterns and generate expected route names
- **Files modified:** cancer-macros/src/redirect.rs
- **Verification:** `cargo check --package app` passes with redirect!("users.index")
- **Committed in:** e19d958 (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were necessary for the macro to work correctly. No scope creep.

## Issues Encountered

None - plan executed as expected after fixing the macro expansion issues.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- resource! macro fully functional with middleware chaining
- Sample app demonstrates pattern with 7-route resource in single line
- Ready for additional convention-over-configuration features

---
*Phase: 04-convention-over-configuration*
*Completed: 2026-01-15*

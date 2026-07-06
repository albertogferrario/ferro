---
phase: 40-auth-middleware
plan: 02
subsystem: auth
tags: [extractors, auth-user, sample-app, cli-templates, documentation]

requires:
  - phase: 40-auth-middleware
    provides: AuthUser<T> and OptionalUser<T> extractors (plan 01)

provides:
  - Working AuthUser example in sample app (/auth/profile route)
  - AuthUser tip in make:auth controller template
  - Handler Extractors documentation section with AuthUser, OptionalUser, Deref, and limitations

affects: [41-api-resources, 44-real-time-improvements]

tech-stack:
  added: []
  patterns: [auth-extractor-in-handlers]

key-files:
  created: []
  modified: [app/src/controllers/auth_controller.rs, app/src/routes.rs, ferro-cli/src/templates/mod.rs, docs/src/features/authentication.md]

key-decisions:
  - "Grouped /auth/profile and /auth/logout under a single group with SessionAuthMiddleware"
  - "Used doc comments (//!) for AuthUser tip in template to match module-level documentation style"

patterns-established:
  - "AuthUser<T> as handler parameter for authenticated routes in sample app"

duration: 3min
completed: 2026-02-10
---

# Phase 40 Plan 02: Sample App, Templates, and Docs Summary

**AuthUser example route in sample app, AuthUser tip in make:auth template, and Handler Extractors docs section with AuthUser/OptionalUser/Deref/limitations**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-10T04:03:47Z
- **Completed:** 2026-02-10T04:06:25Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- GET /auth/profile route in sample app demonstrating AuthUser<users::Model> extractor
- AuthUser tip comment in make:auth controller template teaching developers the extractor pattern
- Handler Extractors section in authentication docs covering AuthUser, OptionalUser, Deref behavior, and FromRequest limitations

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AuthUser example route to sample app** - `c6bfb89` (feat)
2. **Task 2: Update make:auth template and docs** - `f11f2fc` (docs)

## Files Created/Modified

- `app/src/controllers/auth_controller.rs` - Added profile handler using AuthUser<users::Model>
- `app/src/routes.rs` - Added GET /auth/profile in authenticated route group
- `ferro-cli/src/templates/mod.rs` - Added AuthUser tip comment to make:auth controller template
- `docs/src/features/authentication.md` - Added Handler Extractors section with examples

## Decisions Made

- Grouped /auth/profile and /auth/logout under a single `group!("/auth", {...}).middleware(SessionAuthMiddleware::new())` instead of individual middleware attachments, for cleaner route organization
- Used `//!` doc comments for the AuthUser tip in the template to match the existing module-level documentation style

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 40 complete: both extractors implemented (plan 01) and integrated into sample app, templates, and docs (plan 02)
- Ready for Phase 41 (API Resources Basics)
- No blockers

---
*Phase: 40-auth-middleware*
*Completed: 2026-02-10*

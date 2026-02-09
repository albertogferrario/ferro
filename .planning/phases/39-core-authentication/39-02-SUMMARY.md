---
phase: 39-core-authentication
plan: 02
subsystem: auth
tags: [controllers, routes, login, register, logout, middleware]

# Dependency graph
requires:
  - phase: 39-01
    provides: auth-ready User model with find_by_email and DatabaseUserProvider
provides:
  - auth controller with register, login, logout handlers
  - auth routes with guest and session middleware guards
affects: [39-03-make-auth, 39-04-docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Auth::attempt with closure for credential validation"
    - "Validator + manual email uniqueness check pattern"
    - "GuestMiddleware for login/register routes"
    - "SessionAuthMiddleware for logout route"

key-files:
  created:
    - app/src/controllers/auth_controller.rs
  modified:
    - app/src/controllers/mod.rs
    - app/src/routes.rs

key-decisions:
  - "Aliased ferro::AuthMiddleware as SessionAuthMiddleware to avoid conflict with app's existing header-based AuthMiddleware"
  - "Named routes (auth.register, auth.login, auth.logout) for compile-time validated redirects"

patterns-established:
  - "Auth controller pattern: deserialize input, validate with Validator, check uniqueness, hash/verify, Auth::login/logout"
  - "Guest-only route group for registration and login"

# Metrics
duration: 5min
completed: 2026-02-09
---

# Phase 39 Plan 02: Auth Controllers and Routes Summary

**Login and register controllers with routes demonstrating the complete auth flow using Auth facade, validation, and hashing**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-09
- **Completed:** 2026-02-09
- **Tasks:** 2
- **Files created:** 1
- **Files modified:** 2

## Accomplishments
- AuthController with register handler: validates input (name, email, password with confirmation), checks email uniqueness, hashes password, creates user via SeaORM, logs in via Auth::login, returns 201 JSON
- AuthController with login handler: validates input, uses Auth::attempt with closure calling find_by_email + verify, returns user data or 422 with credential error
- AuthController with logout handler: calls Auth::logout, returns 200 JSON
- Routes registered: POST /auth/register and POST /auth/login (guest-only via GuestMiddleware), POST /auth/logout (session-auth required via AuthMiddleware::new())

## Task Commits

Each task was committed atomically:

1. **Task 1: Create auth controller with register, login, logout handlers** - `b2c23c9` (feat)
2. **Task 2: Register auth routes with middleware** - `bc30cdc` (feat)

## Files Created/Modified
- `app/src/controllers/auth_controller.rs` - New file with register, login, logout handlers
- `app/src/controllers/mod.rs` - Added `pub mod auth_controller`
- `app/src/routes.rs` - Added auth route group (guest) and logout route (authenticated)

## Decisions Made
- Aliased `ferro::AuthMiddleware` as `SessionAuthMiddleware` to avoid name conflict with the app's existing header-based `AuthMiddleware`
- Added named routes (`auth.register`, `auth.login`, `auth.logout`) for compile-time validated references

## Deviations from Plan

None significant. The plan suggested `GuestMiddleware::redirect_to("/")` which was followed exactly. The plan used `AuthMiddleware::new()` for the logout route which was also followed.

## Issues Encountered
None.

## User Setup Required
None.

## Next Phase Readiness
- Auth controllers and routes complete, matching the pattern that make:auth (39-03) scaffolds
- No blockers for remaining phase plans

---
*Phase: 39-core-authentication*
*Completed: 2026-02-09*

---
phase: 40-auth-middleware
plan: 01
subsystem: auth
tags: [extractors, fromrequest, handler-injection, auth-user]

requires:
  - phase: 39-core-authentication
    provides: Auth facade, Authenticatable trait, UserProvider, session-based auth

provides:
  - AuthUser<T> extractor for required authentication in handler signatures
  - OptionalUser<T> extractor for optional authentication in handler signatures
  - FromRequest implementations for both extractors

affects: [40-02-sample-app, 44-real-time-improvements]

tech-stack:
  added: []
  patterns: [typed-parameter-extraction, deref-for-ergonomic-access]

key-files:
  created: [framework/src/auth/extract.rs]
  modified: [framework/src/auth/mod.rs, framework/src/lib.rs]

key-decisions:
  - "401 via FrameworkError::domain instead of FrameworkError::Unauthorized (which is 403)"
  - "AuthUser counts as the one FromRequest param per handler (existing framework constraint)"

patterns-established:
  - "Auth extractors use session thread-locals, do not consume request body"
  - "Deref implementation on extractors for ergonomic field access"

duration: 5min
completed: 2026-02-10
---

# Phase 40 Plan 01: Auth User Extractors Summary

**AuthUser<T> and OptionalUser<T> extractors with FromRequest implementations and Deref for ergonomic handler parameter injection**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-10T03:56:47Z
- **Completed:** 2026-02-10T04:01:45Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- AuthUser<T> extractor that returns 401 when not authenticated, typed user when authenticated
- OptionalUser<T> extractor that returns None for guests, Some(user) when authenticated
- Both types re-exported from framework public API via `use ferro::AuthUser` / `use ferro::OptionalUser`
- Deref implementations for ergonomic access (e.g., `user.email` instead of `user.0.email`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create AuthUser and OptionalUser extractors** - `ad014fc` (feat)
2. **Task 2: Re-export extractors from framework public API** - `d7beb5c` (feat)

## Files Created/Modified

- `framework/src/auth/extract.rs` - AuthUser<T> and OptionalUser<T> with FromRequest and Deref impls
- `framework/src/auth/mod.rs` - Added extract module and re-exports
- `framework/src/lib.rs` - Added AuthUser and OptionalUser to public re-exports

## Decisions Made

- Used `FrameworkError::domain("Unauthenticated.", 401)` for AuthUser's unauthenticated error instead of `FrameworkError::Unauthorized` which maps to 403 (authorization failure, not authentication failure)
- AuthUser counts as the one FromRequest param per handler, matching the existing framework constraint that only one body-consuming extractor is allowed per handler signature

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Extractors ready for integration testing in sample app (Plan 40-02)
- Both types work as #[handler] parameters via the existing FromRequest classification in the handler macro
- No blockers for next plan

---
*Phase: 40-auth-middleware*
*Completed: 2026-02-10*

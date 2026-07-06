---
phase: 74-session-absolute-expiry
plan: 01
subsystem: auth
tags: [session, owasp, timeout, security, sea-orm]

# Dependency graph
requires:
  - phase: 73-security-headers
    provides: security middleware foundation
provides:
  - Dual timeout session management (idle + absolute)
  - created_at tracking on sessions entity
  - destroy_for_user for bulk session invalidation
  - CLI migration template with created_at column
affects: [authentication, session-management]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dual timeout enforcement: idle (last_activity) + absolute (created_at)"
    - "Backward compat via nullable created_at with skip-if-NULL logic"
    - "Default trait method with error for unsupported drivers"

key-files:
  modified:
    - framework/src/session/config.rs
    - framework/src/session/store.rs
    - framework/src/session/driver/database.rs
    - framework/src/session/middleware.rs
    - ferro-cli/src/templates/files/backend/migrations/create_sessions_table.rs.tpl
    - ferro-cli/src/templates/files/root/env.example.tpl
    - docs/src/features/authentication.md

key-decisions:
  - "Nullable created_at for backward compat with existing sessions tables"
  - "destroy_for_user default returns error rather than unimplemented panic"
  - "Cookie max_age uses max(idle, absolute) so cookie outlives both server-side checks"

patterns-established:
  - "Dual timeout: both idle and absolute enforced in read() and gc()"
  - "NotSet on update to preserve original column value"

# Metrics
duration: 8min
completed: 2026-02-26
---

# Phase 74 Plan 01: Session Absolute Expiry Summary

**Dual timeout (idle + absolute) session enforcement with created_at tracking and bulk invalidation**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-26
- **Completed:** 2026-02-26
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- SessionConfig now has `idle_lifetime` (2h default) and `absolute_lifetime` (30 days default) with env var support
- DatabaseSessionDriver enforces both timeouts in `read()` and cleans up both in `gc()`
- Sessions entity has nullable `created_at` column; set on INSERT, preserved on UPDATE
- `destroy_for_user` on SessionStore trait enables "logout other devices" and "logout everywhere" flows
- CLI migration template and env.example updated for new projects

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dual timeout config, created_at entity, and driver enforcement** - `2de1cad` (feat)
2. **Task 2: Add destroy_for_user + CLI template + env template + tests** - `a33eae5` (feat)

## Files Created/Modified
- `framework/src/session/config.rs` - Renamed lifetime to idle_lifetime, added absolute_lifetime with env var parsing
- `framework/src/session/store.rs` - Added destroy_for_user to SessionStore trait with default error impl
- `framework/src/session/driver/database.rs` - Dual timeout in read()/gc(), created_at entity field, destroy_for_user impl, unit tests
- `framework/src/session/middleware.rs` - Pass both lifetimes to driver, cookie max_age uses longer timeout
- `ferro-cli/src/templates/files/backend/migrations/create_sessions_table.rs.tpl` - Added created_at column and DeriveIden variant
- `ferro-cli/src/templates/files/root/env.example.tpl` - Added SESSION_ABSOLUTE_LIFETIME variable
- `docs/src/features/authentication.md` - Updated session config table with dual timeout docs

## Decisions Made
- Nullable `created_at` for backward compatibility with existing deployed sessions tables (NULL skips absolute check)
- `destroy_for_user` default trait impl returns FrameworkError rather than panicking, so custom store implementors are not burdened
- Cookie max_age set to `max(idle_lifetime, absolute_lifetime)` so the cookie survives long enough for the server to enforce the real expiry

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Updated authentication docs**
- **Found during:** Task 2 (env template update)
- **Issue:** CLAUDE.md mandates docs update when framework changes; authentication.md referenced old SESSION_LIFETIME without dual timeout context
- **Fix:** Added SESSION_ABSOLUTE_LIFETIME row and clarified SESSION_LIFETIME as idle timeout
- **Files modified:** docs/src/features/authentication.md
- **Verification:** Doc table now reflects both timeout variables
- **Committed in:** a33eae5 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Doc update necessary per project rules. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Dual timeout enforcement is complete and backward-compatible
- Existing deployed apps with old sessions tables will silently skip absolute check (NULL created_at)
- New apps get the full schema via updated CLI migration template

---
*Phase: 74-session-absolute-expiry*
*Completed: 2026-02-26*

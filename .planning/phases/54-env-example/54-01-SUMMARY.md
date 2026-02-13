---
phase: 54-env-example
plan: 01
subsystem: infra
tags: [env, config, cli, templates, dotenv]

# Dependency graph
requires:
  - phase: none
    provides: none
provides:
  - Accurate env.example.tpl matching all framework env var usage
affects: [new-project-scaffold, developer-onboarding]

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - ferro-cli/src/templates/files/root/env.example.tpl

key-decisions:
  - "Keep AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY despite no explicit env::var calls (read by AWS SDK implicitly)"
  - "Keep MAIL_DRIVER because scaffolded app template (mail.rs.tpl) reads it"
  - "Use actual code defaults for all values (SESSION_SECURE=true, not false)"

patterns-established: []

# Metrics
duration: 3min
completed: 2026-02-13
---

# Phase 54 Plan 01: Env Example Summary

**Updated env.example.tpl to exactly match the framework's 63 actual env vars: removed 8 phantom vars, added 23 missing vars, fixed 3 inaccurate defaults**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-13T12:13:10Z
- **Completed:** 2026-02-13T12:15:56Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Complete audit of all env::var, env(), env_optional(), env_required() calls across the entire workspace
- Removed 8 phantom variables that had no corresponding code reads
- Added 23 missing variables that code reads but were absent from the template
- Fixed 3 inaccurate entries (AWS_ENDPOINT -> AWS_URL, SESSION_SECURE default, added MAIL_ENCRYPTION)
- Template now has 63 env vars, all verified against source code

## Task Commits

Each task was committed atomically:

1. **Task 1: Audit env vars - build definitive list from code** - `18d8fd6` (feat)
2. **Task 2: Verify template renders and lint passes** - verification only, no code changes

## Files Created/Modified
- `ferro-cli/src/templates/files/root/env.example.tpl` - Complete rewrite to match actual codebase env var usage

## Decisions Made

1. **Kept AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY** despite no explicit `env::var()` calls - the AWS SDK reads these implicitly through its credential chain, and the storage config doc comments document them as required.
2. **Kept MAIL_DRIVER** because the scaffolded app template (`mail.rs.tpl`) reads it via `env("MAIL_DRIVER", ...)`, even though the framework's notification dispatcher does not.
3. **Used true code defaults** throughout - e.g., SESSION_SECURE defaults to `true` in code, was `false` in old template.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added SERVER_MAX_BODY_SIZE to template**
- **Found during:** Task 1 (codebase audit)
- **Issue:** `SERVER_MAX_BODY_SIZE` is read by `framework/src/config/providers/server.rs` but was not listed in the plan's add list
- **Fix:** Added to template with default 10485760 (10MB)
- **Committed in:** 18d8fd6

**2. [Rule 2 - Missing Critical] Added REDIS_PREFIX and CACHE_DEFAULT_TTL to template**
- **Found during:** Task 1 (codebase audit)
- **Issue:** `framework/src/cache/config.rs` reads REDIS_PREFIX and CACHE_DEFAULT_TTL (separate from ferro-cache's CACHE_PREFIX and CACHE_TTL)
- **Fix:** Added both to CACHE section with their code defaults
- **Committed in:** 18d8fd6

---

**Total deviations:** 2 auto-fixed (both missing critical vars discovered during audit)
**Impact on plan:** All auto-fixes necessary for completeness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 54 complete (1/1 plan), ready for Phase 55 (Split Templates)
- No blockers or concerns

---
*Phase: 54-env-example*
*Completed: 2026-02-13*

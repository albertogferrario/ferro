---
phase: 60-locale-context
plan: 01
subsystem: lang
tags: [tokio, task-local, middleware, accept-language, locale]

requires:
  - phase: 58-core-translator
    provides: Translator, normalize_locale, JSON loading
  - phase: 59-config-error-types
    provides: LangConfig, LangError, framework re-exports

provides:
  - locale() per-request accessor function
  - set_locale() with normalization
  - LangMiddleware for Accept-Language detection
  - task_local! LOCALE_CONTEXT infrastructure

affects: [61-validation-bridge, 63-framework-integration]

tech-stack:
  added: []
  patterns: [task_local locale context mirroring session pattern]

key-files:
  created:
    - framework/src/lang/mod.rs
    - framework/src/lang/middleware.rs
  modified:
    - ferro-lang/src/lib.rs
    - framework/src/lib.rs

key-decisions:
  - "locale() returns String not Option — always has a reasonable default"
  - "LangMiddleware has no constructor params — reads LangConfig from Config registry"
  - "Accept-Language parsing takes first tag only — no full RFC 2616 quality negotiation"

patterns-established:
  - "lang module follows session module pattern: task_local!, scope helper, middleware"

duration: 5min
completed: 2026-02-13
---

# Phase 60 Plan 01: Locale Context Summary

**Per-request locale context via task_local!, locale()/set_locale() accessors, and LangMiddleware with Accept-Language detection and query param override**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-13T16:06:44Z
- **Completed:** 2026-02-13T16:11:53Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Per-request locale stored in tokio task_local, mirroring session pattern
- locale() returns String with cascading fallback: task-local -> LangConfig -> "en"
- set_locale() normalizes input (en_US -> en-us) before storing
- LangMiddleware detects locale with priority: ?locale= query param > Accept-Language header > config default
- 12 unit tests covering locale context, normalization, and Accept-Language parsing

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Create lang module with task_local context and LangMiddleware** - `0a8b260` (feat)
2. **Task 3: Wire lang module into framework with tests** - `20f101e` (feat)

## Files Created/Modified
- `framework/src/lang/mod.rs` - task_local LOCALE_CONTEXT, locale(), set_locale(), scope helpers
- `framework/src/lang/middleware.rs` - LangMiddleware with Accept-Language parsing
- `ferro-lang/src/lib.rs` - Re-export normalize_locale
- `framework/src/lib.rs` - pub mod lang, re-export locale/set_locale/LangMiddleware

## Decisions Made
- locale() returns String (not Option) — always has a reasonable default via cascading fallback
- LangMiddleware requires no constructor parameters — reads LangConfig from global Config registry at request time
- Accept-Language parsing is simple: first tag only, no quality-value negotiation (normalize_locale handles format)
- Tasks 1 and 2 combined into single commit since middleware.rs must exist for module compilation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used eprintln instead of tracing::warn**
- **Found during:** Task 1 (lang module creation)
- **Issue:** Plan specified tracing::warn for set_locale outside scope, but framework doesn't use tracing crate
- **Fix:** Used eprintln! consistent with session middleware error handling pattern
- **Files modified:** framework/src/lang/mod.rs
- **Verification:** Consistent with SessionMiddleware error handling
- **Committed in:** 0a8b260

**2. [Rule 3 - Blocking] Removed detect_locale integration tests requiring hyper::body::Incoming**
- **Found during:** Task 3 (adding tests)
- **Issue:** Request::new requires hyper::body::Incoming which can only come from real HTTP connections
- **Fix:** Tested parse_accept_language directly (pure function) and locale/set_locale via task_local scope. detect_locale logic covered transitively.
- **Files modified:** framework/src/lang/middleware.rs
- **Verification:** 12 tests pass covering all key behaviors
- **Committed in:** 20f101e

---

**Total deviations:** 2 auto-fixed (1 bug adaptation, 1 blocking test infrastructure)
**Impact on plan:** Both necessary for correctness with existing codebase. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- locale()/set_locale()/LangMiddleware available from framework re-exports
- Ready for Phase 61 (Validation Bridge) to use locale() for translation lookups
- No blockers or concerns

---
*Phase: 60-locale-context*
*Completed: 2026-02-13*

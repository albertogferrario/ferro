---
phase: 12-agent-first-polish
plan: 01
subsystem: cli
tags: [error-handling, ux, agent-first, cli]

# Dependency graph
requires: []
provides:
  - Actionable error messages with cause and fix instructions
  - fail_with helper for consistent error formatting
  - Accurate port/host display in `cancer serve`
affects: [12-02, 12-03, 12-04]

# Tech tracking
tech-stack:
  added: []
  patterns: [actionable-error-pattern, fail_with-helper]

key-files:
  created: []
  modified:
    - app/src/main.rs
    - app/src/bootstrap.rs
    - cancer-cli/src/commands/serve.rs

key-decisions:
  - "fail_with helper in main.rs for standardized error output"
  - "Inline detailed error handling in bootstrap.rs for DB init"
  - "Read SERVER_HOST and SERVER_PORT from .env for accurate CLI display"

patterns-established:
  - "Error pattern: Error → Cause → How to fix → Example (numbered list)"
  - "CLI args default (8000) vs framework config default (8080) resolution"

# Metrics
duration: 8min
completed: 2026-01-16
---

# Phase 12: Agent-First Polish - Plan 01 Summary

**Replaced panicking .expect() calls with actionable error messages that guide agents to fix issues**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-16T09:50:00Z
- **Completed:** 2026-01-16T09:58:00Z
- **Tasks:** 4 (2 already done in prior commits, 2 completed in this session)
- **Files modified:** 3

## Accomplishments

- All `.expect()` calls in main.rs replaced with actionable `fail_with` helper
- bootstrap.rs DB init error provides clear fix instructions with example .env values
- `cancer serve` now displays actual SERVER_HOST:SERVER_PORT from .env, not CLI defaults
- Consistent error format: Error → Cause → How to fix (numbered list)

## Task Commits

Each task was committed atomically:

1. **Task 1: Make main.rs errors actionable** - `de37845` (feat) - *Prior commit*
2. **Task 2: Make bootstrap.rs errors actionable** - `93e8921` (feat)
3. **Task 3: Create fail_with helper** - `de37845` (feat) - *Combined with Task 1*
4. **Task 4: Fix serve command port display** - `cd38ef3` (feat)

## Files Created/Modified

- `app/src/main.rs` - Added fail_with helper, replaced all .expect() calls with actionable messages
- `app/src/bootstrap.rs` - Replaced DB::init .expect() with detailed error guidance
- `cancer-cli/src/commands/serve.rs` - Read SERVER_HOST/PORT from .env for accurate display

## Decisions Made

1. **fail_with helper pattern** - Centralized error formatting in main.rs with context + cause + fix list
2. **Inline error for bootstrap** - Single error point, more detailed than fail_with template
3. **ENV precedence for ports** - CLI default (8000) triggers env lookup, explicit CLI arg overrides

## Deviations from Plan

None - plan executed exactly as written. Tasks 1 and 3 were already completed in prior commit `de37845`.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Agent-first error handling pattern established
- Ready for Plan 02 (improved .env.example documentation)

---
*Phase: 12-agent-first-polish*
*Plan: 01*
*Completed: 2026-01-16*

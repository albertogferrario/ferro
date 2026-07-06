---
phase: 86-actions-preconditions
plan: 02
subsystem: api
tags: [serde, validation, builder-pattern, service-projections, cross-phase]

# Dependency graph
requires:
  - phase: 86-01
    provides: ActionDef, InputDef, GuardDef types and FieldDef readable/writable
  - phase: 85-02
    provides: StateMachine with validate(), Warning enum, Transition with guard field
provides:
  - ServiceDef with actions/guards fields and builder methods
  - ServiceDef::validate() as single cross-phase validation entry point
  - Warning::UnusedGuard and Warning::TransitionTriggerWithoutStateMachine variants
  - 97 total tests (93 unit + 4 doctests)
affects: [87-relationships, 88-intent-graph, 89-intent-derivation]

# Tech tracking
tech-stack:
  added: []
  patterns: [cross-phase validation via shared guard pool, single entry point validation]

key-files:
  created: []
  modified: [ferro-projections/src/service.rs, ferro-projections/src/state.rs, ferro-projections/src/action.rs, ferro-projections/src/field.rs]

key-decisions:
  - "ServiceDef::validate() subsumes StateMachine::validate() — single entry point for all service validation"
  - "Guards are a shared pool referenced from both Transition.guard (Phase 85) and ActionDef.preconditions (Phase 86)"
  - "Validation returns Err for undefined references, Ok(warnings) for structural concerns"

patterns-established:
  - "Cross-phase validation: single validate() method checks references across state machine and action subsystems"
  - "Warning enum shared between state machine and service-level validation"

# Metrics
duration: 6min
completed: 2026-02-28
---

# Phase 86, Plan 02: Actions & Preconditions — ServiceDef Integration & Validation

**ServiceDef::validate() as cross-phase entry point with guard pool shared between transitions and action preconditions**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- ServiceDef extended with actions/guards Vec fields and builder methods
- Warning enum gains UnusedGuard and TransitionTriggerWithoutStateMachine variants
- ServiceDef::validate() performs 7-step validation: state machine delegation, guard reference checks from actions and transitions, transition trigger matching, unused guard warnings, trigger-without-state-machine warnings
- 19 new tests covering all validation paths, builder chains, serde round-trips
- Total test count: 93 unit + 4 doctests = 97

## Task Commits

Each task was committed atomically:

1. **Task 1: ServiceDef integration + Warning extension + validate()** - `c825c07` (feat)
2. **Task 2: Comprehensive test suite** - `110a138` (test)

## Files Created/Modified
- `ferro-projections/src/service.rs` - actions/guards fields, builder methods, validate() method, 13 new tests
- `ferro-projections/src/state.rs` - Warning::UnusedGuard and Warning::TransitionTriggerWithoutStateMachine variants
- `ferro-projections/src/action.rs` - 2 new tests (state-independent action, minimal serde)
- `ferro-projections/src/field.rs` - 4 new tests (readable/writable combinations)

## Decisions Made
- ServiceDef::validate() subsumes StateMachine::validate() as the single validation entry point
- Guards form a shared pool referenced by both transitions (Phase 85) and action preconditions (Phase 86)
- Undefined references are hard errors (Err), structural concerns are warnings (Ok(Vec<Warning>))

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 86 complete: ActionDef, InputDef, GuardDef types (Plan 01) + ServiceDef integration and validation (Plan 02)
- ServiceDef::validate() ready for Phase 87 to extend with relationship validation
- Guard pool pattern established for future cross-system reference checks

---
*Phase: 86-actions-preconditions*
*Completed: 2026-02-28*

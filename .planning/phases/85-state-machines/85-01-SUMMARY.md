---
phase: 85-state-machines
plan: 01
subsystem: api
tags: [serde, rust, projections, state-machine, builder-pattern, bfs-validation]

# Dependency graph
requires:
  - phase: 84-service-identity-field-semantics
    provides: ferro-projections crate with ServiceDef, FieldDef, Error types
provides:
  - StateMachine, StateDef, Transition, Warning types with builder APIs
  - BFS validation for reachable states, dead-ends, missing initial
  - Convenience queries (states_for_event, events_from_state)
  - ServiceDef.state_machine() integration
affects: [86-actions-preconditions, 88-intent-core-types, 89-resolved-fields, 90-intent-graph, 92-mcp-introspection, 93-renderers]

# Tech tracking
tech-stack:
  added: []
  patterns: [state-machine-schema, bfs-reachability, warning-vs-error-validation]

key-files:
  created:
    - ferro-projections/src/state.rs
  modified:
    - ferro-projections/src/service.rs
    - ferro-projections/src/lib.rs

key-decisions:
  - "Flat states only, no hierarchical/compound states in v1"
  - "Guards as Option<String>, actions as Vec<String> — string references, not closures"
  - "validate() returns Result<Vec<Warning>, Error> — warnings for structural concerns, errors for fatal issues"
  - "Removed Eq from ServiceDef — StateDef.metadata uses serde_json::Value which only implements PartialEq"
  - "Added PartialEq to StateMachine/StateDef/Transition for ServiceDef PartialEq derivation"

patterns-established:
  - "Warning enum for non-fatal validation issues vs Error for fatal ones"
  - "BFS reachability from initial state for state graph validation"
  - "Convenience query methods (states_for_event, events_from_state) on schema types"

# Metrics
duration: 7min
completed: 2026-02-28
---

# Phase 85, Plan 01: State Machine Schema Types Summary

**StateMachine/StateDef/Transition types with builder APIs, BFS validation, and ServiceDef integration in ferro-projections**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- StateMachine schema with name, display_name, description, initial_state, states, transitions
- StateDef with is_final, on_enter/on_exit side effects, metadata
- Transition with from/event/to, guard, actions, description
- Warning enum (UnreachableState, DeadEndState, NoFinalStates)
- validate() with 6 checks: initial set, initial exists, transition refs valid, BFS reachability, dead-end detection, no-final-states
- Convenience methods: states_for_event(), events_from_state()
- ServiceDef.state_machine() builder integration
- 18 new tests + 2 doctests, all 39 unit tests + 2 doctests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create state.rs with all types + builders + validation** - `fb7c8f3` (feat)
2. **Task 2: Integrate StateMachine into ServiceDef + re-exports** - `20534be` (feat)

## Files Created/Modified
- `ferro-projections/src/state.rs` - StateMachine, StateDef, Transition, Warning with builders, validation, convenience methods, 18 tests
- `ferro-projections/src/service.rs` - Added state_machine field and builder method to ServiceDef
- `ferro-projections/src/lib.rs` - Added mod state and pub use re-exports

## Decisions Made
- Removed `Eq` from `ServiceDef` because `StateDef.metadata` contains `serde_json::Value` which only implements `PartialEq`, not `Eq`. No external code depends on `ServiceDef: Eq`.
- Added `PartialEq` derive to `StateMachine`, `StateDef`, `Transition` to support `ServiceDef`'s `PartialEq` derivation.
- Followed research spec for flat states, string guards/actions, warning-based validation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added PartialEq derives for ServiceDef compatibility**
- **Found during:** Task 2 (ServiceDef integration)
- **Issue:** ServiceDef derives PartialEq, but StateMachine/StateDef/Transition didn't, causing compile error
- **Fix:** Added PartialEq to all three types. Removed Eq from ServiceDef since serde_json::Value doesn't implement Eq.
- **Files modified:** ferro-projections/src/state.rs, ferro-projections/src/service.rs
- **Verification:** All 39 tests pass, clippy clean
- **Committed in:** 20534be (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary for type compatibility. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- StateMachine schema types ready for Phase 86 (actions/preconditions) to define guard/action implementations
- ServiceDef extensible for Phase 87 (relationships) and beyond
- validate() ready for MCP introspection in Phase 92
- states_for_event/events_from_state ready for IntentGraph generation in Phase 89-90

---
*Phase: 85-state-machines*
*Completed: 2026-02-28*

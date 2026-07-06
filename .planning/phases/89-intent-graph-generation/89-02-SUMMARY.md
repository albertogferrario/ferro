---
phase: 89-intent-graph-generation
plan: 02
subsystem: projections
tags: [intent-derivation, state-machine-analysis, relationship-analysis, action-analysis, signal-scoring]

requires:
  - phase: 89-intent-graph-generation
    plan: 01
    provides: derive_intents() engine, field meaning + writability analyzers, aggregation pipeline, normalization
provides:
  - State machine analyzer (Process vs Track discrimination by shape)
  - Relationship analyzer (Browse vs Focus discrimination by cardinality)
  - Action analyzer (Process vs Collect vs Browse discrimination by patterns)
  - Full 5-analyzer derivation pipeline
affects: [90-intent-graph, 91-renderer]

tech-stack:
  added: []
  patterns: [branching/guard detection for Process, linear progression for Track, cardinality-driven Browse/Focus, action pattern analysis]

key-files:
  created: []
  modified: [ferro-projections/src/derive.rs]

key-decisions:
  - "State machine shape (branching+guards vs linear) is the key Process/Track discriminator, not state count alone"
  - "Guard density as ratio (guarded/total transitions) provides proportional Process weight"
  - "Relationship analyzer uses OneToMany/ManyToMany for Browse, OneToOne+Inline for Focus, ManyToOne for Focus"
  - "Action analyzer cross-references transition_trigger for Process, input count>2 for Collect, simple CRUD for Browse"

patterns-established:
  - "State machine analyzer produces both Process and Track signals, letting normalization determine winner"
  - "Action analyzer contributes to both state machine and standalone scoring (transition_trigger signals from both)"
  - "Signal name format: {count}_{SIGNAL_CONSTANT} for count-qualified signals"

duration: 8min
completed: 2026-03-01
---

# Phase 89 Plan 02: Complete 5-Analyzer Derivation Pipeline Summary

**State machine, relationship, and action analyzers complete the full signal pipeline for structural intent derivation**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- State machine analyzer: discriminates Process (branching states, guarded transitions, transition triggers, workflow states) from Track (linear progression, final states, unguarded transitions)
- Relationship analyzer: discriminates Browse (OneToMany/ManyToMany collections, rich relationship graphs) from Focus (OneToOne+Inline, ManyToOne parent references)
- Action analyzer: discriminates Process (transition triggers, preconditions) from Collect (complex inputs >2) and Browse (simple CRUD)
- Full 5-analyzer pipeline integration verified: order management scenario correctly derives Process as primary intent
- 15 new tests (4 state machine + 5 relationship + 5 action + 1 full pipeline integration)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add state machine, relationship, and action signal analyzers** - `db31d4f` (feat)
2. **Task 2: Unit tests for state machine, relationship, and action analyzers** - `3905385` (test)

## Files Created/Modified
- `ferro-projections/src/derive.rs` - Three new analyzer functions, 14 new signal constants, 15 new tests, ActionDef import added to test module

## Decisions Made
- State machine analyzer produces signals for both Process and Track simultaneously, letting the aggregation and normalization pipeline determine the winner based on total weight. This avoids hard-coding a threshold.
- Guard density uses ratio (guarded/total) rather than absolute count, so a service with 1/1 guarded transitions gets the same proportional weight as 5/5.
- Action analyzer has intentional overlap with state machine analyzer for transition_trigger signals. Both analyzers can contribute Process weight from the same underlying data, which amplifies the Process signal when both state machine structure AND action patterns align.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Full 5-analyzer derivation pipeline is complete and tested
- 185 ferro-projections unit tests + 5 doc tests = 190 total
- Ready for Phase 90 (IntentGraph construction from derive_intents output)

---
*Phase: 89-intent-graph-generation*
*Completed: 2026-03-01*

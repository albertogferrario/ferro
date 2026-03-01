---
phase: 94-protocol-patent
plan: 03
subsystem: docs
tags: [mdbook, protocol, data-model, specification]

requires:
  - phase: 94-01
    provides: mdBook project structure with placeholder pages and JSON Schema files
provides:
  - Complete data model specification across 7 pages (README + 6 type pages)
  - All 22 public types documented with JSON examples and normative rules
affects: [94-05]

tech-stack:
  added: []
  patterns: [rfc-2119-conformance-language, normative-vs-informative-split]

key-files:
  created: []
  modified:
    - docs/protocol/src/data-model/README.md
    - docs/protocol/src/data-model/service-def.md
    - docs/protocol/src/data-model/field-def.md
    - docs/protocol/src/data-model/state-machine.md
    - docs/protocol/src/data-model/actions.md
    - docs/protocol/src/data-model/relationships.md
    - docs/protocol/src/data-model/intent.md

key-decisions:
  - "Builder API documented as informative, not normative -- other implementations may use any construction mechanism"
  - "infer_meaning() documented as informative -- one possible algorithm, not protocol-mandated"
  - "default_navigation() mapping documented as informative -- implementations may choose different defaults"
  - "System fields (Identifier, CreatedAt, UpdatedAt) explicitly called out as excluded from intent derivation signal calculations"
  - "Schema-only constraint for state machines emphasized as critical design decision enabling serialization and introspection"
  - "Intent derivation signal types are normative (WHAT each analyzer considers); exact weights are informative (left to implementation)"
  - "Stable tie-breaking order documented as normative: Process > Track > Collect > Browse > Focus > Summarize > Analyze > Custom"

patterns-established:
  - "Each data model page follows: Type Definition table -> Normative Rules -> Serialization Rules -> JSON Example -> Schema link"
  - "Cross-references between types use relative links matching mdBook SUMMARY.md structure"

duration: 12min
completed: 2026-03-01
---

# Phase 94-03: Data Model Specification Summary

**Complete data model specification: all 22 public types documented across 7 pages with JSON examples and normative rules**

## Performance

- **Duration:** 12 min
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments
- Data model overview (README.md) with type hierarchy, serialization conventions, and canonical definition policy
- ServiceDef documented as protocol root type with all 9 fields and complete JSON example
- FieldDef, DataType (10 variants), and FieldMeaning (18 known + Custom) fully specified with rendering guidance
- StateMachine, StateDef, and Transition documented with schema-only constraint and validation rules
- ActionDef, InputDef, and GuardDef specified with shared guard pool and single type vocabulary
- RelationshipDef with two-dimensional design, Cardinality (4 variants) with default navigation, NavigationHint (5 variants)
- Intent (7+1 variants), IntentScore with confidence and tie-breaking, IntentHint with Primary/Exclude overrides

## Task Commits

Each task was committed atomically:

1. **Task 1: data-model README, service-def.md, field-def.md** - `b5d4b7d` (docs)
2. **Task 2: state-machine.md, actions.md** - `c2e9c28` (docs, committed alongside 94-02 summary by parallel agent)
3. **Task 3: relationships.md, intent.md** - `5610cf2` (docs)

## Files Modified
- `docs/protocol/src/data-model/README.md` - Type hierarchy overview, serialization conventions
- `docs/protocol/src/data-model/service-def.md` - ServiceDef root type specification
- `docs/protocol/src/data-model/field-def.md` - FieldDef, DataType, FieldMeaning specification
- `docs/protocol/src/data-model/state-machine.md` - StateMachine, StateDef, Transition specification
- `docs/protocol/src/data-model/actions.md` - ActionDef, InputDef, GuardDef specification
- `docs/protocol/src/data-model/relationships.md` - RelationshipDef, Cardinality, NavigationHint specification
- `docs/protocol/src/data-model/intent.md` - Intent, IntentScore, IntentHint specification

## Decisions Made
- Builder APIs, infer_meaning(), and default_navigation() are all marked informative (not normative) to allow alternative implementations.
- System fields are explicitly documented as excluded from intent signal calculations.
- The normative/informative split follows the pattern established in Plan 94-04: signal types (WHAT) are normative, exact weights (HOW MUCH) are informative.
- Stable tie-breaking order is normative to ensure consistent primary intent selection across implementations.

## Deviations from Plan

### Auto-fixed Issues

**1. Task 2 files committed by parallel agent**
- **Found during:** Task 2 commit
- **Issue:** Files written to disk by this agent were staged and committed by a concurrent 94-02 agent in commit c2e9c28 (plan summary commit)
- **Fix:** Content is correct and committed. Task 2 was effectively completed, just not in a standalone commit.
- **Impact on plan:** None. Content is identical to what was planned.

---

**Total deviations:** 1 (commit attribution, no content impact)

## Issues Encountered
None

## User Setup Required
None

## Next Phase Readiness
- All 22 public types documented across the data model section
- Cross-references between pages are consistent
- JSON examples present for all major types
- Ready for Plan 94-05 (governance section) if planned

---
*Phase: 94-protocol-patent*
*Completed: 2026-03-01*

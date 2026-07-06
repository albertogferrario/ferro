---
phase: 94-protocol-patent
plan: 04
subsystem: docs
tags: [protocol, specification, derivation, rendering, validation, rfc2119]

requires:
  - phase: 94-01
    provides: mdBook project structure and JSON Schema generation
provides:
  - Intent derivation specification with 5-analyzer signal-to-intent mappings
  - Rendering specification with Renderer trait contract and informative layout tables
  - Validation specification with all Error/Warning variants and BFS reachability
affects: [94-05, 94-06]

tech-stack:
  added: []
  patterns: [normative-vs-informative-spec-sections, rfc2119-conformance-language]

key-files:
  created: []
  modified:
    - docs/protocol/src/derivation.md
    - docs/protocol/src/rendering.md
    - docs/protocol/src/validation.md

key-decisions:
  - "Signal types (WHAT each analyzer examines) are normative; exact weights are informative -- allows alternative implementations while preserving interoperability"
  - "Rendering spec is renderer-agnostic: Renderer trait is normative, intent-to-layout mapping is informative"
  - "BFS reachability specified as MUST with explicit algorithm pseudocode"
  - "Validation order recommendation: fatal errors before warnings for useful error messages"

patterns-established:
  - "Normative/informative distinction: normative sections use MUST/SHOULD, informative sections use explicit Note callouts"
  - "Signal mapping tables: consistent format across all analyzer specs (Condition | Target Intent | Rationale)"

duration: 6min
completed: 2026-03-01
---

# Phase 94-04: Derivation, Rendering, and Validation Specification Summary

**Derivation rules for 5 analyzers with normative signal types, renderer-agnostic rendering contract, and validation rules covering all Error/Warning variants with BFS reachability**

## Performance

- **Duration:** 6 min
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Intent derivation specification documents all 5 analyzers (field meaning, writability, state machine, relationship, action) with normative signal-to-intent mappings while keeping weights informative
- Rendering specification defines the Renderer trait contract, RenderContext, RenderMode, and includes informative reference tables for intent-to-layout, field-to-component, and relationship-to-component mappings
- Validation specification covers all 4 Error variants and 9 Warning variants with explicit BFS reachability algorithm and reference resolution rules

## Task Commits

Each task was committed atomically:

1. **Task 1: Write derivation.md** - `98bab85` (docs)
2. **Task 2: Write rendering.md and validation.md** - `01bc85c` (docs)

## Files Created/Modified
- `docs/protocol/src/derivation.md` - Intent derivation specification (146 lines)
- `docs/protocol/src/rendering.md` - Rendering specification (135 lines)
- `docs/protocol/src/validation.md` - Validation specification (188 lines)

## Decisions Made
- Signal types are normative (MUST implement these signal-to-intent mappings) while weights are informative (implementations MAY use any weighting strategy). This distinction allows alternative implementations to remain conformant while tuning behavior differently.
- Rendering spec separates the normative Renderer trait from informative reference mappings. Different renderers (JSON-UI, A2UI, HTML) can map intents to different UI patterns.
- BFS reachability is specified with explicit algorithm pseudocode to ensure implementations detect unreachable states consistently.
- Validation order is recommended (fatal before warnings) but not mandated, to allow implementation flexibility.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Three core behavior specification pages complete (derivation, rendering, validation)
- Combined with Plan 02 (architecture) and Plan 03 (data model pages), the protocol spec's technical content is largely complete
- Remaining plans can address governance pages (conformance, extensions, security) and appendices (examples, changelog)

---
*Phase: 94-protocol-patent*
*Completed: 2026-03-01*

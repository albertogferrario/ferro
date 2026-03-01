---
phase: 89-intent-graph-generation
plan: 01
subsystem: projections
tags: [intent-derivation, signal-analysis, normalization, confidence-scoring]

requires:
  - phase: 88-intent-core-types
    provides: Intent, IntentScore, IntentHint types and ServiceDef.intent_hints
provides:
  - derive_intents() public API for structural intent derivation
  - Field meaning analyzer (Money/Percentage/Quantity -> Summarize, FreeText/ImageUrl/Url -> Focus, EntityName/Category -> Browse, DateTime+numeric -> Analyze, Status -> Track)
  - Writability analyzer (writable ratio -> Collect, read-only ratio -> Summarize)
  - Signal aggregation and normalization pipeline
  - IntentHint override (Primary/Exclude) application
  - Default fallback (Focus 0.5 when no structural signals)
affects: [89-02-intent-graph-generation, 90-intent-graph, 91-renderer]

tech-stack:
  added: []
  patterns: [signal-based scoring, proportional count-weighted signals, max-normalization to [0,1]]

key-files:
  created: [ferro-projections/src/derive.rs]
  modified: [ferro-projections/src/lib.rs]

key-decisions:
  - "Proportional signals (0.3 * count) not binary presence for field meanings"
  - "Browse and Focus get 0.1 baseline to always appear in results"
  - "Tie-breaking by stable priority ordering: Process > Track > Collect > Browse > Focus > Summarize > Analyze"
  - "format_signal_with_sources joins active signal names with + separator"

patterns-established:
  - "Signal type alias: (Intent, f64, String) for analyzer outputs"
  - "Analyzer functions take &ServiceDef, return Vec<Signal>"
  - "is_system_field() excludes Identifier/CreatedAt/UpdatedAt from domain analysis"
  - "Signal name constants as module-level &str to prevent typo bugs"

duration: 12min
completed: 2026-03-01
---

# Phase 89 Plan 01: Intent Derivation Engine Summary

**derive_intents() with field meaning and writability analyzers, aggregation pipeline, normalization, and IntentHint override**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- derive_intents() public API that always returns at least one IntentScore
- Field meaning analyzer: 6 signal rules mapping FieldMeaning variants to intents with count-proportional weights
- Writability analyzer: 4 signal rules based on readable/writable ratios
- Aggregation pipeline: sum weights per intent, collect signal names
- Normalizer: max-normalization to [0.0, 1.0], sorted descending with stable tie-breaking
- IntentHint override: Exclude removes intents, Primary forces position 0 with confidence 1.0
- 27 new tests covering each component in isolation plus integration

## Task Commits

Each task was committed atomically:

1. **Task 1: Create derive.rs with signal analyzers and public API** - `c557dc8` (feat)
2. **Task 2: Unit tests for analyzers, normalizer, hints, fallback** - `c758780` (test)

## Files Created/Modified
- `ferro-projections/src/derive.rs` - Derivation engine: analyzers, aggregation, normalization, hints, public API, 27 tests
- `ferro-projections/src/lib.rs` - Added mod derive, exported derive_intents

## Decisions Made
- Proportional signal weights (0.3 * count for Summarize fields, 0.25 * count for Focus fields) rather than binary presence detection. This scales naturally with field count.
- Browse and Focus receive 0.1 baseline scores to ensure they always appear in results even without specific field signals.
- Stable tie-breaking priority order (Process > Track > Collect > Browse > Focus > Summarize > Analyze) ensures deterministic output.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- derive_intents() is ready for Plan 02 to add the remaining three analyzers (relationship, state machine, action)
- The aggregation pipeline is designed to accept additional Signal vectors from new analyzers
- 170 unit tests + 5 doc tests = 175 total

---
*Phase: 89-intent-graph-generation*
*Completed: 2026-03-01*

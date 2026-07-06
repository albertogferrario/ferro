---
phase: 88-intent-core-types
plan: 01
subsystem: api
tags: [serde, schemars, intent, service-projections]

# Dependency graph
requires:
  - phase: 87-service-relationships
    provides: ServiceDef with relationships, Warning enum, validation framework
provides:
  - Intent enum with 7 structurally-derivable variants + Custom fallback
  - IntentScore for confidence-scored derivation results
  - IntentHint for manual override of structural analysis
  - ServiceDef.intent_hints field with builder and validation
  - Warning::ConflictingIntentHints and Warning::MultiplePrimaryIntentHints
affects: [89-intent-derivation, 90-intent-renderer, 91-protocol]

# Tech tracking
tech-stack:
  added: []
  patterns: [Intent untagged Custom fallback matching FieldMeaning pattern, IntentHint externally tagged enum]

key-files:
  created: [ferro-projections/src/intent.rs]
  modified: [ferro-projections/src/lib.rs, ferro-projections/src/service.rs, ferro-projections/src/state.rs]

key-decisions:
  - "Intent follows FieldMeaning pattern: serde rename_all snake_case + untagged Custom(String) as last variant"
  - "IntentScore uses f64 confidence (NOT Eq) matching ServiceDef pattern"
  - "IntentHint uses externally tagged serde (default) for clear JSON: {primary: browse} or {exclude: process}"
  - "Intent hint conflict detection serializes intent to string via serde_json to avoid coupling Warning to Intent types"

patterns-established:
  - "Intent vocabulary: Browse, Focus, Collect, Process, Summarize, Analyze, Track + Custom"
  - "Manual override pattern: IntentHint::Primary/Exclude for when structural analysis is wrong"

# Metrics
duration: 4min
completed: 2026-02-28
---

# Phase 88: Intent Core Types — Plan 01 Summary

**Intent, IntentScore, IntentHint types with ServiceDef integration and validation warnings for conflicting/multiple primary hints**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-28T23:08:25Z
- **Completed:** 2026-02-28T23:12:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Intent enum with 7 structurally-derivable classification variants + Custom escape hatch
- IntentScore struct with confidence (f64) and matching_signals for Phase 89 output
- IntentHint enum (Primary/Exclude) for manual override of structural analysis
- ServiceDef.intent_hints field with builder, serde skip-if-empty, and validation
- Two new Warning variants: ConflictingIntentHints and MultiplePrimaryIntentHints
- 25 new tests (15 intent + 10 service integration), 143 total (138 unit + 5 doc)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create intent.rs with Intent, IntentScore, IntentHint types** - `875799c` (feat)
2. **Task 2: Integrate IntentHint into ServiceDef with validation** - `b4f8077` (feat)

## Files Created/Modified
- `ferro-projections/src/intent.rs` - Intent, IntentScore, IntentHint types with 15 tests
- `ferro-projections/src/lib.rs` - Module declaration and re-exports
- `ferro-projections/src/service.rs` - intent_hints field, builder, validation, 10 tests
- `ferro-projections/src/state.rs` - ConflictingIntentHints and MultiplePrimaryIntentHints warning variants

## Decisions Made
- Intent follows FieldMeaning pattern exactly: `#[serde(rename_all = "snake_case")]` with `#[serde(untagged)]` on Custom(String) as last variant
- IntentScore does NOT derive Eq (f64 doesn't implement Eq), matching ServiceDef pattern
- IntentHint uses default externally tagged serde for clear JSON structure
- Conflict detection serializes Intent to string via serde_json for Warning decoupling
- No Display impl added to Warning (no existing pattern to match)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed unused `Intent` import from service.rs**
- **Found during:** Task 2 (ServiceDef integration)
- **Issue:** `Intent` was imported but only `IntentHint` was needed directly (pattern matching destructures IntentHint variants)
- **Fix:** Removed `Intent` from the import, kept only `IntentHint`
- **Files modified:** ferro-projections/src/service.rs
- **Verification:** `cargo clippy -D warnings` passes clean
- **Committed in:** b4f8077 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Trivial import cleanup. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Intent vocabulary established for Phase 89 structural analysis engine
- IntentScore ready for Phase 89 to produce scored derivation results
- IntentHint override mechanism ready for ServiceDef consumers
- All types re-exported from crate root and JSON Schema enabled

---
*Phase: 88-intent-core-types*
*Completed: 2026-02-28*

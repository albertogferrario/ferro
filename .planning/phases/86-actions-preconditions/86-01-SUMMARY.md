---
phase: 86-actions-preconditions
plan: 01
subsystem: api
tags: [serde, schemars, builder-pattern, service-projections]

# Dependency graph
requires:
  - phase: 85.1-01
    provides: schemars JsonSchema derivation on projection types
  - phase: 84-01
    provides: FieldDef, DataType, FieldMeaning, ServiceDef core types
provides:
  - ActionDef — business operation schema with inputs, preconditions, effects, transition trigger
  - InputDef — action parameter reusing DataType/FieldMeaning
  - GuardDef — named boolean condition
  - FieldDef readable/writable booleans for field access mode
  - ServiceDef read_only_field/write_only_field convenience builders
affects: [87-relationships, 88-intent-graph, 89-intent-derivation]

# Tech tracking
tech-stack:
  added: []
  patterns: [skip_serializing_if for Vec::is_empty, default_true serde helper reuse]

key-files:
  created: [ferro-projections/src/action.rs]
  modified: [ferro-projections/src/field.rs, ferro-projections/src/service.rs, ferro-projections/src/lib.rs]

key-decisions:
  - "InputDef reuses DataType/FieldMeaning from field.rs — single type vocabulary, no parallel type systems"
  - "readable/writable default to true — backward-compatible with all existing Phase 84/85 JSON"
  - "GuardDef is minimal (name + display_name + description) — evaluation logic lives outside the schema"

patterns-established:
  - "Vec fields use #[serde(default, skip_serializing_if = \"Vec::is_empty\")] for clean JSON"
  - "read_only_field/write_only_field convenience builders on ServiceDef for common access patterns"

# Metrics
duration: 8min
completed: 2026-02-28
---

# Phase 86, Plan 01: Actions & Preconditions — Type Definitions

**ActionDef/InputDef/GuardDef types with builder APIs, FieldDef readable/writable booleans, ServiceDef access-mode builders**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- ActionDef type with name, inputs, preconditions, effects, and transition_trigger fields
- InputDef reusing DataType/FieldMeaning from field module — single type vocabulary
- GuardDef as minimal named boolean condition schema
- FieldDef extended with readable/writable booleans (default true, backward-compatible)
- ServiceDef gains read_only_field() and write_only_field() convenience builders
- All types derive Serialize, Deserialize, JsonSchema, PartialEq, Eq
- 78 total tests (74 unit + 4 doctests), up from 55

## Task Commits

Each task was committed atomically:

1. **Task 1: Create action.rs with ActionDef, InputDef, GuardDef** - `8596075` (feat)
2. **Task 2: Add readable/writable to FieldDef + ServiceDef convenience builders** - `33ef185` (feat)

**Formatting fix:** `0122d39` (style: apply rustfmt)

## Files Created/Modified
- `ferro-projections/src/action.rs` - ActionDef, InputDef, GuardDef types with builder APIs and tests
- `ferro-projections/src/field.rs` - FieldDef readable/writable fields with serde defaults
- `ferro-projections/src/service.rs` - read_only_field/write_only_field convenience builders with tests
- `ferro-projections/src/lib.rs` - mod action + re-exports for ActionDef, InputDef, GuardDef

## Decisions Made
- InputDef reuses DataType and FieldMeaning from field.rs rather than creating parallel types — maintains single type vocabulary
- readable/writable default to true via existing default_true() helper — all prior JSON remains valid
- GuardDef kept minimal (name, display_name, description) — guard evaluation logic is external to the schema

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- ActionDef, InputDef, GuardDef available for Phase 87 (relationships) and Phase 88 (intent graph)
- FieldDef readable/writable ready for Phase 89 intent derivation heuristics
- All types are serializable and have JSON Schema support for MCP introspection

---
*Phase: 86-actions-preconditions*
*Completed: 2026-02-28*

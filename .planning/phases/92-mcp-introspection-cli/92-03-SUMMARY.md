---
phase: 92-mcp-introspection-cli
plan: 03
subsystem: api
tags: [mcp, projections, coverage, intent-derivation]

# Dependency graph
requires:
  - phase: 92-mcp-introspection-cli
    provides: reconstruct_service_def pub(crate), list_projections, list_models tools
  - phase: 89-intent-graph-generation
    provides: derive_intents() function
provides:
  - projection_coverage MCP tool for model/projection cross-referencing
  - Coverage gap detection with CLI scaffolding suggestions
affects: [93-field-test-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: [model-projection-cross-referencing, coverage-gap-detection]

key-files:
  created:
    - ferro-mcp/src/tools/projection_coverage.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs

key-decisions:
  - "Case-insensitive service_name matching for model-projection cross-referencing"
  - "derive_intents via reconstruct_service_def for covered projections (reuses Plan 02 pub(crate) API)"
  - "Suggestion format: ferro make:projection {snake_name} --from-model"

patterns-established:
  - "Coverage report pattern: cross-reference two discovery tools and report gaps with actionable suggestions"

# Metrics
duration: 8min
completed: 2026-03-01
---

# Phase 92 Plan 03: Service Coverage MCP Tool Summary

**`projection_coverage` MCP tool cross-references models with projections, reports coverage percentage, derives primary intents, and suggests CLI commands for uncovered models**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-01T03:06:00Z
- **Completed:** 2026-03-01T03:14:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `projection_coverage` MCP tool registered and discoverable
- Cross-references models (via list_models) with projections (via list_projections) using case-insensitive service_name matching
- Derives primary intent and confidence for covered projections via reconstruct_service_def + derive_intents
- Generates `ferro make:projection {snake_name} --from-model` suggestions for uncovered models
- Coverage percentage computed as (with_projections / total_models) * 100
- 5 new tests: serialization, empty project, suggestion format, snake_case conversion, coverage percentages

## Task Commits

Each task was committed atomically:

1. **Task 1: Add projection_coverage MCP tool** - `9016a37` (feat)
2. **Task 2: Update ROADMAP and STATE for Phase 92 completion** - `203cfb7` (docs)

## Files Created/Modified
- `ferro-mcp/src/tools/projection_coverage.rs` - Coverage report tool: CoverageReport, ModelCoverage, CoverageSummary types and execute function
- `ferro-mcp/src/tools/mod.rs` - Register projection_coverage module
- `ferro-mcp/src/service.rs` - Add ProjectionCoverageParams + tool handler registration

## Decisions Made
- Case-insensitive matching between model name and projection service_name (e.g., "User" matches service_name "user")
- PascalCase to snake_case conversion for suggestion generation
- Primary intent derivation reuses existing reconstruct_service_def + derive_intents pipeline; returns None on reconstruction failure rather than erroring

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pretty-printed JSON uses spaces around colons (`"key": value` not `"key":value`); test assertions adjusted to match

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 92 complete with all 3 plans shipped
- Ready for Phase 93 (Field Test & Polish)

---
*Phase: 92-mcp-introspection-cli*
*Completed: 2026-03-01*

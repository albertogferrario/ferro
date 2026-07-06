---
phase: 134-relocate-renderers-to-output-crates
plan: "01"
subsystem: projections
tags: [ferro-json-ui, ferro-projections, renderer, feature-flag]

# Dependency graph
requires:
  - phase: 133-generalize-renderer-trait
    provides: Generic Renderer trait with associated Output and Context types
provides:
  - JsonUiRenderer in ferro-json-ui behind `projections` feature flag
  - VisualContext and RenderMode importable from ferro_json_ui
  - is_system_field pub in ferro-projections for cross-crate use
  - Pattern: output crates own their Renderer implementation
affects: [135-relocate-renderers-to-output-crates, ferro-projections, ferro-json-ui, ferro-mcp]

# Tech tracking
tech-stack:
  added: [ferro-projections optional dep in ferro-json-ui, ferro-theme optional dep in ferro-json-ui]
  patterns: [Output crates own Renderer implementations; ferro-projections owns only trait and schema types; optional feature flag for cross-crate renderer]

key-files:
  created:
    - ferro-json-ui/src/projection/mod.rs
    - ferro-json-ui/src/projection/field_map.rs
    - ferro-json-ui/src/projection/relationship_map.rs
  modified:
    - ferro-json-ui/Cargo.toml
    - ferro-json-ui/src/lib.rs
    - ferro-projections/src/render/mod.rs

key-decisions:
  - "JsonUiRenderer relocates to ferro-json-ui under projections feature — ferro-projections retains only Renderer trait and schema types"
  - "is_system_field promoted from pub(crate) to pub for cross-crate use by the relocated renderer"
  - "projection/ submodule in ferro-json-ui owns field_map and relationship_map, using crate-root re-exports from ferro-projections"

patterns-established:
  - "Output crate pattern: each renderer lives in its output crate (ferro-json-ui for JSON-UI), not in ferro-projections"
  - "Feature flag gating: projections = [dep:ferro-projections, dep:ferro-theme] keeps the dependency optional"
  - "Cross-crate imports: use ferro_projections::{Type} (crate root re-exports) not private module paths"

requirements-completed: []

# Metrics
duration: ~15min
completed: 2026-04-14
---

# Phase 134 Plan 01: Relocate JsonUiRenderer to ferro-json-ui Summary

**JsonUiRenderer, VisualContext, RenderMode, field_map, and relationship_map relocated from ferro-projections into ferro-json-ui behind a `projections` feature flag, establishing the output-crate ownership pattern**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-14T23:20:00Z
- **Completed:** 2026-04-14T23:29:10Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Created `ferro-json-ui/src/projection/` with three files totalling 3265 lines of inserted code
- Wired `projections` feature flag in ferro-json-ui/Cargo.toml with optional ferro-projections and ferro-theme deps
- Made `is_system_field` pub in ferro-projections/src/render/mod.rs for cross-crate access
- All 567 unit tests and 6 doc tests pass under `cargo test -p ferro-json-ui --features projections`
- ferro-projections still compiles cleanly (`cargo build -p ferro-projections`)

## Task Commits

1. **Task 1: Wire Cargo feature flag and relocate files to ferro-json-ui** - `a127d40f` (feat)

## Files Created/Modified

- `ferro-json-ui/src/projection/mod.rs` - JsonUiRenderer, VisualContext, RenderMode with all intent layout strategies and tests
- `ferro-json-ui/src/projection/field_map.rs` - field_to_column, field_to_display, field_to_input with full test coverage
- `ferro-json-ui/src/projection/relationship_map.rs` - relationship_to_component with test coverage
- `ferro-json-ui/Cargo.toml` - Added projections feature flag and optional deps
- `ferro-json-ui/src/lib.rs` - Added pub mod projection and re-exports behind feature flag
- `ferro-projections/src/render/mod.rs` - is_system_field promoted to pub; doc comment updated

## Decisions Made

- Used crate-root re-exports (`ferro_projections::FieldMeaning`) instead of private module paths (`ferro_projections::field::FieldMeaning`) since ferro-projections modules are private
- Declared `field_map` and `relationship_map` as `pub mod` inside `projection/mod.rs` so they are accessible as projection submodules

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

Minor: The plan specified import paths using private module paths (e.g., `use ferro_projections::field::FieldMeaning`). At compile time these failed because ferro-projections declares its modules as `mod` (private). Fixed by using crate-root re-exports instead. This is consistent with the [110-01] decision documented in STATE.md: "All ferro imports use explicit crate-root exports".

## Next Phase Readiness

- Plan 02 can now update ferro-projections to remove the `visual` feature flag and `json_ui.rs` module (the source is now in ferro-json-ui)
- The output-crate pattern is established and verified working

## Self-Check: PASSED

- ferro-json-ui/src/projection/mod.rs: FOUND
- ferro-json-ui/src/projection/field_map.rs: FOUND
- ferro-json-ui/src/projection/relationship_map.rs: FOUND
- commit a127d40f: FOUND

---
*Phase: 134-relocate-renderers-to-output-crates*
*Completed: 2026-04-14*

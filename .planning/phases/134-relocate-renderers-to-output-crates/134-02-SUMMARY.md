---
phase: 134-relocate-renderers-to-output-crates
plan: "02"
subsystem: projections
tags: [ferro-projections, ferro-json-ui, ferro-mcp, renderer, refactor]

# Dependency graph
requires:
  - phase: 134-01
    provides: JsonUiRenderer/RenderMode/VisualContext relocated to ferro-json-ui behind projections feature
provides:
  - ferro-projections is renderer-free (no visual feature, no ferro-theme dep, no relocated files)
  - ferro-mcp imports JsonUiRenderer/RenderMode/VisualContext from ferro_json_ui
  - framework re-exports visual renderer types from ferro_json_ui when projections feature enabled
affects: [v12.0-json-ui-v2, ferro-projections consumers, ferro-mcp, framework]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Renderer trait stays in ferro-projections; concrete implementations live in output crates"
    - "Framework projections feature now pulls ferro-json-ui/projections for visual types"

key-files:
  created: []
  modified:
    - ferro-projections/src/lib.rs
    - ferro-projections/src/render/mod.rs
    - ferro-projections/Cargo.toml
    - ferro-mcp/Cargo.toml
    - ferro-mcp/src/tools/render_projection.rs
    - framework/Cargo.toml
    - framework/src/lib.rs

key-decisions:
  - "ferro-projections owns Renderer trait and BaseContext only - no concrete renderers"
  - "framework projections feature depends on ferro-json-ui/projections for visual types"

patterns-established:
  - "Renderer implementations belong in their output crate, never in ferro-projections"

requirements-completed: []

# Metrics
duration: 4min
completed: 2026-04-17
---

# Phase 134 Plan 02: Renderer Relocation Cleanup Summary

**ferro-projections stripped to renderer-trait-only: visual feature removed, three relocated files deleted, ferro-mcp and framework re-exports updated to ferro-json-ui.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-04-17T16:02:26Z
- **Completed:** 2026-04-17T16:06:32Z
- **Tasks:** 2
- **Files modified:** 8 (including 3 deleted)

## Accomplishments
- Removed `[features]` section and `ferro-theme` dependency from `ferro-projections/Cargo.toml`
- Deleted `ferro-projections/src/render/{field_map,json_ui,relationship_map}.rs` (all relocated in 134-01)
- Removed `#[cfg(feature = "visual")]` re-exports from `ferro-projections/src/lib.rs`
- Updated `ferro-projections/src/render/mod.rs` doc to reflect clean abstraction (no module refs to deleted files)
- Updated `ferro-mcp/Cargo.toml`: dropped `visual` feature, added `ferro-json-ui/projections`
- Updated `ferro-mcp/src/tools/render_projection.rs` import: `JsonUiRenderer`, `RenderMode`, `VisualContext` now from `ferro_json_ui`
- Full workspace passes `cargo fmt`, `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features`

## Task Commits

1. **Task 1: Clean ferro-projections and delete relocated files** - `1fe8107e` (refactor)
2. **Task 2: Update ferro-mcp imports and feature flags** - `18d7ddf9` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `ferro-projections/src/lib.rs` - Removed visual re-exports
- `ferro-projections/src/render/mod.rs` - Removed old module declarations, updated doc comment
- `ferro-projections/Cargo.toml` - Removed features section and ferro-theme dep
- `ferro-projections/src/render/field_map.rs` - Deleted
- `ferro-projections/src/render/json_ui.rs` - Deleted
- `ferro-projections/src/render/relationship_map.rs` - Deleted
- `ferro-mcp/Cargo.toml` - Dropped visual feature, added projections feature to ferro-json-ui
- `ferro-mcp/src/tools/render_projection.rs` - Split import to use ferro_json_ui for visual types
- `framework/Cargo.toml` - projections feature now depends on ferro-json-ui/projections
- `framework/src/lib.rs` - Re-exports visual types from ferro_json_ui under projections feature gate

## Decisions Made
- The `framework` crate's `projections` feature was updated to depend on `ferro-json-ui/projections` so downstream users still get `JsonUiRenderer`, `RenderMode`, `VisualContext` via the framework re-export surface without needing to import from ferro-json-ui directly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] framework/Cargo.toml referenced removed visual feature**
- **Found during:** Task 1 (cargo build -p ferro-projections)
- **Issue:** `framework/Cargo.toml` had `projections = ["dep:ferro-projections", "ferro-projections/visual"]` which failed after the visual feature was deleted
- **Fix:** Changed to `projections = ["dep:ferro-projections", "dep:ferro-json-ui", "ferro-json-ui/projections"]` and updated `framework/src/lib.rs` re-exports accordingly
- **Files modified:** framework/Cargo.toml, framework/src/lib.rs
- **Verification:** cargo clippy --all --all-targets passes clean
- **Committed in:** 18d7ddf9 (Task 2 commit)

**2. [Rule 3 - Blocking] ferro-mcp Cargo.toml visual feature blocked ferro-projections compilation**
- **Found during:** Task 1 (initial cargo build -p ferro-projections)
- **Issue:** ferro-mcp still had `features = ["visual"]` on its ferro-projections dep, blocking workspace-level build of ferro-projections
- **Fix:** Removed the visual feature from ferro-projections dep in ferro-mcp/Cargo.toml during Task 1 (full import update completed in Task 2)
- **Files modified:** ferro-mcp/Cargo.toml
- **Verification:** cargo build -p ferro-projections succeeds
- **Committed in:** 1fe8107e (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes required to satisfy the workspace compilation requirement. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 134 is complete: renderer relocation is done (134-01 moved files, 134-02 cleaned up references)
- ferro-projections is now a clean schema-only crate: `Renderer` trait + `BaseContext` + `ServiceDef` + domain types, no output dependencies
- v12.0 JSON-UI v2 phases can proceed with a clean ferro-projections that has no rendering coupling

---
*Phase: 134-relocate-renderers-to-output-crates*
*Completed: 2026-04-17*

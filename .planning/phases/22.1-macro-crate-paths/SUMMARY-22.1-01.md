---
phase: 22.1-macro-crate-paths
plan: 01
subsystem: macros
tags: [proc-macro, crate-resolution, dependency]

# Dependency graph
requires: []
provides:
  - ferro_crate() helper function for dynamic crate path resolution
  - proc-macro-crate dependency for reading user's Cargo.toml
affects: [22.1-02, 22.1-03]

# Tech tracking
tech-stack:
  added: [proc-macro-crate v3]
  patterns: [dynamic crate name resolution]

key-files:
  created: [ferro-macros/src/crate_path.rs]
  modified: [ferro-macros/Cargo.toml, ferro-macros/src/lib.rs]

key-decisions:
  - "Try 'ferro' first, fall back to 'ferro_rs' for backwards compatibility"
  - "Return TokenStream directly for easy integration with quote! macros"

patterns-established:
  - "crate_path module pattern for centralized crate name resolution"

# Metrics
duration: 1 min
completed: 2026-01-17
---

# Phase 22.1 Plan 01: Add proc-macro-crate Dependency and Helper Summary

**Added proc-macro-crate v3 dependency and ferro_crate() helper function for dynamic crate path resolution in proc macros**

## Performance

- **Duration:** 1 min
- **Started:** 2026-01-17T01:13:19Z
- **Completed:** 2026-01-17T01:14:33Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added proc-macro-crate v3 dependency for reading user's Cargo.toml at macro expansion time
- Created crate_path.rs module with ferro_crate() helper function
- Implemented fallback logic: tries "ferro" first, then "ferro_rs", then defaults to "::ferro_rs"
- Exported helper as pub(crate) for use by all macros in the crate

## Task Commits

Each task was committed atomically:

1. **Task 1: Add proc-macro-crate dependency** - `8d022b8` (chore)
2. **Task 2: Create crate path helper module** - `215afff` (feat)
3. **Task 3: Export the helper module** - `f8194aa` (feat)

## Files Created/Modified

- `ferro-macros/Cargo.toml` - Added proc-macro-crate = "3" dependency
- `ferro-macros/src/crate_path.rs` - New module with ferro_crate() function
- `ferro-macros/src/lib.rs` - Added module declaration and pub(crate) re-export

## Decisions Made

- **Fallback order:** Try "ferro" first (the crates.io published name), then "ferro_rs" for backwards compatibility
- **Return type:** TokenStream directly so it integrates seamlessly with quote! macros
- **Default:** If both lookups fail, default to `::ferro_rs` to maintain backwards compatibility

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - build completed successfully with expected warnings about unused code (will be used in plan 22.1-02).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ferro_crate() helper is ready for use
- Plan 22.1-02 will update all macros to use this helper
- Warnings about unused ferro_crate() expected until macros are updated

---
*Phase: 22.1-macro-crate-paths*
*Completed: 2026-01-17*

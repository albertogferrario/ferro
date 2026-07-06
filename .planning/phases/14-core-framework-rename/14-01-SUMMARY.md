---
phase: 14-core-framework-rename
plan: 01
subsystem: infra
tags: [cargo, rust, rebrand, ferro]

# Dependency graph
requires:
  - phase: 13-rebrand-audit
    provides: Audit and mapping documentation for rename
provides:
  - Framework crate renamed from cancer-rs to ferro
  - App dependency updated to reference ferro package
  - Macros updated to generate ferro:: imports
affects: [15-supporting-crates-rename, 16-cli-rebrand, 17-mcp-rebrand, 19-sample-app-migration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Alias pattern for gradual migration: `cancer = { path = \"../framework\", package = \"ferro\" }`"

key-files:
  created: []
  modified:
    - framework/Cargo.toml
    - app/Cargo.toml
    - cancer-macros/src/validate.rs
    - cancer-macros/src/request.rs
    - framework/tests/validation_derive.rs

key-decisions:
  - "Keep dependency alias as 'cancer' for code compatibility until Phase 19 imports migration"
  - "Update macro-generated code to use ferro:: imports (required for tests to pass)"

patterns-established:
  - "Package rename with alias: change package name but keep alias for gradual migration"

# Metrics
duration: 8 min
completed: 2026-01-16
---

# Phase 14 Plan 01: Core Framework Package Rename Summary

**Renamed main framework crate from cancer-rs to ferro, updated app dependency alias, and fixed macro-generated imports**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-16T00:40:00Z
- **Completed:** 2026-01-16T00:48:00Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Framework package renamed from `cancer-rs` to `ferro` in Cargo.toml
- App dependency updated to reference `ferro` package while keeping `cancer` alias
- Macro-generated code updated to use `ferro::` imports for test compatibility
- All lib tests and integration tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Rename framework package** - `8c60958` (refactor)
2. **Task 2: Update app dependency alias** - `3039175` (refactor)
3. **Task 3: Full workspace verification** - `037572c` (refactor)
   - Included macro and test file updates required for verification

## Files Created/Modified

- `framework/Cargo.toml` - Changed package name from cancer-rs to ferro
- `app/Cargo.toml` - Updated dependency reference to ferro package (kept cancer alias)
- `cancer-macros/src/validate.rs` - Updated generated code to use ferro:: imports
- `cancer-macros/src/request.rs` - Updated generated code to use ferro:: imports
- `framework/tests/validation_derive.rs` - Updated test imports to use ferro::

## Decisions Made

1. **Keep 'cancer' alias** - The app/Cargo.toml uses `cancer = { path = "../framework", package = "ferro" }` to allow existing `use cancer::` imports to continue working until Phase 19 migrates all imports.

2. **Update macro-generated code** - Macros in cancer-macros generate code that references the framework crate. These had to be updated to use `ferro::` instead of `cancer_rs::` for the generated code to compile correctly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated macro-generated imports to ferro::**

- **Found during:** Task 3 (Full workspace verification)
- **Issue:** cancer-macros generates code with `cancer_rs::` imports which fail when the package is renamed to `ferro`
- **Fix:** Updated validate.rs and request.rs to generate `ferro::` instead of `cancer_rs::`
- **Files modified:** cancer-macros/src/validate.rs, cancer-macros/src/request.rs
- **Verification:** `cargo test -p ferro --test validation_derive` passes
- **Committed in:** 037572c

**2. [Rule 3 - Blocking] Updated integration test imports**

- **Found during:** Task 3 (Full workspace verification)
- **Issue:** framework/tests/validation_derive.rs used `use cancer_rs::` imports
- **Fix:** Changed to `use ferro::` imports
- **Files modified:** framework/tests/validation_derive.rs
- **Verification:** Integration tests pass
- **Committed in:** 037572c

---

**Total deviations:** 2 auto-fixed (both blocking issues)
**Impact on plan:** Required for verification to pass. No scope creep - macros must generate valid code.

## Issues Encountered

- **Doc tests still use cancer_rs imports** - Doc tests in framework source files (e.g., config/mod.rs) still reference `cancer_rs` in their examples. These will be updated in Phase 18 (Documentation Update). For now, lib tests and integration tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Core framework package renamed to `ferro`
- App can import via `cancer` alias (backwards compatible)
- Ready for Phase 15: Supporting Crates Rename (cancer-* to ferro-*)

---
*Phase: 14-core-framework-rename*
*Completed: 2026-01-16*

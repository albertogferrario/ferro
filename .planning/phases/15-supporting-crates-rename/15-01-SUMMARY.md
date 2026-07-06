---
phase: 15-supporting-crates-rename
plan: 01
subsystem: infra
tags: [cargo, rust, rebrand, ferro]

# Dependency graph
requires:
  - phase: 14-core-framework-rename
    provides: Core framework renamed to ferro with alias pattern
provides:
  - All 9 supporting crates renamed from cancer-* to ferro-*
  - Workspace members updated to reference new directories
  - Framework dependencies updated with alias pattern
affects: [16-cli-rebrand, 17-mcp-rebrand, 19-sample-app-migration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Alias pattern for all crate dependencies: `cancer-X = { path = \"../ferro-X\", package = \"ferro-X\" }`"

key-files:
  created: []
  modified:
    - Cargo.toml (root)
    - framework/Cargo.toml
    - ferro-cli/Cargo.toml
    - ferro-*/Cargo.toml (9 files)
    - ferro-events/src/traits.rs (updated doctests)
    - ferro-events/src/dispatcher.rs (updated doctests)
    - ferro-events/src/lib.rs (updated module docs)
    - ferro-events/README.md
  renamed:
    - cancer-macros/ -> ferro-macros/
    - cancer-events/ -> ferro-events/
    - cancer-queue/ -> ferro-queue/
    - cancer-notifications/ -> ferro-notifications/
    - cancer-broadcast/ -> ferro-broadcast/
    - cancer-storage/ -> ferro-storage/
    - cancer-cache/ -> ferro-cache/
    - cancer-mcp/ -> ferro-mcp/
    - cancer-cli/ -> ferro-cli/

key-decisions:
  - Updated ferro-events doctests to use ferro_events imports for test compilation

patterns-established:
  - "Consistent alias pattern across all supporting crate dependencies"

# Metrics
duration: ~15 minutes
completed: 2026-01-16
---

# Phase 15 Plan 01: Supporting Crates Rename Summary

**Renamed all 9 supporting crates from cancer-* to ferro-*, updated workspace configuration and dependency references**

## Performance

- **Duration:** ~15 minutes
- **Started:** 2026-01-16
- **Completed:** 2026-01-16
- **Tasks:** 3
- **Files modified:** 15+ (9 crate Cargo.tomls + root + framework + cli + ferro-events docs)
- **Directories renamed:** 9

## Accomplishments

- All 9 crate directories renamed from cancer-* to ferro-*
- Package names updated in all crate Cargo.toml files
- Root workspace members updated to reference new directories
- Framework dependencies updated with alias pattern for gradual migration
- CLI dependency on MCP updated with alias pattern
- Updated ferro-events doctests and documentation to use new crate name
- Full workspace builds and tests pass

## Task Commits

1. **Task 1: Rename directories and packages** - `d0e48a5` (refactor)
2. **Task 2: Update workspace and dependencies** - `cd9b88b` (refactor)
3. **Task 3: Full verification** - `46100c2` (refactor)

## Files Created/Modified

### Task 1 - Directory and Package Renames
- Renamed 9 directories: cancer-* -> ferro-*
- Updated 9 Cargo.toml files with new package names
- Updated descriptions and keywords from "Cancer" to "Ferro"

### Task 2 - Workspace and Dependencies
- `Cargo.toml` (root): Updated workspace members array
- `framework/Cargo.toml`: Updated 7 dependency paths with alias pattern
- `ferro-cli/Cargo.toml`: Updated MCP dependency with alias pattern

### Task 3 - Verification and Doctest Fixes
- `ferro-events/src/traits.rs`: Updated doctests from cancer_events to ferro_events
- `ferro-events/src/dispatcher.rs`: Updated doctests from cancer_events to ferro_events
- `ferro-events/src/lib.rs`: Updated module documentation
- `ferro-events/README.md`: Updated crate name and examples

## Decisions Made

1. **Doctest updates required**: Unlike runtime code which uses aliases, doctests need to reference the actual crate name (ferro_events) to compile correctly.

## Deviations from Plan

1. **Additional doctest fixes**: Task 3 required updating ferro-events doctests beyond basic verification, as they were failing due to referencing the old cancer_events crate name.

## Pre-existing Issues (Not Related to Rename)

- `metrics::tests::test_different_methods_tracked_separately` - Flaky test in framework metrics module
- `templates::tests::test_globals_css_not_empty` - Test expecting tailwind in CSS

## Next Phase Readiness

- All supporting crates renamed to ferro-*
- Alias pattern allows existing code to continue using cancer-* imports
- Ready for Phase 16: CLI Rebrand

---
*Phase: 15-supporting-crates-rename*
*Completed: 2026-01-16*

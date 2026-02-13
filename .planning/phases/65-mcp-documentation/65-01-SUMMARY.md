---
phase: 65-mcp-documentation
plan: 01
subsystem: mcp
tags: [ferro-mcp, localization, introspection, translation, coverage]

# Dependency graph
requires:
  - phase: 64-cli-scaffolding
    provides: make:lang command, ferro new templates with localization defaults
provides:
  - list_lang_files MCP tool for locale/key/coverage introspection
  - Updated list_commands with make:lang
  - Updated application_info with localization status
  - Updated MCP instructions with localization workflow
affects: [66-tests-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: [env-config-parsing, json-key-flattening-for-mcp]

key-files:
  created:
    - ferro-mcp/src/tools/list_lang_files.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/list_commands.rs
    - ferro-mcp/src/tools/application_info.rs

key-decisions:
  - "Lightweight .env parsing (line-by-line split) instead of adding a dependency"
  - "Coverage report compares each locale against fallback locale only"
  - "LocalizationStatus added as nested struct in FeatureSummary"

patterns-established:
  - "Pattern: MCP localization introspection mirrors ferro-lang loader behavior"

# Metrics
duration: 5min
completed: 2026-02-13
---

# Phase 65 Plan 01: MCP Localization Introspection Summary

**New list_lang_files MCP tool for locale discovery, key inspection, and translation coverage reporting; updated list_commands, application_info, and MCP instructions with localization workflow**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-13T19:17:44Z
- **Completed:** 2026-02-13T19:22:47Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Created `list_lang_files` MCP tool that scans lang/ directory, reports locales with file names and key counts, and generates a coverage report of missing keys per locale vs fallback
- Added `make:lang` to `list_commands` CLI registry
- Added `LocalizationStatus` to `application_info` feature summary with locale count and hints
- Updated `FERRO_MCP_INSTRUCTIONS` with localization feature mention, "Adding Localization" workflow, `list_lang_files` proactive guidance, and Localization tool category

## Task Commits

Each task was committed atomically:

1. **Task 1: Create list_lang_files MCP tool** - `3ae4e1f` (feat)
2. **Task 2: Update list_commands, application_info, and MCP instructions** - `f0a560f` (feat)

## Files Created/Modified
- `ferro-mcp/src/tools/list_lang_files.rs` - New MCP tool: scans lang/ for locales, keys, files, and coverage
- `ferro-mcp/src/tools/mod.rs` - Added `list_lang_files` module declaration
- `ferro-mcp/src/service.rs` - Added ListLangFilesParams, tool handler, and MCP instructions updates
- `ferro-mcp/src/tools/list_commands.rs` - Added make:lang command entry
- `ferro-mcp/src/tools/application_info.rs` - Added LocalizationStatus struct and scan_localization function

## Decisions Made
- Used lightweight .env parsing (line-by-line split on `=`) consistent with other MCP tools, avoiding new dependencies
- Coverage report compares non-fallback locales against the fallback locale's key set
- Added `LocalizationStatus` as a nested struct inside `FeatureSummary` rather than a top-level field

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- MCP localization introspection complete
- Ready for Phase 66 (Tests & Polish)

---
*Phase: 65-mcp-documentation*
*Completed: 2026-02-13*

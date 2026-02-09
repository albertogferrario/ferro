---
phase: 31-mcp-ui-tools
plan: 03
subsystem: mcp
tags: [mcp, json-ui, introspection, code-generation]

requires:
  - phase: 31-01
    provides: json_ui_catalog tool
  - phase: 31-02
    provides: json_ui_inspect and json_ui_generate tools

provides:
  - FERRO_MCP_INSTRUCTIONS with JSON-UI tool documentation
  - generation_context with JSON-UI naming, patterns, imports
  - application_info with JSON-UI view count reporting

affects: [32-documentation]

tech-stack:
  added: []
  patterns:
    - "MCP instructions as discovery surface for JSON-UI tools"
    - "application_info as JSON-UI capability detector"

key-files:
  created: []
  modified:
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/generation_context.rs
    - ferro-mcp/src/tools/application_info.rs

key-decisions:
  - "Added JSON-UI as separate tool category AND in Code Generation category for dual discovery"
  - "JsonUiViewsStatus counts .rs files in src/views/ excluding mod.rs"

patterns-established:
  - "JSON-UI tools discoverable via both workflows and when-to-use sections"
  - "application_info reports feature availability with hints for missing features"

duration: 3min
completed: 2026-02-09
---

# Phase 31 Plan 03: MCP Instructions Update Summary

**Updated FERRO_MCP_INSTRUCTIONS, generation_context, and application_info to make JSON-UI tools discoverable by AI agents**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T09:53:57Z
- **Completed:** 2026-02-09T09:57:21Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- FERRO_MCP_INSTRUCTIONS includes JSON-UI in About section, two new workflows, when-to-use for 3 tools, and new tool category
- generation_context returns JSON-UI naming conventions, file structure, patterns, imports, and anti-patterns
- application_info reports JSON-UI view count by scanning src/views/ directory

## Task Commits

Each task was committed atomically:

1. **Task 1: Update FERRO_MCP_INSTRUCTIONS** - `69be299` (feat)
2. **Task 2: Update generation_context and application_info** - `a2ea80c` (feat)

## Files Created/Modified
- `ferro-mcp/src/service.rs` - Added JSON-UI to FERRO_MCP_INSTRUCTIONS: About section, workflows, when-to-use, tool categories
- `ferro-mcp/src/tools/generation_context.rs` - Added views field to NamingConventions/FileStructure/CommonPatterns/ImportTemplates, added anti-patterns, updated tests
- `ferro-mcp/src/tools/application_info.rs` - Added JsonUiViewsStatus struct and src/views/ scanning

## Decisions Made
- Added JSON-UI tools to both "Code Generation" and a separate "JSON-UI" category for maximum discoverability
- JsonUiViewsStatus counts .rs files in src/views/ excluding mod.rs to avoid double-counting

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 31 (MCP UI Tools) complete: all 3 plans finished
- Ready for Phase 32 (Documentation)

---
*Phase: 31-mcp-ui-tools*
*Completed: 2026-02-09*

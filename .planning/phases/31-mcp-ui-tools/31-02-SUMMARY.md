---
phase: 31-mcp-ui-tools
plan: 02
subsystem: mcp
tags: [mcp, json-ui, introspection, code-generation, regex, walkdir]

# Dependency graph
requires:
  - phase: 30
    provides: CLI AI view generation (COMPONENT_CATALOG, scan_models, scan_routes)
  - phase: 31-01
    provides: json_ui_catalog MCP tool
provides:
  - json_ui_inspect MCP tool for discovering existing views
  - json_ui_generate MCP tool for assembling view generation context
affects: [32-documentation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Regex-based source scanning for views (same pattern as CLI ai.rs)"
    - "Context assembly tool (provides data, agent writes code)"

key-files:
  created:
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs

key-decisions:
  - "Regex-based view scanning over syn AST for speed and simplicity"
  - "json_ui_generate returns context, does NOT call AI API"
  - "COMPONENT_CATALOG embedded as const string (same content as CLI)"
  - "Model scanning reimplemented in ferro-mcp (different crate, can't share)"

patterns-established:
  - "JSON-UI MCP tools follow existing tool pattern: execute() in module + #[tool] in service.rs"
  - "View inspection via regex matching pub fn signatures returning JsonUiView"

# Metrics
duration: 4min
completed: 2026-02-09
---

# Phase 31 Plan 02: JSON-UI Inspect & Generate MCP Tools Summary

**Two project-aware MCP tools for discovering existing JSON-UI views and assembling generation context from models/routes**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-09T09:45:53Z
- **Completed:** 2026-02-09T09:50:23Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- json_ui_inspect scans src/views/*.rs and extracts view metadata (name, title, layout, components, actions)
- json_ui_generate assembles component catalog + model fields + routes + example + conventions for agent-driven view creation
- Both tools handle missing directories gracefully (empty result, not error)
- 7 unit tests across both tools, all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create json_ui_inspect MCP tool** - `88b2a6b` (feat)
2. **Task 2: Create json_ui_generate MCP tool** - `9983fe5` (feat)

## Files Created/Modified
- `ferro-mcp/src/tools/json_ui_inspect.rs` - Scans src/views/ for JSON-UI view functions, extracts metadata
- `ferro-mcp/src/tools/json_ui_generate.rs` - Assembles context for view generation (catalog, models, routes, example, conventions)
- `ferro-mcp/src/tools/mod.rs` - Added module declarations
- `ferro-mcp/src/service.rs` - Registered both tools with params and descriptions

## Decisions Made
- Regex-based view scanning (not syn AST) for speed and simplicity, matching CLI pattern
- json_ui_generate provides structured context rather than calling AI, since the consuming agent IS the LLM
- COMPONENT_CATALOG duplicated as const string in generate module (different crate from CLI, cannot import)
- Model and route scanning reimplemented with regex (same approach as CLI but independent implementation)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- json_ui_inspect and json_ui_generate tools registered and functional
- Ready for Plan 03 (MCP instructions update and existing tool integration)

---
*Phase: 31-mcp-ui-tools*
*Completed: 2026-02-09*

---
phase: 83-api-dx-polish
plan: 02
subsystem: api
tags: [openapi, mcp, routing, x-mcp-extensions]

requires:
  - phase: 80-x-mcp-extensions
    provides: x-mcp OpenAPI extension format and ferro-api-mcp consumer support
provides:
  - Route-level x-mcp customization API (.mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden())
  - Group-level MCP defaults with child-override semantics
  - OpenAPI spec builder consumes explicit overrides with auto-generated fallback
affects: [83-05-local-verification, ferro-api-mcp, docs]

tech-stack:
  added: []
  patterns: [McpDefaults struct for group-to-child MCP propagation]

key-files:
  modified:
    - framework/src/routing/router.rs
    - framework/src/routing/macros.rs
    - framework/src/api/openapi.rs

key-decisions:
  - "RouteInfo derives Default for ergonomic test construction with ..Default::default()"
  - "McpDefaults internal struct propagates group MCP settings to children; child overrides take precedence"
  - "Hidden routes emit only x-mcp-hidden: true with no tool name or description"
  - "update_route_mcp remains pub(crate) since it is only used within the routing module"

patterns-established:
  - "MCP metadata propagation: group defaults merge with route overrides (child wins)"
  - "Hidden routes use continue-skip pattern at OpenAPI emission rather than post-filtering"

duration: 18min
completed: 2026-02-28
---

# Phase 83 Plan 02: x-MCP Route Customization API Summary

**Route-level .mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden() builder methods with OpenAPI spec consumption and 5 new tests**

## Performance

- **Duration:** 18 min
- **Started:** 2026-02-28
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- RouteInfo extended with mcp_tool_name, mcp_description, mcp_hint, mcp_hidden fields
- RouteDefBuilder and GroupDef support .mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden() builder methods
- Group-level MCP defaults propagate to child routes with child-override semantics
- OpenAPI spec builder respects explicit overrides, emits x-mcp-hidden for hidden routes, adds x-mcp-hint when present
- 5 new test cases covering override, hidden, hint, auto-generated fallback, and description override

## Task Commits

Each task was committed atomically:

1. **Task 1: Add x-mcp metadata to RouteInfo and route builders** - `fac552e` (feat)
2. **Task 2: Consume x-mcp metadata in OpenAPI spec builder** - `3f337e2` (feat)

## Files Created/Modified

- `framework/src/routing/router.rs` - RouteInfo with MCP fields, update_route_mcp(), Default derive
- `framework/src/routing/macros.rs` - MCP builder methods on RouteDefBuilder and GroupDef, McpDefaults propagation
- `framework/src/api/openapi.rs` - Spec builder consumes MCP overrides, 5 new tests

## Decisions Made

- RouteInfo derives Default for ergonomic test construction
- McpDefaults internal struct for group-to-child propagation (child overrides win)
- Hidden routes emit only x-mcp-hidden: true, no tool name or description
- update_route_mcp kept pub(crate) since only used within routing module

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Route-level MCP customization complete and tested
- Plan 05 (local verification) can validate end-to-end x-mcp flow

---
*Phase: 83-api-dx-polish*
*Completed: 2026-02-28*

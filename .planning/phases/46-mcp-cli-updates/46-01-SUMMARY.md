---
phase: 46-mcp-cli-updates
plan: 01
subsystem: mcp
tags: [mcp, introspection, api-resources, authorization, policy]

# Dependency graph
requires:
  - phase: 41-api-resources-basics
    provides: ApiResource derive macro and Resource trait
  - phase: 39-core-authentication
    provides: Policy trait for authorization
provides:
  - list_resources MCP tool for API resource introspection
  - list_policies MCP tool for authorization policy introspection
affects: [46-02, 46-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "syn visitor pattern for scanning derive macros (list_resources)"
    - "String matching for impl trait scanning (list_policies)"

key-files:
  created:
    - ferro-mcp/src/tools/list_resources.rs
    - ferro-mcp/src/tools/list_policies.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs

key-decisions:
  - "syn visitor for resources, string matching for policies — consistent with existing tool patterns"

patterns-established:
  - "Resource scanning via derive attribute detection"
  - "Policy scanning via impl block string matching with ability extraction"

# Metrics
duration: 4min
completed: 2026-02-10
---

# Phase 46 Plan 01: list_resources + list_policies MCP Tools Summary

**Two new MCP introspection tools: list_resources scans for #[derive(ApiResource)] structs with field details; list_policies scans for impl Policy<Model> with ability methods**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-10T07:20:07Z
- **Completed:** 2026-02-10T07:24:04Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- list_resources tool scans all .rs files under src/ for #[derive(ApiResource)] structs, extracting field names, types, and resource attributes (skip, rename)
- list_policies tool scans all .rs files under src/ for impl Policy<Model> patterns, extracting policy struct names, guarded models, and implemented abilities
- Both tools registered in MCP service with descriptions, usage guidance, and workflow integration
- MCP instructions updated with new tool entries in categories, workflows, and "When to Use" sections

## Task Commits

Each task was committed atomically:

1. **Task 1: Create list_resources and list_policies MCP tools** - `31b9dfb` (feat)
2. **Task 2: Register list_resources and list_policies in MCP service** - `a268d95` (feat)

## Files Created/Modified
- `ferro-mcp/src/tools/list_resources.rs` - API resource scanner using syn visitor pattern
- `ferro-mcp/src/tools/list_policies.rs` - Authorization policy scanner using string matching
- `ferro-mcp/src/tools/mod.rs` - Module declarations for new tools
- `ferro-mcp/src/service.rs` - Tool handlers, descriptions, and MCP instructions update

## Decisions Made
- syn visitor for resources, string matching for policies -- consistent with existing tool patterns (list_models uses syn, list_jobs introspection uses syn, but policies use impl blocks rather than derive attributes so string matching is more appropriate)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Ready for 46-02-PLAN.md (list_rate_limiters + list_broadcast_channels + MCP instructions update)
- Both new tools follow established patterns and integrate into the existing MCP service

---
*Phase: 46-mcp-cli-updates*
*Completed: 2026-02-10*

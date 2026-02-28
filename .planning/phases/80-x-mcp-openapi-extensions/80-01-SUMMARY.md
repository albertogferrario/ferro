---
phase: 80-x-mcp-openapi-extensions
plan: 01
subsystem: openapi
tags: [openapi, mcp, x-mcp, vendor-extensions, ai-tools]

# Dependency graph
requires:
  - phase: 80
    provides: phase research and plan
provides:
  - mcp_tool_name helper for snake_case AI tool names
  - mcp_description helper for AI-optimized descriptions
  - x-mcp-tool-name and x-mcp-description extensions on every API operation
affects: [80-02]

# Tech tracking
tech-stack:
  added: []
  patterns: [utoipa ExtensionsBuilder for vendor extension emission]

key-files:
  created: []
  modified:
    - framework/src/api/openapi.rs

key-decisions:
  - "Reuse existing extract_resource_name and singularize helpers for consistency with auto_summary"
  - "ExtensionsBuilder.add() auto-prefixes x- so keys are idempotent"
  - "Tool names use snake_case (list_users, get_user) matching MCP convention"

patterns-established:
  - "mcp_tool_name/mcp_description pattern mirrors auto_summary with method+path dispatch"
  - "Extensions inserted on OperationBuilder before build() call"

# Metrics
duration: 5min
completed: 2026-02-28
---

# Phase 80 Plan 01: Framework x-mcp Extension Emission Summary

**x-mcp vendor extensions emitted from build_openapi_spec with AI-friendly tool names and descriptions**

## Performance

- **Duration:** 5 min
- **Completed:** 2026-02-28
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- `mcp_tool_name(method, path)` generates snake_case tool names: list_users, get_user, create_user, update_user, delete_user
- `mcp_description(method, path)` generates action-oriented descriptions for AI consumption
- `build_openapi_spec()` now emits `x-mcp-tool-name` and `x-mcp-description` extensions on every API operation via `ExtensionsBuilder`
- 16 unit tests for helper functions covering GET/POST/PUT/PATCH/DELETE for collection and single-resource paths, nested resources, and fallback
- Updated `build_spec_basic` test to verify extensions on GET and DELETE operations
- New `spec_extensions_in_json` test verifies extensions survive JSON serialization round-trip

## Task Commits

Work committed atomically alongside plan 02 changes:

1. **Task 1+2: mcp helpers + extension wiring** - `4a25aa4` (feat)

## Files Created/Modified
- `framework/src/api/openapi.rs` - Added ExtensionsBuilder import, mcp_tool_name and mcp_description helpers, extension emission in build_openapi_spec loop, 16 helper tests, extension assertions in build_spec_basic, spec_extensions_in_json test

## Decisions Made
- Reused existing extract_resource_name/singularize/has_path_param helpers to keep pattern consistent with auto_summary
- ExtensionsBuilder handles x- prefix automatically, so keys are idempotent whether prefixed or not
- Tool names follow snake_case convention (list_users not listUsers) to match MCP tool naming standards

## Deviations from Plan

Commits were bundled with plan 02 changes since both plans modified the same file and were executed in the same session.

## Issues Encountered
None

## User Setup Required
None - extensions are emitted automatically for all API routes.

## Next Phase Readiness
- All 42 openapi tests pass
- Full workspace compiles cleanly (fmt + clippy + tests)
- x-mcp extensions available for downstream consumption by ferro-api-mcp spec parser (plan 02)

---
*Phase: 80-x-mcp-openapi-extensions*
*Completed: 2026-02-28*

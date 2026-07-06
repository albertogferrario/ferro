---
phase: 92-mcp-introspection-cli
plan: 02
subsystem: api
tags: [mcp, cli, validation, projections, servicedef]

# Dependency graph
requires:
  - phase: 91-framework-integration
    provides: MCP projection introspection tools (list, inspect, render)
provides:
  - CLI projection:check command for structural validation
  - MCP validate_projection tool for structured validation results
  - pub(crate) reconstruct_service_def reusable across MCP tools
affects: [92-mcp-introspection-cli]

# Tech tracking
tech-stack:
  added: []
  patterns: [regex-based ServiceDef reconstruction, source-level validation]

key-files:
  created:
    - ferro-cli/src/commands/projection_check.rs
    - ferro-mcp/src/tools/validate_projection.rs
  modified:
    - ferro-cli/src/main.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/Cargo.toml
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/tools/render_projection.rs
    - ferro-mcp/src/service.rs

key-decisions:
  - "reconstruct_service_def promoted to pub(crate) for reuse across MCP tools"
  - "CLI projection:check feature-gated behind projections feature flag (default on)"
  - "Warnings don't fail validation — only Err from validate() sets exit code 1"

patterns-established:
  - "Source-level regex reconstruction shared between CLI and MCP via separate implementations"

# Metrics
duration: 6min
completed: 2026-03-01
---

# Phase 92 Plan 02: Projection Validation Summary

**CLI `ferro projection:check` and MCP `validate_projection` tool for ServiceDef structural validation**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-01T02:57:13Z
- **Completed:** 2026-03-01T03:03:32Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- CLI command `ferro projection:check` scans, reconstructs, and validates all projections with colored output
- MCP `validate_projection` tool returns structured JSON with warnings/errors/valid status
- `reconstruct_service_def` made `pub(crate)` for reuse across MCP tools
- Feature-gated behind `projections` feature in ferro-cli (default enabled)
- 11 new tests across CLI (5) and MCP (6)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ferro projection:check CLI command** - `2fced76` (feat)
2. **Task 2: Add validate_projection MCP tool** - `538fae1` (feat)

## Files Created/Modified
- `ferro-cli/src/commands/projection_check.rs` - CLI command: scan, reconstruct, validate projections
- `ferro-cli/Cargo.toml` - Added ferro-projections optional dep + projections feature flag
- `ferro-cli/src/commands/mod.rs` - Register projection_check module (feature-gated)
- `ferro-cli/src/main.rs` - Register projection:check command variant + match arm
- `ferro-mcp/src/tools/validate_projection.rs` - MCP tool: single/all validation with structured results
- `ferro-mcp/src/tools/mod.rs` - Register validate_projection module
- `ferro-mcp/src/tools/render_projection.rs` - Made reconstruct_service_def pub(crate)
- `ferro-mcp/src/service.rs` - Added ValidateProjectionParams + tool handler

## Decisions Made
- reconstruct_service_def promoted to pub(crate) for reuse (was private in render_projection)
- CLI and MCP have independent reconstruction implementations — CLI uses its own copy to avoid coupling ferro-cli to ferro-mcp internals
- Warnings produce exit code 0 (clean), only validation errors produce exit code 1
- Feature gated behind `projections` feature flag with default-on for seamless opt-out

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Projection validation available via CLI and MCP
- Ready for remaining 92 plans (if any) or phase 93

---
*Phase: 92-mcp-introspection-cli*
*Completed: 2026-03-01*

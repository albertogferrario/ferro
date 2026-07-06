---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "09"
subsystem: ferro-mcp
tags: [mcp-tool, route-lookup, levenshtein, json-ui]
dependency_graph:
  requires: ["162-07"]
  provides: [json_ui_verify_action MCP tool]
  affects: [ferro-mcp/src/service.rs, ferro-mcp/src/tools/list_routes.rs]
tech_stack:
  added: [strsim 0.11]
  patterns: [pure lookup helper + async execute wrapper, rmcp tool_router macro]
key_files:
  created:
    - ferro-mcp/src/tools/json_ui_verify_action.rs
  modified:
    - ferro-mcp/Cargo.toml
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/tools/list_routes.rs
    - ferro-mcp/src/service.rs
decisions:
  - "Used McpError::ToolError (not a new variant) for oversized-input rejection"
  - "Added Clone + PartialEq to RouteInfo (required by VerifyActionResult derive)"
  - "find_handler is pub(crate) for testability; execute wraps it with project I/O"
metrics:
  duration_minutes: 2
  completed: "2026-05-16"
  tasks_completed: 3
  files_changed: 5
---

# Phase 162 Plan 09: json_ui_verify_action MCP Tool Summary

One-liner: Levenshtein-backed MCP tool that confirms a handler name is registered as a named route, returning the closest candidate on miss.

## What Was Built

`json_ui_verify_action` is a new ferro-mcp tool (D-09) that accepts `{ handler: String, method: Option<String> }` and returns:

- `{ found: true, route: RouteInfo, candidate: null, message }` on exact match
- `{ found: false, route: null, candidate: "closest.name", message }` on miss

The tool reads route names from the existing registry via `list_routes::execute` — no second source of truth (D-10). Input is capped at 256 chars before Levenshtein runs (T-162-09-01 mitigation).

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add strsim 0.11 dependency | e7cdd290 | ferro-mcp/Cargo.toml, Cargo.lock |
| 2 | Create json_ui_verify_action.rs + mod.rs registration | e427b4e6 | json_ui_verify_action.rs, mod.rs, list_routes.rs |
| 3 | Wire into MCP dispatcher | b7b47d38 | service.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing derives] Added Clone + PartialEq to RouteInfo**
- **Found during:** Task 2 (first compile attempt)
- **Issue:** `VerifyActionResult` derives `PartialEq` and holds `Option<RouteInfo>`; `RouteInfo` had neither `Clone` nor `PartialEq`
- **Fix:** Added `Clone, PartialEq` to `RouteInfo`'s derive in list_routes.rs
- **Files modified:** ferro-mcp/src/tools/list_routes.rs
- **Commit:** e427b4e6

## Tests

5 unit tests, all passing:
- `verify_action_found_returns_route_info` — exact match returns route, no candidate
- `verify_action_found_filters_by_method` — GET route does not match POST query
- `verify_action_not_found_returns_closest_levenshtein_candidate` — "dashboar.show" → "dashboard.show" (distance 1)
- `verify_action_empty_route_list_returns_no_candidate` — empty list, no candidate
- `verify_action_rejects_oversized_handler_input` — 257-char input returns Err

## Known Stubs

None.

## Threat Flags

None beyond those in the plan's threat model (T-162-09-01 mitigated; T-162-09-02 accepted).

## Self-Check: PASSED

- `ferro-mcp/src/tools/json_ui_verify_action.rs` exists
- `pub async fn execute` present in file
- `pub(crate) fn find_handler` present in file
- `strsim::levenshtein` call present in file
- `pub mod json_ui_verify_action` in mod.rs
- Commits e7cdd290, e427b4e6, b7b47d38 in git log
- 215 tests pass, fmt clean, clippy clean

---
phase: 240-crud-input-schema-derivation-list-query-polish
plan: "03"
subsystem: mcp-renderer-write-dispatch
tags: [ferro-mcp-server, crud, write-boundary, tool-emission, nti-envelope, phase-205-guard]

requires:
  - phase: 240-02
    provides: "build_create/update/delete_input_schema builders"

provides:
  - "fn render_create_tool(service: &ServiceDef) -> Result<Option<Tool>, ProjError>"
  - "fn render_update_tool(service: &ServiceDef) -> Result<Option<Tool>, ProjError>"
  - "fn render_delete_tool(service: &ServiceDef) -> Result<Option<Tool>, ProjError>"
  - "flag-gated CRUD verb emission in render_exposed_tools (after ActionDef loop, before disambiguation)"
  - "CRUD verb NTI detection block in handle_write_call (before find_action)"
  - "crud_tool_call_nti_parses_as_valid_mcp_content test (Phase 205 guard extension)"

affects:
  - "240-04 — dispatch.rs range/sort filter enforcement is independent; no dependency on 240-03"
  - "241 — Phase 241 removes the NTI block and wires execution"

tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN within each task — RED commit first (failing test), then GREEN commit"
    - "render_create/update/delete_tool mirror render_action_tool shape exactly (build schema → map to object → construct Tool with annotations)"
    - "NTI detection loop mirrors confirmation prefix routing pattern — same strip_prefix idiom"
    - "disambiguate_write_tool_collisions untouched; CRUD verb names embed service name, globally unique"

key-files:
  created: []
  modified:
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/write_dispatch.rs
    - ferro-mcp-server/src/jsonrpc.rs

key-decisions:
  - "CRUD verb emission placed after ActionDef loop and BEFORE disambiguation — CRUD names (create_order) embed the service name and can never collide, making them safe to add before the collision pass"
  - "NTI detection loop placed before tenant check and find_action — a CRUD verb call must never reach -32601 regardless of tenant state; the NTI envelope is structurally inert (no DB access, no auth check)"
  - "NTI envelope uses CallToolResult::structured with is_error=false to satisfy the Phase 205 regression guard; the agent sees a usable response shape, not a protocol error"

requirements-completed: [CRUD-01, CRUD-02]

duration: 5min
completed: "2026-06-23"
---

# Phase 240 Plan 03: Write Tool Surface — CRUD Verb Emission + NTI Envelope Summary

**Three tool-emitter helpers + flag-gated CRUD verb emission in `render_exposed_tools` + structured NTI routing in `handle_write_call` + Phase 205 guard extension — the write tool listing surface is complete; tools appear in `tools/list` with correct schemas and return a valid MCP response on call (not a protocol error)**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-06-23T17:29:02Z
- **Completed:** 2026-06-23T17:33:42Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

### Task 1: Emit create_/update_/delete_ tools in render_exposed_tools

- `render_create_tool`: name = `create_{svc}`, schema from `build_create_input_schema`, `destructive(false)`
- `render_update_tool`: name = `update_{svc}`, schema from `build_update_input_schema`, `destructive(false)`
- `render_delete_tool`: name = `delete_{svc}`, schema from `build_delete_input_schema`, `destructive(true)`
- Emission block added in `render_exposed_tools` after `for action in &service.actions` loop and before `disambiguate_write_tool_collisions` call — gated on `service.creatable` / `service.updatable` / `service.deletable`
- `disambiguate_write_tool_collisions` left entirely untouched; CRUD verb names are globally unique by construction
- Four RED tests added first (failing on bare list_order output), then four GREEN tests pass after implementation

### Task 2: CRUD verb NTI envelope in write_dispatch + Phase 205 guard extension

- CRUD verb detection block inserted in `handle_write_call` after confirmation prefix routing and **before** the tenant check + `find_action` call (line ordering confirmed by grep)
- Returns `CallToolResult::structured({ error_kind: "not_yet_implemented", message: "... (Phase 241)" })` — never falls through to `-32601`
- Loop iterates `["create_", "update_", "delete_"]` prefixes; matches on `services.iter().any(|s| s.mcp_exposed && s.name == svc_name)` so only valid tool names are intercepted
- `crud_tool_call_nti_parses_as_valid_mcp_content` test added in `jsonrpc.rs` mirroring the Phase 205 guard shape: `service.creatable(true)`, `McpContext { scope: Some("read_write") }`, call `create_order`, assert `CallToolResult` parses, `is_error=Some(false)`, `structured_content["error_kind"] == "not_yet_implemented"`
- Original `tools_call_result_parses_as_valid_mcp_content` (list_ path) still passes unchanged

## Task Commits

1. **Task 1 RED** — `80a198b1`: test(240-03): add RED tests for CRUD verb tool emission in renderer
2. **Task 1 GREEN** — `d57d31cf`: feat(240-03): emit create_/update_/delete_ tools in render_exposed_tools
3. **Task 2 RED** — `0e798e99`: test(240-03): add RED test for CRUD verb NTI envelope (Phase 205 guard extension)
4. **Task 2 GREEN** — `ce6ebf9a`: feat(240-03): CRUD verb NTI envelope in write_dispatch + Phase 205 guard extension

## Files Created/Modified

- `ferro-mcp-server/src/renderer.rs` — `render_create_tool` (line 244), `render_update_tool` (line 270), `render_delete_tool` (line 297); CRUD emission block in `render_exposed_tools` (lines 90–109); four tests (lines 650–800)
- `ferro-mcp-server/src/write_dispatch.rs` — NTI detection block (lines 155–168), before find_action
- `ferro-mcp-server/src/jsonrpc.rs` — `crud_tool_call_nti_parses_as_valid_mcp_content` test (line 415)

## Decisions Made

- CRUD verb emission placed after ActionDef loop and before disambiguation — the collision pass is safe to run on CRUD names because they embed the service name (create_order can never collide with update_order), so no disambiguation code change was needed.
- NTI detection runs before the tenant check (not after) — the envelope is inert (no DB access), so it makes no sense to gate it on auth. More importantly, if the tenant check ran first and failed, the response would be a `-32603` error, not a protocol-valid NTI result. Auth for CRUD execution is Phase 242.
- `is_error=false` in the NTI envelope is correct — the tool was found and responded validly; the "not yet implemented" message is informational, not an error state from the protocol's perspective.

## Deviations from Plan

None — plan executed exactly as written. Both TDD RED/GREEN cycles completed without incident. `cargo fmt --all` reformatted multi-line function signatures to single-line form on both GREEN commits; applied before committing; no logic change.

## Known Stubs

The NTI detection block (`error_kind: "not_yet_implemented"`) is an intentional, documented stub. It is not a content stub — it advertises the tool in `tools/list` (with correct schema) and returns a valid MCP response on call. Phase 241 removes this block and wires actual execution. This is the defined scope boundary for Phase 240.

## Threat Flags

None. The NTI block contains only `error_kind` + a generic message — no column names, table names, or schema internals. No write execution path is reached. The Phase 205 regression guard is extended to cover CRUD verb calls.

---

## Self-Check: PASSED

- FOUND: ferro-mcp-server/src/renderer.rs
- FOUND: ferro-mcp-server/src/write_dispatch.rs
- FOUND: ferro-mcp-server/src/jsonrpc.rs
- FOUND: 240-03-SUMMARY.md
- FOUND: 80a198b1 (RED Task 1)
- FOUND: d57d31cf (GREEN Task 1)
- FOUND: 0e798e99 (RED Task 2)
- FOUND: ce6ebf9a (GREEN Task 2)

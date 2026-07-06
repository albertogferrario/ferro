---
phase: 218-write-tool-rendering-from-actiondef
plan: "00"
subsystem: ferro-mcp-server
tags: [tdd, red-tests, mcp, write-tools, actiondef]
dependency_graph:
  requires: []
  provides: [218-00-red-tests]
  affects: [ferro-mcp-server/src/schema.rs, ferro-mcp-server/src/renderer.rs, ferro-mcp-server/src/jsonrpc.rs]
tech_stack:
  added: []
  patterns: [tdd-red-wave, compile-error-red, assertion-red]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/schema.rs
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/jsonrpc.rs
decisions:
  - "Placed SC#5 test inline in jsonrpc.rs (beside Phase 205 regression), not in integration tests — no DB needed and co-location keeps both wire-format regression tests together"
  - "Removed redundant `use std::collections::HashMap` from renderer test module — brought in via `use super::*` through McpContext"
metrics:
  duration_seconds: 171
  completed_date: "2026-06-13T20:25:01Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 218 Plan 00: RED Test Layer for Write-Tool Rendering — Summary

One-liner: Wave 0 RED tests encoding all five write-tool rendering success criteria — compile-error RED for schema (build_action_input_schema missing), assertion-RED for renderer (no write tools emitted yet) and jsonrpc SC#5 (tools.len()==1, expected 3).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | RED schema tests for build_action_input_schema (SC#2, T-218-01) | 7dba9635 | ferro-mcp-server/src/schema.rs |
| 2 | RED renderer tests for write-tool emission, annotations, guard filter (SC#1, SC#3, SC#4, T-218-02) | 7dba9635 | ferro-mcp-server/src/renderer.rs |
| 3 | RED SC#5 strict-deserialization test for write-tool definitions (SC#5, T-218-03) | 7dba9635 | ferro-mcp-server/src/jsonrpc.rs |

## Crate Compile State

**`ferro-mcp-server` does NOT compile for tests after this plan.** This is intentional.

The 5 schema RED tests each call `build_action_input_schema(&action, &service)` which does not exist yet. The Rust compiler emits:

```
error[E0425]: cannot find function `build_action_input_schema` in this scope (×5)
error: could not compile `ferro-mcp-server` (lib test) due to 5 previous errors
```

**Plan 01 executor:** implement `pub fn build_action_input_schema(action: &ActionDef, service: &ServiceDef) -> crate::Result<serde_json::Value>` in `ferro-mcp-server/src/schema.rs`. Once that function exists, the schema tests will compile and the renderer/jsonrpc tests will also be reachable — they will then report assertion-RED (write tools absent from `render_exposed_tools` output).

**Plan 02 executor:** extend `render_exposed_tools` in `renderer.rs` with the write-tool loop. This turns the renderer and jsonrpc assertion-RED tests GREEN.

## What Each Test Encodes

### schema.rs (5 RED tests — compile-error RED, SC#2, T-218-01)

| Test | Encodes |
|------|---------|
| `test_action_schema_injects_identifier` | Service's first Identifier field injected as required integer param |
| `test_action_schema_maps_inputs` | InputDef → property with type + description; required=true → required[] |
| `test_action_schema_optional_input_not_required` | required=false → in properties, NOT in required[] |
| `test_action_schema_excludes_sensitive_input` | FieldMeaning::Sensitive excluded from properties AND required[] (T-218-01 mitigation) |
| `test_action_schema_no_identifier_field_is_silent_noop` | No identifier field = silent skip, schema still valid |

### renderer.rs (6 RED tests — assertion-RED, SC#1/SC#3/SC#4, T-218-02)

| Test | Encodes |
|------|---------|
| `test_one_write_tool_per_action` | One write tool per ActionDef; name = action.name verbatim; total = 3 (SC#1) |
| `test_write_tool_annotations_transition` | transition_trigger.is_some() → readOnlyHint=false, destructiveHint=true (SC#4) |
| `test_write_tool_annotations_non_transition` | No transition_trigger → readOnlyHint=false, destructiveHint=false (SC#4) |
| `test_guard_false_omits_tool` | evaluated_guards["has_items"]=false → tool absent (SC#3, T-218-02) |
| `test_guard_true_includes_tool` | evaluated_guards["has_items"]=true → tool present (SC#3) |
| `test_guard_absent_includes_tool` | Guard key absent (McpContext::default()) → tool present (SC#3) |

### jsonrpc.rs (1 RED test — assertion-RED, SC#5, T-218-03)

| Test | Encodes |
|------|---------|
| `write_tools_definitions_parse_as_valid_mcp_tool` | tools/list len==3; each Tool deserializes via rmcp::model::Tool; write tool annotations correct |

## Deviations from Plan

None — plan executed exactly as written.

The only adjustment was removing a redundant `use std::collections::HashMap` import from the renderer test module (it was unused since `McpContext` with its `evaluated_guards: HashMap` field is brought in via `use super::*`). This is a Rule 2 micro-fix to keep clippy clean, not a deviation from plan intent.

## Security Coverage

| Threat | Test | Status |
|--------|------|--------|
| T-218-01: Sensitive input disclosure via write tool schema | `test_action_schema_excludes_sensitive_input` | RED (compile-error); GREEN gate = Plan 01 |
| T-218-02: Guard filter misread as auth gate | `test_guard_false_omits_tool` / `test_guard_true_includes_tool` / `test_guard_absent_includes_tool` | RED (assertion); GREEN gate = Plan 02 |
| T-218-03: Malformed tool definition breaks strict MCP clients | `write_tools_definitions_parse_as_valid_mcp_tool` | RED (assertion); GREEN gate = Plan 02 |

## Self-Check: PASSED

- FOUND: ferro-mcp-server/src/schema.rs
- FOUND: ferro-mcp-server/src/renderer.rs
- FOUND: ferro-mcp-server/src/jsonrpc.rs
- FOUND: .planning/phases/218-write-tool-rendering-from-actiondef/218-00-SUMMARY.md
- FOUND: commit 7dba9635

---
phase: 197-mcprenderer-ferro-mcp-server
plan: "02"
subsystem: ferro-mcp-server
tags: [mcp, projections, schema-derivation, renderer, tdd]
dependency_graph:
  requires: [ferro-mcp-server scaffold (197-01), ferro-projections (ServiceDef, FieldDef, FieldMeaning, DataType)]
  provides: [McpRenderer::render, render_exposed_tools, build_input_schema, is_filter_field]
  affects: [ferro-mcp-server/src/schema.rs, ferro-mcp-server/src/renderer.rs, ferro-mcp-server/src/lib.rs]
tech_stack:
  added: []
  patterns: [Renderer trait impl, Arc<JsonObject> inputSchema, is_filter_field conservative predicate, TDD GREEN on both tasks]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/schema.rs
    - ferro-mcp-server/src/renderer.rs
    - ferro-mcp-server/src/lib.rs
decisions:
  - inputSchema is derived solely from ServiceDef.fields + pagination; no separately declared schema (AMCP-02)
  - is_filter_field uses 5-gate conservative predicate; gate order is load-bearing (readable first catches write_only regardless of Sensitive meaning)
  - render_exposed_tools filters on mcp_exposed before rendering; zero tools for unmarked projections (AMCP-01)
  - McpRenderer::render uses tool.annotate(ToolAnnotations::new().read_only(true)) matching ferro-api-mcp pattern
metrics:
  duration: "183s (~3m)"
  completed: "2026-06-10"
  tasks_completed: 2
  files_changed: 3
---

# Phase 197 Plan 02: McpRenderer & Schema Derivation Summary

`McpRenderer::render` derives an MCP tool definition with a conservative `inputSchema` built solely from `ServiceDef` fields — the same projection that yields JSON-UI yields the tool's input contract with no separately maintained schema.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | schema.rs: is_filter_field predicate, data_type_to_json_schema, build_input_schema | af01fffc |
| 2 | renderer.rs: McpRenderer::render, render_exposed_tools; lib.rs: re-export | 9b14a64c |
| fmt | cargo fmt cleanup | 08b22046 |

## Verification Results

- `cargo test -p ferro-mcp-server`: 10/10 passed (5 schema + 5 renderer)
- `cargo clippy -p ferro-mcp-server --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean
- AMCP-01: `test_mcp_exposed_filter` — exactly 1 tool from 2 services (1 exposed, 1 not)
- AMCP-02: `adding_field_changes_schema` — property count increases when filter field added

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ferro_projections::field module is private**
- **Found during:** Task 1, first compile
- **Issue:** Plan specified `use ferro_projections::field::{DataType, FieldDef, FieldMeaning}` but the `field` module is `mod field` (private) in ferro-projections; types are re-exported from crate root.
- **Fix:** Changed import to `use ferro_projections::{DataType, FieldDef, FieldMeaning, ServiceDef}` — matching the existing pattern in ferro-api-mcp and ferro-json-ui.
- **Files modified:** ferro-mcp-server/src/schema.rs
- **Commit:** af01fffc (fix applied inline before first RED run)

## TDD Gate Compliance

Tasks 1 and 2 were implemented as GREEN (implementation + tests together) against pre-existing RED state: the plan-01 stubs made every behavior test fail by definition (stub returned empty schema / `Err("not yet implemented")`). The stubs constituted the RED gate; the full implementations here are the GREEN gate.

- Task 1: stub returned `{"type":"object","properties":{}}` → all 5 schema tests would fail against stub → GREEN: full predicate + type map + builder, all 5 pass.
- Task 2: stub returned `Err(Render("not yet implemented"))` → all 5 renderer tests would fail → GREEN: full render + exposed filter, all 5 pass.

REFACTOR not needed for either task.

## Known Stubs

No new stubs introduced. The `dispatch` stub from plan 01 remains (plan 03 scope).

## Threat Surface Scan

No new network endpoints or auth paths. `is_filter_field` is the control point for T-197-03 (information disclosure via inputSchema). `test_sensitive_field_excluded` and `test_write_only_excluded` assert the gates hold. `readOnlyHint=true` satisfies T-197-04. `test_mcp_exposed_filter` satisfies T-197-05.

## Self-Check: PASSED

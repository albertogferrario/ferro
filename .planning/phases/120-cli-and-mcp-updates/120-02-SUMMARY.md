---
phase: 120
plan: "02"
subsystem: ferro-mcp
tags: [mcp, json-ui, v2, catalog, inspect, templates, generation-context]
dependency_graph:
  requires: [117-catalog-json-schema, 119-page-loader]
  provides: [mcp-json-ui-v2-tools]
  affects:
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/generation_context.rs
tech_stack:
  added: []
  patterns: [serde_json::Value flat map scan, global_catalog() API, json schema exposure]
key_files:
  modified:
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/generation_context.rs
decisions:
  - "json_ui_catalog exposes json_schema + component_schemas fields — one MCP call gets full schema (D-04)"
  - "json_ui_inspect scans src/views/*.json not .rs — BUILTIN_TYPES const removed (D-05)"
  - "json_ui_generate VIEW_EXAMPLE is now a v2 JSON spec; conventions point to render_file (D-07)"
  - "code_templates json_view category: 3 v1 Rust templates replaced with v2 JSON specs + new json_view_handler (D-06)"
  - "generation_context json_ui_view pattern updated to v2 render_file flow (D-08 extension)"
metrics:
  duration: ~25min
  completed: 2026-04-21
  tasks_completed: 5
  files_modified: 5
---

# Phase 120 Plan 02: MCP Tools v2 Update — Summary

MCP JSON-UI tools fully updated for v2 flat spec format: `json_ui_catalog` exposes JSON Schema fields, `json_ui_inspect` rewrites from regex to JSON file walk, `json_ui_generate` emits v2 spec examples and conventions, code templates replace v1 Rust builder patterns with v2 JSON specs, and `generation_context` updates the json_ui_view pattern to use `JsonUi::render_file`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | json_ui_catalog: add json_schema + component_schemas | cabbd577 | ferro-mcp/src/tools/json_ui_catalog.rs |
| 2 | json_ui_inspect: v2 JSON file scanner | cabbd577 | ferro-mcp/src/tools/json_ui_inspect.rs |
| 3 | json_ui_generate: v2 VIEW_EXAMPLE + ViewConventions | cabbd577 | ferro-mcp/src/tools/json_ui_generate.rs |
| 4 | code_templates: v2 JSON templates + json_view_handler | cabbd577 | ferro-mcp/src/tools/code_templates.rs |
| 5 | generation_context: v2 json_ui_view pattern (D-08) | cabbd577 | ferro-mcp/src/tools/generation_context.rs |

## Implementation

### json_ui_catalog (D-04)

Added two fields to `JsonUiCatalog`:
- `json_schema: serde_json::Value` — full v2 spec schema from `global_catalog().json_schema()`
- `component_schemas: HashMap<String, serde_json::Value>` — per-component props schemas for all 39 built-in + 1 Map plugin component

Schema map built in `execute()` by iterating `cat.components_sorted()` and `cat.plugin_components_sorted()` with `cat.component_schema(name)`. Previous field shape preserved (CONTEXT D-24).

### json_ui_inspect (D-05)

Complete rewrite of `execute()`:
- Scans `src/views/*.json` (not `.rs`)
- Parses each file as `serde_json::Value`
- Extracts `title` from `spec["title"]`, `layout` from `spec["layout"]`
- Collects `components_used` from `spec["elements"][*]["type"]` — deduplicated and sorted
- Collects `actions` from `spec["elements"][*]["action"]` as `"METHOD handler"` strings
- Invalid JSON files are silently skipped (not errors)
- `BUILTIN_TYPES` const removed — component type identity now comes from the parsed spec data
- `inspect_component()` updated to use `global_catalog().components_sorted()` for built-in detection (no stale hardcoded list)
- `TODO(Phase 120)` comment fulfilled

### json_ui_generate (D-07)

- `VIEW_EXAMPLE` replaced with a v2 JSON spec (User List example with Card, Text, DataTable)
- `ViewConventions` updated:
  - `file_location`: `"src/views/{name}.json"` (was `.rs`)
  - `function_signature`: handler using `JsonUi::render_file(...)` (was `pub async fn view() -> Response`)
  - `import_pattern`: `"use ferro::{JsonUi, Response, handler, Request};"` (was Spec/Element imports)
  - `layout_default`: `"dashboard"` (was `"app"`)
- `list_existing_views()` scans `.json` not `.rs` files

### code_templates (D-06)

`json_view_templates()` rewritten:
- `basic_view`: v2 JSON spec with Card + Text heading (no imports — JSON files need none)
- `list_view`: v2 JSON spec with DataTable, create Button, Pagination
- `form_view`: v2 JSON spec with Form + Input fields and action
- `json_view_handler` (new): Rust handler template using `JsonUi::render_file`

All four templates use `category: "json_view"` — filter by category still works. No `Spec::builder()` or `Element::new()` in any `json_view` template.

### generation_context (D-08 extension)

`common_patterns.json_ui_view` updated from v1 `Spec::builder()` Rust code to a v2 comment block showing both the JSON spec file and the paired Rust handler using `JsonUi::render_file`. `imports.json_ui_view` updated from Spec/Element imports to `use ferro::{handler, Request, Response, JsonUi};`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Updated generation_context.rs v1 reference**
- **Found during:** Task 4 (D-08 verification grep)
- **Issue:** `generation_context.rs` `common_patterns.json_ui_view` contained v1 `Spec::builder()` code. The CONTEXT.md deferred section said "out of scope if no hits" — but grep found hits.
- **Fix:** Updated `json_ui_view` pattern to v2 JSON spec + `render_file` handler. Updated `imports.json_ui_view` to v2 handler imports.
- **Files modified:** `ferro-mcp/src/tools/generation_context.rs`
- **Commit:** cabbd577

## Verification

D-08 grep after all changes:
```
grep -rn "Spec::builder()|Element::new|JsonUiView|Component::" ferro-mcp/src/tools/ --include="*.rs"
```

All remaining hits are:
- Test assertion strings confirming v1 is absent
- `JsonUiViewList` struct name (MCP output type, not a v1 component type)
- `BUILDER_API` documentation constant in `json_ui_catalog.rs` (intentional — documents the spec builder API for reference)

No v1 generation or template code remains.

## Test Results

- `cargo clippy --package ferro-mcp --all-targets -- -D warnings`: clean
- `cargo test --package ferro-mcp --all-features`: 212 passed, 0 failed

## Known Stubs

None. All catalog, schema, and template data is sourced from `global_catalog()` or is static v2 JSON.

## Self-Check: PASSED

- [x] ferro-mcp/src/tools/json_ui_catalog.rs modified (json_schema + component_schemas added)
- [x] ferro-mcp/src/tools/json_ui_inspect.rs rewritten (JSON file scan)
- [x] ferro-mcp/src/tools/json_ui_generate.rs updated (v2 example + conventions)
- [x] ferro-mcp/src/tools/code_templates.rs updated (v2 JSON templates)
- [x] ferro-mcp/src/tools/generation_context.rs updated (v2 pattern)
- [x] Commit cabbd577 exists
- [x] 212 tests pass
- [x] No v1 generation references remain

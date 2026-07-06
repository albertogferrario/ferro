---
phase: 120
plan: "04"
subsystem: ferro-mcp
tags: [json-ui, mcp, v2, inspect, code-templates]
dependency_graph:
  requires: [Phase 117 Catalog & JSON Schema, Phase 119 Page Loader]
  provides: [v2 json_ui_inspect scanner, v2 json_view code templates]
  affects: [ferro-mcp]
tech_stack:
  added: []
  patterns: [json file walking, serde_json value extraction, flat element map parsing]
key_files:
  modified:
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/code_templates.rs
decisions:
  - "Rewrote inspect_component() to use global_catalog() iterator instead of static BUILTIN_TYPES array"
  - "Used flat directory walk with recursion for scan_json_views() — handles subdirectory nesting"
  - "json_view_handler template added as 4th entry in category json_view (filter compat preserved)"
metrics:
  duration: "~15 minutes"
  completed: "2026-04-21T13:31:53Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 120 Plan 04: json_ui_inspect v2 Scanner + code_templates v2 JSON Format Summary

Rewrote the `json_ui_inspect` MCP tool to scan v2 JSON spec files instead of v1 Rust source patterns, and replaced all three `json_view` code templates with v2 flat JSON spec format plus a new paired Rust handler template.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Rewrite json_ui_inspect.rs for v2 JSON file scanning | 9476aef4 | ferro-mcp/src/tools/json_ui_inspect.rs |
| 2 | Rewrite json_view_templates() for v2 JSON spec format | e3466bf3 | ferro-mcp/src/tools/code_templates.rs |

## What Was Built

### Task 1: json_ui_inspect v2 Scanner

Replaced the v1 regex-based scanner (which matched `JsonUiView`, `Component::X`, `pub fn ... -> JsonUiView` patterns in `.rs` files) with a JSON file walker:

- `scan_json_views(project_root, views_dir)` walks `src/views/` recursively
- For each `*.json` file: parses with `serde_json`, extracts `title`, `layout`, `elements[*].type` (deduplicated+sorted), `elements[*].action.handler`
- `inspect_component()` rewritten to use `global_catalog().components_sorted()` iterator instead of the static `BUILTIN_TYPES` array (which was stale at 20 entries vs 39 built-in components)
- Removed: `BUILTIN_TYPES` const, `regex::Regex` import, all v1 pattern strings
- Added: tempfile-based integration tests for v2 spec parsing, filter, and non-JSON skipping

### Task 2: code_templates v2 JSON Format

Replaced all three `json_view` templates with v2 flat JSON spec strings:

- `basic_view`: minimal spec with `Card` + `Text` elements
- `list_view`: spec with `Card`, `Text`, `Button` (create action), `DataTable` (data_path binding), `Pagination`
- `form_view`: spec with `Card`, `Text`, `Form` (POST action), two `Input` elements
- Added `json_view_handler`: Rust handler template using `JsonUi::render_file("views/{{view_name}}.json", data)` — pairs with a JSON spec file
- All `imports` fields for JSON spec templates are empty (no Rust imports needed for spec files)
- Category `json_view` preserved across all 4 templates — filter compatibility maintained

## Verification

- `cargo build -p ferro-mcp`: clean
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings`: clean
- `cargo test -p ferro-mcp`: 208 passed, 0 failed
- `grep -rn "Spec::builder|Element::new|JsonUiView|Component::" ferro-mcp/src/tools/`: zero hits in generation/template paths

## Deviations from Plan

None — plan executed as specified.

## Self-Check: PASSED

- ferro-mcp/src/tools/json_ui_inspect.rs: modified and compiles
- ferro-mcp/src/tools/code_templates.rs: modified and compiles
- Commit 9476aef4: exists in git log
- Commit e3466bf3: exists in git log
- All 208 ferro-mcp tests pass

---
phase: 120
plan: "01"
subsystem: ferro-cli, ferro-mcp
tags: [json-ui-v2, cli, mcp, code-generation, ai-generation]
dependency_graph:
  requires: [115-spec-v2-data-structures, 117-catalog-and-json-schema, 119-page-loader]
  provides: [v2-cli-generation, v2-mcp-tools, v2-code-templates]
  affects: [ferro-cli, ferro-mcp]
tech_stack:
  added: []
  patterns: [two-pass-ai-generation, tool-use-structured-output, json-file-walk]
key_files:
  created: []
  modified:
    - ferro-cli/src/commands/make_json_view.rs
    - ferro-cli/src/ai.rs
    - ferro-cli/src/templates/make.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/generation_context.rs
decisions:
  - "`call_anthropic_structured` is a separate function (not a flag) — request body shape differs from plain text calls"
  - "Two-pass orchestration lives in `generate_json_view()` in ai.rs, not in the command itself"
  - "component_schemas includes plugin components — agents don't know which they'll need at call time"
  - "inspect_component() uses `components_sorted()` (not `component_schema()`) to distinguish built-in from plugin — both are in per_component_schemas"
metrics:
  duration: "~20min"
  completed: "2026-04-21"
  tasks: 1
  files: 8
---

# Phase 120 Plan 01: CLI & MCP Updates Summary

JSON-UI v2 spec generation end-to-end: CLI generates `.json` files via two-pass AI, MCP tools expose v2 conventions and JSON Schema, all v1 builder-pattern references removed from generation paths.

## What Was Built

### ferro-cli

**`make_json_view.rs`** — Generates `src/views/{name}.json` (v2 flat spec) instead of `src/views/{name}.rs`. Removed all `mod.rs` update logic (JSON files are not Rust modules). Default layout changed from "app" to "dashboard". Usage instructions updated to show `JsonUi::render_file` pattern.

**`ai.rs`** — Three new functions replacing the old `build_view_context`:
- `call_anthropic()` — retained but stripped of the `//!` assistant prefill (no longer appropriate for JSON output); max_tokens reduced to 1024 for Pass 1
- `call_anthropic_structured()` — new function using Anthropic `tool_use` mechanism with `emit_spec` tool and `tool_choice: {type: "tool", name: "emit_spec"}` for schema-constrained output; max_tokens 4096
- `generate_json_view()` — two-pass orchestrator: Pass 1 builds plain-text component plan using `catalog.prompt()`, Pass 2 generates full spec constrained to `catalog.json_schema()` via `call_anthropic_structured`, then validates with `Spec::from_json` + `catalog.validate()`

**`templates/make.rs`** — `json_view_template()` now returns a JSON spec string (not Rust code). Added `json_view_handler_template()` for the paired Rust handler.

### ferro-mcp

**`json_ui_catalog.rs`** — Two new fields on `JsonUiCatalog`:
- `json_schema: serde_json::Value` — full spec schema from `global_catalog().json_schema()`
- `component_schemas: HashMap<String, serde_json::Value>` — per-component props schemas for all built-in and plugin components

**`json_ui_inspect.rs`** — Complete rewrite of the scanner. Removed v1 regex patterns (`JsonUiView`, `Component::`, `BUILTIN_TYPES`). New `scan_json_views()` walks `src/views/*.json`, parses each as `serde_json::Value`, extracts `title`, `layout`, `elements[*].type` (deduplicated, sorted), and `elements[*].action.handler`. Fixed `inspect_component()` to use `components_sorted()` (not `component_schema()`) to distinguish built-in from plugin components — both appear in `per_component_schemas` so the schema presence check alone is insufficient.

**`json_ui_generate.rs`** — `VIEW_EXAMPLE` updated to a complete v2 JSON spec. `ViewConventions` updated: `file_location` → `.json`, `function_signature` → `JsonUi::render_file` call, `import_pattern` → `use ferro::{JsonUi, Response};`, `layout_default` → "dashboard". `list_existing_views()` now scans `.json` files.

**`code_templates.rs`** — `json_view_templates()` rewritten: all three templates (`basic_view`, `list_view`, `form_view`) are now v2 JSON spec strings with empty `imports` (no Rust imports needed for spec files). Added fourth template `json_view_handler` (category `json_view`) with the paired Rust handler using `JsonUi::render_file`.

**`generation_context.rs`** — Updated `common_patterns.json_ui_view` to show v2 spec + paired handler. Updated `file_structure.views` to `.json`. Updated `imports.json_ui_view` to `use ferro::{JsonUi, Response};`. Updated avoid-list to remove stale `.layout("app")` v1 hint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `inspect_component()` built-in/plugin discrimination**
- **Found during:** Task 1 (test run)
- **Issue:** Used `cat.component_schema(component_type).is_some()` to detect built-ins, but `per_component_schemas` includes plugin components too — Map was incorrectly classified as built-in
- **Fix:** Changed to `cat.components_sorted().any(|spec| spec.name.eq_ignore_ascii_case(component_type))` which only iterates the built-in component list
- **Files modified:** ferro-mcp/src/tools/json_ui_inspect.rs
- **Commit:** 9f82c44a

**2. [Rule 2 - Missing] Update `generation_context.rs` v1 references**
- **Found during:** D-08 grep verification pass
- **Issue:** `generation_context.rs` contained v1 builder code in `json_ui_view` pattern, v1 imports in `imports.json_ui_view`, stale `.rs` path in `file_structure.views`, and a v1-specific avoid hint
- **Fix:** Updated all four fields to v2 format and conventions
- **Files modified:** ferro-mcp/src/tools/generation_context.rs
- **Commit:** 9f82c44a

## Verification

D-08 grep check — no v1 builder references in generation/template paths:
```
grep -rn "Spec::builder\|Element::new\|JsonUiView\|Component::" ferro-cli/src ferro-mcp/src --include="*.rs"
```
Remaining hits are all in:
- `BUILDER_API`/`ACTION_API` const strings (API documentation, intentional)
- `json_ui_catalog.rs` test assertions for those const strings
- `module.rs` (out of this phase's scope — module scaffold templates)

All tests pass: 210 passed, 0 failed.

## Self-Check: PASSED

Files exist:
- ferro-cli/src/commands/make_json_view.rs: FOUND
- ferro-cli/src/ai.rs: FOUND
- ferro-cli/src/templates/make.rs: FOUND
- ferro-mcp/src/tools/json_ui_generate.rs: FOUND
- ferro-mcp/src/tools/json_ui_catalog.rs: FOUND
- ferro-mcp/src/tools/json_ui_inspect.rs: FOUND
- ferro-mcp/src/tools/code_templates.rs: FOUND
- ferro-mcp/src/tools/generation_context.rs: FOUND

Commit exists: 9f82c44a (verified via git log)

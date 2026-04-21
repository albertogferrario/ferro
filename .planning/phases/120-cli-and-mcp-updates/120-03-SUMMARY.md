---
phase: 120
plan: "03"
subsystem: ferro-mcp
tags: [mcp, json-ui, catalog, json-schema]
dependency_graph:
  requires: [phase-117-catalog-and-json-schema]
  provides: [json_ui_catalog-json-schema-fields]
  affects: [ferro-mcp]
tech_stack:
  added: []
  patterns: [catalog-accessor-delegation]
key_files:
  modified:
    - ferro-mcp/src/tools/json_ui_catalog.rs
decisions:
  - "component_schemas always covers all components regardless of filter argument (sourced from global_catalog(), not from the filtered component list)"
metrics:
  duration: "~5 min"
  completed: "2026-04-21"
  tasks_completed: 1
  files_modified: 1
---

# Phase 120 Plan 03: json_ui_catalog JSON Schema Fields Summary

Added `json_schema` and `component_schemas` fields to the MCP `json_ui_catalog` tool so downstream agents get machine-readable schemas in a single MCP call without needing a second tool.

## What Was Built

**`JsonUiCatalog` struct additions** in `ferro-mcp/src/tools/json_ui_catalog.rs`:

- `json_schema: serde_json::Value` — full v2 spec JSON Schema document sourced from `global_catalog().json_schema()`. Validates a complete JSON-UI spec: requires `$schema`, `root`, `elements`; `$defs` contains `Element` (discriminated `oneOf` over all 39 built-in + plugin Props), `Action`, `Visibility`.
- `component_schemas: HashMap<String, serde_json::Value>` — per-component Props-only schemas keyed by type name. Covers all built-in (39) and plugin components (Map + any registered). Agents use these as structured-output constraints when generating element props for a specific component.

**Population logic** in `execute()`:
- `json_schema`: delegated to `cat.json_schema().clone()` — zero extra computation, reuses the pre-built schema from the Phase 117 catalog.
- `component_schemas`: single chained iterator over `components_sorted()` + `plugin_components_sorted()`, collecting `(name, props_schema.clone())` pairs. Not filtered by the `component` argument — always returns the full map regardless of component filter.

**Tests added** (6 new, 1 updated):
- `test_serialization`: updated to assert `json_schema` and `component_schemas` keys are present in serialized JSON.
- `test_json_schema_is_full_spec_schema`: verifies `$id` = `"ferro-json-ui/v2"` and required fields.
- `test_component_schemas_covers_all_builtins`: asserts all 39 named built-ins have entries; count = built-ins + plugins.
- `test_component_schemas_are_props_only`: Card schema has `title` in `properties`, does NOT have Element envelope shape.
- `test_component_schemas_includes_plugin_components`: Map plugin appears in `component_schemas`.
- `test_filter_returns_all_schema_fields`: even when filtering by `Button`, `json_schema` is still the full spec schema and `component_schemas` still includes `Card`.

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add json_schema + component_schemas to JsonUiCatalog | 87e1d001 | ferro-mcp/src/tools/json_ui_catalog.rs |

## Deviations from Plan

None — plan executed exactly as written. D-04 from the CONTEXT specified the exact field names, types, and population strategy; implementation followed it directly.

## Known Stubs

None. Both fields are fully wired to `global_catalog()` which is populated at first call.

## Threat Flags

None. This change only adds read-only fields to an existing MCP response struct. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Self-Check: PASSED

- ferro-mcp/src/tools/json_ui_catalog.rs: FOUND
- Commit 87e1d001: FOUND
- SUMMARY.md: FOUND
- All 210 ferro-mcp tests passing (cargo test -p ferro-mcp --all-features)
- cargo clippy -p ferro-mcp --all-targets -- -D warnings: PASSED (0 warnings)

---
phase: 218-write-tool-rendering-from-actiondef
plan: "01"
subsystem: ferro-mcp-server
tags: [tdd, green-tests, mcp, write-tools, actiondef, schema, sensitive-exclusion]
dependency_graph:
  requires: [218-00-red-tests]
  provides: [218-01-schema-green, build_action_input_schema]
  affects: [ferro-mcp-server/src/schema.rs]
tech_stack:
  added: []
  patterns: [tdd-green-wave, single-source-of-truth, sensitive-field-exclusion]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/schema.rs
decisions:
  - "Promoted data_type_to_json_schema to pub(crate) — single DataType→JSON mapping shared between read (build_input_schema) and write (build_action_input_schema) paths"
  - "InputDef is only needed in test scope; kept top-level import minimal (ActionDef, DataType, FieldDef, FieldMeaning, ServiceDef) and placed InputDef in the #[cfg(test)] mod import"
  - "Identifier injection is a silent no-op when the service has no Identifier field — some actions create new records and have no record ID to inject"
metrics:
  duration_seconds: 97
  completed_date: "2026-06-13T20:41:18Z"
  tasks_completed: 1
  files_modified: 1
---

# Phase 218 Plan 01: build_action_input_schema Implementation — Summary

One-liner: Implement `build_action_input_schema` in schema.rs — reusing `pub(crate) data_type_to_json_schema`, injecting the Identifier field, excluding Sensitive inputs — turning all 5 Plan 00 schema RED tests GREEN.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Promote data_type_to_json_schema and implement build_action_input_schema (SC#2, T-218-01) | 39eeb9fb | ferro-mcp-server/src/schema.rs |

## What Changed

### `ferro-mcp-server/src/schema.rs`

Three changes in one file:

1. **Top-level import extended:** `ActionDef` added to `use ferro_projections::{...}` — provides the function signature type for `build_action_input_schema`.

2. **`data_type_to_json_schema` promoted:** `fn` → `pub(crate) fn` (single-token change). This is the D-02 single-source-of-truth guarantee: no duplicated DataType match table exists. `grep -c "DataType::Integer =>" schema.rs` returns 1.

3. **`build_action_input_schema` added** (64 new lines including doc comment): derives the write-tool `inputSchema` from `ActionDef.inputs`. Implementation:
   - Finds the first `FieldMeaning::Identifier` field in `service.fields`, injects it as a required integer property with description "ID of the {display_name} record to act on".
   - Iterates `action.inputs`, skipping any with `FieldMeaning::Sensitive` (T-218-01 mitigation).
   - Maps each non-sensitive input via `data_type_to_json_schema(input.data_type)`, forwards `input.description`, adds to `required[]` iff `input.required`.
   - Returns `{ "type": "object", "properties": {...}, "required": [...] }`.

## Test Results

```
running 10 tests
test schema::tests::test_action_schema_excludes_sensitive_input ... ok
test schema::tests::test_action_schema_injects_identifier ... ok
test schema::tests::test_action_schema_maps_inputs ... ok
test schema::tests::test_action_schema_no_identifier_field_is_silent_noop ... ok
test schema::tests::test_action_schema_optional_input_not_required ... ok
test schema::tests::test_input_schema_derivation ... ok
test schema::tests::test_pagination_params_in_schema ... ok
test schema::tests::test_sensitive_field_excluded ... ok
test schema::tests::test_write_only_excluded ... ok
test schema::tests::test_write_only_excluded ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out
```

All 5 Plan 00 schema RED tests are GREEN. All 5 pre-existing schema tests remain GREEN.

## Crate Compile State

`ferro-mcp-server` now **compiles**. The Plan 00 intentional compile-error (missing `build_action_input_schema`) is resolved.

Renderer and jsonrpc tests (Plans 00 SC#1/SC#3/SC#4/SC#5) compile but remain assertion-RED — write tools are not yet emitted by `render_exposed_tools`. These are deferred to Plan 02 as designed.

## Deviations from Plan

None — plan executed exactly as written.

The test import was consolidated from `use ferro_projections::{ActionDef, DataType, FieldMeaning, InputDef, ServiceDef};` to `use ferro_projections::InputDef;` since `ActionDef`, `DataType`, `FieldMeaning`, `ServiceDef` are already in scope via `use super::*`. This keeps the import minimal and avoids clippy `unused_imports` warnings — consistent with how Plan 00 handled the same issue.

## Security Coverage

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-218-01: Sensitive input disclosure | `matches!(input.meaning, FieldMeaning::Sensitive) { continue; }` in build_action_input_schema | GREEN — verified by `test_action_schema_excludes_sensitive_input` |
| T-218-03: Malformed schema shape | Always emits `{type:"object", properties, required}` | Partial — strict rmcp deser proof deferred to Plan 02 SC#5 |

## Known Stubs

None. The function is fully implemented. Renderer and jsonrpc tests remain assertion-RED because `render_exposed_tools` does not yet emit write tools — that is Plan 02 work, not a stub in this plan's scope.

## Self-Check: PASSED

- FOUND: ferro-mcp-server/src/schema.rs
- FOUND: commit 39eeb9fb
- FOUND: `pub(crate) fn data_type_to_json_schema`
- FOUND: `pub fn build_action_input_schema`
- DataType match table count: 1 (single source of truth confirmed)
- Schema tests: 10/10 GREEN

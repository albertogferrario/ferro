---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
plan: "01"
subsystem: ferro-projections
tags: [servicedef, metadata, tenant-scoping, mcp-policy, tdd]
dependency_graph:
  requires: []
  provides: [ServiceDef.tenant_column, ServiceDef.mcp_ability]
  affects: [ferro-mcp-server/dispatch, app/controllers/mcp, app/projections/order]
tech_stack:
  added: []
  patterns: [skip_serializing_if Option::is_none, consuming builder mut self -> Self]
key_files:
  created: []
  modified:
    - ferro-projections/src/service.rs
    - docs/protocol/schemas/service-def.json
    - docs/protocol/schemas/protocol.json
    - docs/protocol/schemas/ (remaining schema files regenerated)
decisions:
  - "tenant_column and mcp_ability use skip_serializing_if (not serde default) — Option<String> fields that should be absent from JSON when None, consistent with display_name/description"
  - "Fields placed immediately after mcp_exposed to cluster MCP-related metadata together"
  - "Protocol schemas regenerated as part of task — generate_protocol_schemas integration test runs on cargo test"
metrics:
  duration: "~4 minutes"
  completed: "2026-06-10T18:47:24Z"
  tasks_completed: 1
  files_changed: 19
requirements: [AMCP-10, AMCP-11]
---

# Phase 200 Plan 01: ServiceDef tenant_column + mcp_ability Metadata Fields Summary

Two plain-metadata `Option<String>` fields added to `ServiceDef` — `tenant_column` (FK column for tenant-scoped dispatch) and `mcp_ability` (Gate ability name for MCP authorization) — establishing the contract the rest of Phase 200 implements against.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add tenant_column and mcp_ability fields to ServiceDef | aa15ec57 | ferro-projections/src/service.rs + 18 schema files |

## What Was Built

`ServiceDef` in `ferro-projections/src/service.rs` gained two new optional fields:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub tenant_column: Option<String>,

#[serde(skip_serializing_if = "Option::is_none")]
pub mcp_ability: Option<String>,
```

With corresponding consuming builder methods:

```rust
pub fn tenant_column(mut self, col: impl Into<String>) -> Self { ... }
pub fn mcp_ability(mut self, ability: impl Into<String>) -> Self { ... }
```

Three new tests added:
- `tenant_and_ability_default_none_when_absent` — deserialization from minimal JSON leaves both `None`
- `tenant_column_and_mcp_ability_builder_sets_values` — builder chain sets expected values
- `tenant_column_and_mcp_ability_skip_serializing_when_none` — serialization omits absent keys

## TDD Gate Compliance

RED phase confirmed: compile error (`no field tenant_column`, `method not found`) before implementation.
GREEN phase: all 3 new tests pass after adding fields, initializers, and builder methods.

## Verification Results

- `cargo test -p ferro-projections`: 234 unit tests + 1 integration test + 8 doc-tests = 243 total, all pass
- `cargo clippy -p ferro-projections --all-targets -- -D warnings`: clean
- `cargo fmt -p ferro-projections -- --check`: clean (applied fmt before commit)
- `cargo tree -p ferro-projections | grep -E '^\s*(ferro|framework) '`: no output (OK — no framework dep)
- All 4 grep acceptance criteria: PASS

## Deviations from Plan

None — plan executed exactly as written. Protocol schema files were regenerated as a natural side effect of the `generate_protocol_schemas` integration test running during `cargo test`. These are tracked schema artifacts and their update is correct.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. Both fields are plain `Option<String>` schema metadata on an existing struct. Serialization uses `skip_serializing_if` (D-T-200-INFO: field declarations, not data, safe to serialize). No framework/ferro dependency added (D-T-200-COUPLE: verified via `cargo tree`).

## Self-Check

- [x] `ferro-projections/src/service.rs` — modified, committed in aa15ec57
- [x] `docs/protocol/schemas/service-def.json` — contains `tenant_column` and `mcp_ability`
- [x] Commit aa15ec57 exists: `git log --oneline | grep aa15ec57`

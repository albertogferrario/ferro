---
phase: 240-crud-input-schema-derivation-list-query-polish
plan: "02"
subsystem: mcp-schema
tags: [ferro-mcp-server, schema-derivation, crud, write-boundary, list-query, range-filters]

requires:
  - phase: 240-01
    provides: "ServiceDef::is_write_excluded_field predicate"

provides:
  - "pub fn is_range_filter_field(field: &FieldDef) -> bool"
  - "pub fn build_create_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>"
  - "pub fn build_update_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>"
  - "pub fn build_delete_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>"
  - "build_input_schema extended with __ne/__in/__gt/__gte/__lt/__lte/sort params"

affects:
  - "240-03 — renderer.rs will call build_create/update/delete_input_schema to emit CRUD tools"
  - "240-04 — dispatch.rs will consume is_range_filter_field for runtime filter enforcement"

tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN within each task — RED commit first (failing test), then GREEN commit (implementation)"
    - "DataType-based gate in is_range_filter_field vs meaning-based gate in is_filter_field — deliberate divergence so Money/Quantity/Percentage get range params"
    - "Shared is_write_excluded_field predicate between create and update — single source of truth for agent-writable field set (T-240-04)"
    - "Patch semantics in update: identifier injected as sole required; all data fields optional"

key-files:
  created: []
  modified:
    - ferro-mcp-server/src/schema.rs

key-decisions:
  - "is_range_filter_field uses DataType gate (not meaning gate) so Money/Quantity/Percentage float fields get range comparisons even though is_filter_field's meaning allowlist excludes them (D-10)"
  - "build_create_input_schema does not inject the Identifier — a new record has no id yet; update and delete inject it as the sole required param"
  - "confirmation_token appears in build_delete_input_schema properties but NOT in required[] — schema advertises the parameter; enforcement is Phase 241/242 (D-08)"
  - "sort param inserted into build_input_schema properties unconditionally — dispatch.rs Plan 04 handles validation and SQL ORDER BY"

requirements-completed: [CRUD-01, CRUD-02, CRUD-04]

duration: 5min
completed: "2026-06-23"
---

# Phase 240 Plan 02: CRUD Input-Schema Derivation + List Query Polish Summary

**`is_range_filter_field` + three write-schema builders (`build_create/update/delete_input_schema`) + `build_input_schema` extended with `__ne`/`__in`/`__gt`/`__gte`/`__lt`/`__lte`/`sort` params — the schema-derivation core that makes the agent write surface correct and the list query surface richer**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-23T17:21:17Z
- **Completed:** 2026-06-23T17:26:07Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

### Task 1: is_range_filter_field + extended build_input_schema

- `is_range_filter_field` added immediately after `is_filter_field` with its own DataType-based gate 5 (Integer/Float/DateTime/Date); deliberately does NOT call `is_filter_field` — D-10 requires Money/Quantity/Percentage to pass
- `build_input_schema` extended with three new param blocks after the existing equality loop:
  - `__ne` + `__in` for every `is_filter_field` field (D-09); `__in` has `type: array` with `items` set to the scalar type
  - `__gt/__gte/__lt/__lte` for every `is_range_filter_field` field (D-10)
  - `sort` param (`type: string`, prefix `-` for descending) (D-11)
- Equality params and `limit`/`offset` untouched (D-02 back-compat)
- Three named tests: `test_range_params_in_schema`, `test_ne_in_params_in_schema`, `test_sort_param_in_schema`; plus `test_existing_params_backcompat`

### Task 2: build_create/update/delete_input_schema builders

- `build_create_input_schema`: iterates `service.fields`, skips any field where `service.is_write_excluded_field(field, exclude_sm_status)` returns true; no Identifier injection (create has no id); `required[]` populated for `field.required` data fields
- `build_update_input_schema`: injects Identifier first as sole required param (T-240-05 / Pitfall 7); then data fields added to properties only — patch semantics (D-06)
- `build_delete_input_schema`: injects Identifier as required; inserts optional `confirmation_token` in properties (D-08); enforcement Phase 241/242
- Both create and update call `service.is_write_excluded_field` — shared predicate, no drift (T-240-04)
- Five table tests: `test_create_schema_exclusions`, `test_create_schema_status_sm`, `test_update_schema_patch_semantics`, `test_update_schema_status_sm`, `test_delete_schema`

## Task Commits

1. **Task 1 RED** — `40cda6d8`: test(240-02): add RED tests for is_range_filter_field + range/ne/in/sort schema params
2. **Task 1 GREEN** — `4731e99f`: feat(240-02): add is_range_filter_field + extend build_input_schema with range/ne/in/sort params
3. **Task 2 RED** — `05c8a121`: test(240-02): add RED tests for build_create/update/delete_input_schema builders
4. **Task 2 GREEN** — `882941e5`: feat(240-02): add build_create/update/delete_input_schema write-schema builders

## Files Created/Modified

- `ferro-mcp-server/src/schema.rs` — `is_range_filter_field` (lines 52–69), extended `build_input_schema` (lines 132–178), `build_create_input_schema` (lines 249–283), `build_update_input_schema` (lines 284–336), `build_delete_input_schema` (lines 337–374), plus 8 new tests

## Decisions Made

- `is_range_filter_field` has its own gate 5 (DataType-based), not delegating to `is_filter_field`. The meaning allowlist in `is_filter_field` deliberately excludes Money/Quantity/Percentage for equality (not useful to filter `total = 100`), but range comparisons on those types (`total__gt = 50`) are useful. Two separate predicates preserve both semantics.
- `build_create_input_schema` does not inject the Identifier — on create the id doesn't exist yet. Update and delete inject it as the sole required param in required[].
- `confirmation_token` in the delete schema is optional (not required). The schema advertises the parameter so agents know to request a token first; the actual enforcement (rejecting un-tokenized deletes) is Phase 241/242.

## Deviations from Plan

None — plan executed exactly as written. The TDD RED/GREEN cycle for both tasks proceeded without incident. Rustfmt required expanding one compact assert in the backcompat test (`assert_eq!(props["limit"]["default"], 25, ...)` → multi-line form); applied `cargo fmt --all` before final commit; no logic change.

## Known Stubs

None. All four functions are fully implemented — no hardcoded empty values, placeholder text, or unwired data flows. The `confirmation_token` in `build_delete_input_schema` is intentionally schema-only (not enforced at runtime); this is documented as D-08 and is not a stub — enforcement is Phase 241/242's explicit scope.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes at trust boundaries. Both mitigations from the threat register (T-240-04 and T-240-05) are implemented and covered by tests.

---
*Phase: 240-crud-input-schema-derivation-list-query-polish*
*Completed: 2026-06-23*

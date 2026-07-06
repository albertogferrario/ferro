---
phase: 117-catalog-and-json-schema
plan: "04"
subsystem: ferro-json-ui
tags: [catalog, validation, pre-dispatch, jsonschema, three-stage-pipeline]
dependency_graph:
  requires: [117-03 full_schema assembly, cached jsonschema::Validator (SCHEMA-03)]
  provides: [Catalog::validate(&Spec) -> Result<(), Vec<CatalogError>>]
  affects: [ferro-json-ui/src/catalog.rs]
tech_stack:
  added: []
  patterns: [three-stage validation pipeline, Stage 1 short-circuit, on-demand per-component validator_for, cached full-spec validator reuse]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/catalog.rs
decisions:
  - "instance_path is a method on ValidationError (not a field) in jsonschema 0.46 — call as err.instance_path()"
  - "null props are skipped in Stage 2 — Value::Null means no props provided; required field enforcement is the schema gate, not null rejection"
  - "build_builtins_only() reused for validate() tests to avoid BadPlugin_117 pollution (same pattern as Plan 03)"
  - "Stage 1 short-circuit proven by combining unknown type with bad envelope — only UnknownType surfaces, not SpecInvalid"
  - "#[allow(dead_code)] removed from Catalog struct — fields consumed by validate() and json_schema()"
metrics:
  duration: "~5 minutes"
  completed: "2026-04-18T13:33:54Z"
  tasks_completed: 1
  files_changed: 1
---

# Phase 117 Plan 04: Catalog::validate Three-Stage Pipeline Summary

Implemented `Catalog::validate(&self, spec: &Spec) -> Result<(), Vec<CatalogError>>` with a three-stage pipeline satisfying CAT-03 and SCHEMA-03. All 7 new validation tests pass; full suite (376 lib + 19 integration + 5 doc-tests) clean.

## What Was Built

**`ferro-json-ui/src/catalog.rs`** (+281 lines from Plan 03):

### `Catalog::validate(&self, spec: &Spec) -> Result<(), Vec<CatalogError>>`

Three-stage pipeline:

**Stage 1 — type_name whitelist (O(1) per element)**

Iterates `spec.elements` and checks each `element.type_name` against `self.components` and `self.plugin_components`. First unknown type pushes `CatalogError::UnknownType { element_id, type_name }`. If any unknown type found: **short-circuit return** — Stages 2 and 3 do not run. This prevents the full-spec oneOf from emitting dozens of noisy "no variant matched" errors when the real problem is a typo in a type name.

**Stage 2 — per-element Props validation**

For each element, looks up `self.per_component_schemas.get(&el.type_name)`. Skips null props (`Value::Null` means no props object was provided — required fields are the gate, not null rejection). Calls `jsonschema::validator_for(schema)` on demand (CONTEXT D-12 escape hatch for future precompilation). `iter_errors` collects per-elem errors as `"{instance_path()}: {err}"` strings. Pushes `CatalogError::PropsInvalid` if any. Errors accumulate across all elements.

**Stage 3 — full-spec envelope validation (SCHEMA-03)**

Serializes the full `Spec` via `serde_json::to_value`. Runs `self.validator.iter_errors(&spec_value)` — the cached `jsonschema::Validator` compiled once in `Catalog::build()`. Envelope errors become `CatalogError::SpecInvalid { errors: Vec<String> }`. Errors accumulate alongside Stage 2 errors.

### Tests added (7 new, all passing)

| Test | What it proves |
|------|----------------|
| `validate_positive_per_type` | Text, Button, Badge, Separator pass with minimal valid props |
| `validate_unknown_type` | `UnknownType` returned for unrecognized type name |
| `validate_missing_required_prop` | `PropsInvalid` for `Card` with empty props (missing required `title`) |
| `validate_bad_schema_version` | `SpecInvalid` when `$schema` doesn't match `"ferro-json-ui/v2"` const |
| `validate_pre_dispatch_short_circuits` | Unknown type + bad envelope → only `UnknownType` surfaces (Stages 2 & 3 not run) |
| `validator_is_cached_not_recompiled` | 100 validate() calls on one Catalog, no panic or regression |
| `validate_accumulates_multiple_errors_across_elements` | Card (missing title) + Button (missing label) → 2 PropsInvalid errors |

### Dead code annotation removed

`#[allow(dead_code)]` on `Catalog` struct removed — `components`, `plugin_components`, `per_component_schemas`, and `validator` are all consumed by `validate()`.

## Props Types with Required Fields (useful for test targeting)

| Component | Required fields (non-Option, no #[serde(default)]) |
|-----------|-----------------------------------------------------|
| `CardProps` | `title: String` |
| `ButtonProps` | `label: String` |
| `InputProps` | `field: String`, `label: String` |
| `TableProps` | `columns: Vec<Column>`, `data_path: String` |
| `TextProps` | `content: String` (likely — drives the test helper) |

`CardProps.title` was used for `validate_missing_required_prop` — confirmed required by schemars (no `Option<>`, no `#[serde(default)]`).

## jsonschema 0.46 API Notes

- `err.instance_path` is a **method** not a field — call as `err.instance_path()`. The plan noted this as a potential issue; it was the only compile error encountered.
- `validator.iter_errors(&value)` returns an iterator of `ValidationError` — works exactly as described in RESEARCH §5.

## Cache Reuse Verification (SCHEMA-03)

`grep 'validator_for.*full_schema' ferro-json-ui/src/catalog.rs` returns exactly 2 hits:
- Line 589: `Catalog::build()` — compiles and stores `self.validator`
- Line 761: `build_builtins_only()` — test-only equivalent

Neither hit is inside `validate()`. `validate()` calls only `self.validator.iter_errors(...)`. Cache reuse is structural, not behavioral — the field is a `jsonschema::Validator` stored on the struct.

The `validator_is_cached_not_recompiled` test runs 100 validate() calls without regression. No wall-clock measurement captured; compile cost is zero (no `validator_for` call inside validate).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `instance_path` is a method, not a field**
- **Found during:** First compile attempt
- **Issue:** Plan template used `err.instance_path` (field access). jsonschema 0.46 exposes `instance_path()` as a method returning a `Display`-friendly path.
- **Fix:** Changed both occurrences to `err.instance_path()` (per-element and envelope loops).
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 01b114f8

## Verification Results

```
cargo fmt --all -- --check                                              → clean (after rustfmt auto-fix)
cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings → clean
cargo test -p ferro-json-ui --lib catalog::                             → 26 passed, 0 failed
cargo test -p ferro-json-ui --all-features                              → 376 lib + 11 + 8 + 5 = all ok
```

SCHEMA-03 verified: `validate()` does not call `jsonschema::validator_for` on the full schema — it reuses `self.validator`.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. Validation errors contain instance paths and jsonschema messages only — no user-supplied values echoed back.

## Next Step

Plan 05 implements `catalog.prompt()` + `catalog.component_schema(name)`.

## Self-Check: PASSED

- `ferro-json-ui/src/catalog.rs` — FOUND (confirmed 1276 lines post-Plan 04)
- Commit `01b114f8` — FOUND
- `pub fn validate(&self, spec: &crate::spec::Spec) -> Result<(), Vec<CatalogError>>` — FOUND
- `// === Stage 1:` in validate — FOUND
- `// === Stage 2:` in validate — FOUND
- `// === Stage 3:` in validate — FOUND
- `self.validator.iter_errors(` in validate — FOUND
- No `validator_for.*full_schema` inside validate — CONFIRMED (grep returns 0 hits inside validate)
- All 7 validate tests — PASS

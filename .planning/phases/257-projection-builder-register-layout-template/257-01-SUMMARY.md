---
phase: 257-projection-builder-register-layout-template
plan: "01"
subsystem: ferro-json-ui
tags: [spec-builder, catalog-validation, each-directive, fill-viewport, tdd]
dependency_graph:
  requires: []
  provides: [ElementBuilder.each, SpecBuilder.fill_viewport, catalog-each-guard]
  affects: [ferro-json-ui/src/spec.rs, ferro-json-ui/src/catalog.rs]
tech_stack:
  added: []
  patterns: [consuming-setter, tdd-red-green, catalog-validation-guard]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/src/catalog.rs
decisions:
  - "Stage 3 template guard: remove props key entirely from envelope copy (not null-out) — the component oneOf arm validates props against required-field schemas; null fails, absent is skipped"
metrics:
  duration_minutes: 23
  completed_date: "2026-07-06"
  tasks_completed: 3
  files_modified: 2
requirements: [POS-10]
---

# Phase 257 Plan 01: Projection Builder Register Layout Template — Primitives Summary

Two consuming setters and a targeted catalog validation fix: the self-contained ferro-json-ui primitives that Plan 02's `emit_register_root` depends on.

## What Was Built

**ElementBuilder.each(path, as_) — D-12.** Public consuming setter on `ElementBuilder` setting the `$each` iteration directive. Serializes to `{"$each": {"path": "..", "as": ".."}}` via the existing `EachDirective` serde rename; round-trips cleanly through deserialization. Placed alongside the existing `prop`/`child` setters in the `impl ElementBuilder` block; `NestedElement` deliberately unchanged (Phase 163 deferral note stands).

**SpecBuilder.fill_viewport(bool) — D-13.** Public consuming setter on `SpecBuilder`. Added `fill_viewport_: bool` field (defaults to `false`), setter method, and changed the hardcoded `fill_viewport: false` in `build()` to `fill_viewport: self.fill_viewport_`. All existing callers are unaffected (default false preserved).

**Catalog::validate $each template-element guard — D-14.** Fixed the pre-existing limitation that rejected `$each` template elements whose data-bound props include non-String fields (e.g. `TileProps.price_cents: Option<u64>`).

- **Stage 2** (per-element Props validation): Added `if el.each.is_some() { continue; }` immediately after the existing null-props guard. `strip_expr_objects` turns `{"$data":..}` into `""` which passes String schemas but fails `anyOf[integer,null]`; template elements skip schema validation because concrete types are only knowable after `$each` expansion at render time.
- **Stage 3** (full-spec envelope validation): For each template element in the serialized envelope copy, `props` key is **removed** (not null-ed). The envelope schema's component oneOf arm validates `props` against required-field schemas — null fails those required fields; absent is simply skipped by the `additionalProperties` validator. `strip_expr_objects` is unchanged.

## Tasks

| Task | Commit | Result |
|------|--------|--------|
| 1: ElementBuilder.each setter (D-12) | 65577123 | TDD green |
| 2: SpecBuilder.fill_viewport setter (D-13) | 65577123 | TDD green |
| 3: Catalog $each template guard (D-14) | 701bcaf7 | Tests green |

## Tests Added

- `spec::tests::each_builder_round_trip` — builds Element via `.each("/data/items","p")`, asserts `$each` JSON key with `path`/`as`, round-trips through serde.
- `spec::tests::fill_viewport_builder` — asserts both `true` (explicit set) and `false` (default) paths.
- `catalog::tests::catalog_each_template_null_data` — TileGrid + Tile `$each` template with `$data`-bound `price_cents`; `spec.data = null`; `catalog.validate` returns `Ok`.
- `catalog::tests::catalog_each_template_populated_data` — same spec with populated data array; covers the `validate_directives` path-resolves-to-array branch.

## Deviations from Plan

**1. [Rule 1 - Bug] Stage 3 guard: remove props instead of null-out**

- **Found during:** Task 3 first test run
- **Issue:** The plan specified `obj.insert("props", Value::Null)`. Setting props to null still fails the component oneOf because each component variant's schema has `required: ["item_id", "name", ...]` on the props object — null does not satisfy those required fields.
- **Fix:** Used `obj.remove("props")` instead. The envelope schema treats props as optional on the element shape (no `required: ["props"]` in the element allOf); removing the key causes the validator to skip props validation entirely.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 701bcaf7

## CI Gate

Full CI-exact gate green:
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets --all-features -- -D warnings` — clean (50s)
- `cargo test --all-features` — 757 tests pass, exit 0
- `cargo doc --no-deps -p ferro-json-ui` — clean
- Schema export: no churn (docs/protocol/schemas/ unchanged)

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The `$each` validation bypass is strictly scoped to `each.is_some()` per T-257-01; `validate_directives` enforces structural $each rules; `resolve_expressions` enforces concrete types at render time. No threat flags.

## Self-Check: PASSED

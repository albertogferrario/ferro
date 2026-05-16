---
phase: 163
plan: "04"
subsystem: ferro-json-ui
tags: [validation, spec, directives, each, if, error-handling]
dependency_graph:
  requires: [163-01, 163-02, 163-03]
  provides: [directive-validation-gate]
  affects: [ferro-json-ui/src/spec.rs]
tech_stack:
  added: []
  patterns: [fail-fast-validation, best-effort-path-checking, reserved-name-list]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs
decisions:
  - "Nested $each rejected at validation time as NestedEach (planner decision #3)"
  - "Mismatched-each children (direct children with different path/as) rejected as MismatchedEach (planner decision #4)"
  - "Path resolution checks are best-effort: skipped when spec.data is null"
  - "validate_directives inserted between validate_no_dangling and detect_cycle"
metrics:
  duration: "158s"
  completed: "2026-05-16"
  tasks: 1
  files: 1
---

# Phase 163 Plan 04: Directive Validator Gates Summary

Added parse-time validation of `$each` and `$if` directives to `Spec::from_json` and `Spec::builder().build()`. Bad specs now fail fast with typed, actionable errors before any data binding or expansion occurs.

## What Was Built

Five new `SpecError` variants and a `validate_directives` function called from `validate_structure` between `validate_no_dangling` and `detect_cycle`.

### New SpecError Variants

- `EachPathNotArray { element_id, path }` — `$each.path` resolves to a non-array value in `spec.data`
- `IfPathMissing { element_id, path }` — `$if.path` references a key absent from `spec.data`
- `EachAsReservedName { element_id, name }` — `$each.as` is one of six reserved names
- `NestedEach { outer, inner }` — transitive descendant of an `$each` element is also `$each`-templated
- `MismatchedEach { parent, parent_path, child, child_path }` — direct child has `$each` with different `{path, as}` than its parent

### Implementation Details

- `RESERVED_EACH_AS` constant: `["data", "root", "_root", "_each", "this", "self"]`
- `validate_directives(spec)` performs four checks per `$each` element:
  1. Reserved-name check on `as`
  2. Path-to-array check against `spec.data` (best-effort, skipped when `data` is null)
  3. Mismatched-each direct-child check
  4. Nested-each transitive-descendant check (grandchildren and deeper)
- `check_visibility_paths` helper walks the `Visibility` tree recursively to check every condition path
- `$if` path checking is also best-effort: skipped when `spec.data` is null

### Call Order in validate_structure

```
validate_ids → RootMissing → validate_no_dangling → validate_directives → validate_footer_ids → detect_cycle → check_depth
```

### Tests

11 inline unit tests added to `spec.rs`, covering:
1. `validate_each_path_not_array_fires` — non-array path with data present
2. `validate_each_path_not_array_skipped_when_data_null` — best-effort skip
3. `validate_each_as_reserved_data_rejected` — `"data"` reserved name
4. `validate_each_as_reserved_root_rejected` — `"root"` reserved name
5. `validate_each_as_non_reserved_accepted` — `"order"` and `"row"` are valid
6. `validate_if_path_missing_fires` — missing key with data present
7. `validate_if_path_missing_skipped_when_data_null` — best-effort skip
8. `validate_nested_each_rejected` — transitive descendant (A → mid → B, B has `$each`)
9. `validate_mismatched_each_child_rejected` — direct child with different path
10. `validate_correlated_each_child_accepted` — same `{path, as}` on parent and child is valid
11. `validate_directives_called_between_no_dangling_and_cycle` — structural assertion via `include_str!`

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add 5 SpecError variants + validate_directives + 11 tests | e99776cf | ferro-json-ui/src/spec.rs |

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` exists and contains all 5 new variants
- Commit `e99776cf` exists in git log
- All 444 tests pass (`cargo test -p ferro-json-ui --all-features`)
- `cargo clippy --all --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean

---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: "05"
subsystem: ferro-json-ui
tags: [spec-builder, nested-dsl, ergonomics, tdd]
decisions_addressed: [D-06, D-07]

dependency_graph:
  requires: [163-01, 163-02]
  provides: [NestedElement, SpecBuilder::element_nested, flatten_nested]
  affects: [ferro-json-ui/src/spec.rs]

tech_stack:
  added: []
  patterns:
    - "NestedElement: nested-tree builder type, flattens at build() time"
    - "flatten_nested: recursive walker producing {parent}-{idx} auto-IDs"
    - "D-07 contract: nested form lives only at build-call time; Spec.elements stays flat"

key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs

decisions:
  - "[D-06] NestedElement is a parallel type alongside ElementBuilder, not a .nest() method on ElementBuilder — type-level distinction makes the nested/flat boundary visible in the type system"
  - "[D-07] Runtime Spec shape is unchanged; nested form is builder sugar only"
  - "NestedElement does not expose $each/$if directive setters — Rust callers write loops directly"

metrics:
  duration: ~10min
  completed: "2026-05-16"
  tasks_completed: 1
  files_modified: 1
---

# Phase 163 Plan 05: NestedElement ergonomic nested-tree DSL Summary

NestedElement struct with child-DSL and SpecBuilder::element_nested that auto-flattens a nested element tree to the canonical flat Spec.elements map with {parent}-{idx} positional IDs at build time.

## What Was Built

`NestedElement` type added to `ferro-json-ui/src/spec.rs` (Section C2). It mirrors `ElementBuilder` but holds `children: Vec<NestedElement>` instead of `children: Vec<String>`. The private `flatten_nested` recursive walker converts the nested tree to the flat element map at `element_nested` call time — the runtime `Spec` shape is unchanged (D-07).

API surface added:
- `pub struct NestedElement` with `new`, `prop`, `child`, `action`, `visible` builder methods
- `pub fn SpecBuilder::element_nested(id, NestedElement) -> Self`
- `fn flatten_nested(elements, id, NestedElement)` (private, recursive)
- `NestedElement::build_for_test` (test-only, `#[cfg(test)]`)

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add NestedElement + SpecBuilder::element_nested + inline tests | 0fb5dadd | ferro-json-ui/src/spec.rs |

## Verification

All seven new tests pass:
- `nested_element_builder_basics` — leaf node with props, no children
- `nested_builder_flattens_one_level` — root + child auto-IDed as root-0
- `nested_builder_flattens_two_levels` — root > section > text at MAX_NESTING_DEPTH=3
- `nested_builder_auto_ids_match_position` — three siblings become parent-0, parent-1, parent-2
- `nested_builder_root_set_from_first_call` — first element_nested call sets root
- `nested_builder_preserves_action_and_visible` — action + visible pass through flatten_nested
- `nested_builder_and_flat_builder_produce_equivalent_specs` — serde_json::to_value round-trip equality

Existing flat API tests (Plan 03 resolve_actions) continue to pass.

`cargo test -p ferro-json-ui --all-features` exits 0.
`cargo build -p ferro-json-ui` exits 0.
`cargo clippy -p ferro-json-ui --all-targets -- -D warnings` clean.
`cargo fmt --all -- --check` clean.

## Deviations from Plan

**1. [Rule 1 - Bug] Fixed incorrect Visibility/Action type names in test 6**
- **Found during:** GREEN phase compilation
- **Issue:** Plan's test 6 sketch used `Action::Navigate { path }` (no such variant — Action is a struct with a `handler` field) and `crate::visibility::Condition` / `crate::visibility::Operator` (actual types are `VisibilityCondition` / `VisibilityOperator`)
- **Fix:** Used `Action::new("home.index")` and `VisibilityCondition { operator: VisibilityOperator::Exists, ... }`
- **Files modified:** ferro-json-ui/src/spec.rs (test only)
- **Commit:** 0fb5dadd (same commit, corrected before GREEN compile)

## Known Stubs

None.

## Threat Flags

None. `flatten_nested` auto-generates IDs via `{parent}-{idx}`; collision with caller-provided IDs surfaces as `SpecError::DuplicateId` via the existing `validate_structure` call (T-163-05-01 mitigated as planned).

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` exists and contains all required symbols
- Commit `0fb5dadd` exists in git log
- All 7 nested tests pass; full suite (473 tests) passes

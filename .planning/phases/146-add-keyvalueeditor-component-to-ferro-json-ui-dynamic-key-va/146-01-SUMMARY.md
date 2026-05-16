---
phase: 146
plan: "01"
subsystem: ferro-json-ui
tags: [tdd, red-tests, keyvalueeditor, component, runtime]
dependency_graph:
  requires: []
  provides: [146-01-red-tests]
  affects: [ferro-json-ui/src/render.rs, ferro-json-ui/src/component.rs, ferro-json-ui/src/runtime/mod.rs]
tech_stack:
  added: []
  patterns: [TDD RED phase, test-first component scaffolding]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render.rs
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/runtime/mod.rs
decisions:
  - "Tests reference KeyValueEditorProps and render_key_value_editor which do not yet exist — compile failure is the design (RED state)"
  - "SwitchProps missing-field errors are pre-existing and unrelated to this plan"
  - "runtime/mod.rs test arrays updated only; no mod declaration or SOURCE push added (those are Plan 03)"
metrics:
  duration: "~5 minutes"
  completed: "2026-04-22"
  tasks_completed: 3
  files_modified: 3
---

# Phase 146 Plan 01: RED Tests for KeyValueEditor Summary

Seven RED render tests, two RED serde round-trip tests, and two updated runtime test arrays establish the TDD contract for the KeyValueEditor component before any implementation lands.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add 7 render_key_value_editor tests to render.rs | 1986cdef | ferro-json-ui/src/render.rs |
| 2 | Add RED serde round-trip tests to component.rs | caad9d19 | ferro-json-ui/src/component.rs |
| 3 | Update runtime/mod.rs test arrays | e47e2e10 | ferro-json-ui/src/runtime/mod.rs |

## Test Counts

| File | Tests Added |
|------|-------------|
| ferro-json-ui/src/render.rs | 7 (`render_key_value_editor_*`) + helper `kv_props_minimal` |
| ferro-json-ui/src/component.rs | 2 (`key_value_editor_serde_roundtrip`, `key_value_editor_allow_custom_keys_defaults_to_true`) |
| ferro-json-ui/src/runtime/mod.rs | 0 new tests; 2 arrays updated with `"setupKeyValueEditor"` and `"setupKeyValueEditor();"` |

## RED State Verification

`cargo build -p ferro-json-ui --tests` fails with:

```
error[E0412]: cannot find type `KeyValueEditorProps` in this scope
error[E0422]: cannot find struct, variant or union type `KeyValueEditorProps` in this scope
```

Runtime tests (`bundle_contains_all_setup_functions`, `dispatcher_invokes_every_setup`) will fail with `bundle missing setupKeyValueEditor` once the compile errors from Plans 02/03 are resolved.

## TDD Gate Compliance

This plan is the RED gate for Phase 146. Plans 02 (implementation) and 03 (runtime module) flip these tests from RED to GREEN.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — this plan adds only test code.

## Threat Flags

None — test-only changes introduce no new network endpoints, auth paths, or schema changes.

## Self-Check: PASSED

- ferro-json-ui/src/render.rs: 7 `render_key_value_editor_*` functions confirmed (`grep -c` = 7)
- ferro-json-ui/src/component.rs: `mod key_value_editor_tests` + 2 test functions confirmed
- ferro-json-ui/src/runtime/mod.rs: `"setupKeyValueEditor"` count = 1, `"setupKeyValueEditor();"` count = 1
- Commits 1986cdef, caad9d19, e47e2e10 exist in git log
- Build fails with expected unresolved-name errors (RED state confirmed)

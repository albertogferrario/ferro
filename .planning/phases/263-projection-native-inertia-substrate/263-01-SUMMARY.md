---
phase: 263-projection-native-inertia-substrate
plan: "01"
subsystem: ferro-projections
tags: [projections, schema, derivation, serde]
dependency_graph:
  requires: []
  provides: [schema_contract, SchemaContract, FieldContract, ActionContract, InputContract]
  affects: [ferro-projections crate root]
tech_stack:
  added: []
  patterns: [pure-fn derivation sibling to derive_intents, From<&T> converters, serde skip directives]
key_files:
  created:
    - ferro-projections/src/schema_contract.rs
    - ferro-projections/tests/schema_contract.rs
  modified:
    - ferro-projections/src/lib.rs
decisions:
  - InputContract includes `meaning: FieldMeaning` field (present in InputDef; preserves full semantic vocabulary)
  - ActionContract.preconditions uses Vec<String> with serde default+skip_serializing_if (matches ActionDef pattern)
  - From<&FieldDef/ActionDef/InputDef> converters chosen over inline closures (idiomatic, testable)
metrics:
  duration: ~10 minutes
  completed_date: "2026-07-27"
  tasks_completed: 2
  files_changed: 3
requirements_satisfied: [SUBST-01]
---

# Phase 263 Plan 01: SchemaContract Types + Pure schema_contract Fn — Summary

Pure schema-level derivation (`schema_contract(&ServiceDef) -> SchemaContract`) added to `ferro-projections` as the natural sibling of `derive_intents`, exposing field access modes, meanings, action definitions (preconditions + inputs), and declared guards in a serializable contract.

## What Was Built

### `SchemaContract` shape (exact fields)

```rust
pub struct SchemaContract {
    pub name: String,
    pub display_name: Option<String>,          // #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Vec<FieldContract>,
    pub actions: Vec<ActionContract>,
    pub guards: Vec<String>,                   // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub has_state_machine: bool,
}

pub struct FieldContract {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    pub required: bool,
    pub readable: bool,
    pub writable: bool,
    pub is_list: bool,
}

pub struct ActionContract {
    pub name: String,
    pub display_name: Option<String>,          // #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,           // #[serde(skip_serializing_if = "Option::is_none")]
    pub preconditions: Vec<String>,            // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputContract>,            // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_transition: bool,
}

pub struct InputContract {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,                 // included — present in InputDef
    pub required: bool,
}
```

### `InputContract` fields as discovered from `action.rs`

`InputDef` has: `name: String`, `data_type: DataType`, `meaning: FieldMeaning`, `required: bool`, `description: Option<String>`.

`InputContract` maps all fields except `description` (omitted — not structurally meaningful for the schema contract; the contract describes type shape, not human docs). `meaning` is included (it is a first-class semantic field on `InputDef`, not boilerplate).

### Builder method substitutions in tests

No substitutions needed. All builder methods used (`read_only_field`, `optional_field`, `field`, `guard`, `action`, `precondition`) exist verbatim in `service.rs` and `action.rs`.

## Test Results

```
cargo test -p ferro-projections schema_contract
  4 unit tests (inline #[cfg(test)] mod):  PASS
  schema_contract::tests::schema_contract_field_set
  schema_contract::tests::read_only_field_has_correct_access_flags
  schema_contract::tests::action_preconditions_and_transition
  schema_contract::tests::schema_contract_serde_round_trip

cargo test -p ferro-projections --test schema_contract
  3 integration tests (ferro-projections/tests/schema_contract.rs):  PASS
  schema_contract_field_names_and_access
  schema_contract_actions_and_preconditions
  schema_contract_serde_round_trip
```

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 09edf23f | feat(263-01): add SchemaContract types and pure schema_contract fn |
| 2 | 92662fa6 | feat(263-01): add schema_contract integration test (SUBST-01) |

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. `schema_contract` is a pure synchronous transform of a developer-authored `ServiceDef` with no external input. No threat flags.

## Self-Check: PASSED

- `ferro-projections/src/schema_contract.rs` exists: FOUND
- `ferro-projections/tests/schema_contract.rs` exists: FOUND
- `ferro-projections/src/lib.rs` contains `pub use schema_contract::`: FOUND
- Commit 09edf23f exists in git log: FOUND
- Commit 92662fa6 exists in git log: FOUND
- No async/tokio/sea_orm in schema_contract.rs: CONFIRMED
- All 7 tests (4 unit + 3 integration) green: CONFIRMED

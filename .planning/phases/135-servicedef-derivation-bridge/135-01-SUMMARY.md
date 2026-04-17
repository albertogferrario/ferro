---
phase: 135-servicedef-derivation-bridge
plan: "01"
subsystem: ferro-projections
tags: [projections, derivation, metadata, service-def]
dependency_graph:
  requires: []
  provides: [ModelMetadata, FieldMetadata, ServiceDef::from_model, DataType::from_column_type]
  affects: [ferro-mcp]
tech_stack:
  added: []
  patterns: [intermediate-metadata-struct, type-inference-from-strings, snake-to-title-derivation]
key_files:
  created: []
  modified:
    - ferro-projections/src/field.rs
    - ferro-projections/src/service.rs
    - ferro-projections/src/lib.rs
decisions:
  - "ModelMetadata/FieldMetadata decouple ferro-projections from ORM types — callers populate from their own parsing"
  - "from_model() is an inherent method on ServiceDef, not a free function, for builder API consistency"
  - "snake_to_title() is private to service.rs — not part of the public surface"
metrics:
  duration: "~8min"
  completed: "2026-04-17"
  tasks: 2
  files_modified: 3
---

# Phase 135 Plan 01: ServiceDef Derivation Bridge — Core Types Summary

One-liner: ModelMetadata intermediate struct + DataType::from_column_type() type inference + ServiceDef::from_model() automatic derivation, enabling ORM-agnostic ServiceDef construction from field metadata.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add DataType::from_column_type() with tests | 34b3af09 | ferro-projections/src/field.rs |
| 2 | Add ModelMetadata, FieldMetadata, ServiceDef::from_model() | 61040ec0 | ferro-projections/src/service.rs, ferro-projections/src/lib.rs, ferro-projections/src/field.rs (fmt) |

## What Was Built

**Task 1 — DataType::from_column_type()**

Added an inherent method on `DataType` that maps Rust/SeaORM column type strings to the correct `DataType` variant. Key behaviors:
- Strips `Option<T>` wrappers before matching (`Option<String>` -> `String`)
- Maps all integer sizes (i8/i16/i32/i64/u8/u16/u32/u64) to `Integer`
- Maps `f32`/`f64`/`Decimal` to `Float`
- Maps `bool` to `Boolean`
- Maps `Uuid`/`uuid::Uuid` to `Uuid`
- Maps `DateTime<Utc>` and `chrono::*` patterns to `DateTime`
- Maps `NaiveDate` to `Date`
- Maps `Vec<u8>` to `Binary`
- Maps `Json`/`serde_json::Value` to `Json`
- Falls back to `String` for unrecognized types
- Two tests: `from_column_type_mappings` and `from_column_type_option_stripping`

**Task 2 — ModelMetadata, FieldMetadata, ServiceDef::from_model()**

- `ModelMetadata`: name, display_name, table, fields — ORM-agnostic intermediate representation
- `FieldMetadata`: name, column_type (raw string), is_primary_key, is_nullable
- `ServiceDef::from_model()`: derives a complete ServiceDef from ModelMetadata
  - Calls `DataType::from_column_type()` for type inference
  - Calls `infer_meaning()` for semantic field meaning
  - System fields (id, created_at, updated_at, or any primary key) get `writable: false`
  - `is_nullable: true` maps to `required: false`
  - Display name derived via `snake_to_title()` if not explicitly provided
- `snake_to_title()`: private helper, "order_item" -> "Order Item"
- Both types re-exported from ferro_projections crate root
- 6 tests: basic, system_fields_read_only, nullable_to_required, display_name_override, snake_to_title, round_trip_model_to_intents

## Verification

```
cargo fmt --all -- --check       ✓
cargo clippy --all --all-targets -- -D warnings   ✓
cargo test -p ferro-projections --all-features    ✓ (all 229+ tests pass)
```

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- ferro-projections/src/field.rs: FOUND — contains `pub fn from_column_type`
- ferro-projections/src/service.rs: FOUND — contains `pub struct ModelMetadata`, `pub struct FieldMetadata`, `pub fn from_model`
- ferro-projections/src/lib.rs: FOUND — contains `ModelMetadata` and `FieldMetadata` in pub use
- Commit 34b3af09: FOUND
- Commit 61040ec0: FOUND

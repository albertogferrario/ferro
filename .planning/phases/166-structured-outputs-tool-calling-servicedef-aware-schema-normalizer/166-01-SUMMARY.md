---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
plan: "01"
subsystem: ferro-ai
tags: [schema, schemars, ferro-projections, error-handling, wave-0, probe-tests]
dependency_graph:
  requires: [165-04]
  provides: [schema-module-shell, error-variants, probe-tests-A1-A2]
  affects: [ferro-ai]
tech_stack:
  added: [schemars-1.2.0, ferro-projections-dep, jsonschema-0.46-dev]
  patterns: [thiserror-variants, schemars-schema_for-to_value, tdd-probe]
key_files:
  created: [ferro-ai/src/schema/mod.rs]
  modified: [ferro-ai/Cargo.toml, ferro-ai/src/error.rs, ferro-ai/src/lib.rs]
decisions:
  - "Intent uses const-per-branch anyOf (not enum array) because variants have doc comments — closing algorithm must collect const values, not extract anyOf[0].enum"
  - "FieldMeaning uses enum-array anyOf (no per-variant docs) — closing algorithm extracts anyOf[0].enum directly"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-08T03:44:27Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 3
---

# Phase 166 Plan 01: Wave 0 Foundation — Deps, Error Variants, Schema Probe Summary

Wave 0 foundation: schemars + ferro-projections deps added, Error enum extended with three variants, schema module shell created with passing structural probe tests that definitively resolve research assumptions A1 and A2.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add new dependencies to ferro-ai/Cargo.toml | 79bdee4d | ferro-ai/Cargo.toml |
| 2 | Extend Error enum with schema/tool variants | 5f6e31c2 | ferro-ai/src/error.rs |
| 3 | Create schema module shell + Wave 0 structural probe test | 469a1531 | ferro-ai/src/schema/mod.rs, ferro-ai/src/lib.rs |

## Decisions Made

**Verified schema shapes for Plan 02/03 closing algorithm:**

`FieldMeaning` (no per-variant doc comments):
- Emits `anyOf` with 2 branches
- Branch 0: `{"type":"string","enum":["identifier",...18 values...]}` — single closed enum array
- Branch 1: `{"type":"string"}` — open string (Custom escape hatch)
- Closing algorithm: extract `anyOf[0]["enum"]` → emit `{"type":"string","enum":[...extracted...]}`

`Intent` (has per-variant doc comments):
- Emits `anyOf` with 8 branches (7 known + 1 open)
- Branches 0–6: `{"const":"browse","description":"...","type":"string"}` — individual const per variant
- Branch 7: `{"description":"Escape hatch...","type":"string"}` — open string (no const, no enum)
- Closing algorithm: collect `const` values from branches where `branch["const"].is_string()` → emit `{"type":"string","enum":[...collected...]}`

## Deviations from Plan

### Auto-corrected Assumptions

**[Rule 1 - Bug] Corrected research assumption A2 for Intent schema shape**
- **Found during:** Task 3 (TDD RED phase — probe test failed)
- **Issue:** Research assumed `Intent` would emit `anyOf` with first branch as `{"enum":[...]}` (same as `FieldMeaning`). The actual shape uses individual `const` branches per variant because each `Intent` variant has a doc comment. schemars 1.x uses `const` when a variant has a description, and collapses to `enum` when variants have no individual descriptions.
- **Fix:** Updated the `schema_probe_intent_any_of_shape` test to assert the actual `const`-per-branch shape and updated the module doc comment to document both shapes for Plan 02/03 implementors.
- **Impact:** Plan 02's closing algorithm must handle two distinct anyOf patterns: `enum`-array (FieldMeaning) and `const`-collection (Intent). The Plan 02 implementation must be aware of this distinction.
- **Files modified:** ferro-ai/src/schema/mod.rs
- **Commit:** 469a1531

## Verification Results

- `cargo build -p ferro-ai` exits 0 with new deps
- `cargo test -p ferro-ai schema_probe` green — both probe tests pass
- `cargo test -p ferro-ai test_error_is_retryable` green — no regression
- `cargo clippy --all --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean

## Known Stubs

None — this plan creates only the shell (no normalizer implementation yet). The `schema/mod.rs` module is intentionally empty of production code; Plan 02 adds `for_structured_output()`.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes at trust boundaries. The new `ferro-projections` dependency is a leaf crate with no HTTP surface. No threat flags.

## Self-Check: PASSED

- ferro-ai/src/schema/mod.rs exists: confirmed (created in this plan)
- ferro-ai/src/lib.rs contains `pub mod schema;`: confirmed
- ferro-ai/src/error.rs contains `ToolIterationLimit(u32)`: confirmed
- Commits 79bdee4d, 5f6e31c2, 469a1531 exist in git log: confirmed

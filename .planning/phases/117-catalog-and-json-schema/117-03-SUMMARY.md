---
phase: 117-catalog-and-json-schema
plan: "03"
subsystem: ferro-json-ui
tags: [catalog, json-schema, oneOf, validator, schema-assembly]
dependency_graph:
  requires: [117-02 BUILTIN_SPECS population, Catalog::build() discovery impl]
  provides: [assemble_full_schema, catalog.json_schema(), validator compiled once (SCHEMA-03)]
  affects: [ferro-json-ui/src/catalog.rs]
tech_stack:
  added: []
  patterns: [hoist_defs hoisting, element-level oneOf discriminator, jsonschema::validator_for once at build time]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/catalog.rs
decisions:
  - "oneOf placed at Element level (discriminates on element.type), not under Element.properties.props — props objects lack a type field, so props-level oneOf was structurally wrong"
  - "hoist_defs() collects per-component $defs into shared root $defs — required for $ref resolution (e.g., #/$defs/ConfirmDialog from Action schema embedded in Button/Form variants)"
  - "build_builtins_only() test helper added under #[cfg(test)] to isolate Plan 03 tests from BadPlugin_117 global registry pollution left by Plan 02's combined plugin test"
  - "Re-added #[allow(dead_code)] on Catalog struct — components, plugin_components, per_component_schemas, validator not yet consumed outside tests; Plan 04 removes it"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-18T13:28:30Z"
  tasks_completed: 1
  files_changed: 1
---

# Phase 117 Plan 03: Full Schema Assembly + Validator Compilation Summary

Implemented `assemble_full_schema()`, `catalog.json_schema()`, and the real `jsonschema::Validator` compilation in `Catalog::build()`. Replaced both Plan 02 stubs (`full_schema = Value::Null`, placeholder validator). All 19 catalog tests pass; full ferro-json-ui suite clean.

## What Was Built

**`ferro-json-ui/src/catalog.rs`** (995 lines, +281 from Plan 02):

- **`hoist_defs(schema, shared_defs)`** — strips `$defs` from a schemars-generated schema object and merges its entries into a shared map. Required because schemars emits nested type definitions (e.g., `ConfirmDialog`, `DialogVariant`, `ActionOutcome`) under the schema root's `$defs`; when those schemas are embedded as `allOf` variants in a larger assembled schema, all `$ref` pointers must resolve from the *assembled* root.

- **`assemble_full_schema(per_component)`** — hand-builds the full JSON Schema document:
  1. Generates `action_schema` and `visibility_schema` via `schemars::schema_for!`; sanitizes each; hoists their nested `$defs` (e.g., `ConfirmDialog`, `NotifyVariant`, `VisibilityCondition`, `VisibilityOperator`) to `shared_defs`.
  2. Sorts component names for deterministic oneOf output (CONTEXT D-18).
  3. For each component: clones its props schema, hoists its `$defs`, wraps in `allOf [ { type const discriminator on element }, { props + children + action + visible } ]`.
  4. Inserts `Element` (the oneOf), `Action`, `Visibility` into `shared_defs`.
  5. Returns the root document with `$schema`, `$id`, `type`, `required`, `properties`, `$defs: shared_defs`.

- **`Catalog::build()`** — Plan 02 stubs replaced:
  ```rust
  let full_schema = assemble_full_schema(&per_component_schemas)?;
  let validator = jsonschema::validator_for(&full_schema)
      .map_err(|e| CatalogError::BuildFailed(...))?;
  ```

- **`Catalog::json_schema(&self) -> &Value`** — zero-copy accessor returning `&self.full_schema` (D-15).

- **`Catalog::build_builtins_only()`** — `#[cfg(test)]` helper that builds from `BUILTIN_SPECS` only, bypassing the global plugin registry. Used by all Plan 03 tests to avoid pollution from `BadPlugin_117` registered by Plan 02's combined plugin test.

- **7 new unit tests** (all passing):
  - `json_schema_has_spec_envelope_shape` — `$id`, `type`, `required` fields
  - `json_schema_has_action_and_visibility_defs` — `$defs/Action`, `$defs/Visibility`, `$defs/Element`
  - `json_schema_oneof_covers_all_builtins` — all 39 BUILTIN_TYPES have discriminators
  - `json_schema_is_valid` — meta-validates via `jsonschema::draft202012::meta::is_valid`
  - `validator_is_compiled_once_and_usable` — minimal valid spec passes validation
  - `validator_rejects_wrong_schema_version` — wrong `$schema` const rejected
  - `oneof_variants_are_deterministic_sorted` — byte-equal output across two builds

## Schema Size

Assembled schema string length (at test time, 39 built-ins, no plugins): approximately 55–65 KB (not measured at runtime in this plan; Plan 04 can expose it via MCP tooling).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `$ref` pointers from per-component schemas didn't resolve in assembled root**
- **Found during:** First test run — `BuildFailed("Pointer '/$defs/ConfirmDialog' does not exist")`
- **Issue:** schemars emits nested type definitions under a per-component schema's own `$defs`. When embedded as `allOf[1]` in the oneOf, `$ref: "#/$defs/ConfirmDialog"` resolves from the assembled document root — but only `Element`, `Action`, `Visibility` were present there. Types like `ConfirmDialog`, `DialogVariant`, `ActionOutcome`, `NotifyVariant`, `VisibilityCondition`, `VisibilityOperator` were missing.
- **Fix:** Added `hoist_defs()` to strip and merge each schema's `$defs` into a shared map before constructing the assembled root.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 46b0097f

**2. [Rule 1 - Bug] oneOf placed under `Element.properties.props` caused all elements to fail validation**
- **Found during:** `validator_is_compiled_once_and_usable` — element `{ "type": "Text", "props": { "content": "hi" } }` rejected
- **Issue:** The plan spec placed the discriminated oneOf under `Element.properties.props`. Each variant's `allOf[0]` pinned `"type": { "const": "Text" }` — but this discriminator applied to the *props* sub-object, not the element object. Props objects like `{ "content": "hi" }` don't have a `type` field.
- **Fix:** Moved the oneOf to the Element level. Each variant is now `allOf [ { element.type const }, { props: propsSchema, children: ..., action: ..., visible: ... } ]`. The `type` discriminator matches `element.type` as intended.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 46b0097f (same commit, discovered during same test run)

**3. [Rule 1 - Bug] Plan 03 tests poisoned by BadPlugin_117 global registry entry from Plan 02**
- **Found during:** Second test run (after fixing `$ref` hoisting) — `json_schema_has_spec_envelope_shape` etc. failed with `BuildFailed("plugin 'BadPlugin_117' returned an invalid JSON Schema")`
- **Issue:** `build_discovers_plugins_and_rejects_invalid_schema` registers `BadPlugin_117` permanently in the global plugin registry. When Plan 03 tests call `Catalog::build()` after that test runs, the build aborts due to the invalid plugin.
- **Fix:** Added `Catalog::build_builtins_only()` under `#[cfg(test)]` — builds from `BUILTIN_SPECS` only, skipping plugin discovery. All Plan 03 tests use this helper. Plan 02 tests that call `Catalog::build()` are unaffected (they ran before `BadPlugin_117` was registered, or test the rejection behavior itself).
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 46b0097f

**4. [Rule 2 - Missing critical functionality] Clippy -D warnings required `#[allow(dead_code)]` re-added**
- **Found during:** `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings`
- **Issue:** `components`, `plugin_components`, `per_component_schemas`, and `validator` are `pub(crate)` but not yet consumed outside test code. Clippy treats test-only usage as dead for the lib target under `-D warnings`.
- **Fix:** Re-added `#[allow(dead_code)]` on `Catalog` struct with a comment that Plan 04 removes it when `validate()` is wired up.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 46b0097f

## Verification Results

```
cargo fmt --all -- --check                                    → clean
cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings → clean
cargo test -p ferro-json-ui --lib catalog::                  → 19 passed, 0 failed
cargo test -p ferro-json-ui --all-features                   → all tests ok, no regressions
```

## Known Stubs

None — both Plan 02 stubs (`full_schema = Value::Null`, placeholder validator) replaced with real implementations.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Next Step

Plan 04 implements `catalog.validate(&Spec)` — the full per-element validation pipeline using the compiled `validator` and `per_component_schemas`. Plan 04 also removes `#[allow(dead_code)]` from `Catalog`.

## Self-Check: PASSED

- `ferro-json-ui/src/catalog.rs` — FOUND (995 lines)
- Commit `46b0097f` — FOUND
- `fn assemble_full_schema(` — FOUND
- `pub fn json_schema(&self) -> &Value` — FOUND
- `assemble_full_schema(&per_component_schemas)` in `Catalog::build` — FOUND
- `jsonschema::validator_for(&full_schema)` in `Catalog::build` — FOUND

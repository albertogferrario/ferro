---
phase: 117-catalog-and-json-schema
plan: "02"
subsystem: ferro-json-ui
tags: [catalog, builtin-specs, schemars, schema-sanitizer, plugin-discovery]
dependency_graph:
  requires: [117-01 catalog.rs scaffold, jsonschema 0.46 dep]
  provides: [BUILTIN_SPECS 39-entry table, Catalog::build() discovery impl, sanitize_schema()]
  affects: [ferro-json-ui/src/catalog.rs]
tech_stack:
  added: []
  patterns: [schemars::schema_for! per-Props invocation, fn() -> Value table entries, recursive JSON walker]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/catalog.rs
decisions:
  - "Used #[allow(dead_code)] on Catalog struct again — full_schema and validator are stub-populated; Plan 03 removes the allow"
  - "Combined good+bad plugin test into one function to avoid global registry leakage across test functions"
  - "sanitize_schema walks keys via collected Vec<String> to avoid simultaneous borrow of obj"
  - "Slot fields: Card=[footer], Modal=[footer], PageHeader=[actions]; Tabs and KanbanBoard outer Props get [] per CONTEXT D-05 / RESEARCH L-3"
metrics:
  duration: "~4 minutes"
  completed: "2026-04-18T14:00:40Z"
  tasks_completed: 1
  files_changed: 1
---

# Phase 117 Plan 02: BUILTIN_SPECS Population + Discovery Implementation Summary

Populated the 39-entry `BUILTIN_SPECS` static table, implemented `Catalog::build()` with the full discovery pipeline (built-ins + plugins), added the `sanitize_schema()` walker for `definitions` → `$defs` rewriting, and added 12 unit tests. All success criteria met.

## What Was Built

**`ferro-json-ui/src/catalog.rs`** (714 lines, +567 from Plan 01 scaffold):

- **`BUILTIN_SPECS` static table** — 39 entries of `(&str, &str, SchemaFn, &[&str])` in exact `BUILTIN_TYPES` order. Each `schema_fn` closure calls `serde_json::to_value(schemars::schema_for!(TProps)).unwrap()`. Grouping comments match `render/mod.rs` for reviewability.

- **`sanitize_schema()`** — recursive `Value` walker that renames `definitions` → `$defs` and rewrites `#/definitions/X` `$ref` strings to `#/$defs/X`. Idempotent. Keys collected into `Vec<String>` before iteration to avoid borrow conflicts.

- **`Catalog::build()`** — replaces `unimplemented!` stub:
  1. Runtime drift guard: `BUILTIN_SPECS.len() != BUILTIN_TYPES.len()` → `CatalogError::BuildFailed`
  2. Built-in loop: for each BUILTIN_SPECS entry, calls `schema_fn()`, runs `sanitize_schema()`, inserts into `components` and `per_component_schemas`
  3. Plugin discovery loop: iterates `registered_plugin_types()`, skips built-in shadows (D-19), calls `with_plugin()` for raw schema, sanitizes, meta-validates via `jsonschema::validator_for()` — failure → `BuildFailed` with plugin name (H-3)
  4. Placeholder stubs: `full_schema = Value::Null`, trivial `{ "type": "object" }` validator for Plan 03 to overwrite

- **Slot fields wired**: `Card` → `["footer"]`, `Modal` → `["footer"]`, `PageHeader` → `["actions"]`; all other 36 components → `[]`

- **12 unit tests** (all passing):
  - `builtin_types_count_is_39` (Plan 01 drift guard, retained)
  - `builtin_specs_len_matches_dispatch`
  - `builtin_specs_names_match_dispatch`
  - `build_populates_all_builtins`
  - `build_card_has_footer_slot`
  - `build_modal_has_footer_slot`
  - `build_pageheader_has_actions_slot`
  - `build_text_has_no_slots`
  - `build_populates_per_component_schemas`
  - `sanitize_schema_rewrites_definitions_to_dollar_defs`
  - `sanitize_schema_is_idempotent`
  - `build_discovers_plugins_and_rejects_invalid_schema` (combined good+bad plugin test)

## Verification Results

```
rg 'schema_for!' ferro-json-ui/src/catalog.rs | wc -l   → 39
wc -l ferro-json-ui/src/catalog.rs                       → 714
cargo fmt --all -- --check                               → clean
cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings  → clean
cargo test -p ferro-json-ui --lib catalog::              → 12 passed, 0 failed
cargo test -p ferro-json-ui --all-features               → all tests ok, no regressions
```

## Schemars Output Shape Notes

Schemars 0.8.x emits `definitions` + `$ref: "#/definitions/X"` for types with enum/nested struct references (e.g., `ButtonVariant`, `Action`, `Column`). The sanitizer correctly rewrites both the key and the `$ref` strings. No `$schema` or `$id` meta-keys needed adjustment — schemars 0.8 does not emit them by default.

## Plugin Discovery Behaviour

No plugins are registered by default in the test environment. The combined plugin test (`build_discovers_plugins_and_rejects_invalid_schema`) registers `GoodPlugin_117` (valid schema → appears in `plugin_components`) then `BadPlugin_117` (`{ "type": 42 }` → `CatalogError::BuildFailed("plugin 'BadPlugin_117' returned an invalid JSON Schema")`). Built-in shadow prevention (D-19) verified by design — any plugin type matching a BUILTIN_SPECS name is skipped.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `Result<Catalog, _>` match arm used `{other:?}` without `Debug` on `Catalog`**
- **Found during:** First compile run
- **Issue:** The test's catch-all `other => panic!("... {other:?}")` required `Catalog: Debug` which is not derived. Clippy caught this as a compile error.
- **Fix:** Replaced the single-arm match with explicit `Err(other)` and `Ok(_)` arms, avoiding the need for `Catalog: Debug`.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** 252fe972 (same task commit)

**2. [Rule 1 - Bug] rustfmt required reformatting of two expressions**
- **Found during:** `cargo fmt --all -- --check`
- **Issue:** `.map_err(|e| { ... })?` closure block and inline `assert!` message needed reformatting to rustfmt's line-length rules.
- **Fix:** `cargo fmt --all` applied automatically.
- **Files modified:** `ferro-json-ui/src/catalog.rs`

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| `full_schema = Value::Null` | `catalog.rs` | Plan 03 assembles the oneOf JSON Schema document |
| `validator` compiled from `{ "type": "object" }` placeholder | `catalog.rs` | Plan 03 replaces with validator compiled from `full_schema` |

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced. Plugin meta-validation (T-117-02-01) and `definitions`→`$defs` sanitization (T-117-02-04) are implemented as specified.

## Next Step

Plan 03 assembles `full_schema` via a hand-built `oneOf` over all component Props schemas, compiles the `jsonschema::Validator` from it, and removes both stub values and the `#[allow(dead_code)]` on `Catalog`.

## Self-Check: PASSED

- `ferro-json-ui/src/catalog.rs` — FOUND (714 lines)
- Commit `252fe972` — FOUND

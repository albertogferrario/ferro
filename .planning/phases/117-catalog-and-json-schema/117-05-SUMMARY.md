---
phase: 117-catalog-and-json-schema
plan: "05"
subsystem: ferro-json-ui/catalog
tags: [catalog, component-schema, accessor, zero-copy]
dependency_graph:
  requires: [117-04]
  provides: [CAT-04, component_schema, components_sorted, plugin_components_sorted]
  affects: [ferro-mcp/json_ui_catalog, Phase-120-AI-generation, Plan-06-prompt, Plan-07-CLI]
tech_stack:
  added: []
  patterns: [zero-copy-borrow, build_builtins_only-test-isolation]
key_files:
  modified:
    - ferro-json-ui/src/catalog.rs
decisions:
  - "Use build_builtins_only() in all Plan 05 tests — Catalog::build() is sensitive to the global plugin registry which build_discovers_plugins_and_rejects_invalid_schema contaminates with BadPlugin_117. Consistent with existing Plan 03/04 test pattern."
  - "component_schema delegates directly to per_component_schemas.get(type_name): single-line body, zero allocation, &Value lifetime tied to &self (CONTEXT D-15, D-19)."
  - "Sorted iterators allocate a Vec<&ComponentSpec> of fixed size (39 built-ins, O(n log n) sort) on each call — accepted per CONTEXT D-12 escape hatch pattern; cache deferred to follow-up if profiling demands it."
metrics:
  duration: "~8 minutes"
  completed: "2026-04-18T13:38:13Z"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 117 Plan 05: Catalog Accessor Methods Summary

Zero-copy per-component schema accessor and sorted iteration helpers added to `Catalog`, satisfying CAT-04 (ROADMAP success criterion 5).

## What Was Built

Three public methods added to `impl Catalog` in `ferro-json-ui/src/catalog.rs`:

**`component_schema(&self, type_name: &str) -> Option<&serde_json::Value>`**
- Single `self.per_component_schemas.get(type_name)` call — no clone, no allocation beyond `Option`
- Unified lookup across built-ins and plugins (both are in `per_component_schemas` after `Catalog::build`)
- Returns Props-only schema, NOT the Element envelope (CONTEXT D-19)
- `&Value` lifetime bounded by `&self` (CONTEXT D-15)
- Satisfies ROADMAP SC-5: `catalog.component_schema("Card")` returns `Some(&Value)`

**`components_sorted(&self) -> impl Iterator<Item = &ComponentSpec>`**
- Built-in specs in ascending name order
- Required by Plan 06 `prompt()` for deterministic output (CONTEXT D-18)
- Required by ferro-mcp `json_ui_catalog.rs` consumer rewrite (Plan 06 migration, RESEARCH §9)

**`plugin_components_sorted(&self) -> impl Iterator<Item = &ComponentSpec>`**
- Plugin specs in ascending name order
- Preserves the built-in / plugin split that `CatalogResponse` (CONTEXT D-24) requires

## Tests Added (4 new, 30 total in `catalog::tests`)

| Test | Asserts |
|------|---------|
| `component_schema_returns_props_only` | CardProps schema has `properties.title`; NOT an Element wrapper (no `children`+`props` keys) |
| `component_schema_none_for_unknown` | Unknown name and empty string both return `None` |
| `component_schema_resolves_every_builtin` | All 39 names in `BUILTIN_TYPES` have a schema entry |
| `components_sorted_yields_ascending_by_name` | Built-in iterator yields names in sorted order; plugin iterator also sorted |

All four tests use `build_builtins_only()` to avoid global plugin registry pollution from `build_discovers_plugins_and_rejects_invalid_schema`.

## CardProps Schema Shape Verification

`component_schema_returns_props_only` confirmed that CardProps schema has a top-level `properties` map containing `title` — no `$defs` traversal required. The `sanitize_schema` pass in Plan 02 correctly flattened schemars output into a direct `properties` object. The diagnostic fallback path (Step 6) was not needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Switched all new tests from `Catalog::build()` to `Catalog::build_builtins_only()`**
- **Found during:** First test run (3 of 4 new tests failed)
- **Issue:** Plan spec called `Catalog::build()` in all four tests. The global plugin registry is contaminated by `build_discovers_plugins_and_rejects_invalid_schema` (which registers `BadPlugin_117`), causing `build()` to return `BuildFailed`.
- **Fix:** All four Plan 05 tests switched to `build_builtins_only()`, consistent with the pattern already used by Plans 03/04 tests in the same file.
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Commit:** b692343b

## Commits

| Hash | Message |
|------|---------|
| b692343b | feat(117-05): add component_schema, components_sorted, plugin_components_sorted accessors |

## Verification

```
cargo fmt --all -- --check          → clean
cargo clippy -p ferro-json-ui \
  --all-targets --all-features \
  -- -D warnings                    → clean
cargo test -p ferro-json-ui \
  --lib catalog::                   → 30/30 passed
```

## Known Stubs

None.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced. The new accessor methods are read-only borrows of existing in-memory state.

## Next Step

Plan 06 implements `Catalog::prompt()` and replaces the hand-maintained `COMPONENT_CATALOG` const string across ferro-mcp (`json_ui_catalog.rs`) and ferro-cli. It will consume `components_sorted()` and `plugin_components_sorted()` for deterministic output.

## Self-Check: PASSED

- `ferro-json-ui/src/catalog.rs` exists and contains all three new pub fn declarations
- Commit b692343b verified in git log
- 30/30 catalog tests pass

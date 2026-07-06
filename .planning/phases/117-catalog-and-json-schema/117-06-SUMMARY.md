---
phase: 117-catalog-and-json-schema
plan: "06"
subsystem: ferro-json-ui/catalog
tags: [catalog, prompt, consumer-migration, delete-component-catalog]
dependency_graph:
  requires: [117-05]
  provides: [CAT-02, Catalog::prompt, ROADMAP-SC-7]
  affects: [Plan-07-CLI, Phase-120-AI-generation, ferro-mcp/json_ui_catalog, ferro-mcp/json_ui_generate, ferro-cli/ai]
tech_stack:
  added: []
  patterns: [schemars-driven-prompt-rendering, global_catalog-consumer-pattern]
key_files:
  modified:
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/lib.rs
    - ferro-cli/src/ai.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
decisions:
  - "`prompt()` emits a Markdown-ish text summary (7964 bytes), not the raw JSON Schema — satisfies CONTEXT D-17 (≤ 8 KB budget) and the ROADMAP caveat that 40-80 KB schemas do not fit in AI context windows."
  - "Byte-identical text shape across builds via `components_sorted()` / `plugin_components_sorted()` — CONTEXT D-18 determinism preserved."
  - "ferro-mcp json_ui_catalog preserves its public `JsonUiCatalog` / `CatalogComponent` / `PropInfo` struct shape (CONTEXT D-24); only the body is rewritten to source from `global_catalog()`."
  - "`BUILDER_API` and `ACTION_API` const strings stay hand-maintained in json_ui_catalog.rs — they describe DSL ergonomics that live outside the component schema surface."
  - "`test_button_has_variants` renamed to `test_button_has_props`: struct-shaped Props expose enum variants through `PropInfo.type_name` (pipe-joined), so the top-level `variants` field is `None` for components like Button. Enum-shaped Props (if any) would surface via `derive_variants`."
requirements_completed:
  - CAT-02
metrics:
  duration: "~35 minutes (Task 1 prior commit + Task 2 follow-on)"
  completed: "2026-04-18T14:09:48Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Phase 117 Plan 06: Catalog::prompt() + Consumer Migration Summary

`Catalog::prompt()` shipped as the concise-text AI system prompt surface, and every `COMPONENT_CATALOG` consumer migrated to `global_catalog().prompt()`. ROADMAP SC-7 closed.

## What Was Built

**`Catalog::prompt()`** (Task 1, commit `4e60c527`) — emits a ≤ 8 KB Markdown-ish text summary of every built-in and plugin component: headings, descriptions, props lines with type hints, and slot documentation. Six private helpers (`render_component_section`, `render_props_line`, `render_field_type`, `rust_for_json_type`, `render_enum_inline`, `wrap_optional`) walk per-component JSON Schemas to produce deterministic output.

**Consumer migration** (Task 2, commits `b5911f9d`, `ad1b3dc2`, `9af40599`, `80e3117c`) — three consumer files migrated; one 4.7 KB hand-maintained const deleted.

## Size Measurements

| Artifact | Bytes |
|---|---|
| `cat.prompt().len()` (built-ins only, runtime) | **7964** |
| Deleted `COMPONENT_CATALOG` const (doc + body, ferro-json-ui/src/lib.rs lines 86-176) | **4768** |

The prompt is 67% larger than the legacy const because it documents all 39 built-in components versus the legacy 28. Still comfortably under the 8 KB budget (CONTEXT D-17).

## Consumer Migrations

| File | Replacement expression |
|---|---|
| `ferro-cli/src/ai.rs` | `let catalog_prompt = global_catalog().prompt();` bound before the `format!` block; f-string interpolates `{catalog_prompt}`. |
| `ferro-mcp/src/tools/json_ui_generate.rs` | `component_catalog: global_catalog().prompt()` as the `JsonUiGenerationContext` field value. |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `global_catalog().components_sorted()` / `.plugin_components_sorted()` feed a per-spec `to_catalog_component` closure that derives `PropInfo` entries and `variants: Option<Vec<String>>` from each Props schema. `BUILDER_API` / `ACTION_API` const strings preserved. |

## Snapshot Tests Updated

One test in `ferro-mcp/src/tools/json_ui_catalog.rs`:

- **`test_button_has_variants` → `test_button_has_props`**: struct-shaped Props surface their enum variants through `PropInfo.type_name` (e.g., `variant: "default|secondary|destructive|outline|ghost|link"`). The top-level `variants` field is therefore `None` for struct Props. Enum-shaped Props (top-level `oneOf` of `const` strings) would populate it — none currently exist.
- **`catalog_returns_complete_data` allow-list expansion**: `CalendarCell`, `Collapsible`, `Toast`, `Checklist` added to the `no_required` allow-list — their schemars output correctly has no required fields (all optional), which is accurate schema-driven behavior.

No other snapshot tests existed for the old const string shape.

## Workspace Grep Verification

```
$ rg 'COMPONENT_CATALOG' --type rust
(no matches)
```

ROADMAP SC-7 satisfied. No unexpected consumers found; only the three documented in RESEARCH §9.

## Task Commits

| Task | Commit | Message |
|---|---|---|
| Task 1: `Catalog::prompt()` + 5 unit tests | `4e60c527` | feat(117-06): implement Catalog::prompt() with 5 unit tests |
| Task 2 step 1: delete const | `b5911f9d` | refactor(117-06): delete COMPONENT_CATALOG const from ferro-json-ui |
| Task 2 step 2: ferro-cli migration | `ad1b3dc2` | refactor(117-06): migrate ferro-cli/ai.rs to global_catalog().prompt() |
| Task 2 step 3: json_ui_generate migration | `9af40599` | refactor(117-06): migrate json_ui_generate to global_catalog().prompt() |
| Task 2 step 4: json_ui_catalog rewrite | `80e3117c` | refactor(117-06): rewrite json_ui_catalog to source from global_catalog() |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tightened `rg COMPONENT_CATALOG --type rust` cleanup to catch doc comment references**
- **Found during:** Task 2 grep verification.
- **Issue:** After deleting the const, two doc comments in `ferro-json-ui/src/catalog.rs` still referenced `COMPONENT_CATALOG` by name (the crate-level `//!` header and a `ComponentSpec.description` field comment). `rg --type rust` returned those two hits, blocking the acceptance criterion "`rg 'COMPONENT_CATALOG' --type rust` returns ZERO hits".
- **Fix:** Rewrote both comments to describe the current responsibility without naming the deleted const ("the hand-maintained component reference string", "Short imperative description used in prompt output and catalog tooling").
- **Files modified:** `ferro-json-ui/src/catalog.rs`
- **Verification:** `rg 'COMPONENT_CATALOG' --type rust` returns no matches.
- **Committed in:** `b5911f9d` (folded into the const-deletion commit — same logical cleanup).

---

**Total deviations:** 1 auto-fixed. Impact: cosmetic cleanup, no scope creep.

## Verification

```
cargo fmt --all -- --check            → clean
cargo clippy --all --all-targets \
  --all-features -- -D warnings       → clean
cargo test --all-features             → 2203 tests passed, 0 failed
rg 'COMPONENT_CATALOG' --type rust    → no matches
```

## Threat Flags

T-117-06-02 (plugin description instruction injection) remains the only live concern and is covered by the existing hardcoded `"Plugin component."` default in BUILTIN_SPECS — arbitrary plugin-authored description text does NOT enter `prompt()` output today. When a future phase lets plugins author descriptions, add sanitization before re-exposing via `prompt()`.

## Next Step

Plan 07 ships the `ferro json-ui:schema` CLI command, exposing `catalog.json_schema()` and `catalog.component_schema(type)` to external tooling for the 40-80 KB machine-readable schema path. The AI-prompt path (`prompt()`) and the tool-introspection path (`json_schema()`) are now cleanly separated per the ROADMAP caveat.

## Self-Check: PASSED

- `ferro-json-ui/src/lib.rs` no longer contains `pub const COMPONENT_CATALOG`
- `pub use catalog::{global_catalog, Catalog, CatalogError, ComponentSpec};` still present in lib.rs
- `ferro-cli/src/ai.rs` contains `global_catalog().prompt()`
- `ferro-mcp/src/tools/json_ui_generate.rs` uses `global_catalog()`
- `ferro-mcp/src/tools/json_ui_catalog.rs` uses `cat.components_sorted()` + `cat.plugin_components_sorted()`; `BUILDER_API` and `ACTION_API` preserved
- Workspace grep for `COMPONENT_CATALOG` returns zero `.rs` hits
- All four Task 2 commits present in `git log`
- fmt / clippy / test all green

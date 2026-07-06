---
phase: 164
plan: "03"
subsystem: ferro-json-ui / ferro-mcp
tags: [json-ui, component, catalog, data-path, raw-html, d-15, d-17a]
dependency_graph:
  requires: []
  provides: [ImageProps.data_path, DescriptionListProps.data_path, RawHtml-component]
  affects: [ferro-json-ui, ferro-mcp, ferro-json-ui/projection]
tech_stack:
  added: []
  patterns: [data-path-override, verbatim-html-emission, safety-docstring]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
decisions:
  - "data_path on Image and DescriptionList uses Option<String> with fallback to static field — mirrors 3 existing precedents in same file"
  - "RawHtml is a narrow primitive (single html: String field) — explicitly not a generic Plugin dispatch per Phase 115 D-01"
  - "items on DescriptionListProps changed from required to #[serde(default)] to allow data_path as sole source"
  - "src on ImageProps changed from required to #[serde(default)] for consistency with data_path usage"
  - "DescriptionList and RawHtml added to MCP no_required list since all their props now have defaults"
metrics:
  duration: "~30 minutes"
  completed: "2026-05-17"
  tasks_completed: 3
  files_modified: 7
---

# Phase 164 Plan 03: data_path overrides + RawHtml component Summary

Bundles D-15 (Image/DescriptionList data_path) and D-17a (RawHtml component) with atomic catalog count maintenance (S-4). Both touch the same files; splitting would break the BUILTIN count invariant between waves.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add data_path to ImageProps + DescriptionListProps | 658560ed | component.rs, render/atoms.rs |
| 2 | Add RawHtmlProps + render_raw_html + catalog integration | 2b3b530b | component.rs, atoms.rs, mod.rs, catalog.rs, lib.rs, projection/builder.rs, ferro-mcp/json_ui_catalog.rs |
| 3 | Pre-commit gate (fmt + clippy + tests) | 8cebb694 | atoms.rs (fmt only) |

## Catalog Count Assertion Sites Updated

All four sites bumped from 40 to 41:

1. `ferro-json-ui/src/render/mod.rs:530` — `assert_eq!(BUILTIN_TYPES.len(), 41)`
2. `ferro-json-ui/src/catalog.rs:1046` — `assert_eq!(crate::render::BUILTIN_TYPES.len(), 41)`
3. `ferro-json-ui/src/catalog.rs:1052` — `assert_eq!(BUILTIN_SPECS.len(), 41)`
4. `ferro-mcp/src/tools/json_ui_catalog.rs:289` — count literal `41` + "all 41 built-in components"

## New Tests Added

**Task 1 (data_path on Image and DescriptionList):**
- `render::atoms::tests::image_data_path_resolves_src_from_data` — data_path overrides static src
- `render::atoms::tests::image_data_path_none_uses_static_src` — backward compat without data_path
- `render::atoms::tests::image_data_path_missing_in_data_falls_back_to_src` — fallback on missing path
- `render::atoms::tests::description_list_data_path_overrides_static_items` — data_path overrides items
- `render::atoms::tests::description_list_data_path_serde_roundtrip` — serde round-trip

**Task 2 (RawHtml component):**
- `render::atoms::tests::raw_html_props_serde_roundtrip` — props round-trip
- `render::atoms::tests::render_raw_html_emits_verbatim` — verbatim emission in `<div data-ferro-raw-html>`
- `render::atoms::tests::render_raw_html_null_props_emits_diagnostic` — decode failure → HTML comment
- `render::atoms::tests::builtin_types_includes_raw_html` — BUILTIN_TYPES has RawHtml + len == 41

## lib.rs Re-export

`ferro-json-ui/src/lib.rs` re-export block includes `RawHtmlProps` in the `pub use component::{...}` list.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] projection/builder.rs DescriptionListProps struct literals**
- **Found during:** Task 2 compilation
- **Issue:** Two struct literals `DescriptionListProps { items, columns: None }` in `projection/builder.rs` were missing the new `data_path` field, causing compile errors
- **Fix:** Added `data_path: None` to both struct literals (replace_all)
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** 2b3b530b

**2. [Rule 1 - Bug] MCP test_components_have_props assertion for DescriptionList**
- **Found during:** Task 2 test run
- **Issue:** `test_components_have_props` in ferro-mcp asserted DescriptionList has at least one required prop; after `items` became `#[serde(default)]`, the schema has no required fields — a correct schema change, not a regression
- **Fix:** Added "DescriptionList" and "RawHtml" to the `no_required` exception list in the MCP test
- **Files modified:** `ferro-mcp/src/tools/json_ui_catalog.rs`
- **Commit:** 2b3b530b

**3. [Rule 3 - Environment] Worktree target dir disk space**
- **Found during:** Task 3 full test run
- **Issue:** `/dev/disk3s5` at 100% capacity; `cargo test --all-features` failed with "No space left on device" errors in the worktree's own `target/` directory
- **Fix:** Ran tests using `CARGO_TARGET_DIR=/Users/alberto/repositories/albertogferrario/ferro/target` (the main repo's already-built target). All 482 ferro-json-ui + 219 ferro-mcp tests pass.
- **Note:** The disk space constraint is an environment issue, not a code issue. The code and tests are correct.

## Downstream Notes

- **Plan 10 (docs)** must add documentation sections for:
  - `Image.data_path` — how to use, fallback behavior, example JSON
  - `DescriptionList.data_path` — same
  - `RawHtml` — the component itself, Safety warning, when to use vs. registered plugins
- **Gestiscilo F7** (`/dashboard/analisi/statistiche`) — unblocked by Image/DescriptionList data_path
- **Gestiscilo F9** (`/dashboard/settings`) — unblocked by RawHtml component

## Self-Check: PASSED

- `658560ed` exists in git log: confirmed
- `2b3b530b` exists in git log: confirmed
- `8cebb694` exists in git log: confirmed
- `ferro-json-ui/src/component.rs` modified: confirmed (RawHtmlProps + data_path on Image + DescList)
- `ferro-json-ui/src/render/atoms.rs` modified: confirmed (render_raw_html + data_path resolution)
- `ferro-json-ui/src/render/mod.rs` modified: confirmed (BUILTIN_TYPES 41, dispatch arm, count assertion)
- `ferro-json-ui/src/catalog.rs` modified: confirmed (BUILTIN_SPECS 41, count assertions)
- `ferro-json-ui/src/lib.rs` modified: confirmed (RawHtmlProps re-exported)
- `ferro-mcp/src/tools/json_ui_catalog.rs` modified: confirmed (count 41, RawHtml in expected names)
- All count assertions: 40 → 41 across all 4 sites, no residual 40s
- Tests: 460 ferro-json-ui lib tests pass; 17 ferro-mcp json_ui_catalog tests pass

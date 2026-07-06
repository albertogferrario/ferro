---
phase: 164
plan: "06"
subsystem: ferro-json-ui / ferro-mcp
tags: [json-ui, kanban, data-path, d-13a, runtime-friction, f3]
dependency_graph:
  requires: [164-05]
  provides: [KanbanBoardProps.data_path, render_kanban_board-data-path-branch]
  affects: [ferro-json-ui, ferro-mcp, ferro-json-ui/projection]
tech_stack:
  added: []
  patterns: [data-path-override, runtime-column-resolution]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
decisions:
  - "data_path wins over static columns when both are set — mirrors DataTableProps pattern (RESEARCH Open Question 6 default)"
  - "columns changed from required to #[serde(default, skip_serializing_if = Vec::is_empty)] — data_path can be the sole source"
  - "filter_map silently drops malformed KanbanColumnProps entries from runtime data (T-164-06-01 mitigate)"
  - "KanbanBoard added to MCP no_required exception list — consistent with DescriptionList and RawHtml (Plan 03)"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-17"
  tasks_completed: 3
  files_modified: 4
---

# Phase 164 Plan 06: KanbanBoard data_path (D-13a) Summary

Adds `data_path: Option<String>` to `KanbanBoardProps` and teaches `render_kanban_board` to resolve columns from handler data at render time. Closes V7-RUNTIME-FRICTION F3 — dashboard kanban was blocked because `columns` required static inlining.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add KanbanBoardProps.data_path + make columns skippable | 7409372c | component.rs |
| 2 | Branch render_kanban_board on data_path | d41972ab | render/containers.rs |
| 3 | Pre-commit gate (fmt + clippy + tests) | 1a25d352 | component.rs, containers.rs, projection/builder.rs, ferro-mcp/json_ui_catalog.rs |

## Key Decision: data_path Wins Over Static Columns

When both `data_path` and `columns` are set, `data_path` takes precedence. This matches the RESEARCH Open Question 6 default and mirrors `DataTableProps.data_path` (REQUIRED field — data always drives the table). The precedence is documented in the field's rustdoc.

## Call Sites Updated

One call site needed updating (Rule 1 auto-fix):

- `ferro-json-ui/src/projection/builder.rs:368` — `KanbanBoardProps { columns, mobile_default_column }` literal was missing `data_path: None`. Added.

## MCP Exception List Updated

`"KanbanBoard"` added to the `no_required` exception list in `ferro-mcp/src/tools/json_ui_catalog.rs` — consistent with how Plan 03 handled `DescriptionList` and `RawHtml` after their required fields gained `#[serde(default)]`.

## New Tests Added

**Task 1 (KanbanBoardProps serde):**
- `component::kanban_board_props_tests::kanban_board_props_serde_static_columns`
- `component::kanban_board_props_tests::kanban_board_props_serde_data_path`
- `component::kanban_board_props_tests::kanban_board_props_serde_neither`
- `component::kanban_board_props_tests::kanban_board_props_empty_columns_skipped_on_serialize`

**Task 2 (render_kanban_board):**
- `render::containers::tests::render_kanban_board_data_path_resolves_columns`
- `render::containers::tests::render_kanban_board_static_columns_fallback`
- `render::containers::tests::render_kanban_board_data_path_missing_renders_empty`
- `render::containers::tests::render_kanban_board_data_path_wins_over_static`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] projection/builder.rs KanbanBoardProps struct literal**
- **Found during:** Task 3 (clippy)
- **Issue:** `KanbanBoardProps { columns, mobile_default_column }` in `projection/builder.rs:368` was missing the new `data_path` field, causing `E0063`
- **Fix:** Added `data_path: None` to the struct literal
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** 1a25d352

**2. [Rule 1 - Bug] MCP test_components_have_props assertion for KanbanBoard**
- **Found during:** Task 3 (full test suite)
- **Issue:** `test_components_have_props` in ferro-mcp panicked because `KanbanBoard` now has no required props — correct schema change, not a regression, matches the pattern established in Plan 03 for DescriptionList and RawHtml
- **Fix:** Added `"KanbanBoard"` to the `no_required` exception list in `ferro-mcp/src/tools/json_ui_catalog.rs`
- **Files modified:** `ferro-mcp/src/tools/json_ui_catalog.rs`
- **Commit:** 1a25d352

## Downstream Notes

- **Plan 10 (docs)** must add documentation for `KanbanBoard.data_path`:
  - How to use, precedence over static columns, example JSON with `data_path`
  - Document both the `data_path` path (this plan, D-13a) AND the `$each` templated path (D-13b) in `docs/src/json-ui/expressions.md`
- **Gestiscilo F3** (`/dashboard/*` kanban views) — unblocked by this plan
- **Plan 07 or later** (if needed): `column_template` + grouping-key factory pattern was explicitly deferred — `data_path` alone covers the documented consumer use case

## Known Stubs

None. The feature is fully wired: `data_path` → `resolve_path` → `Vec<KanbanColumnProps>` deserialization → render loop.

## Self-Check: PASSED

- `7409372c` exists in git log: confirmed
- `d41972ab` exists in git log: confirmed
- `1a25d352` exists in git log: confirmed
- `ferro-json-ui/src/component.rs` modified: confirmed (KanbanBoardProps.data_path added, columns skippable, 4 tests)
- `ferro-json-ui/src/render/containers.rs` modified: confirmed (resolve_path import, data_path branch, 4 render tests)
- `ferro-json-ui/src/projection/builder.rs` modified: confirmed (data_path: None added)
- `ferro-mcp/src/tools/json_ui_catalog.rs` modified: confirmed (KanbanBoard in no_required list)
- fmt: clean
- clippy --all-targets: clean
- cargo test --all-features: all pass

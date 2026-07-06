---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "04"
subsystem: ferro-json-ui
tags: [plugin, rich-text-editor, quill, component, catalog]
dependency_graph:
  requires: [162-01, 162-02, 162-03]
  provides: [RichTextEditorPlugin, RichTextEditorProps]
  affects: [ferro-json-ui/src/plugins, ferro-json-ui/src/component.rs, ferro-json-ui/src/lib.rs]
tech_stack:
  added: [Quill 2.0.3 (CDN, init_script IIFE)]
  patterns: [JsonUiPlugin trait, global_plugin_registry, plugin_components catalog path]
key_files:
  created:
    - ferro-json-ui/src/plugins/rich_text_editor.rs
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/plugins/mod.rs
    - ferro-json-ui/src/plugin.rs
    - ferro-json-ui/src/lib.rs
decisions:
  - RichTextEditor implemented as plugin only — BUILTIN_SPECS entry rejected by drift guard
  - Schema discoverability via plugin_components catalog path (auto-populated from global registry)
  - SRI hashes marked TODO pending production verification (T-162-04-02)
metrics:
  duration: ~25 minutes
  completed: "2026-05-16T17:10:08Z"
  tasks_completed: 3
  files_changed: 5
---

# Phase 162 Plan 04: RichTextEditor v2 Plugin Summary

RichTextEditor re-implemented as a Quill 2.0.3 `JsonUiPlugin` (D-18). Emits container div + hidden input; init script mirrors editor HTML to form field on text-change. Two consumer sites in gestiscilo documenti templates unblocked.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | RichTextEditorProps struct + schema smoke tests | 5218123b | component.rs |
| 2 | RichTextEditorPlugin + global registry registration | 52de8a55 | plugins/rich_text_editor.rs, plugins/mod.rs, plugin.rs, lib.rs |
| 2b | rustfmt formatting fix | 4b54fd0f | plugins/rich_text_editor.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] BUILTIN_SPECS catalog entry would break drift guard**

- **Found during:** Task 3
- **Issue:** The plan instructed adding `RichTextEditor` to `BUILTIN_SPECS`. The catalog enforces `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` via a runtime drift guard in `Catalog::build()` and two test assertions. Adding to `BUILTIN_SPECS` without adding to `BUILTIN_TYPES` causes 11 catalog tests to fail with count mismatch (41 vs 40).
- **Fix:** Did not add to `BUILTIN_SPECS`. Plugin components are automatically discovered by `Catalog::build()` from `global_plugin_registry()` and stored in `plugin_components` (separate from `components`). `RichTextEditor` is fully discoverable via `cat.plugin_components["RichTextEditor"]` without any BUILTIN_SPECS entry. The plan note "plugins are dispatched via the plugin registry after the built-in match falls through" already described this correctly; the action step was inconsistent with the note.
- **Files modified:** ferro-json-ui/src/catalog.rs (no net change — import addition reverted)
- **Commits:** no separate commit needed (revert was in-session)

## Tests

- `component::schema_smoke_tests::schema_for_rich_text_editor_props_generates` — passes
- `component::schema_smoke_tests::rich_text_editor_props_serde_roundtrip` — passes
- `plugins::rich_text_editor::tests::rich_text_editor_plugin_component_type_is_rich_text_editor` — passes
- `plugins::rich_text_editor::tests::rich_text_editor_plugin_assets_include_quill_2_0_3` — passes
- `plugins::rich_text_editor::tests::rich_text_editor_plugin_init_script_binds_data_ferro_quill` — passes
- `plugins::rich_text_editor::tests::rich_text_editor_plugin_render_emits_container_and_hidden_input` — passes
- Full suite: 453 tests pass, 0 fail

## Known Stubs

- **SRI hashes:** `ferro-json-ui/src/plugins/rich_text_editor.rs` lines 96-105 — Quill CSS and JS `Asset::new()` calls lack `.integrity()` SRI hashes. Marked `TODO(162-04)` with compute instructions. Must be verified and added before production deployment (T-162-04-02). Plan 162-10 (verification gate) will surface these.

## Threat Surface

All three threat register items addressed:

| Threat ID | Status |
|-----------|--------|
| T-162-04-01 (XSS on submit) | Documented in `RichTextEditorProps` rustdoc: sanitization is consumer's responsibility |
| T-162-04-02 (supply chain / SRI) | TODO markers present; SRI hashes not yet populated (see Known Stubs) |
| T-162-04-03 (XSS via field/label) | `html_escape()` applied to `field`, `label`, and `initial` before HTML emission |

## Self-Check: PASSED

- `ferro-json-ui/src/plugins/rich_text_editor.rs` — FOUND
- `ferro-json-ui/src/component.rs` contains `RichTextEditorProps` — FOUND
- Commits 5218123b, 52de8a55, 4b54fd0f — FOUND in git log

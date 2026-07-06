---
phase: 150
plan: 01
subsystem: ferro-json-ui
tags: [tdd, red-phase, richtexteditor, tests]
dependency_graph:
  requires: []
  provides: [RED test contract for RichTextEditor component]
  affects:
    - ferro-json-ui/src/render.rs
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/runtime/mod.rs
tech_stack:
  added: []
  patterns: [TDD RED gate, substring-assertion tests, serde round-trip tests]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render.rs
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/runtime/mod.rs
decisions:
  - rte_props_minimal helper function placed inside mod tests alongside test fns
  - JsonUiView accessible in render.rs test module via use super::*
  - cargo fmt applied as separate commit after initial test authoring
metrics:
  duration: ~8min
  completed: "2026-05-01"
  tasks: 3
  files: 3
---

# Phase 150 Plan 01: RichTextEditor RED Tests Summary

RED test scaffold for the RichTextEditor component: nine render tests, two serde/default tests, and two extended runtime bundle assertion arrays — all referencing types that do not yet exist.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add nine render_rich_text_editor unit tests to render.rs | 7f5cee6d | ferro-json-ui/src/render.rs |
| 2 | Add RED serde round-trip + theme-default tests to component.rs | a7231bda | ferro-json-ui/src/component.rs |
| 3 | Extend runtime test arrays to require setupRichTextEditor | d60fa697 | ferro-json-ui/src/runtime/mod.rs |
| fmt | Apply cargo fmt to test additions | 15319c8d | ferro-json-ui/src/render.rs, ferro-json-ui/src/component.rs |

## Test Counts

| File | New Tests |
|------|-----------|
| ferro-json-ui/src/render.rs | 9 (render_rich_text_editor_default_formats, render_rich_text_editor_custom_formats, render_rich_text_editor_with_value_html, render_rich_text_editor_with_label, render_rich_text_editor_with_error, render_rich_text_editor_with_placeholder, render_rich_text_editor_required_emits_hidden, render_rich_text_editor_html_escapes_dynamic_attrs, render_rich_text_editor_emits_quill_sri_assets_via_pipeline) |
| ferro-json-ui/src/component.rs | 2 (rich_text_editor_serde_roundtrip, rich_text_editor_theme_defaults_to_snow) |
| ferro-json-ui/src/runtime/mod.rs | 0 new tests; 2 arrays extended with "setupRichTextEditor" and "setupRichTextEditor();" entries |

## RED Diagnostic Strings

Build output from `cargo build -p ferro-json-ui --tests`:

```
error[E0412]: cannot find type `RichTextEditorProps` in this scope
error[E0422]: cannot find struct, variant or union type `RichTextEditorProps` in this scope
error[E0425]: cannot find function `render_rich_text_editor` in this scope
error[E0599]: no variant or associated item named `RichTextEditor` found for enum `component::Component`
```

Runtime test failures (once compile issues are resolved by Plans 02-04):
- `bundle_contains_all_setup_functions`: "bundle missing setupRichTextEditor"
- `dispatcher_invokes_every_setup`: "dispatcher missing setupRichTextEditor();"

## Insertion Points

- **render.rs**: New tests appended immediately before closing `}` of `mod tests` block (was line 9138, now extended to line 9362).
- **component.rs**: New `#[cfg(test)] mod rich_text_editor_tests { ... }` block appended after the last `#[cfg(test)] mod image_source_tests { ... }` block (was line 4238).
- **runtime/mod.rs**: `"setupRichTextEditor"` added after `"setupKeyValueEditor"` in `bundle_contains_all_setup_functions` array (line 131); `"setupRichTextEditor();"` added after `"setupKeyValueEditor();"` in `dispatcher_invokes_every_setup` array (line 164).

## Deviations from Plan

**1. [Rule 1 - Style] cargo fmt applied as separate commit**

- **Found during:** Post-task 2
- **Issue:** `cargo fmt --all -- --check` failed on the newly added test code in render.rs and component.rs (vec! formatting, method chaining line breaks).
- **Fix:** Ran `cargo fmt --all` and committed formatting fixes as a separate `style(150-01)` commit.
- **Files modified:** ferro-json-ui/src/render.rs, ferro-json-ui/src/component.rs
- **Commit:** 15319c8d

## Known Stubs

None — this plan adds test-only code with no production stubs.

## Threat Flags

None — only test code added; no new production surface.

## Self-Check: PASSED

- ferro-json-ui/src/render.rs: 9 `fn render_rich_text_editor_*` functions confirmed
- ferro-json-ui/src/component.rs: `mod rich_text_editor_tests`, `fn rich_text_editor_serde_roundtrip`, `fn rich_text_editor_theme_defaults_to_snow` confirmed
- ferro-json-ui/src/runtime/mod.rs: `"setupRichTextEditor"` (count=1) and `"setupRichTextEditor();"` (count=1) confirmed
- Commits 7f5cee6d, a7231bda, d60fa697, 15319c8d confirmed in git log
- `cargo build -p ferro-json-ui --tests` exits non-zero with RichTextEditor unresolved errors
- `cargo fmt --all -- --check` passes

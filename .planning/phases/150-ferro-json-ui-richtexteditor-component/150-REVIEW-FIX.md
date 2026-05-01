---
phase: 150-ferro-json-ui-richtexteditor-component
fixed_at: 2026-05-01T00:00:00Z
review_path: .planning/phases/150-ferro-json-ui-richtexteditor-component/150-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 150: Code Review Fix Report

**Fixed at:** 2026-05-01T00:00:00Z
**Source review:** .planning/phases/150-ferro-json-ui-richtexteditor-component/150-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: HTML initial value silently ignored — only Delta path is rehydrated

**Files modified:** `ferro-json-ui/src/runtime/rich_text_editor.rs`
**Commit:** 01fae6d0
**Applied fix:** Added an `else if` branch after the Delta path. When `initialDelta` is null and `hostText.length > 0`, clears the host element, decodes the HTML-escaped server value through a temporary `div` (entity decode via `textContent`/`innerHTML`), and loads it into Quill via `clipboard.dangerouslyPasteHTML`. Quill's clipboard pipeline filters it through the `formats` allowlist, so no additional sanitization step is needed.

### WR-02: Duplicate DOM `id` when two editors share the same `name`

**Files modified:** `ferro-json-ui/src/component.rs`
**Commit:** a560a8a3
**Applied fix:** Extended the `name` field doc comment on `RichTextEditorProps` to state explicitly that `name` must be unique within the page (not just within the form), explaining why: the renderer uses `name` as the DOM `id` of the editor host and `err-{name}` as the error element id, so collisions break label association and the error-element selector. A structural fix (namespaced or random suffix) is deferred to a follow-up phase as noted in the review.

### WR-03: `<a target=...>` passed through without `rel="noopener noreferrer"`

**Files modified:** `ferro-json-ui/src/runtime/rich_text_editor.rs`
**Commit:** 3cf0fb57
**Applied fix:** Replaced the single `lower !== 'href' && lower !== 'target'` guard in `stripDisallowedAttributes` with explicit branches: `href` is kept unconditionally; `target` is kept but when its value is `_blank`, `rel="noopener noreferrer"` is set on the element; all other attributes on `<a>` are removed. This closes the tabnabbing vector while preserving the ability to have external links.

### WR-04: `cssEscapeId` regex is incomplete

**Files modified:** `ferro-json-ui/src/runtime/rich_text_editor.rs`
**Commit:** 0c7b8922
**Applied fix:** Replaced the hard-coded regex polyfill with a two-path implementation: delegates to `CSS.escape` when the standard API is available (all modern browsers), falling back to the hand-rolled regex only for very old environments. The fallback regex is extended to also cover `!`, `@`, `,`, `%`, `?`, `;`, and space — the characters the reviewer identified as missing.

---

_Fixed: 2026-05-01T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

---
phase: 146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va
reviewed: 2026-04-22T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/render.rs
  - ferro-json-ui/src/resolve.rs
  - ferro-json-ui/src/runtime/key_value_editor.rs
  - ferro-json-ui/src/runtime/mod.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 146: Code Review Report

**Reviewed:** 2026-04-22T00:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 146 adds the `KeyValueEditor` component: a `KeyValueEditorProps` struct, an HTML renderer (`render_key_value_editor`), and a vanilla-JS runtime module that wires add/delete/sync events. The component is correctly plumbed into the `Component` enum, serialization/deserialization, the action resolver leaf set, and the runtime bundle dispatcher. HTML escaping is applied uniformly to all user-controlled values. Two correctness gaps were found in `resolve.rs` and `render.rs`, and two minor quality observations are noted.

## Warnings

### WR-01: `resolve_errors` silently skips `KeyValueEditor` error injection

**File:** `ferro-json-ui/src/resolve.rs:471`

**Issue:** `resolve_errors_node` lists `Component::KeyValueEditor(_)` in the "leaf components with no form field semantics" catch-all arm and does nothing. However `KeyValueEditorProps` has both a `field: String` and an `error: Option<String>`, exactly like `Input`, `Select`, `Checkbox`, and `Switch`. When a caller uses `resolve_errors` or `resolve_errors_all` to propagate server-side validation errors onto form components, `KeyValueEditor` fields are silently skipped and their `error` slot is never populated.

**Fix:**
```rust
// In resolve_errors_node, add a new arm before the leaf catch-all:
Component::KeyValueEditor(props) => {
    set_field_error(&mut props.error, &props.field, errors, all);
}
```

### WR-02: Template row aria attributes carry `aria-invalid` and `aria-describedby` when the editor has an error

**File:** `ferro-json-ui/src/render.rs:1875-1879` and `1912`, `1933`

**Issue:** `aria_attrs` is computed once from the editor-level `error` state and then embedded in both live rows and the `<template data-kv-row-template>` row. When the editor renders in an error state, all dynamically added rows (cloned from the template) will carry `aria-invalid="true"` and `aria-describedby="err-{field}"` even though those rows themselves are not invalid. This is technically incorrect ARIA: `aria-invalid` on an input means that specific input's value is wrong, not that a containing component has an error.

**Fix:** Separate the aria attributes for row inputs from the editor-level error. Rows inside the template should always be rendered without `aria-invalid`; the error is on the editor wrapper only.

```rust
// Compute aria_attrs only for live (prefilled) rows, not the template.
// Pass a flag or separate string for template rows:
let render_row = |key_value: Option<(&str, &str)>, is_template: bool| -> String {
    let row_aria = if !is_template { &aria_attrs[..] } else { "" };
    // use row_aria instead of aria_attrs in input/select formatting
    ...
};

// Prefilled rows:
html.push_str(&render_row(Some((k.as_str(), v.as_str())), false));

// Template row:
html.push_str(&render_row(None, true));
```

## Info

### IN-01: `label` `for` attribute points to the hidden input, not the visible rows container

**File:** `ferro-json-ui/src/render.rs:1954-1956`

**Issue:** The rendered label uses `for="{field_escaped}"`, which refers to `id="{field_escaped}"` on the hidden `<input type="hidden">`. Hidden inputs cannot receive focus, so the label's `for` association has no effect for keyboard or screen reader users.

**Fix:** Either omit the `for` attribute on the label (use it as a visual label only), or assign a distinct `id` to the `[data-kv-rows]` container and point `for` there. The rows container is the logical focus target for the editor group.

```rust
// Option A: drop the `for` attribute
html.push_str(&format!(
    r#"<label class="block text-sm font-medium text-text">{label_escaped}</label>"#
));

// Option B: add an id to the rows container and link
html.push_str(&format!(
    r#"<div class="space-y-2" id="{field_escaped}-rows" data-kv-rows>"#
));
// label:
html.push_str(&format!(
    r#"<label class="block text-sm font-medium text-text" for="{field_escaped}-rows">{label_escaped}</label>"#
));
```

### IN-02: Stray comment markers in `component.rs` (single-slash instead of double-slash)

**File:** `ferro-json-ui/src/component.rs:1194`, `3655`, `3677`

**Issue:** Three comment lines use a single `/` instead of `//`. These are compile errors or doc-comment fragments that appear to be copy-paste artifacts. Examples from the grep output:
- `/ Unknown type: treat as a plugin component.` (line ~1194)
- `/ Round-trip: deserialize back into Component, assert structural equality.` (line ~3655)
- `/ Serde default: when allow_custom_keys is absent from JSON input, it must be true.` (line ~3677)

If these are actual source lines (not artifacts of the grep display), they would fail to compile. Verify and correct to `//`.

**Fix:**
```rust
// Unknown type: treat as a plugin component.
// Round-trip: deserialize back into Component, assert structural equality.
// Serde default: when allow_custom_keys is absent from JSON input, it must be true.
```

---

_Reviewed: 2026-04-22T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

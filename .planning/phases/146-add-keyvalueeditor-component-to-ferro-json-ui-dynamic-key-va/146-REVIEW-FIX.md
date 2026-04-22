---
phase: 146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va
fixed_at: 2026-04-22T00:00:00Z
fix_scope: critical_warning
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 146: Code Review Fix Report

**Fix scope:** Critical + Warning
**Findings in scope:** 2
**Fixed:** 2
**Skipped:** 0

## WR-01: `resolve_errors` silently skips `KeyValueEditor` error injection

**Status:** Already fixed at time of review.

`resolve_errors_node` in `ferro-json-ui/src/resolve.rs` already contained the correct arm:

```rust
Component::KeyValueEditor(props) => {
    set_field_error(&mut props.error, &props.field, errors, all);
}
```

No code change required.

## WR-02: Template row aria attributes carry `aria-invalid` when editor has an error

**Status:** Fixed (render.rs already had the fix; test updated to match).

The renderer at `render.rs:1900` already implemented the `render_row(…, is_template: bool)` closure with `let row_aria = if is_template { "" } else { &aria_attrs[..] };`, correctly suppressing `aria-invalid`/`aria-describedby` on template rows.

However, the test `render_key_value_editor_error_state` used empty props (no initial data), so there were no live rows and the assertion `html.contains(r#"aria-invalid="true"#)` failed. The test was updated to:

- Use a prefilled row (`data_path` + initial data) to provide a live row that should carry `aria-invalid`.
- Assert `aria-invalid` is present on the live row.
- Assert the template row does NOT carry `aria-invalid`.

**Commit:** `fix(146): update error_state test to reflect WR-02 aria fix`

---

_Fixed: 2026-04-22T00:00:00Z_

---
phase: 181-json-ui-input-error-prop-inline-render
reviewed: 2026-05-31T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - framework/src/json_ui/mod.rs
  - ferro-json-ui/src/resolve.rs
  - ferro-json-ui/src/render/form.rs
  - docs/src/json-ui/forms.md
  - docs/src/SUMMARY.md
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
---

# Phase 181: Code Review Report

**Reviewed:** 2026-05-31
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Phase 181 delivers three coherent changes: Fix A (merge runtime data into spec before resolve), Fix B (singular `error` string on the resolve pass), and D-06 (error-state class swap + ARIA + error `<p>` id parity across Checkbox, CheckboxList, Switch, and Input-file). The resolve.rs and form.rs changes are clean and internally consistent. Tests are thorough and well-scoped.

One warning-level logic gap exists in `mod.rs`: the JSON API render paths (`render_json`, `render_json_with_errors`) do not call `merge_data` before resolving expressions, creating an asymmetry with the HTML paths that Fix A repaired. The `$data` binding escape hatch documented in `forms.md` silently fails on those JSON paths when the binding key lives in handler-supplied data rather than embedded spec data.

Two info-level items: a doc claim that overstates automation in the blessed path, and a pre-existing `text-muted-foreground` / `text-text-muted` inconsistency in `CheckboxList`.

---

## Warnings

### WR-01: render_json / render_json_with_errors skip merge_data — asymmetry with HTML paths

**File:** `framework/src/json_ui/mod.rs:214-222` and `277-289`

**Issue:** Fix A correctly calls `spec.clone().merge_data(data.clone())` in `render_with_config` (line 84) and `render_with_errors_config` (line 268) before resolving expressions. The JSON API counterparts (`render_json`, `render_json_with_errors`) call `Self::resolve(spec)` / `Self::resolve_with_errors(spec, errors)` on the original, un-merged spec. `resolve_expressions` reads from `spec.data` (see `expression.rs:36`), so any `$data` binding that references a key in the handler-supplied `data` argument will resolve to `null` on the JSON paths.

The documented escape hatch (`forms.md` §Escape Hatch) uses `JsonUi::render`, so the breakage is latent rather than active today. It will surface if a caller uses `render_json` / `render_json_with_errors` with the same `$data`-driven error binding pattern.

**Fix:**

```rust
pub fn render_json(spec: &Spec, data: &serde_json::Value) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let spec = Self::resolve(&spec_with_data);
    let effective_data = if data.is_null() { &spec.data } else { data };
    let payload = serde_json::json!({
        "spec": spec,
        "data": effective_data,
    });
    Ok(HttpResponse::json(payload))
}

pub fn render_json_with_errors(
    spec: &Spec,
    data: &serde_json::Value,
    errors: &HashMap<String, Vec<String>>,
) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let spec = Self::resolve_with_errors(&spec_with_data, errors);
    let effective_data = if data.is_null() { &spec.data } else { data };
    let payload = serde_json::json!({
        "spec": spec,
        "data": effective_data,
    });
    Ok(HttpResponse::json(payload))
}
```

Note: `effective_data` must continue to reference `data` (the caller's argument), not `spec_with_data.data`, so that the null-fallback behavior remains correct and the returned `"data"` field is not polluted with the merged spec data.

---

## Info

### IN-01: Blessed-path doc example misleads on session-flash automation

**File:** `docs/src/json-ui/forms.md:28-33`

**Issue:** The GET handler example passes `ValidationError::default()` — an always-empty map — to `render_validation_error`, which means the sample code never displays any error. The following paragraph compounds this by stating "The blessed path reads it automatically from `_flash.old._validation_errors`", which is incorrect: `render_validation_error` takes an explicit `&ValidationError` argument and does no session reading internally. The handler must retrieve the error value from the session (via `req.validation_errors()` or an equivalent) and pass it in. The discrepancy between the code sample and the prose description will confuse readers about what automation actually exists.

**Fix:** Replace the placeholder `ValidationError::default()` with a call that reads from the session flash, and reword the subsequent sentence to accurately describe the API:

```rust
// In the GET handler
let ve = req.validation_errors(); // reads _flash.old._validation_errors
JsonUi::render_validation_error(&spec, &data, &ve)
```

Prose: "The handler reads the `ValidationError` from the session flash via `req.validation_errors()` and passes it to `render_validation_error`. The framework then matches field names against the spec's form-control elements and populates each matching `error` prop."

---

### IN-02: CheckboxList description paragraph uses wrong muted-text class

**File:** `ferro-json-ui/src/render/form.rs:594`

**Issue:** Pre-existing before Phase 181, but visible in the reviewed file. The description `<p>` for `CheckboxList` emits `text-muted-foreground` while every other description paragraph in the same renderer uses `text-text-muted`:

- `render_input` description: `text-text-muted` (line 316)
- `render_select` description: `text-text-muted` (line 425)
- `render_checkbox` description: `text-text-muted` (line 512)
- `render_switch` description: `text-text-muted` (line 730)
- `render_checkbox_list` description: `text-muted-foreground` (line 594) — inconsistent

`text-muted-foreground` is a shadcn/ui convention that likely resolves to a different token than the project's semantic `text-text-muted`. Whether they map to the same CSS custom property depends on the active theme, but they are not textually identical.

**Fix:**

```rust
// ferro-json-ui/src/render/form.rs:594 — change
"<p class=\"text-sm text-muted-foreground mb-2\">{}</p>",
// to
"<p class=\"text-sm text-text-muted mb-2\">{}</p>",
```

---

_Reviewed: 2026-05-31_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

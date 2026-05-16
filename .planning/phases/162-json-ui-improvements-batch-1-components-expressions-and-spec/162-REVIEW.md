---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
reviewed: 2026-05-16T23:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - ferro-json-ui/Cargo.toml
  - ferro-json-ui/src/action.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/layout.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/plugin.rs
  - ferro-json-ui/src/plugins/mod.rs
  - ferro-json-ui/src/plugins/rich_text_editor.rs
  - ferro-json-ui/src/projection/component_map.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/data.rs
  - ferro-json-ui/src/render/form.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/spec.rs
  - ferro-mcp/Cargo.toml
  - ferro-mcp/src/service.rs
  - ferro-mcp/src/tools/code_templates.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - ferro-mcp/src/tools/json_ui_verify_action.rs
  - ferro-mcp/src/tools/list_routes.rs
  - ferro-mcp/src/tools/mod.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 162: Code Review Report

**Reviewed:** 2026-05-16T23:00:00Z
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Phase 162 delivers 25 decisions covering new components (CheckboxList), render-time improvements (DataTable row-action URL interpolation), spec validation (FooterMissing, duplicate-footer warning), six strum derives, and supporting MCP tooling and documentation. The implementation is substantially correct: HTML escaping is applied consistently across all new render paths, the SRI hash test pins both Quill assets, strum/serde agreement is verified by round-trip tests, and the catalog/MCP count invariant is maintained at 40 built-ins + 2 plugins.

Three issues were found that warrant attention before the phase is fully closed. None are functional regressions from the perspective of the existing test suite, but two affect browser security guarantees.

---

## Warnings

### WR-01: Quill CDN assets missing `crossorigin` attribute — SRI verification non-functional in browsers

**File:** `ferro-json-ui/src/plugins/rich_text_editor.rs:96-100`

**Issue:** The Quill CSS and JS assets are constructed with `.integrity(QUILL_CSS_SRI)` / `.integrity(QUILL_JS_SRI)` but without `.crossorigin(...)`. For cross-origin resources loaded via CDN, browsers require the `crossorigin` attribute to send a CORS request and receive CORS headers before they can verify the SRI hash. Without `crossorigin`, browsers either silently skip SRI verification or block the resource, depending on implementation. The result is that the pinned sha384 hashes provide no actual integrity protection at runtime.

The Map plugin (`ferro-json-ui/src/plugins/map.rs:225,231`) correctly sets `.crossorigin("")` on all its CDN assets. The RichTextEditor plugin is inconsistent with that established pattern.

**Fix:** Add `.crossorigin("")` (anonymous CORS) to both Quill asset builders:

```rust
fn css_assets(&self) -> Vec<Asset> {
    vec![Asset::new(QUILL_CSS_URL)
        .integrity(QUILL_CSS_SRI)
        .crossorigin("")]
}

fn js_assets(&self) -> Vec<Asset> {
    vec![Asset::new(QUILL_JS_URL)
        .integrity(QUILL_JS_SRI)
        .crossorigin("")]
}
```

The existing `rich_text_editor_plugin_assets_carry_sri_hashes` test should be extended to assert `crossorigin` is `Some("")`.

---

### WR-02: `json_ui_verify_action` Levenshtein candidate returned with no distance threshold

**File:** `ferro-mcp/src/tools/json_ui_verify_action.rs:74-82`

**Issue:** `find_handler` always returns the closest candidate by Levenshtein distance even when that candidate is semantically unrelated. In a production application with many routes, searching for a completely wrong handler name (e.g. `"aaaa.bbbb"`) will suggest the lexicographically closest route name regardless of the actual edit distance. This can actively mislead agents — the suggestion `candidate: Some("completely_different.route")` looks authoritative but shares no meaningful relationship with the query.

D-09 in CONTEXT specifies the tool should return "Err(NotFound) with the closest-by-Levenshtein candidate name" without mentioning a threshold. However, without a threshold, the tool's error message is potentially misleading for large route tables. The DoS cap (256-char input) is present, but the unbounded suggestion is a usability issue that can become a logic error if downstream code treats a returned candidate as a near-match.

**Fix:** Cap the candidate suggestion to a reasonable edit distance threshold. A threshold of `min(handler.len() / 2, 8)` (half the input length up to 8) is a commonly used heuristic. Return `candidate: None` when the best distance exceeds the threshold.

```rust
let threshold = (handler.len() / 2).min(8);
let candidate = routes
    .iter()
    .filter_map(|r| {
        r.name
            .as_ref()
            .map(|n| (n.clone(), strsim::levenshtein(n, handler)))
    })
    .min_by_key(|(_, dist)| *dist)
    .filter(|(_, dist)| *dist <= threshold)
    .map(|(name, _)| name);
```

---

### WR-03: `register_built_in_plugins` is a dead public export that silently double-registers built-in plugins when called

**File:** `ferro-json-ui/src/plugins/mod.rs:16-19`, `ferro-json-ui/src/lib.rs:78`

**Issue:** `global_plugin_registry()` in `plugin.rs:153-158` initializes the `OnceLock` by directly registering `MapPlugin` and `RichTextEditorPlugin` inline. The separately exported `register_built_in_plugins()` function calls `register_plugin(MapPlugin)` and `register_plugin(RichTextEditorPlugin)` — which, when called, will trigger `global_plugin_registry()` (initializing the registry with both plugins already), then overwrite them again with fresh instances. The function is currently uncalled (no call sites in the workspace outside the definition file), so it causes no runtime harm, but:

1. It is a dead public export (`pub use plugins::register_built_in_plugins` in `lib.rs`) that creates a misleading API surface.
2. If a consumer calls it, the redundant re-registration is a silent no-op today (because `PluginRegistry::register` overwrites), but it creates brittleness: a future stateful plugin initializer in `register()` would be called twice.

The CONTEXT decision (D-18) notes that `RichTextEditorPlugin` should "use the existing v2 plugin surface" — the existing surface is the `OnceLock` auto-registration in `global_plugin_registry`, not a separate `register_built_in_plugins` call.

**Fix:** Remove the `register_built_in_plugins` function from `plugins/mod.rs` and its re-export from `lib.rs:78`. The `OnceLock` initialization already handles built-in registration.

If the function is intentionally kept as a user-facing API for explicit initialization in test environments, add a doc comment explaining the double-registration behavior and why it is safe.

---

## Info

### IN-01: `selected_path` resolution in `CheckboxList` silently drops non-string array elements

**File:** `ferro-json-ui/src/render/form.rs:497-507`

**Issue:** When `selected_path` resolves to a JSON array, only elements for which `v.as_str()` returns `Some` are included in the `selected` set. Numeric values in the array (e.g. `[1, 2, "three"]`) are silently dropped. This matches the documented prop type (`Vec<String>`), so it is not a bug in the strict sense, but it is a silent data-loss path with no diagnostic. An author who passes integer IDs in `selected_path` will see no checkboxes pre-selected and no error message.

**Fix:** Consider logging a debug comment to the rendered output (matching the `<!-- ferro-json-ui: ... -->` diagnostic pattern) when the resolved array contains non-string elements, or document the behavior explicitly in `CheckboxListProps.selected_path` rustdoc.

---

### IN-02: `initial_esc` in RichTextEditor `<div>` content is never displayed to the user

**File:** `ferro-json-ui/src/plugins/rich_text_editor.rs:74,81`

**Issue:** `initial_esc = html_escape(&initial)` is placed as the inner content of the editor container div (line 81). The IIFE unconditionally overwrites Quill's `root.innerHTML` from `input.value` (line 112) whenever `input.value` is truthy, making the div content irrelevant. The `input.value` attribute is separately HTML-escaped (line 84), which is correct — browsers decode HTML entities in attribute values, so `input.value` in JavaScript returns the original unescaped string, and `quill.root.innerHTML = input.value` correctly initializes Quill with the rich HTML.

The div content (line 81) is harmless but dead — it is overwritten before the user sees it. The variable `initial_esc` is used correctly for the `value=""` attribute but unnecessarily placed in the div body. This creates no security or functional issue, but adds a small amount of misleading server-sent content.

**Fix (optional):** Set the div content to an empty string rather than `initial_esc`, and add a comment explaining that Quill initialization reads from the hidden input:

```rust
html.push_str(&format!(
    "<div id=\"{field_esc}-editor\" data-ferro-quill data-ferro-field=\"{field_esc}\"></div>"
));
```

This is a minor clarity improvement, not a required fix.

---

## Suggested Fixes

Priority order for addressing the findings:

1. **WR-01 (SRI/crossorigin)** — One-line fix per asset, update test assertion. Ensures Quill SRI actually functions in browsers. Maps to the security bar the phase set for itself (T-162-04-02).

2. **WR-02 (Levenshtein threshold)** — Add threshold filter to `find_handler`. Prevents misleading candidate suggestions in large route tables. Low-risk change.

3. **WR-03 (dead export)** — Remove or document `register_built_in_plugins`. Eliminates a confusing public API that implies a required initialization step that is actually automatic.

4. **IN-01** and **IN-02** are informational; fix at discretion.

---

_Reviewed: 2026-05-16T23:00:00Z_
_Reviewer: Claude (gsd-code-reviewer, sonnet-4-6)_
_Depth: standard_

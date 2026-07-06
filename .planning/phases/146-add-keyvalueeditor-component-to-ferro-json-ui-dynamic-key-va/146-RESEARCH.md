# Phase 146: Add KeyValueEditor Component to ferro-json-ui — Research

**Researched:** 2026-04-22
**Domain:** ferro-json-ui component system — Rust HTML rendering, vanilla JS IIFE runtime
**Confidence:** HIGH

## Summary

This phase adds a single new component (`KeyValueEditor`) to `ferro-json-ui`. The codebase has a mature, well-established pattern for adding components: a props struct in `component.rs`, two match arms (serialize + deserialize), a render function in `render.rs`, a dispatch arm in `render_component`, and a public re-export in `lib.rs`. The runtime follows an equally rigid pattern: a new `src/runtime/<name>.rs` file with a `pub(super) const SOURCE: &str` holding vanilla ES5 JS, then wired into the IIFE assembler in `runtime/mod.rs`.

All insertion points were verified by reading the actual source. Line numbers are exact as of the current state of the repository. The `data_path` resolution for JSON objects is already handled by the existing `resolve_path` function (returns `&Value`), which is the correct primitive to use — `resolve_path_string` would serialize the object as a JSON string rather than returning the typed map. The `<template>` approach for JS row cloning is the clean solution for new rows and is fully supported by all modern browsers with no extra dependencies.

Security posture is straightforward: `html_escape` is already present in `render.rs` as `pub(crate)` and must be called on every dynamic string emitted into HTML. The JS runtime receives no untrusted data directly — it reads from DOM inputs the user already controls.

**Primary recommendation:** Follow the established four-file pattern exactly. No new dependencies are needed. The only non-obvious decision is using `resolve_path` (not `resolve_path_string`) to extract object entries for pre-filling rows.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 through D-17** — All implementation decisions are locked. Key constraints:

- `KeyValueEditorProps` fields: `field: String`, `label: Option<String>`, `suggested_keys: Vec<String>`, `allow_custom_keys: bool` (default `true`), `data_path: Option<String>`, `error: Option<String>`
- `allow_custom_keys = true` → key input is `<input type="text">` with `list="{field}-suggestions"` pointing to a `<datalist>`
- `allow_custom_keys = false` → key input is `<select>` with `<option>` elements; no chevron SVG decoration
- Row layout: `grid grid-cols-[1fr_1fr_auto] gap-2 items-center` per row
- Serialization target: JSON object `{"key1": "value1"}` into hidden `<input name="{field}" type="hidden">`
- `data-kv-editor` attribute on wrapper; `data-kv-field` carries the field name
- New runtime module: `ferro-json-ui/src/runtime/key_value_editor.rs`
- Vanilla JS ES5 only — `var`, no arrow functions, no closures capturing `this`
- Empty rows (blank key) excluded from serialization; duplicate keys = last-write-wins

### Claude's Discretion

- Row template element placement (use `<template data-kv-row-template>` inside the wrapper)
- Exact Tailwind classes — follow existing Input/Select patterns
- Test coverage approach

### Deferred Ideas (OUT OF SCOPE)

- Array-format serialization
- Drag-to-reorder rows
- Client-side duplicate key prevention
- `min_rows` / `max_rows` constraints
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Props definition + serde | Rust library (`component.rs`) | — | All component schemas live here |
| HTML rendering | Rust library (`render.rs`) | — | Server-side HTML generation |
| Row cloning JS | Browser (`runtime/key_value_editor.rs`) | — | DOM mutation is client-side only |
| JSON serialization to hidden field | Browser (`runtime/key_value_editor.rs`) | — | Runs on user interaction, not at render time |
| data_path pre-fill | Rust library (`render.rs`) | — | Resolved at server render time from handler data |
| Public API export | `lib.rs` | — | All user-facing types re-exported from crate root |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0 | Props serialization/deserialization | Used by every component in the crate |
| serde_json | 1.0 | JSON value resolution, object entry iteration | Already a direct dependency |
| schemars | 1.x | `JsonSchema` derive on props struct | Used on all props structs without `Action` fields |

**No new dependencies needed.** [VERIFIED: ferro-json-ui/Cargo.toml]

---

## Architecture Patterns

### System Architecture Diagram

```
Handler data (serde_json::Value)
         │
         ▼
  resolve_path(data, data_path)
         │
         ▼  Option<&Value::Object>
  render_key_value_editor(props, data)
         │
         ├──► for each entry: emit <div data-kv-row> with key + value inputs
         ├──► emit <template data-kv-row-template> (empty row prototype)
         ├──► emit <datalist> if suggested_keys non-empty
         ├──► emit <button data-kv-add>
         └──► emit <input type="hidden" name="{field}" value="{serialized_json}">
                              │
                              ▼
                  browser: setupKeyValueEditor()
                              │
                  ┌───────────┼────────────┐
                  ▼           ▼            ▼
           add click    delete click   input event
                              │
                  syncHiddenField(editor)
                              │
                  write JSON.stringify(obj) → hidden input.value
```

### Recommended Project Structure

No new directories needed. Files to create/modify:

```
ferro-json-ui/src/
├── component.rs          # add KeyValueEditorProps struct + Component::KeyValueEditor variant
├── render.rs             # add render_key_value_editor() + dispatch arm
├── lib.rs                # add KeyValueEditorProps to re-exports
└── runtime/
    ├── mod.rs            # import key_value_editor, push SOURCE, add dispatcher call + update tests
    └── key_value_editor.rs   # NEW: pub(super) const SOURCE with setupKeyValueEditor()
```

### Pattern 1: Props Struct

**What:** `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]` — all four derives plus `JsonSchema` because `KeyValueEditorProps` has no `Action` fields (unlike `SwitchProps` which skips `JsonSchema`).

**serde defaults:** `Vec<String>` fields need `#[serde(default)]`; `bool` with default `true` needs `#[serde(default = "default_true")]`.

```rust
// Source: verified pattern from CheckboxProps / InputProps in component.rs
fn default_true() -> bool { true }

/// Props for KeyValueEditor component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KeyValueEditorProps {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub suggested_keys: Vec<String>,
    #[serde(default = "default_true")]
    pub allow_custom_keys: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

[VERIFIED: component.rs reading of CheckboxProps, SwitchProps, InputProps patterns]

### Pattern 2: Component Enum Insertion

**Insertion point:** After `Image(ImageProps)` at line 957, before `Plugin(PluginProps)`.

```rust
// In Component enum (component.rs ~line 956-957)
Image(ImageProps),
KeyValueEditor(KeyValueEditorProps),   // ADD HERE
Plugin(PluginProps),
```

Serialize match arm — after `Component::Image` arm (~line 1022), before `Component::Plugin`:

```rust
Component::KeyValueEditor(p) => serialize_tagged(serializer, "KeyValueEditor", p),
```

Deserialize match arm — after `"Image"` arm (~line 1153), before the `_` catch-all:

```rust
"KeyValueEditor" => serde_json::from_value::<KeyValueEditorProps>(value)
    .map(Component::KeyValueEditor)
    .map_err(de::Error::custom),
```

[VERIFIED: component.rs lines 917-1165]

### Pattern 3: render_component Dispatch

**Insertion point:** After the `// Form field components.` block in `render_component` (~line 313), alongside `render_input`, `render_select`, `render_checkbox`, `render_switch`.

```rust
// Source: render.rs ~line 309-313 pattern
Component::Input(props) => render_input(props, data),
Component::Select(props) => render_select(props, data),
Component::Checkbox(props) => render_checkbox(props, data),
Component::Switch(props) => render_switch(props, data),
Component::KeyValueEditor(props) => render_key_value_editor(props, data),  // ADD
```

[VERIFIED: render.rs lines 288-350]

### Pattern 4: data_path Resolution for JSON Objects

**Critical difference from other form components:** `data_path` for `KeyValueEditor` resolves to a JSON object, not a string. Use `resolve_path` (returns `Option<&Value>`), not `resolve_path_string`.

```rust
// Source: data.rs — resolve_path returns Option<&Value>, handles Object correctly
use crate::data::resolve_path;

let initial_entries: Vec<(String, String)> = if let Some(ref dp) = props.data_path {
    resolve_path(data, dp)
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let val_str = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string(other).unwrap_or_default(),
                    };
                    (k.clone(), val_str)
                })
                .collect()
        })
        .unwrap_or_default()
} else {
    vec![]
};
```

`resolve_path_string` would serialize the entire object as `{"key":"val"}` — not useful for row seeding. [VERIFIED: data.rs lines 15-57]

### Pattern 5: datalist Rendering

The `list="{field}-suggestions"` pattern is used by `InputProps::list`. For `KeyValueEditor`, the `<datalist>` is driven by `suggested_keys: Vec<String>` (not from the data `Value`), so rendering is simpler than the existing `render_input` datalist (which reads from `data[list_id]`).

```rust
// Datalist from suggested_keys — Vec<String> driven, not data-driven
if !props.suggested_keys.is_empty() {
    html.push_str(&format!(
        "<datalist id=\"{}-suggestions\">",
        html_escape(&props.field)
    ));
    for key in &props.suggested_keys {
        html.push_str(&format!("<option value=\"{}\">", html_escape(key)));
    }
    html.push_str("</datalist>");
}
```

[VERIFIED: render.rs lines 1488-1498 for existing pattern; adaptation for Vec<String> source is straightforward]

### Pattern 6: Runtime Module

**File structure (canonical: form_guards.rs):**

```rust
// ferro-json-ui/src/runtime/key_value_editor.rs
pub(super) const SOURCE: &str = r#"
    // ── Key-value editor ─────────────────────────────────────────────────

    function setupKeyValueEditor() {
        var editors = document.querySelectorAll('[data-kv-editor]');
        for (var i = 0; i < editors.length; i++) {
            initKeyValueEditor(editors[i]);
        }
    }

    function initKeyValueEditor(editor) {
        var rowsContainer = editor.querySelector('[data-kv-rows]');
        var addBtn = editor.querySelector('[data-kv-add]');
        var tmpl = editor.querySelector('[data-kv-row-template]');

        if (!rowsContainer || !addBtn || !tmpl) return;

        addBtn.addEventListener('click', function() {
            var clone = tmpl.content.cloneNode(true);
            rowsContainer.appendChild(clone);
            syncHiddenField(editor);
        });

        rowsContainer.addEventListener('click', function(e) {
            var delBtn = e.target.closest('[data-kv-delete]');
            if (!delBtn) return;
            var row = delBtn.closest('[data-kv-row]');
            if (row) {
                row.parentNode.removeChild(row);
                syncHiddenField(editor);
            }
        });

        rowsContainer.addEventListener('input', function(e) {
            if (e.target.hasAttribute('data-kv-key') || e.target.hasAttribute('data-kv-value')) {
                syncHiddenField(editor);
            }
        });
    }

    function syncHiddenField(editor) {
        var fieldName = editor.getAttribute('data-kv-field');
        var hiddenInput = editor.querySelector('input[name="' + fieldName + '"][type="hidden"]');
        if (!hiddenInput) return;
        var rows = editor.querySelectorAll('[data-kv-row]');
        var obj = {};
        for (var i = 0; i < rows.length; i++) {
            var keyEl = rows[i].querySelector('[data-kv-key]');
            var valEl = rows[i].querySelector('[data-kv-value]');
            var k = keyEl ? keyEl.value.trim() : '';
            var v = valEl ? valEl.value : '';
            if (k !== '') {
                obj[k] = v;
            }
        }
        hiddenInput.value = JSON.stringify(obj);
    }
"#;
```

[VERIFIED: form_guards.rs and kanban.rs — exact module format confirmed]

### Pattern 7: mod.rs IIFE Assembly

Three changes in `runtime/mod.rs`:

1. Add `mod key_value_editor;` to the module declarations at the top.
2. Add `s.push_str(key_value_editor::SOURCE);` to the `LazyLock` body after `kanban::SOURCE`.
3. Add `setupKeyValueEditor();` to the `ferroRuntime()` dispatcher string.
4. Update the `bundle_contains_all_setup_functions` test array to include `"setupKeyValueEditor"`.
5. Update the `dispatcher_invokes_every_setup` test array to include `"setupKeyValueEditor();"`.

[VERIFIED: runtime/mod.rs lines 1-162 — exact location of all five changes confirmed]

### Pattern 8: Initial JSON Serialization at Render Time

The hidden field `value` attribute is written at render time with the current object state. When `data_path` resolves to an object, serialize it. When no data, use `{}`.

```rust
let initial_json = if initial_entries.is_empty() {
    "{}".to_string()
} else {
    let obj: serde_json::Map<String, serde_json::Value> = initial_entries
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_else(|_| "{}".to_string())
};
// Emit: <input type="hidden" id="{field}" name="{field}" value="{initial_json}">
```

[VERIFIED: D-11 and D-17 from CONTEXT.md; data.rs confirms resolve_path handles objects]

### Anti-Patterns to Avoid

- **`resolve_path_string` for object data_path:** Returns JSON-serialized string, not iterable entries. Use `resolve_path(data, dp).and_then(|v| v.as_object())` instead.
- **JS string building for new rows:** Build HTML strings in JS to create new rows. Use `<template data-kv-row-template>` + `cloneNode(true)` — the UI spec mandates this.
- **Chevron SVG on select in key input:** `render_select` adds a chevron decoration. The `allow_custom_keys = false` key input is a compact inline select — no chevron per UI spec.
- **`JsonSchema` skip on KeyValueEditorProps:** Only `SwitchProps` skips `JsonSchema` (because it contains `Option<Action>`). `KeyValueEditorProps` has no `Action` fields, so `JsonSchema` must be derived.
- **Arrow functions or `const`/`let` in runtime JS:** All existing modules use `var` and named function declarations. ES5 compatibility is a hard requirement (D-15).
- **Per-row event listeners:** Wire `click` and `input` to the rows container with delegation, not to individual rows. Rows added dynamically after setup would not get per-row listeners.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON object serialization | Custom serializer | `serde_json::to_string` | Already a dep; handles all edge cases |
| HTML escaping | String replace | `html_escape()` in render.rs (pub(crate)) | Already handles `&`, `<`, `>`, `"`, `'` |
| JSON pointer resolution | Custom path walker | `resolve_path()` in data.rs | Handles nested paths, array indices, missing segments |
| Datalist rendering | Custom pattern | Mirror render_input datalist block | Established pattern, consistent output |

---

## Common Pitfalls

### Pitfall 1: Wrong data.rs function for object resolution
**What goes wrong:** Calling `resolve_path_string(data, dp)` when the resolved value is a JSON object returns `Some("{\"key\":\"val\"}")` — a JSON string, not an iterable map. Attempting to split or parse it manually is fragile.
**Why it happens:** `resolve_path_string` is used by all other form components because they need a single string value. `KeyValueEditor` needs object entries.
**How to avoid:** Use `resolve_path(data, dp).and_then(|v| v.as_object())` to get `Option<&Map<String, Value>>`, then iterate entries.
**Warning signs:** Initial rows show JSON string as key or value text, e.g. `{"metadata":"value"}` as a single key.

### Pitfall 2: Template element not inside the wrapper
**What goes wrong:** If the `<template data-kv-row-template>` is outside the `[data-kv-editor]` wrapper, the JS `editor.querySelector('[data-kv-row-template]')` returns `null` and add-row fails silently.
**How to avoid:** Render the `<template>` inside the outer wrapper `<div data-kv-editor ...>`, after the `<div data-kv-rows>` section. Match the exact HTML structure in the UI spec.

### Pitfall 3: Forgetting to update both runtime/mod.rs test arrays
**What goes wrong:** Adding `setupKeyValueEditor` to the dispatcher string but not to the test arrays causes `bundle_contains_all_setup_functions` or `dispatcher_invokes_every_setup` to pass vacuously (they use an explicit list). Easy to miss the second array.
**How to avoid:** Update both test `for fn_name in [...]` and `for call in [...]` arrays in `runtime/mod.rs` tests. CI `-D warnings` will not catch this — it is a functional test gap.

### Pitfall 4: Error state classes on delete/add buttons
**What goes wrong:** Applying `border-destructive` or `focus-visible:ring-destructive` to the delete or add-row buttons when `error` is set.
**How to avoid:** Error state only affects key and value input elements (`data-kv-key`, `data-kv-value`). Delete button and add-row button are not affected. Match UI spec exactly.

### Pitfall 5: Missing `html_escape` on dynamic values
**What goes wrong:** Emitting pre-filled key or value strings directly into `value="..."` attributes without escaping allows XSS if handler data contains `"` or `>` characters.
**How to avoid:** Call `html_escape()` on every key and value string from `initial_entries`, on `field`, on `label`, on `error`, and on every `suggested_keys` entry.

---

## Code Examples

### Exact render function structure (verified from render_select pattern)

```rust
// Source: render.rs render_select, render_input — verified patterns
fn render_key_value_editor(props: &KeyValueEditorProps, data: &Value) -> String {
    // 1. Resolve initial entries from data_path (use resolve_path, not resolve_path_string)
    // 2. Compute has_error, border_class, focus_ring_class
    // 3. Open outer wrapper: <div class="space-y-1" data-kv-editor data-kv-field="{field}">
    // 4. Optionally emit label
    // 5. Open rows container: <div class="space-y-2" data-kv-rows>
    // 6. For each initial entry: emit row div
    // 7. Close rows container
    // 8. Emit <template data-kv-row-template> with empty row
    // 9. Emit <datalist> if suggested_keys non-empty
    // 10. Emit add-row button
    // 11. Emit hidden input with initial_json
    // 12. Optionally emit error paragraph
    // 13. Close outer wrapper
}
```

### Hidden field with initial JSON

```rust
// Source: D-11, D-17 from CONTEXT.md
html.push_str(&format!(
    "<input type=\"hidden\" id=\"{}\" name=\"{}\" value=\"{}\">",
    html_escape(&props.field),
    html_escape(&props.field),
    html_escape(&initial_json)
));
```

### lib.rs re-export insertion

```rust
// Source: lib.rs lines 59-71 — add KeyValueEditorProps to the component:: import list
pub use component::{
    // ... existing exports ...
    KeyValueEditorProps,
    // ... rest ...
};
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — standard cargo test |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req | Behavior | Test Type | Command | Exists? |
|-----|----------|-----------|---------|---------|
| R1 | `render_key_value_editor` emits `data-kv-editor` wrapper | unit | `cargo test -p ferro-json-ui render_key_value_editor` | No — Wave 0 |
| R2 | Pre-filled rows from data_path | unit | same | No — Wave 0 |
| R3 | Error state classes on inputs | unit | same | No — Wave 0 |
| R4 | `allow_custom_keys = false` emits `<select>` key input | unit | same | No — Wave 0 |
| R5 | `suggested_keys` non-empty emits `<datalist>` | unit | same | No — Wave 0 |
| R6 | Empty data_path emits hidden field with `{}` | unit | same | No — Wave 0 |
| R7 | `setupKeyValueEditor` present in bundle | unit | `cargo test -p ferro-json-ui bundle_contains_all_setup_functions` | No — existing test needs update |
| R8 | `setupKeyValueEditor();` in dispatcher | unit | `cargo test -p ferro-json-ui dispatcher_invokes_every_setup` | No — existing test needs update |
| R9 | Serde round-trip: serialize then deserialize KeyValueEditor component | unit | `cargo test -p ferro-json-ui` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Tests for `render_key_value_editor` in `render.rs` `#[cfg(test)]` block — assert on HTML structure for: empty state, pre-filled rows, error state, select variant, datalist presence
- [ ] Serde round-trip test for `KeyValueEditorProps` serialization/deserialization
- [ ] Update `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` test arrays in `runtime/mod.rs`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `html_escape()` on all dynamic HTML output; server validates JSON field value |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via pre-filled key/value in HTML attributes | Tampering | `html_escape()` on all entries from `data_path` resolution |
| XSS via `suggested_keys` in datalist | Tampering | `html_escape()` on every `suggested_keys` entry |
| XSS via `label` / `error` in HTML | Tampering | `html_escape()` on label and error strings |
| JSON injection in hidden field `value` | Tampering | `html_escape()` on `initial_json` string (handles `"` in values) |

**Note:** The JS runtime (`syncHiddenField`) reads from DOM inputs — no eval, no innerHTML construction with untrusted data. The serialized value written to the hidden field is constructed via `JSON.stringify`, which is safe by default. The only XSS surface is server-side rendering, covered by `html_escape`.

---

## Open Questions

1. **`default_true` helper function — shared or per-module?**
   - What we know: Rust serde default functions must be free functions. `SwitchProps` doesn't need this because it uses `Option<bool>` for all toggles.
   - What's unclear: Is there already a `default_true` or similar helper elsewhere in `component.rs`?
   - Recommendation: Define `fn default_allow_custom_keys() -> bool { true }` locally in `component.rs` above `KeyValueEditorProps`. Check for duplicates first with a quick grep.

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies. This phase is pure Rust crate code modification with no new binaries, services, or external tools.

---

## Sources

### Primary (HIGH confidence)
- `ferro-json-ui/src/component.rs` (lines 232-377, 916-1167) — verified enum variant positions, serialize/deserialize match arm positions, existing props struct derives
- `ferro-json-ui/src/render.rs` (lines 288-350, 1370-1617) — verified `render_component` dispatch, `render_input`, `render_select` patterns; `html_escape` signature
- `ferro-json-ui/src/runtime/mod.rs` (all) — verified IIFE assembly pattern, test arrays
- `ferro-json-ui/src/runtime/form_guards.rs` (all) — canonical runtime module pattern
- `ferro-json-ui/src/data.rs` (all) — verified `resolve_path` vs `resolve_path_string` behavior for objects
- `ferro-json-ui/src/lib.rs` (all) — verified re-export block structure
- `ferro-json-ui/Cargo.toml` — confirmed no new dependencies needed
- `.planning/phases/146-.../146-CONTEXT.md` — all implementation decisions
- `.planning/phases/146-.../146-UI-SPEC.md` — HTML skeleton, data attributes, JS pseudocode

### Secondary (MEDIUM confidence)
- None — all claims verified directly from codebase.

### Tertiary (LOW confidence)
- None.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `default_true` helper does not already exist in `component.rs` | Standard Stack / Props Struct | Low — if it exists, just reuse it; no functional impact |

---

## Metadata

**Confidence breakdown:**
- Insertion points (line numbers): HIGH — read directly from source
- Props struct pattern: HIGH — verified against CheckboxProps, InputProps, SwitchProps
- Runtime module pattern: HIGH — verified against form_guards.rs and mod.rs
- data_path object resolution: HIGH — verified resolve_path behavior in data.rs tests
- Datalist pattern: HIGH — verified in render_input lines 1488-1498

**Research date:** 2026-04-22
**Valid until:** 2026-05-22 (stable codebase, no external deps)

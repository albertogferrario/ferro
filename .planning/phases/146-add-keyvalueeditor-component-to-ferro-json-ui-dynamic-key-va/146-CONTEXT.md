# Phase 146: Add KeyValueEditor component to ferro-json-ui — Context

**Gathered:** 2026-04-22
**Status:** Ready for planning
**Mode:** `--auto` (single-pass, recommended defaults selected for all gray areas)

<domain>
## Phase Boundary

Add a `KeyValueEditor` component to `ferro-json-ui`. The component renders a dynamic list of key-value rows (key `<input>` + value `<input>` per row) with an "Add" button and per-row delete. On every mutation the component serializes the current pairs as a JSON object into a hidden `<input>` field — the field that the server reads.

Suggested keys are exposed via a native `<datalist>` on the key input so the browser shows autocomplete without extra JS logic. An `allow_custom_keys` flag controls whether the user may type keys not in the suggestion list. Pre-fill from handler data is supported via `data_path` (resolves to a JSON object whose entries seed the initial rows).

Runtime wiring lives in a new `ferro-json-ui/src/runtime/key_value_editor.rs` module, following the existing single-concern module pattern, assembled into `FERRO_RUNTIME_JS` in `runtime/mod.rs`.

**Primary files touched:**
- `ferro-json-ui/src/component.rs` — `KeyValueEditorProps` struct + `Component::KeyValueEditor` variant
- `ferro-json-ui/src/render.rs` — `render_key_value_editor()` function
- `ferro-json-ui/src/runtime/key_value_editor.rs` — new JS module (`setupKeyValueEditor`)
- `ferro-json-ui/src/runtime/mod.rs` — wire new module into IIFE bundle

**Out of scope:** array-format serialization, duplicate keys, ordered pairs, configurable debounce, frontend-only state ($state/$bindState), drag-to-reorder rows.

</domain>

<decisions>
## Implementation Decisions

### Props structure
- **D-01:** `KeyValueEditorProps` has the following fields:
  - `field: String` — name of the hidden `<input>` that receives the serialized JSON
  - `label: Option<String>` — optional visible label for the editor block
  - `suggested_keys: Vec<String>` — keys exposed as `<datalist>` options; empty = no suggestions
  - `allow_custom_keys: bool` (default `true`) — if false, key input is a `<select>` from `suggested_keys` only
  - `data_path: Option<String>` — JSON pointer path; value must resolve to a JSON object; each entry seeds one row
  - `error: Option<String>` — validation error rendered below the editor
- **D-02:** Mirrors the field/label/data_path/error shape of existing form components (Input, Select, Checkbox, Switch).

### Suggested keys UX
- **D-03:** When `allow_custom_keys` is `true` (default), the key input is `<input type="text">` with a `list="…"` pointing to a `<datalist>` populated from `suggested_keys`. Native browser autocomplete; no extra JS required.
- **D-04:** When `allow_custom_keys` is `false`, the key input renders as `<select>` with `<option>` elements from `suggested_keys`. Value input remains a free-text `<input>`.

### Row add/remove UI
- **D-05:** Rows render as a compact two-column layout (`key` | `value` | `×`) — three columns, key and value expand equally, delete button is narrow.
- **D-06:** An "+ Add row" button appears below the row list. Clicking it appends a new empty row.
- **D-07:** The `×` delete button on each row removes that row immediately. No confirmation prompt.
- **D-08:** Empty rows (both key and value blank) are excluded from the serialized JSON on sync.

### JSON serialization format
- **D-09:** Serialization target: JSON object `{"key1": "value1", "key2": "value2"}`. Written to the hidden `<input>` as the field's `value` attribute.
- **D-10:** Duplicate keys are not supported — if two rows share the same key, last-write wins when serializing. The UI does not prevent duplicate key entry for v1.
- **D-11:** The hidden field is initialized with the serialized JSON at render time (from `data_path` or empty object `{}`). The JS runtime syncs it on every add, remove, or input event.

### Runtime architecture
- **D-12:** New `ferro-json-ui/src/runtime/key_value_editor.rs` file. Exports `pub(super) const SOURCE: &str` containing `setupKeyValueEditor()` — same pattern as `form_guards.rs`, `kanban.rs`, etc.
- **D-13:** `setupKeyValueEditor()` uses `data-kv-editor` attribute on the wrapper element as the selector. Wires "add row" click, "delete row" click, and `input` event delegation for serialization.
- **D-14:** `runtime/mod.rs`: import the module, push `SOURCE` into the bundle, add `setupKeyValueEditor();` to the `ferroRuntime()` dispatcher.
- **D-15:** Vanilla JS only — `var`, ES5-compatible, matching existing modules. No closures capturing `this`; rely on `data-*` attributes for state.

### Data binding
- **D-16:** `data_path` follows the same resolution logic as existing components: slash-separated JSON pointer (e.g. `/data/metadata`). The resolved value must be a JSON object (`serde_json::Value::Object`); other types are treated as absent.
- **D-17:** The hidden field `value` attribute is written at render time with the serialized JSON (or `{}` if no data). The JS runtime does not perform an initial parse of the HTML — it only syncs on user interaction.

### Claude's Discretion
- Row template used by the "Add row" JS: embed as an HTML `<template>` element in the rendered output so the runtime can `cloneNode(true)` it — avoids building HTML strings in JS.
- Exact Tailwind classes for row layout, button styling, and error display — follow the existing Input/Select rendering patterns (semantic tokens, `border-border`, `border-destructive` for errors).
- Test coverage: follow existing `render_*` test patterns in `render.rs` (assert on rendered HTML structure). Add JS-bundle tests for `setupKeyValueEditor` presence in `runtime/mod.rs` tests.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing component patterns
- `ferro-json-ui/src/component.rs` — `InputProps`, `SelectProps`, `CheckboxProps`, `SwitchProps` for field/label/data_path/error shape and serde attributes
- `ferro-json-ui/src/render.rs` — `render_input`, `render_select`, `render_checkbox`, `render_switch` for rendering conventions (semantic tokens, html_escape, ARIA attributes, error classes)

### Runtime module pattern
- `ferro-json-ui/src/runtime/form_guards.rs` — canonical example of a runtime module (SOURCE const, vanilla JS, data-* attributes)
- `ferro-json-ui/src/runtime/mod.rs` — IIFE assembly, dispatcher, tests that verify each setup function is present and called

### Data binding pattern
- `ferro-json-ui/src/data.rs` — `resolve_path_string` used by existing render functions for `data_path` resolution

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `resolve_path_string(data, path)` in `data.rs`: resolves a slash-separated JSON pointer from `&Value` → `Option<String>`; used by all form components for `data_path`
- `html_escape(&str)` in `render.rs`: must be called on all dynamic values emitted into HTML attributes and text nodes
- `InputProps::list` + `<datalist>` rendering (render_input, ~line 1488): exact pattern for rendering a `<datalist>` from a list of strings — reuse directly for `suggested_keys`
- `SelectProps` + `render_select`: pattern for rendering a `<select>` with `<option>` elements when `allow_custom_keys` is false

### Established Patterns
- Component enum: serde tagged `{"type": "KeyValueEditor", ...}` via `serialize_tagged` + custom `Deserialize` match arm
- Props struct: `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]` — add `JsonSchema` only if no `Action` fields (SwitchProps skips JsonSchema due to Action's custom serde)
- Runtime module: `pub(super) const SOURCE: &str = r#"..."#;` with vanilla JS, `data-*` attribute selectors, event delegation
- HTML `<template>` element for JS-clonable row prototype: supported in all modern browsers, avoids JS string building

### Integration Points
- `Component` enum (component.rs ~line 917): add `KeyValueEditor(KeyValueEditorProps)` variant
- `render_component` dispatch (render.rs ~line 288): add `Component::KeyValueEditor(p) => render_key_value_editor(p, data)`
- Serialize/Deserialize match arms (component.rs ~line 986 / ~line 1051): add `KeyValueEditor` case
- `runtime/mod.rs`: push `key_value_editor::SOURCE`, add `setupKeyValueEditor();` to dispatcher and update all runtime bundle tests
- `lib.rs` re-exports: export `KeyValueEditorProps` from the crate public API

</code_context>

<specifics>
## Specific Ideas

- The `<template>` element approach for row cloning was identified as the cleanest way to avoid JS string-building for new rows — row template is rendered by Rust inside the component wrapper.
- The `data-kv-editor` attribute on the wrapper (not `id`) keeps the selector reusable when multiple `KeyValueEditor` components appear on the same page.
- Hidden field name = `field` prop; its `id` should also be `field` for JS targeting consistency with other form components.

</specifics>

<deferred>
## Deferred Ideas

- Array-format serialization (`[{"key": "k", "value": "v"}]`) — needed if duplicate keys or ordering matter; defer until a real use case arrives.
- Drag-to-reorder rows — UI complexity; not needed for v1.
- Client-side duplicate key prevention (red highlight) — nice UX but not required for correctness.
- `min_rows` / `max_rows` constraints — out of scope for v1.

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va*
*Context gathered: 2026-04-22*

# Phase 146: Add KeyValueEditor component to ferro-json-ui — Pattern Map

**Mapped:** 2026-04-22
**Files analyzed:** 5
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | model | transform | `CheckboxProps`, `SwitchProps`, `InputProps` (lines 232–384) | exact |
| `ferro-json-ui/src/render.rs` | renderer | request-response | `render_input` (lines 1372–1519), `render_select` (lines 1521–1617) | exact |
| `ferro-json-ui/src/runtime/key_value_editor.rs` | utility | event-driven | `ferro-json-ui/src/runtime/form_guards.rs` (all) | exact |
| `ferro-json-ui/src/runtime/mod.rs` | config | batch | `runtime/mod.rs` existing IIFE pattern (lines 1–162) | exact |
| `ferro-json-ui/src/lib.rs` | config | transform | `lib.rs` lines 59–71 re-export block | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/component.rs` — Props struct + enum variant + serde arms

**Analog:** `CheckboxProps` / `SwitchProps` / `InputProps` in `ferro-json-ui/src/component.rs`

**Props struct derive pattern** (lines 332–354, `CheckboxProps`):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CheckboxProps {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**`default_true` helper — already exists** (lines 584–586); do NOT add a duplicate:
```rust
fn default_true() -> bool {
    true
}
```
Usage example from `ChecklistProps` (line 575): `#[serde(default = "default_true")]`

**`KeyValueEditorProps` struct to add** — mirrors the shape above, with `Vec<String>` and `bool` fields added:
```rust
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
Note: `JsonSchema` is included because `KeyValueEditorProps` has no `Action` fields. `SwitchProps` (lines 356–384) omits it — do not copy that omission here.

**Component enum variant insertion** — after `Image(ImageProps)` at line 956, before `Plugin(PluginProps)`:
```rust
Image(ImageProps),
KeyValueEditor(KeyValueEditorProps),   // INSERT HERE
Plugin(PluginProps),
```

**Serialize match arm** (copy pattern from lines 1022–1023, insert before Plugin arm):
```rust
Component::Image(p) => serialize_tagged(serializer, "Image", p),
Component::KeyValueEditor(p) => serialize_tagged(serializer, "KeyValueEditor", p),
Component::Plugin(p) => p.serialize(serializer),
```

**Deserialize match arm** (copy pattern from lines 1153–1155, insert before the `_` catch-all):
```rust
"Image" => serde_json::from_value::<ImageProps>(value)
    .map(Component::Image)
    .map_err(de::Error::custom),
"KeyValueEditor" => serde_json::from_value::<KeyValueEditorProps>(value)
    .map(Component::KeyValueEditor)
    .map_err(de::Error::custom),
_ => {
    // Unknown type: treat as a plugin component.
```

---

### `ferro-json-ui/src/render.rs` — `render_key_value_editor()` + dispatch arm

**Analog:** `render_input` (lines 1372–1519) and `render_select` (lines 1521–1617)

**Dispatch arm insertion** (after `Component::Switch` arm at line 313, inside the `// Form field components.` block):
```rust
// Form field components.
Component::Input(props) => render_input(props, data),
Component::Select(props) => render_select(props, data),
Component::Checkbox(props) => render_checkbox(props, data),
Component::Switch(props) => render_switch(props, data),
Component::KeyValueEditor(props) => render_key_value_editor(props, data),  // ADD
```

**Error state pattern** (lines 1393–1403 from `render_input`):
```rust
let has_error = props.error.is_some();
let border_class = if has_error {
    "border-destructive"
} else {
    "border-border"
};
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```

**Label pattern** (line 1406–1410 from `render_input`):
```rust
let mut html = String::from("<div class=\"space-y-1\">");
html.push_str(&format!(
    "<label class=\"block text-sm font-medium text-text\" for=\"{}\">{}</label>",
    html_escape(&props.field),
    html_escape(&props.label)
));
```

**Datalist from a `Vec<String>` field** (adapted from lines 1488–1497 which read from `data`; `KeyValueEditor` uses `suggested_keys` directly — no data lookup needed):
```rust
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

**Hidden input with initial JSON** (mirrors lines 1384–1390 from `render_input` hidden branch):
```rust
html.push_str(&format!(
    "<input type=\"hidden\" id=\"{}\" name=\"{}\" value=\"{}\">",
    html_escape(&props.field),
    html_escape(&props.field),
    html_escape(&initial_json)
));
```

**Error paragraph pattern** (lines 1509–1516 from `render_input`):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
html.push_str("</div>");
```

**data_path resolution — CRITICAL: use `resolve_path`, not `resolve_path_string`** (`data.rs` lines 15–41). All other form components call `resolve_path_string` because they need a single string. `KeyValueEditor` must iterate object entries:
```rust
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

**Initial JSON serialization for the hidden field**:
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
```

**Full render function structure** (ordered steps):
```rust
fn render_key_value_editor(props: &KeyValueEditorProps, data: &Value) -> String {
    // 1. Resolve initial_entries via resolve_path(data, dp).and_then(|v| v.as_object())
    // 2. Compute initial_json from entries (or "{}")
    // 3. Compute has_error, border_class, focus_ring_class
    // 4. Open: <div class="space-y-1" data-kv-editor data-kv-field="{field}">
    // 5. Optionally emit label (html_escape label text)
    // 6. Open rows container: <div class="space-y-2" data-kv-rows>
    // 7. For each initial entry: emit <div data-kv-row class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center">
    //    key input (text or select based on allow_custom_keys) + value input + delete button
    // 8. Close rows container
    // 9. Emit <template data-kv-row-template> with empty row (same structure as step 7)
    // 10. Emit <datalist> if suggested_keys non-empty (only when allow_custom_keys = true)
    // 11. Emit add-row button: <button type="button" data-kv-add>
    // 12. Emit hidden input: <input type="hidden" id="{field}" name="{field}" value="{initial_json}">
    // 13. Optionally emit error paragraph with id="err-{field}"
    // 14. Close outer wrapper </div>
}
```

**`allow_custom_keys = false` key input** — inline `<select>` without chevron decoration (unlike `render_select` which wraps in `<div class="relative">` and adds SVG chevron; the key select is compact and inline):
```rust
// allow_custom_keys = false: key cell is a plain <select>
html.push_str(&format!(
    "<select data-kv-key name=\"\" class=\"rounded-md border {} px-2 py-1.5 text-sm {}\">",
    border_class, focus_ring_class
));
for key in &props.suggested_keys {
    html.push_str(&format!("<option value=\"{}\">{}</option>",
        html_escape(key), html_escape(key)));
}
html.push_str("</select>");
```

**Test pattern** — add inline `#[cfg(test)]` tests at the bottom of `render.rs` following the existing test block structure. Assert on substrings of rendered HTML (no snapshots):
```rust
#[test]
fn render_key_value_editor_empty_state() {
    let props = KeyValueEditorProps { field: "meta".to_string(), .. };
    let html = render_key_value_editor(&props, &serde_json::Value::Null);
    assert!(html.contains("data-kv-editor"));
    assert!(html.contains(r#"name="meta""#));
    assert!(html.contains(r#"value="{}""#));
}
```

---

### `ferro-json-ui/src/runtime/key_value_editor.rs` — NEW runtime module

**Analog:** `ferro-json-ui/src/runtime/form_guards.rs` (entire file)

**File structure** (verbatim file shape from `form_guards.rs`, lines 1–77):
```rust
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

**JS rules enforced by the existing codebase (D-15):**
- `var` only — no `const`, no `let`
- Named function declarations — no arrow functions
- Event delegation on containers — no per-row event listeners
- `data-*` attributes for all state — no closures capturing variables from outer scope
- No `innerHTML` construction with untrusted values

---

### `ferro-json-ui/src/runtime/mod.rs` — IIFE assembly wiring

**Analog:** existing `runtime/mod.rs` (all, lines 1–162)

**Five changes required:**

1. **Module declaration** — add after `mod kanban;` at line 11:
```rust
mod kanban;
mod key_value_editor;   // ADD
mod modals;
```

2. **SOURCE push** — add after `s.push_str(kanban::SOURCE);` at line 38:
```rust
s.push_str(kanban::SOURCE);
s.push_str(key_value_editor::SOURCE);   // ADD
```

3. **Dispatcher call** — add `setupKeyValueEditor();` to the `ferroRuntime()` string (lines 39–54). Place after `setupKanban();`:
```
setupKanban();\n\
setupKeyValueEditor();\n\
```

4. **`bundle_contains_all_setup_functions` test array** (lines 115–127) — add `"setupKeyValueEditor"`:
```rust
for fn_name in [
    "setupSSE",
    "setupTabs",
    // ... existing ...
    "setupKanban",
    "setupKeyValueEditor",   // ADD
] {
```

5. **`dispatcher_invokes_every_setup` test array** (lines 146–158) — add `"setupKeyValueEditor();"`:
```rust
for call in [
    "setupSSE();",
    // ... existing ...
    "setupKanban();",
    "setupKeyValueEditor();",   // ADD
] {
```

---

### `ferro-json-ui/src/lib.rs` — public re-export

**Analog:** `lib.rs` lines 59–71 existing `component::` import list

**Single line addition** — add `KeyValueEditorProps` to the `pub use component::{...}` block (alphabetical position between `KanbanColumnProps` and `ModalProps`):
```rust
pub use component::{
    // ... existing exports ...
    KanbanBoardProps, KanbanColumnProps,
    KeyValueEditorProps,   // ADD
    ModalProps,
    // ... rest ...
};
```

Also add to `COMPONENT_CATALOG` const (lines 102–182) — a new section entry following the `### Switch` entry pattern:
```
### KeyValueEditor
Props: field (String), label (Option<String>), suggested_keys (Vec<String>), allow_custom_keys (bool, default true), data_path (Option<String>), error (Option<String>)
```

---

## Shared Patterns

### html_escape — apply to all dynamic HTML output
**Source:** `ferro-json-ui/src/render.rs` (`pub(crate) fn html_escape`) — already in scope inside `render.rs`
**Apply to:** every dynamic string emitted into HTML: `field`, `label`, `error`, each `suggested_keys` entry, each pre-filled key and value string, and `initial_json`

### Error state — border and focus ring classes
**Source:** `render_input` lines 1393–1403
**Apply to:** key input elements and value input elements when `has_error` is true. Does NOT apply to delete button or add-row button.
```rust
let border_class = if has_error { "border-destructive" } else { "border-border" };
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```

### Error paragraph with ARIA id
**Source:** `render_input` lines 1509–1516
**Apply to:** `render_key_value_editor` — use same `id="err-{field}"` so server-rendered ARIA linkage is consistent.

### `serialize_tagged` helper
**Source:** `component.rs` lines 964–977 — already defined, no change needed.

### serde arm pattern for new Component variants
**Source:** `component.rs` lines 1038–1165 (Deserialize impl match block)
Both serialize and deserialize arms follow an identical three-line pattern. Copy from the `Image` arms immediately above the `Plugin` fallback.

---

## No Analog Found

None — all five files have exact or near-exact analogs in the codebase.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/` (component.rs, render.rs, data.rs, lib.rs, runtime/*)
**Files scanned:** 7 source files read directly
**Pattern extraction date:** 2026-04-22

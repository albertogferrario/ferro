# Phase 162: JSON-UI Improvements Batch 1 — Pattern Map

**Mapped:** 2026-05-16
**Files analyzed:** 18 new/modified files
**Analogs found:** 18 / 18 (all files have codebase analogs)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` (add `CheckboxListProps`) | model/props | CRUD | `CheckboxProps` (line 323), `SelectProps` (line 259) | exact |
| `ferro-json-ui/src/component.rs` (add `SwitchProps.compact`) | model/props | request-response | `SwitchProps` (line 348) | exact — add one field |
| `ferro-json-ui/src/component.rs` (add `ImageProps.inline_svg`) | model/props | request-response | `ImageProps` (line 449) | exact — add one field |
| `ferro-json-ui/src/component.rs` (add `RichTextEditorProps`) | model/props | request-response | `CheckboxProps` (line 323), `InputProps` (line 222) | role-match |
| `ferro-json-ui/src/component.rs` (add strum derives) | model/props | request-response | `AlertVariant` (line 81), `ButtonVariant` (line 50) | exact — add derive |
| `ferro-json-ui/src/action.rs` (add strum derives) | model/props | request-response | `DialogVariant` (line 12), `NotifyVariant` (line 43) | exact — add derive |
| `ferro-json-ui/src/render/form.rs` (add `render_checkbox_list`) | render function | request-response | `render_checkbox` (line 398), `render_switch` (line 465) | exact |
| `ferro-json-ui/src/render/form.rs` (update `render_switch` for compact) | render function | request-response | `render_switch` (line 465) | exact — CSS toggle |
| `ferro-json-ui/src/render/atoms.rs` (update `render_image` for inline_svg) | render function | request-response | `render_image` (line 365) | exact — add branch |
| `ferro-json-ui/src/render/data.rs` (extend `template_actions`) | render function | transform | `template_actions` (line 285) | exact — extend loop |
| `ferro-json-ui/src/render/mod.rs` (bump count + dispatch) | registry | request-response | existing `BUILTIN_TYPES` array (line 41) + count assert (line 526) | exact |
| `ferro-json-ui/src/catalog.rs` (add BUILTIN_SPECS entries) | registry | request-response | existing `BUILTIN_SPECS` entries (line 123) | exact |
| `ferro-json-ui/src/spec.rs` (add `FooterMissing` + `validate_footer_ids`) | validator | request-response | `validate_no_dangling` (line 457), `SpecError::DanglingChild` (line 107) | exact |
| `ferro-json-ui/src/layout.rs` (remove card wrapper) | layout | request-response | `AuthLayout.render` (line 367) | exact — delete 3 lines |
| `ferro-json-ui/src/plugins/rich_text_editor.rs` (NEW) | plugin | request-response | `MapPlugin` in `plugins/map.rs` | exact |
| `ferro-mcp/src/tools/json_ui_verify_action.rs` (NEW) | MCP tool | request-response | `list_routes.rs` (full file) | exact |
| `ferro-mcp/src/tools/mod.rs` (register new tool) | registry | — | existing `pub mod` list (line 1-60) | exact |
| `ferro-mcp/src/tools/code_templates.rs` (add migration templates) | MCP tool | request-response | `handler_templates()` function (line 81) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` (bump count assertion) | MCP tool | request-response | `test_all_components_present` (line 235) | exact |
| `docs/src/json-ui/migration-v1-to-v2.md` (NEW) | documentation | — | `docs/src/json-ui/components.md` | role-match |
| `docs/src/json-ui/plugins.md` (NEW) | documentation | — | `docs/src/json-ui/components.md` | role-match |
| `docs/src/SUMMARY.md` (add nav entries) | config | — | existing nav list | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/component.rs` — `CheckboxListProps` (NEW struct)

**Analog:** `CheckboxProps` (lines 322–345) and `SelectProps` (lines 258–281)

**`CheckboxProps` pattern** (lines 322–345 — the per-option checkbox primitive to iterate):
```rust
/// Props for Checkbox component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CheckboxProps {
    /// Form field name for data binding.
    pub field: String,
    /// HTML value attribute. When set, the checkbox submits this value instead of "1".
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
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**`SelectProps` naming convention for options fields** (lines 258–281 — `options: Vec<SelectOption>` + `data_path`):
```rust
pub struct SelectProps {
    pub field: String,
    pub label: String,
    pub options: Vec<SelectOption>,          // <-- field name: "options" not "items"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,           // <-- for pre-fill; use "options_path" for the data-driven options array
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    // ...
}
```

**`SelectOption` (already exists at line 136 — reuse directly):**
```rust
pub struct SelectOption {
    pub value: String,
    pub label: String,
}
```

**Key conventions for `CheckboxListProps`:**
- Use `options: Vec<SelectOption>` (not `items`) — matches `SelectProps` naming
- Use `options_path: Option<String>` for data-driven array (not `default_value_path`)
- Use `selected_path: Option<String>` for pre-selected values — matches CONTEXT.md D-01
- All optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`
- `Vec<SelectOption>` static field uses `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
- Derive line MUST include `JsonSchema` — consumed by catalog.rs schema generation
- No `Default` derive on the struct (required `field: String` makes Default impossible)

---

### `ferro-json-ui/src/component.rs` — `SwitchProps.compact` (ADD field)

**Analog:** `SwitchProps` lines 347–370 (add one field at the end):
```rust
pub struct SwitchProps {
    pub field: String,
    pub label: String,
    // ... existing optional fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    // ADD HERE:
    /// When true, applies `scale-75 origin-left` CSS to the switch container
    /// for compact inline display (e.g. per-row settings toggles).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
}
```

No derives change on `SwitchProps` — it already has `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]`.

---

### `ferro-json-ui/src/component.rs` — `ImageProps.inline_svg` (ADD field)

**Analog:** `ImageProps` lines 448–461 (add one field alongside `src`):
```rust
/// Props for Image component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImageProps {
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
    // ADD HERE — parallel field approach (preserves "src" wire format):
    /// Server-rendered inline SVG string. When set, the SVG is emitted
    /// verbatim in a `<div aria-label="{alt}">` wrapper; no `<img>` tag.
    /// Server-only: content is NOT sanitized. `alt` is required.
    /// Use case: server-constructed bar charts, QR codes, icons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_svg: Option<String>,
}
```

Note: `Eq` must be dropped from the derive list if the SVG string type prevents `Eq` — but `String` implements `Eq`, so the existing derive is fine.

**Factory method** (add as an `impl ImageProps` block, consistent with CONTEXT D-17):
```rust
impl ImageProps {
    /// Convenience constructor for inline SVG images.
    pub fn inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: String::new(),
            alt: alt.into(),
            aspect_ratio: None,
            placeholder_label: None,
            inline_svg: Some(svg.into()),
        }
    }
}
```

---

### `ferro-json-ui/src/component.rs` — `RichTextEditorProps` (NEW struct)

**Analog:** `InputProps` (lines 222–256) — same field shape for a text input with data binding:
```rust
pub struct InputProps {
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // ...
}
```

`RichTextEditorProps` follows the same shape. Note: RichTextEditor is a leaf element whose render is handled by the plugin system, not by a function in `render/form.rs`.

---

### `ferro-json-ui/src/component.rs` + `ferro-json-ui/src/action.rs` — strum derives (ADD derives to 6 enums)

**Analog:** `AlertVariant` (lines 81–89 of component.rs) and `DialogVariant` (lines 12–18 of action.rs):
```rust
// BEFORE (component.rs line 81):
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertVariant { ... }

// AFTER:
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema,
         strum::AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]   // must match serde wire format
pub enum AlertVariant { ... }
```

Apply this pattern to all six enums:
- `component.rs`: `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant`
- `action.rs`: `DialogVariant`, `NotifyVariant`

The `#[strum(serialize_all = "snake_case")]` attribute ensures `AlertVariant::Success.as_ref()` returns `"success"` — matching the JSON wire format.

**Cargo.toml change required** (`ferro-json-ui/Cargo.toml`):
```toml
strum = { version = "0.26", features = ["derive"] }
```

---

### `ferro-json-ui/src/render/form.rs` — `render_checkbox_list` (NEW function)

**Analog:** `render_checkbox` (lines 393–457) — the per-item render pattern; and `render_switch` (lines 465+) — the props-decode pattern:

**Props decode pattern** (copy from `render_checkbox` lines 398–407):
```rust
pub(crate) fn render_checkbox_list(el: &Element, spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: CheckboxListProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode CheckboxList props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };
    // ...
}
```

**options resolution pattern** (derive from SelectProps data_path logic in render/form.rs around line 310):
```rust
// Resolve options: data-driven path wins over static vec.
let options: Vec<SelectOption> = if let Some(ref path) = props.options_path {
    match resolve_path(data, path).and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| serde_json::from_value::<SelectOption>(v.clone()).ok())
            .collect(),
        None => props.options.clone(),
    }
} else {
    props.options.clone()
};
```

**selected_path resolution pattern** (derive from `resolve_checked` helper in same file):
```rust
// Resolve pre-selected values from data path.
let selected: Vec<String> = props.selected_path.as_deref()
    .and_then(|path| resolve_path(data, path))
    .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
    .unwrap_or_default();
```

**per-option HTML pattern** (copy from `render_checkbox` lines 417–440):
```rust
// Wrap in <fieldset> for a11y.
let mut html = String::from("<fieldset class=\"space-y-2\">");
if let Some(ref label) = props.label {
    html.push_str(&format!(
        "<legend class=\"text-sm font-medium text-text\">{}</legend>",
        html_escape(label)
    ));
}
for option in &options {
    let is_checked = selected.contains(&option.value);
    let checkbox_id = format!("{}_{}", props.field, option.value);
    html.push_str("<div class=\"flex items-center gap-2\">");
    html.push_str(&format!(
        "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" \
         class=\"h-4 w-4 rounded-sm border-border text-primary ...\"",
        html_escape(&checkbox_id),
        html_escape(&props.field),
        html_escape(&option.value)
    ));
    if is_checked { html.push_str(" checked"); }
    if props.disabled == Some(true) { html.push_str(" disabled"); }
    html.push('>');
    html.push_str(&format!(
        "<label class=\"text-sm font-medium text-text\" for=\"{}\">{}</label>",
        html_escape(&checkbox_id),
        html_escape(&option.label)
    ));
    html.push_str("</div>");
}
// ... error / description after the loop (copy from render_checkbox lines 442–455)
html.push_str("</fieldset>");
html
```

**Test pattern** (copy from `data.rs` tests — `mk_element` + `mk_spec` helpers, lines 356–375):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mk_element(type_name: &str, props: serde_json::Value) -> Element {
        Element {
            type_name: type_name.to_string(),
            props,
            children: Vec::new(),
            action: None,
            visible: None,
        }
    }

    fn mk_spec(root: &str, el: Element) -> Spec {
        let mut spec = Spec::builder()
            .element("__tmp__", Element::new("Text"))
            .build()
            .expect("builder accepts trivial spec");
        spec.root = root.to_string();
        spec.elements.clear();
        spec.elements.insert(root.to_string(), el);
        spec
    }

    #[test]
    fn checkbox_list_renders_one_checkbox_per_option() { ... }

    #[test]
    fn checkbox_list_selected_path_prechecks_matching_options() { ... }
}
```

---

### `ferro-json-ui/src/render/form.rs` — `render_switch` compact update

**Analog:** `render_switch` lines 465–530 — find the switch container div and add CSS class conditionally:
```rust
// In render_switch, when building the switch container div:
let compact_class = if props.compact == Some(true) {
    " scale-75 origin-left"
} else {
    ""
};
// Apply to the outermost wrapper div:
html.push_str(&format!(
    "<div class=\"flex items-center gap-3{compact_class}\">",
));
```

---

### `ferro-json-ui/src/render/atoms.rs` — `render_image` inline SVG branch

**Analog:** `render_image` lines 365–394 — add a branch before the standard `<img>` path:
```rust
pub(crate) fn render_image(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ImageProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Image", e),
    };

    // D-17: inline SVG branch — emit verbatim, no <img> tag.
    // Server-only; content is NOT sanitized; alt is required.
    if let Some(ref svg) = props.inline_svg {
        return format!(
            "<div aria-label=\"{}\">{}</div>",
            html_escape(&props.alt),
            svg   // verbatim — intentionally not escaped
        );
    }

    // ... existing img path unchanged ...
}
```

---

### `ferro-json-ui/src/render/data.rs` — `template_actions` extension (D-03/D-04)

**Analog:** `template_actions` lines 285–316 — the existing substitution loop:
```rust
fn template_actions(
    actions: &[DropdownMenuAction],
    row: &Value,
    row_key_value: &str,
) -> Vec<DropdownMenuAction> {
    let id_value: Option<String> = row.get("id").and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    });

    actions
        .iter()
        .map(|a| {
            let mut cloned = a.clone();
            let base_url = cloned.action.url.clone()
                .or_else(|| Some(cloned.action.handler.clone()));
            if let Some(mut url) = base_url {
                // EXISTING: row_key and id substitution
                url = url.replace("{row_key}", row_key_value);
                if let Some(ref id) = id_value {
                    url = url.replace("{id}", id);
                }
                // ADD (D-04): iterate all row columns for named-column placeholders.
                // Do this BEFORE row_key/id so column keys always get priority.
                if let Some(obj) = row.as_object() {
                    for (col_key, col_val) in obj {
                        let placeholder = format!("{{{col_key}}}");
                        let val_str = match col_val {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        url = url.replace(&placeholder, &val_str);
                    }
                }
                cloned.action.url = Some(url);
            }
            cloned
        })
        .collect()
}
```

**Missing-key behavior:** placeholder text left unsubstituted — no panic, no silent removal. `String::replace` with no match is a no-op, so this is already the behavior — no special handling needed.

---

### `ferro-json-ui/src/render/mod.rs` — count bump + dispatch

**Analog:** `BUILTIN_TYPES` array (lines 41–85) and count assertion (line 526):

**BUILTIN_TYPES:** Add after `"Switch"` (in the Form controls group):
```rust
"CheckboxList",   // D-01
```
And add after all existing entries (in whatever group fits — form controls or a new "plugins" section):
```rust
"RichTextEditor", // D-18
```

**Dispatch arm** (in `render_element` match, copy the `"Checkbox"` arm pattern):
```rust
"CheckboxList" => form::render_checkbox_list(el, spec, data, depth),
"RichTextEditor" => {
    // Plugins handle their own rendering via the registry.
    // This arm ensures the type is recognized as built-in for catalog purposes.
    // Actual rendering: render_element falls through to plugin dispatch after built-in check.
    // NOTE: verify with RESEARCH.md D-18 — may be plugin-only (not in BUILTIN_TYPES).
    // Confirm approach before implementing.
    String::new()
}
```

**Count assertion** (line 526 — update both the comment and the number):
```rust
// BEFORE:
assert_eq!(BUILTIN_TYPES.len(), 39);
// AFTER (CheckboxList + RichTextEditor = +2):
assert_eq!(BUILTIN_TYPES.len(), 41);
```

---

### `ferro-json-ui/src/catalog.rs` — BUILTIN_SPECS entries

**Analog:** Existing `Checkbox` and `Switch` entries (lines 337–348):
```rust
(
    "Checkbox",
    "Boolean checkbox with label, description, data binding.",
    || to_value(schema_for!(CheckboxProps)).unwrap(),
    &[],
),
(
    "Switch",
    "Toggle switch (visual alternative to Checkbox); auto-submit when `action` set.",
    || to_value(schema_for!(SwitchProps)).unwrap(),
    &[],
),
```

**Add CheckboxList** (after Switch — keep form-controls group together):
```rust
(
    "CheckboxList",
    "Multi-select checkbox group from static options or data-driven array. \
     Each checked option submits as field=value.",
    || to_value(schema_for!(CheckboxListProps)).unwrap(),
    &[],
),
```

**Add RichTextEditor** (after CheckboxList or in a plugin-backed section):
```rust
(
    "RichTextEditor",
    "Rich text editor using Quill 2.0.3. Emits hidden input with HTML content. \
     Requires plugin registration via JsonUiPlugin.",
    || to_value(schema_for!(RichTextEditorProps)).unwrap(),
    &[],
),
```

**Import block** (catalog.rs line 29 area — add new types):
```rust
use crate::component::{
    // ... existing types ...
    CheckboxListProps,
    RichTextEditorProps,
};
```

---

### `ferro-json-ui/src/spec.rs` — `SpecError::FooterMissing` + `validate_footer_ids`

**Analog:** `SpecError::DanglingChild` (line 107) and `validate_no_dangling` (line 457):

**New `SpecError` variant** (insert after `DanglingChild`):
```rust
// BEFORE (line 107):
#[error("element '{element}' references child '{child}' which does not exist")]
DanglingChild { element: String, child: String },

// ADD AFTER:
#[error("element '{element_id}' has footer reference '{footer_id}' not found in elements")]
FooterMissing { element_id: String, footer_id: String },
```

**`validate_footer_ids` function** (insert in `validate_structure`, after `validate_no_dangling`):
```rust
fn validate_footer_ids(spec: &Spec) -> Result<(), SpecError> {
    for (element_id, el) in &spec.elements {
        // `footer` key lives inside props — handle gracefully when props is null.
        let footer_ids: Vec<String> = el
            .props
            .get("footer")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        for footer_id in &footer_ids {
            if !spec.elements.contains_key(footer_id) {
                return Err(SpecError::FooterMissing {
                    element_id: element_id.clone(),
                    footer_id: footer_id.clone(),
                });
            }
            // D-08 warning: same ID in both footer and children.
            if el.children.contains(footer_id) {
                eprintln!(
                    "ferro-json-ui: element '{}' has '{}' in both footer and children",
                    element_id, footer_id
                );
            }
        }
    }
    Ok(())
}
```

**Update `validate_structure`** (line 416) to call the new function:
```rust
fn validate_structure(spec: &Spec) -> Result<(), SpecError> {
    validate_ids(&spec.elements)?;
    if !spec.elements.contains_key(&spec.root) {
        return Err(SpecError::RootMissing(spec.root.clone()));
    }
    validate_no_dangling(&spec.elements)?;
    validate_footer_ids(spec)?;   // ADD — D-07/D-08
    detect_cycle(&spec.elements, &spec.root)?;
    check_depth(&spec.elements, &spec.root)?;
    Ok(())
}
```

**Test pattern** (copy from spec.rs test block style, lines 534+):
```rust
#[test]
fn from_json_rejects_missing_footer_id() {
    let err = Spec::from_json(r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "card",
        "elements": {
            "card": {
                "type": "Card",
                "props": {"title": "T", "footer": ["ghost"]}
            }
        }
    }"#).unwrap_err();
    match err {
        SpecError::FooterMissing { element_id, footer_id } => {
            assert_eq!(element_id, "card");
            assert_eq!(footer_id, "ghost");
        }
        other => panic!("expected FooterMissing, got {other:?}"),
    }
}
```

---

### `ferro-json-ui/src/layout.rs` — remove card wrapper (D-05)

**Analog:** `AuthLayout.render` lines 367–384 — exact lines to change:

**BEFORE** (lines 371–379):
```rust
let body = format!(
    r#"<div class="min-h-screen flex items-center justify-center">
    <div class="w-full max-w-md">
        <div class="bg-card rounded-lg shadow-md p-8">
            {wrapper}
        </div>
    </div>
</div>"#,
);
```

**AFTER** (remove the inner `bg-card` div):
```rust
let body = format!(
    r#"<div class="min-h-screen flex items-center justify-center">
    <div class="w-full max-w-md">
        {wrapper}
    </div>
</div>"#,
);
```

**Test update required:** The existing `auth_layout_centers_content` test (near line 810) currently asserts the HTML contains `bg-card rounded-lg shadow-md p-8`. Invert that assertion: the wrapper must NOT contain that string after D-05.

---

### `ferro-json-ui/src/plugins/rich_text_editor.rs` (NEW file)

**Analog:** `ferro-json-ui/src/plugins/map.rs` (full file) — the MapPlugin is the canonical plugin example.

**Structural pattern** (map.rs lines 86–100):
```rust
pub struct RichTextEditorPlugin;

impl JsonUiPlugin for RichTextEditorPlugin {
    fn component_type(&self) -> &str {
        "RichTextEditor"
    }

    fn props_schema(&self) -> serde_json::Value {
        // Return a JSON Schema object describing RichTextEditorProps fields.
        serde_json::json!({ "type": "object", "properties": { ... } })
    }

    fn render(&self, props: &serde_json::Value, data: &serde_json::Value) -> String {
        // Decode RichTextEditorProps, emit container div + hidden input.
        // The IIFE in init_script() activates the Quill editor.
        todo!()
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![
            Asset::new("https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css")
                .integrity("sha256-...")    // verify SRI hash before hardcoding
                .crossorigin(""),
        ]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![
            Asset::new("https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js")
                .integrity("sha256-...")    // verify SRI hash before hardcoding
                .crossorigin(""),
        ]
    }

    fn init_script(&self) -> Option<String> {
        Some(r#"(function(){
            document.querySelectorAll('[data-ferro-quill]').forEach(function(el){
                var field = el.dataset.ferroField;
                var quill = new Quill(el, { theme: 'snow' });
                var input = document.getElementById(field + '-value');
                quill.on('text-change', function(){
                    if (input) input.value = quill.root.innerHTML;
                });
            });
        })();"#.to_string())
    }
}
```

**Global registry registration** (in `plugin.rs` line 155 area — add alongside MapPlugin):
```rust
pub fn global_plugin_registry() -> &'static RwLock<PluginRegistry> {
    GLOBAL_PLUGIN_REGISTRY.get_or_init(|| {
        let mut registry = PluginRegistry::new();
        registry.register(crate::plugins::MapPlugin);
        registry.register(crate::plugins::RichTextEditorPlugin);  // ADD D-18
        RwLock::new(registry)
    })
}
```

**`plugins/mod.rs` export** (add `pub mod rich_text_editor; pub use rich_text_editor::RichTextEditorPlugin;`).

---

### `ferro-mcp/src/tools/json_ui_verify_action.rs` (NEW file)

**Analog:** `list_routes.rs` (full file) — follow this file's structure exactly.

**Imports pattern** (list_routes.rs lines 1–12):
```rust
use crate::error::{McpError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
```

**Additional import for Levenshtein:**
```rust
use strsim;
```

**Input/output types** (follow RouteInfo shape from list_routes.rs line 33):
```rust
#[derive(Debug, Serialize)]
pub struct VerifyActionResult {
    pub found: bool,
    pub route: Option<RouteInfo>,
    pub candidate: Option<String>,  // closest Levenshtein match when not found
    pub message: String,
}
```

**Core `execute` function** (async, mirrors list_routes.rs line 95):
```rust
pub async fn execute(
    project_root: &Path,
    handler: &str,
    method: Option<&str>,
) -> Result<VerifyActionResult> {
    use crate::tools::list_routes;
    let routes_info = list_routes::execute(project_root).await?;
    let routes = routes_info.routes;

    // Search for exact match.
    let found = routes.iter().find(|r| {
        r.name.as_deref() == Some(handler)
            && method.map(|m| r.method.eq_ignore_ascii_case(m)).unwrap_or(true)
    });

    if let Some(route) = found {
        return Ok(VerifyActionResult {
            found: true,
            route: Some(/* clone route */),
            candidate: None,
            message: format!("Route '{}' found", handler),
        });
    }

    // Not found — find closest Levenshtein candidate.
    let candidate = routes
        .iter()
        .filter_map(|r| r.name.as_ref().map(|n| (n, strsim::levenshtein(n, handler))))
        .min_by_key(|(_, dist)| *dist)
        .map(|(name, _)| name.clone());

    Ok(VerifyActionResult {
        found: false,
        route: None,
        candidate,
        message: format!("Route '{}' not found", handler),
    })
}
```

**Cargo.toml change required** (`ferro-mcp/Cargo.toml`):
```toml
strsim = "0.11"
```

---

### `ferro-mcp/src/tools/mod.rs` — register new tool

**Analog:** existing `pub mod` list (lines 1–60) — add alphabetically:
```rust
pub mod json_ui_verify_action;   // ADD — D-09 (alphabetically after json_ui_inspect)
```

---

### `ferro-mcp/src/tools/code_templates.rs` — migration templates (D-22)

**Analog:** `handler_templates()` function (lines 81–126) — exact `CodeTemplate` struct shape:

```rust
fn migration_v1_to_v2_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "render_file_migration".to_string(),
            category: "migration_v1_to_v2".to_string(),
            description: "Replace v1 JsonUiView builder with v2 JsonUi::render_file".to_string(),
            code: r#"// v2: load spec from JSON file and merge handler data
JsonUi::render_file("src/views/{{module}}/{{page}}.json", json!({
    "data": { /* your handler data */ }
}))"#.to_string(),
            imports: vec!["use ferro_json_ui::JsonUi;".to_string()],
            placeholders: vec![
                Placeholder {
                    name: "{{module}}".to_string(),
                    description: "Controller module name".to_string(),
                    example: "account".to_string(),
                },
                Placeholder {
                    name: "{{page}}".to_string(),
                    description: "Page name".to_string(),
                    example: "settings".to_string(),
                },
            ],
        },
        // ... 6 more templates following the same pattern, one per D-20 section
    ]
}
```

**Register in `build_templates()`** (after existing `api_templates()` call, line 76):
```rust
templates.extend(migration_v1_to_v2_templates());
```

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` — count assertion bump

**Analog:** `test_all_components_present` (lines 235–300) — update two things:

**Count assertion** (line 237–241):
```rust
// BEFORE:
assert_eq!(
    catalog.components.len(),
    39,
    "Catalog should contain all 39 built-in components, got {}",
    catalog.components.len()
);

// AFTER (+CheckboxList +RichTextEditor = 41):
assert_eq!(
    catalog.components.len(),
    41,
    "Catalog should contain all 41 built-in components, got {}",
    catalog.components.len()
);
```

**Expected names list** (add to `expected` array at line 245+):
```rust
"CheckboxList",    // D-01/D-02
"RichTextEditor",  // D-18
```

---

## Triple-Lockstep Coordination

Three files must stay in exact sync when adding catalog components. Violation causes either a panic at first catalog use or a failing CI test.

| File | Line | What changes | Current value | After Phase 162 |
|------|------|-------------|---------------|-----------------|
| `ferro-json-ui/src/render/mod.rs` | 526 | Count assertion | `assert_eq!(BUILTIN_TYPES.len(), 39)` | `assert_eq!(BUILTIN_TYPES.len(), 41)` |
| `ferro-json-ui/src/catalog.rs` | ~123 | BUILTIN_SPECS array length | 39 entries | 41 entries (+CheckboxList, +RichTextEditor) |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 237–241 | Count assertion in test | `39` | `41` |

Both additions (`CheckboxList` and `RichTextEditor`) MUST be applied atomically in Wave 1 — do not add one without the other, or the count assertions diverge.

Also: `BUILTIN_TYPES` array (render/mod.rs lines 41–85) and the dispatch match in `render_element` must both get arms for both new types.

---

## Shared Patterns

### Props Decode Pattern (ALL new render functions)

Every render function in `render/form.rs` and `render/atoms.rs` uses this error path. Copy verbatim:
```rust
let props: XxxProps = match serde_json::from_value(el.props.clone()) {
    Ok(p) => p,
    Err(e) => {
        return format!(
            "<!-- ferro-json-ui: failed to decode Xxx props: {} -->",
            html_escape(&e.to_string())
        );
    }
};
```

**Source:** `render_checkbox` lines 398–407, `render_switch` lines 466–473, `render_form` lines 40–48.

### HTML Escape (ALL new render functions)

All user-supplied string output passes through `html_escape()`. Import is already available via `use super::html_escape;`. Never emit raw strings from props into HTML output.

### Serde Conventions (ALL new Props structs)

- Required fields: no `#[serde(...)]` attribute needed
- Optional fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Optional Vec fields: `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
- All enums: `#[serde(rename_all = "snake_case")]`
- All Props structs: `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]`

### SpecError Variant Pattern (D-07)

**Source:** `spec.rs` lines 99–120 — `thiserror::Error` derive, structured payloads, no formatted strings.
```rust
#[error("element '{element_id}' has footer reference '{footer_id}' not found in elements")]
FooterMissing { element_id: String, footer_id: String },
```

### Plugin Registration Pattern (D-18)

**Source:** `plugin.rs` lines 152–158. New plugins register in `global_plugin_registry()` alongside MapPlugin. The `JsonUiPlugin` trait requires: `component_type()`, `props_schema()`, `render()`, `css_assets()`, `js_assets()`, `init_script()`.

### MCP Tool Registration Pattern (D-09)

**Source:** `ferro-mcp/src/tools/mod.rs` — add `pub mod tool_name;` to the module list. The tool's `execute()` function is called from the MCP dispatch layer (in `ferro-mcp/src/lib.rs` or equivalent). Follow `list_routes::execute(project_root)` calling convention for async tools.

---

## Test Placement Map

| Test | File | Closest Existing Test to Follow |
|------|------|--------------------------------|
| `checkbox_list_renders_one_checkbox_per_option` | `ferro-json-ui/src/render/form.rs` inline `#[cfg(test)]` | `render_checkbox` tests (form.rs, check for `type="checkbox"`) |
| `checkbox_list_selected_path_prechecks_matching_options` | same | same |
| `schema_for_checkbox_list_props_generates` | `ferro-json-ui/src/component.rs` inline `#[cfg(test)]` | `schema_for_checkbox_props_generates` (line 866 area) |
| `builtin_types_count_matches_dispatch` | `ferro-json-ui/src/render/mod.rs` inline `#[cfg(test)]` | existing test line 521 — update assertion value |
| `test_all_components_present` | `ferro-mcp/src/tools/json_ui_catalog.rs` | existing test line 235 — update count and expected list |
| `data_table_url_template_replaces_column_key` | `ferro-json-ui/src/render/data.rs` inline `#[cfg(test)]` | `table_renders_rows_from_data_path` (data.rs line 380) using `mk_element` + `mk_spec` |
| `data_table_url_template_missing_key_leaves_placeholder` | same | same |
| `auth_layout_centers_content` | `ferro-json-ui/src/layout.rs` inline `#[cfg(test)]` | existing test (update to assert NO `bg-card` class) |
| `from_json_rejects_missing_footer_id` | `ferro-json-ui/src/spec.rs` inline `#[cfg(test)]` | existing `from_json_rejects_*` tests (spec.rs line 534+) |
| `spec_warns_duplicate_footer_child` | same | same — use `eprintln!` capture or just check no panic |
| `switch_compact_adds_scale_class` | `ferro-json-ui/src/render/form.rs` | `render_switch` test pattern |
| `image_inline_svg_renders_without_img_tag` | `ferro-json-ui/src/render/atoms.rs` | `render_image` test (assert no `<img` in output) |
| `alert_variant_as_ref_str_matches_wire_format` | `ferro-json-ui/src/component.rs` | schema smoke test pattern |
| `json_ui_verify_action_found` | `ferro-mcp/src/tools/json_ui_verify_action.rs` | unit test with hard-coded route list (no HTTP call) |
| `json_ui_verify_action_not_found_suggests_closest` | same | same |
| `code_templates_returns_migration_patterns` | `ferro-mcp/src/tools/code_templates.rs` | any existing category filter test |

---

## No Analog Found

No files in Phase 162 are without codebase analogs. All patterns have direct precedents.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/tools/`, `docs/src/json-ui/`
**Files scanned:** 18 source files read directly
**Key risk files (triple-lockstep):**
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/render/mod.rs` line 526
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/catalog.rs` BUILTIN_SPECS array
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp/src/tools/json_ui_catalog.rs` line 237
**Pattern extraction date:** 2026-05-16

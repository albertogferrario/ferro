# Phase 147: DetailForm component — Pattern Map

**Mapped:** 2026-04-23
**Files analyzed:** 6 modified (+ 1 docs)
**Analogs found:** 6 / 6 (100% — every new surface has a direct in-crate precedent)

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | new types + enum variant + serde arms + factory + tests | request-response (shape) | `FormProps` (L190-203), `DescriptionItem`/`DescriptionListProps` (L427-440), `Tab` (L444-450), `KeyValueEditorProps` (L397-416), `ComponentNode::form` factory (L1245-1252), `key_value_editor_serde_roundtrip` (L3624-3673) | exact |
| `ferro-json-ui/src/render.rs` | render function + dispatch arm + plugin-walk arm + tests | request-response (HTML emit) | `render_form` (L971-1031), `render_description_list` (L2427-2439), `render_input` label block (L1407-1412), `render_button` variant strings (L2029-2117), form tests (L4263-4372), key_value_editor tests (L8441-8620) | exact |
| `ferro-json-ui/src/resolve.rs` | three match arms (one per pass) + tests | batch / transform | `Component::Form` arms at L46-51 / L219-224 / L399-403; `resolve_form_action` test (L593-633) | exact |
| `ferro-json-ui/src/lib.rs` | public re-exports + `COMPONENT_CATALOG` entry | public API | `pub use component::{…, FormProps, KeyValueEditorProps, …}` block (L59-71); `### KeyValueEditor` catalog block (L140-142) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `CatalogComponent` entry + exhaustive-list insertion | catalog entry | `CatalogComponent { name: "DescriptionList", … }` (L504-522), exhaustive list (L1114-1154) | exact |
| `docs/src/json-ui/components.md` | new `### DetailForm` doc section | documentation | `### DescriptionList` doc section (L415-471) | role-match |

**Files NOT modified (deliberate):**
- `ferro-json-ui/src/runtime/*` — D-20 forbids runtime JS (server-side mode toggle only).
- `ferro-json-ui/src/visibility.rs`, `ferro-json-ui/src/layout.rs` — unrelated surfaces.

---

## Pattern Assignments

### 1. `ferro-json-ui/src/component.rs`

This file receives six discrete edits. Each has a named analog.

#### 1a. `EditMode` enum — new type

**Analog:** `HttpMethod` at `ferro-json-ui/src/action.rs:21-30`; also `FormMaxWidth` at `component.rs:180-185` (local-enum placement precedent).

**Insertion point:** Near the top of `component.rs`, alongside other small local enums. `FormMaxWidth` is defined right before `FormProps` (L180-185, near imports at the top of the component definitions). Place `EditMode` near the other form-family enums. An acceptable location is just before `DetailFormProps` itself (keep the type near its primary consumer).

**Pattern to copy — HttpMethod shape (with `Default` added per D-01):**

```rust
// action.rs:20-30
/// HTTP method for action requests.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    #[default]
    Post,
    Put,
    Patch,
    Delete,
}
```

**Concrete DetailForm emission (D-01, D-02):**

```rust
/// Which display mode the component uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EditMode {
    #[default]
    View,
    Edit,
}

impl EditMode {
    /// Parse a URL query-parameter value into an `EditMode`.
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some(s) if s.eq_ignore_ascii_case("edit") => EditMode::Edit,
            _ => EditMode::View,
        }
    }
}
```

**Ordering convention:** `EditMode` carries `Copy` (scalar enum) — same as `DialogVariant` / `NotifyVariant` in `action.rs`. `KeyValueEditorProps` at L397-416 uses `JsonSchema`. This enum also keeps `JsonSchema` per D-01.

---

#### 1b. `DetailField` struct — new type

**Analog:** `Tab` at `component.rs:442-450` (contains `Vec<ComponentNode>` → skips `JsonSchema`).

**Pattern to copy — Tab:**

```rust
// component.rs:442-450
/// A single tab within a Tabs component.
// JsonSchema skipped: contains Vec<ComponentNode> — Component has custom Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
}
```

**Insertion point:** Directly before `DetailFormProps`. Group with other helper row-shape structs (`DescriptionItem` at L427-432, `Tab` at L444-450).

**Emission rule (D-03):** Struct must open with the same `// JsonSchema skipped: …` comment. Use `ComponentNode` (not `Option<ComponentNode>`) because Edit mode always renders *something* in `<dd>`. Add a `DetailField::new(label, value, input)` convenience constructor (per Claude's Discretion) mirroring `ComponentNode::input(…)` ergonomics.

---

#### 1c. `DetailFormProps` struct — new type

**Primary analog:** `FormProps` at `component.rs:187-203`. Secondary: `KeyValueEditorProps` at `component.rs:386-416` for doc-comment style.

**Pattern to copy — FormProps:**

```rust
// component.rs:187-203
/// Props for Form component.
// JsonSchema skipped: contains Vec<ComponentNode> — Component has custom Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormProps {
    pub action: Action,
    pub fields: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<crate::action::HttpMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
}
```

**Emission rule (D-04):**
- Same `// JsonSchema skipped: …` comment — update text to mention `Vec<DetailField>`.
- Every `Option` field uses `#[serde(default, skip_serializing_if = "Option::is_none")]` (verified pattern across the file).
- `mode: EditMode` uses `#[serde(default)]` (falls back to `View`).
- Does NOT include `guard` (D-04 + out-of-scope list in CONTEXT § Deferred).

---

#### 1d. `Component::DetailForm` variant

**Analog:** `Component::Form(FormProps)` at `component.rs:952`; `Component::KeyValueEditor(KeyValueEditorProps)` at `component.rs:989`.

**Pattern to copy — the enum definition (component.rs:947-991):**

```rust
// JsonSchema skipped: custom Serialize/Deserialize impl
#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    Card(CardProps),
    Table(TableProps),
    Form(FormProps),
    // ... 38 other variants ...
    KeyValueEditor(KeyValueEditorProps),
    Plugin(PluginProps),
}
```

**Ordering convention (from RESEARCH):** ordering is **not alphabetical** — variants are grouped by family, with `Plugin` always last. KeyValueEditor (phase 146) was inserted immediately before `Plugin`. For phase 147, the safe position is **immediately after `KeyValueEditor`, before `Plugin`** (same family — "form-shaped things holding an Action"). This matches the rule used in every other match block in the crate.

```rust
    KeyValueEditor(KeyValueEditorProps),
    DetailForm(DetailFormProps),          // ← NEW, inserted here
    Plugin(PluginProps),
```

---

#### 1e. Serialize arm

**Analog:** `Component::Form(p) => serialize_tagged(serializer, "Form", p),` at `component.rs:1017`; KeyValueEditor at L1056.

**Pattern to copy (component.rs:1012-1060):**

```rust
impl Serialize for Component {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Component::Card(p) => serialize_tagged(serializer, "Card", p),
            // ...
            Component::KeyValueEditor(p) => serialize_tagged(serializer, "KeyValueEditor", p),
            Component::Plugin(p) => p.serialize(serializer),
        }
    }
}
```

**Ordering convention:** Position must match the enum declaration order in §1d (right after `KeyValueEditor`, before `Plugin`). The file already violates strict alphabetical ordering everywhere — follow the enum order, period.

**Emission:**
```rust
Component::KeyValueEditor(p) => serialize_tagged(serializer, "KeyValueEditor", p),
Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),   // NEW
Component::Plugin(p) => p.serialize(serializer),
```

---

#### 1f. Deserialize arm

**Analog:** `"Form" => serde_json::from_value::<FormProps>(value).map(Component::Form).map_err(de::Error::custom),` at `component.rs:1079-1081`; `"KeyValueEditor"` at L1190-1192.

**Pattern to copy (component.rs:1064-1204):**

```rust
impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_str = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::missing_field("type"))?;

        match type_str {
            "Card" => serde_json::from_value::<CardProps>(value)
                .map(Component::Card)
                .map_err(de::Error::custom),
            // ...
            "KeyValueEditor" => serde_json::from_value::<KeyValueEditorProps>(value)
                .map(Component::KeyValueEditor)
                .map_err(de::Error::custom),
            _ => {
                // Unknown type: treat as a plugin component.
                // ...
            }
        }
    }
}
```

**Ordering convention:** Arms are grouped by family, matching the Serialize block. Insert `"DetailForm"` immediately after `"KeyValueEditor"` at L1192 and before the `_ =>` plugin catch-all at L1193.

**Emission:**
```rust
"KeyValueEditor" => serde_json::from_value::<KeyValueEditorProps>(value)
    .map(Component::KeyValueEditor)
    .map_err(de::Error::custom),
"DetailForm" => serde_json::from_value::<DetailFormProps>(value)   // NEW
    .map(Component::DetailForm)
    .map_err(de::Error::custom),
_ => { /* plugin fallback */ }
```

---

#### 1g. `ComponentNode::detail_form` factory

**Analog:** `ComponentNode::form` at `component.rs:1244-1252`; `ComponentNode::description_list` at `component.rs:1354-1362` (parallel factory).

**Pattern to copy (component.rs:1244-1252):**

```rust
/// Create a Form component node.
pub fn form(key: impl Into<String>, props: FormProps) -> Self {
    Self {
        key: key.into(),
        component: Component::Form(props),
        action: None,
        visibility: None,
    }
}
```

**Ordering convention:** Factories are grouped roughly by variant order in the enum. KeyValueEditor did not add a factory in phase 146; do not use that gap as precedent — D-18 explicitly requires one. Place `detail_form` alongside the form-family factories (after `form` at L1245 is the cleanest fit, since it mirrors `Component::Form` mechanically).

**Emission rule:** Include a rustdoc paragraph citing the structural-coherence contract (§5 of 147-UI-SPEC) and the Option-A empty-label rule (§9 of 147-UI-SPEC). This is a hard requirement of the UI-SPEC acceptance test §14.7.

---

#### 1h. Tests (new `#[cfg(test)] mod detail_form_tests`)

**Analog:** `key_value_editor_serde_roundtrip` at `component.rs:3620-3673`.

**Pattern to copy (component.rs:3619-3673):**

```rust
#[cfg(test)]
mod key_value_editor_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_value_editor_serde_roundtrip() {
        let original = Component::KeyValueEditor(KeyValueEditorProps { … });
        let serialized = serde_json::to_value(&original).expect("serialize …");
        assert_eq!(serialized.get("type").and_then(|v| v.as_str()), Some("KeyValueEditor"));
        let deserialized: Component = serde_json::from_value(serialized).expect("deserialize …");
        // Match deserialized and assert all fields.
        assert_eq!(original, deserialized, "PartialEq round-trip failed");
    }
}
```

**Tests to add** (per RESEARCH §Test Patterns):
- `detail_form_serde_roundtrip` — full prop tree, assert `type=DetailForm` + `mode=edit`.
- `edit_mode_default_is_view`
- `edit_mode_from_query_exact_edit`
- `edit_mode_from_query_case_insensitive` (EDIT, Edit, eDiT)
- `edit_mode_from_query_none_is_view`
- `edit_mode_from_query_unknown_is_view` (empty, "view", "anything-else")
- `edit_mode_serializes_as_snake_case` — `EditMode::Edit ↔ "edit"`

---

### 2. `ferro-json-ui/src/render.rs`

Four edits: new function, dispatch arm, plugin-walk arm, tests.

#### 2a. `render_detail_form` function — new

**Analog (form/method spoofing):** `render_form` at `render.rs:971-1031`.
**Analog (dl scaffold):** `render_description_list` at `render.rs:2427-2439`.
**Analog (input label block showing unconditional `<label>` emission that the empty-string rule targets):** `render_input` at `render.rs:1407-1412`.
**Analog (button variant strings):** `render_button` at `render.rs:2029-2117`.

**Insertion point:** Place `fn render_detail_form(props: &DetailFormProps, data: &Value) -> String` immediately **after `render_form`** (render.rs:971-1031) — same family, keeps the method-spoofing reference visible in adjacent lines.

**Copy-paste anchor 1 — form + method spoofing (render.rs:971-1011):**

```rust
fn render_form(props: &FormProps, data: &Value) -> String {
    let effective_method = props
        .method
        .as_ref()
        .unwrap_or(&props.action.method)
        .clone();

    let (form_method, needs_spoofing) = match effective_method {
        HttpMethod::Get => ("get", false),
        HttpMethod::Post => ("post", false),
        HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
    };

    let action_url = props.action.url.as_deref().unwrap_or("#");
    let mut html = match &props.guard {
        Some(g) => format!("<form action=\"{}\" method=\"{}\" data-form-guard=\"{}\" class=\"…\">", html_escape(action_url), form_method, html_escape(g)),
        None => format!("<form action=\"{}\" method=\"{}\" class=\"…\">", html_escape(action_url), form_method),
    };

    if needs_spoofing {
        let method_value = match effective_method {
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            _ => unreachable!(),
        };
        html.push_str(&format!(
            "<input type=\"hidden\" name=\"_method\" value=\"{method_value}\">"
        ));
    }
    // ... fields, close tag, max-width wrap ...
}
```

**Copy-paste anchor 2 — `<dl>` scaffold (render.rs:2427-2439):**

```rust
fn render_description_list(props: &DescriptionListProps) -> String {
    let columns = props.columns.unwrap_or(1);
    let mut html = format!("<dl class=\"grid grid-cols-{columns} gap-4\">");
    for item in &props.items {
        html.push_str(&format!(
            "<div><dt class=\"text-sm font-medium text-text-muted\">{}</dt><dd class=\"mt-1 text-sm text-text\">{}</dd></div>",
            html_escape(&item.label),
            html_escape(&item.value)
        ));
    }
    html.push_str("</dl>");
    html
}
```

**Copy-paste anchor 3 — Button variant class strings (render.rs:2029-2117, distilled by RESEARCH §Button Variant Class Strings):**

- Shared base: `inline-flex items-center justify-center rounded-md font-medium transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`
- Primary ("Salva"): `bg-primary text-primary-foreground hover:bg-primary/90`
- Outline ("Modifica" / "Annulla"): `border border-border bg-background text-text hover:bg-surface`
- Size default: `px-4 py-2 text-sm`
- Action bar wrapper: `<div class="flex gap-2 justify-end">…</div>`

**Emission structure (D-05 through D-14, §5 of UI-SPEC):**

```rust
fn render_detail_form(props: &DetailFormProps, data: &Value) -> String {
    // 1. Build shared <dl> body — same HTML string in both modes (D-05).
    let mut dl = String::from("<dl class=\"grid grid-cols-1 gap-4\">");
    for field in &props.fields {
        dl.push_str("<div>");
        dl.push_str(&format!(
            "<dt class=\"text-sm font-medium text-text-muted\">{}</dt>",
            html_escape(&field.label)
        ));
        match props.mode {
            EditMode::View => dl.push_str(&format!(
                "<dd class=\"mt-1 text-sm text-text\">{}</dd>",
                html_escape(&field.value)
            )),
            EditMode::Edit => dl.push_str(&format!(
                "<dd class=\"mt-1 text-sm text-text\">{}</dd>",
                render_node(&field.input, data)
            )),
        }
        dl.push_str("</div>");
    }
    dl.push_str("</dl>");

    // 2. Build action bar (flex gap-2 justify-end).
    // View: <a> "Modifica" (outline).
    // Edit: <a> "Annulla" (outline) + <button type="submit"> "Salva" (primary).

    // 3. Assemble:
    //    - View: <div>{dl}{action_bar}</div>
    //    - Edit: <form action=… method=… [+hidden _method]>{dl}{action_bar}</form>
    //      using the exact effective_method / form_method / needs_spoofing block
    //      lifted from render_form above.
}
```

**Token/class inheritance rule (§1-§4 of UI-SPEC):** Emit ONLY classes already present in `render_form`, `render_description_list`, `render_input`, or `render_button`. Zero new classes. The UI checker acceptance test §14.2 will verify this.

---

#### 2b. Dispatch arm in `render_component`

**Analog:** `Component::Form(props) => render_form(props, data),` at `render.rs:305`.

**Pattern to copy (render.rs:288-340):**

```rust
fn render_component(component: &Component, data: &Value) -> String {
    match component {
        // ...
        Component::Card(props) => render_card(props, data),
        Component::Form(props) => render_form(props, data),
        Component::Modal(props) => render_modal(props, data),
        // ...
        Component::KeyValueEditor(props) => render_key_value_editor(props, data),
        // ...
    }
}
```

**Ordering convention:** Variants are grouped by family with comment headers (`// Container components.`, `// Form field components.`, etc.). DetailForm belongs with **Container components** — insert immediately after `Component::Form(props) => render_form(props, data),` (L305) and before `Component::Modal(props) => render_modal(props, data),` (L306).

**Emission:**
```rust
Component::Form(props) => render_form(props, data),
Component::DetailForm(props) => render_detail_form(props, data),   // NEW
Component::Modal(props) => render_modal(props, data),
```

---

#### 2c. Plugin-walk arm in `collect_plugin_types_node`

**Analog:** `Component::Form(props) => { for field in &props.fields { collect_plugin_types_node(field, types); } }` at `render.rs:114-118`.

**Pattern to copy (render.rs:101-197):**

```rust
fn collect_plugin_types_node(node: &ComponentNode, types: &mut HashSet<String>) {
    match &node.component {
        Component::Plugin(props) => { types.insert(props.plugin_type.clone()); }
        Component::Form(props) => {
            for field in &props.fields {
                collect_plugin_types_node(field, types);
            }
        }
        // ... 8 more container arms ...
        // Leaf components have no children to recurse into.
        Component::Table(_)
        | Component::Button(_)
        | /* ... */
        | Component::KeyValueEditor(_) => {}
        Component::KanbanBoard(props) => { /* ... */ }
    }
}
```

**Ordering convention:** Container arms listed first; leaf catch-all (`|`-separated) in the middle; KanbanBoard after the leaf list (a historical quirk). Insert the DetailForm container arm **next to Form (L114-118)**.

**Emission:**
```rust
Component::Form(props) => {
    for field in &props.fields {
        collect_plugin_types_node(field, types);
    }
}
Component::DetailForm(props) => {   // NEW
    for field in &props.fields {
        collect_plugin_types_node(&field.input, types);
    }
}
```

**Critical gotcha (Pitfall 2 in RESEARCH):** Do NOT place `Component::DetailForm(_)` in the leaf catch-all block at L160-189. DetailForm is a container (it holds child `ComponentNode`s inside `DetailField.input`).

---

#### 2d. Render tests

**Analogs:** Form tests at `render.rs:4263-4372`; KeyValueEditor tests at `render.rs:8441-8620`.

**Pattern to copy (render.rs:4263-4291):**

```rust
#[test]
fn form_renders_action_url_and_method() {
    let view = JsonUiView::new().component(ComponentNode {
        key: "f".to_string(),
        component: Component::Form(FormProps {
            action: Action { handler: "users.store".to_string(), url: Some("/users".to_string()), method: HttpMethod::Post, /* ... */ },
            fields: vec![],
            method: None,
            guard: None,
            max_width: None,
        }),
        action: None,
        visibility: None,
    });
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("action=\"/users\""));
    assert!(html.contains("method=\"post\""));
    assert!(html.contains("class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\""));
}
```

**Tests to add** (per RESEARCH §Render test template and UI-SPEC §14 acceptance):
- `render_detail_form_view_mode`
- `render_detail_form_edit_mode`
- `render_detail_form_edit_method_spoofing_put`
- `render_detail_form_edit_method_spoofing_patch`
- `render_detail_form_edit_method_spoofing_delete`
- `render_detail_form_edit_get_no_spoofing`
- `render_detail_form_view_xss_escapes_label`
- `render_detail_form_view_xss_escapes_edit_url`
- `render_detail_form_edit_xss_escapes_cancel_url`
- `render_detail_form_custom_labels`
- `render_detail_form_view_action_bar_below_dl` (ordering check)
- `render_detail_form_scaffold_invariance` — assert the `<dl>…</dl>` substring is byte-for-byte identical between View and Edit modes on the same `fields` (structural coherence §5 of UI-SPEC).

**Insertion point:** Add a new comment-banner section inside the existing `mod tests` — follow the `// ── 19. Form ────────────────` convention at L4261. Pick the next available number.

---

### 3. `ferro-json-ui/src/resolve.rs`

Three match arms (one per pass) plus two tests. Every pass is mechanical.

#### 3a. `resolve_component_node` arm

**Analog:** `Component::Form` arm at `resolve.rs:46-51`:

```rust
Component::Form(props) => {
    resolve_action(&mut props.action, resolver);
    for field in &mut props.fields {
        resolve_component_node(field, resolver);
    }
}
```

**Ordering convention:** Container arms listed first by family; leaf `|`-chain at the bottom. Insert DetailForm arm immediately after the Form arm (L46-51).

**Emission:**
```rust
Component::DetailForm(props) => {
    resolve_action(&mut props.action, resolver);
    for field in &mut props.fields {
        resolve_component_node(&mut field.input, resolver);
    }
}
```

**Critical gotcha:** `DetailField.input` is a `ComponentNode` (not `Component`), so the recursive call is `resolve_component_node(&mut field.input, …)` — do NOT unwrap to `field.input.component`.

Also: remove `Component::DetailForm(_)` from the leaf catch-all at L129-154 (or rather, never add it there).

---

#### 3b. `collect_unresolved_node` arm

**Analog:** `Component::Form` arm at `resolve.rs:219-224`:

```rust
Component::Form(props) => {
    collect_unresolved_action(&props.action, unresolved);
    for field in &props.fields {
        collect_unresolved_node(field, unresolved);
    }
}
```

**Emission:**
```rust
Component::DetailForm(props) => {
    collect_unresolved_action(&props.action, unresolved);
    for field in &props.fields {
        collect_unresolved_node(&field.input, unresolved);
    }
}
```

**Insertion point:** Right after the `Form` arm at L219-224.

---

#### 3c. `resolve_errors_node` arm

**Analog:** `Component::Form` arm at `resolve.rs:399-403`:

```rust
Component::Form(props) => {
    for field in &mut props.fields {
        resolve_errors_node(field, errors, all);
    }
}
```

**Emission:**
```rust
Component::DetailForm(props) => {
    for field in &mut props.fields {
        resolve_errors_node(&mut field.input, errors, all);
    }
}
```

**Insertion point:** Right after the `Form` arm at L399-403.

**Critical gotcha:** DetailForm does NOT hold a validation `error` slot at the component level (unlike `Input`, `Select`, `Checkbox`, `Switch`, `KeyValueEditor`). It only recurses. See `Component::KeyValueEditor(props)` arm at L472-474 — DO NOT copy that shape, copy the `Form` shape.

---

#### 3d. Resolver tests

**Analog:** `resolve_form_action` at `resolve.rs:592-633`:

```rust
#[test]
fn resolve_form_action() {
    let mut view = JsonUiView::new().component(ComponentNode {
        key: "form".to_string(),
        component: Component::Form(FormProps {
            action: make_action("users.store"),
            fields: vec![ComponentNode { /* Input */ }],
            method: None, guard: None, max_width: None,
        }),
        action: None, visibility: None,
    });

    resolve_actions(&mut view, test_resolver);

    match &view.components[0].component {
        Component::Form(props) => assert_eq!(props.action.url, Some("/users".to_string())),
        _ => panic!("expected Form"),
    }
}
```

**Tests to add:**
- `resolve_detail_form_action` — assert `props.action.url` populates from the resolver.
- `resolve_errors_propagates_into_detail_form_fields` — build a DetailForm containing an Input; set errors; assert `InputProps.error` is populated after `resolve_errors`.
- Negative test: `resolve_does_not_touch_edit_or_cancel_url` — D-16 says they are raw hrefs. Set both to arbitrary strings; assert they are untouched after `resolve_actions`.

**Insertion point:** Inside `#[cfg(test)] mod tests` (starts at L478). Place after `resolve_form_action` (L592-633).

---

### 4. `ferro-json-ui/src/lib.rs`

Two edits: public re-export block + `COMPONENT_CATALOG` entry.

#### 4a. Public re-exports

**Analog:** the `pub use component::{…}` block at `lib.rs:59-71`.

**Pattern to copy (lib.rs:59-71):**

```rust
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CheckboxProps, ChecklistItem, ChecklistProps, CollapsibleProps,
    Column, ColumnFormat, Component, ComponentNode, DataTableProps, DescriptionItem,
    DescriptionListProps, DropdownMenuAction, DropdownMenuProps, EmptyStateProps, FormMaxWidth,
    FormProps, FormSectionProps, GapSize, GridProps, HeaderProps, IconPosition, ImageProps,
    InputProps, InputType, KanbanBoardProps, KanbanColumnProps, KeyValueEditorProps, ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PageHeaderProps, PaginationProps,
    PluginProps, ProductTileProps, ProgressProps, SelectOption, SelectProps, SeparatorProps,
    SidebarGroup, SidebarNavItem, SidebarProps, Size, SkeletonProps, SortDirection, StatCardProps,
    SwitchProps, Tab, TableProps, TabsProps, TextElement, TextProps, ToastProps, ToastVariant,
};
```

**Ordering convention:** **Alphabetical** within the block. Confirmed by inspection (`ActionCardProps` → `ActionCardVariant` → `AlertProps` → … → `ToastVariant`). Insert:
- `DetailField` — between `DescriptionListProps` and `DropdownMenuAction`
- `DetailFormProps` — between `DetailField` and `DropdownMenuAction`
- `EditMode` — between `EmptyStateProps` and `FormMaxWidth`

**Emission diff:**
```
    Column, ColumnFormat, Component, ComponentNode, DataTableProps, DescriptionItem,
-   DescriptionListProps, DropdownMenuAction, DropdownMenuProps, EmptyStateProps, FormMaxWidth,
+   DescriptionListProps, DetailField, DetailFormProps, DropdownMenuAction, DropdownMenuProps,
+   EditMode, EmptyStateProps, FormMaxWidth,
```

---

#### 4b. `COMPONENT_CATALOG` entry

**Analog:** `### KeyValueEditor` block at `lib.rs:140-142`:

```rust
### KeyValueEditor
Props: field (String), label (Option<String>), suggested_keys (Vec<String>), allow_custom_keys (bool, default true), data_path (Option<String> — must resolve to a JSON object), error (Option<String>)
Serializes to hidden `<input name="{field}" type="hidden" value="{...json...}">`. When `allow_custom_keys` is true, the key input is a text field with a `<datalist>` from `suggested_keys`; when false, the key input is a `<select>` restricted to `suggested_keys`. Runtime syncs the hidden field on every add/delete/input event.
```

**Ordering convention:** This catalog is roughly grouped by family (form-family together). Insert `### DetailForm` right after the `### Form` block at L116-117 — this groups form-family components and matches the rendering narrative ("DetailForm is the View/Edit twin of DescriptionList+Form").

**Required content (D-19 + UI-SPEC §14.8):** One-sentence description plus the Option-A authoring rule (empty `label` on inner input). The §9 rule must be restated so `ferro-mcp`-driven agents discover it via introspection.

**Emission template:**
```
### DetailForm
Props: mode (EditMode: view|edit), action (Action), fields (Vec<DetailField {label, value, input}>), edit_url (String), cancel_url (String), edit_label (Option<String>, default "Modifica"), save_label (Option<String>, default "Salva"), cancel_label (Option<String>, default "Annulla"), method (Option<HTTP method override>)
Split-mode detail page with inline edit: View mode renders a <dl> with "Modifica" link; Edit mode wraps the same <dl> in a <form> with "Salva"/"Annulla" actions. Mode is URL-driven via ?mode=edit (server-side only; no JS). Authoring rule: when DetailField.input is an Input/Select/Textarea component, the caller MUST set its label to "" — the <dt> provides the visible label. DetailForm does not mutate caller-supplied props.
```

---

### 5. `ferro-mcp/src/tools/json_ui_catalog.rs`

**Analog entry:** `CatalogComponent { name: "DescriptionList", … }` at `json_ui_catalog.rs:504-522`.

**Pattern to copy (json_ui_catalog.rs:504-522):**

```rust
CatalogComponent {
    name: "DescriptionList".to_string(),
    description: "Key-value pairs displayed as a description list.".to_string(),
    props: vec![
        prop("items", "Vec<DescriptionItem>", true, "Items: { label, value, format? }"),
        prop("columns", "Option<u8>", false, "Number of columns for layout"),
    ],
    variants: None,
},
```

**Helper fn (json_ui_catalog.rs:1090-1097):**
```rust
fn prop(name: &str, type_name: &str, required: bool, description: &str) -> PropInfo {
    PropInfo { name: name.to_string(), type_name: type_name.to_string(), required, description: description.to_string() }
}
```

**Ordering convention — two insertions:**

1. In `build_component_catalog()` (ends at L1031 with `Image` as the last entry), insert a new `CatalogComponent { name: "DetailForm", … }` near the form-family entries. The catalog is NOT alphabetical; follow the loose family grouping.

2. **EXHAUSTIVE-LIST TEST (L1113-1154):** Add `"DetailForm"` to the `expected` array AND update the count at L1107-1111:

```rust
// json_ui_catalog.rs:1106-1111
assert_eq!(
    catalog.components.len(),
    39,    // ← becomes 40 after DetailForm (41 if KeyValueEditor backfill)
    "Catalog should contain all 39 built-in components, got {}",
    catalog.components.len()
);
```

**Backfill note (RESEARCH §Pitfall 6 — VERIFIED by grep):** `KeyValueEditor` is **NOT** in `json_ui_catalog.rs` — neither as a `CatalogComponent` nor in the exhaustive list. This phase should backfill it (same edit, same file) while adding DetailForm. New count becomes `41`. Plan should explicitly list this as a sub-task, not defer it (RESEARCH §Open Questions).

**Emission template — `DetailForm` CatalogComponent:**
```rust
CatalogComponent {
    name: "DetailForm".to_string(),
    description: "Split-mode detail page with inline edit: View renders <dl> + Modifica link; \
                  Edit wraps the same <dl> in a <form> with Salva/Annulla actions. \
                  Mode is URL-driven (?mode=edit); server-side only. When DetailField.input is an Input, \
                  its `label` must be empty string — the <dt> provides the visible label."
        .to_string(),
    props: vec![
        prop("mode", "EditMode", true, "View (default) or Edit"),
        prop("action", "Action", true, "Form submit target used in Edit mode"),
        prop("fields", "Vec<DetailField>", true, "Rows: { label, value, input: ComponentNode }"),
        prop("edit_url", "String", true, "Href for the 'Modifica' link (View mode)"),
        prop("cancel_url", "String", true, "Href for the 'Annulla' link (Edit mode)"),
        prop("edit_label", "Option<String>", false, "Override for 'Modifica' label"),
        prop("save_label", "Option<String>", false, "Override for 'Salva' label"),
        prop("cancel_label", "Option<String>", false, "Override for 'Annulla' label"),
        prop("method", "Option<HttpMethod>", false, "HTTP method override (else uses action.method)"),
    ],
    variants: None,
},
```

---

### 6. `docs/src/json-ui/components.md`

**Analog:** `### DescriptionList` section at `docs/src/json-ui/components.md:415-471`.

**Pattern to copy (components.md:415-471):**

```markdown
### DescriptionList

Key-value pairs displayed as a description list. Reuses `ColumnFormat` for value formatting.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `items` | `Vec<DescriptionItem>` | Yes | - | Key-value items |
| `columns` | `Option<u8>` | No | `None` | Number of columns for layout |

**DescriptionItem** defines a key-value pair:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Item label |
| `value` | `String` | Yes | Item value |
| `format` | `Option<ColumnFormat>` | No | Display format |

```rust
use ferro::{ComponentNode, Component, DescriptionListProps, DescriptionItem, ColumnFormat};

ComponentNode {
    key: "user-info".to_string(),
    component: Component::DescriptionList(DescriptionListProps { … }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "user-info", "type": "DescriptionList", … }
```
```

**Ordering convention:** Sections are grouped by family; `### DescriptionList` (L415) and `### Form` (L725) are the two closest anchors. Insert `### DetailForm` immediately after `### DescriptionList` — it's semantically the View/Edit twin.

**Required content** (per CLAUDE.md docs rule + UI-SPEC §14.7):
1. Rust construction example (use `DetailField::new` + `ComponentNode::detail_form`).
2. JSON round-trip example.
3. Props table for `DetailFormProps`.
4. Fields table for `DetailField`.
5. Variants table for `EditMode`.
6. "When to use" paragraph contrasting `DetailForm` with `DescriptionList + Form`.
7. **Option-A authoring rule** (§9 of UI-SPEC) verbatim — "When DetailField.input is an Input, set `label: \"\"` — the `<dt>` provides the visible label."

---

## Shared Patterns

### Pattern S-1: Three-pass resolver participation

**Source:** `ferro-json-ui/src/resolve.rs:46-51`, `:219-224`, `:399-403` (the three `Component::Form` arms).

**Apply to:** Any new component variant that holds an `Action` AND contains child `ComponentNode`s.

**Rule:** Every `match` in `resolve.rs` has a leaf catch-all. Adding a container variant requires **three separate arm insertions**, one per function:
1. `resolve_component_node` — recurse into children + resolve `Action`.
2. `collect_unresolved_node` — same shape, reading-only equivalent.
3. `resolve_errors_node` — recurse into children only (no `Action` visit).

Missing any one silently breaks a resolver pass without compile-time error (because the catch-all absorbs the variant).

### Pattern S-2: Tagged-enum add

**Source:** `ferro-json-ui/src/component.rs:997-1010` (`serialize_tagged`), `:1012-1060` (Serialize), `:1064-1204` (Deserialize).

**Apply to:** Any new `Component` variant.

**Rule (three edits):**
1. Add enum variant near the family grouping, keeping `Plugin` last (L991).
2. Add `Component::{Name}(p) => serialize_tagged(serializer, "{Name}", p),` arm.
3. Add `"{Name}" => serde_json::from_value::<{Name}Props>(value).map(Component::{Name}).map_err(de::Error::custom),` arm before the `_ =>` plugin fallback.

All three use the same string literal for the type name — MUST match exactly.

### Pattern S-3: `html_escape` discipline on every dynamic string

**Source:** `ferro-json-ui/src/render.rs:2917` (`pub(crate) fn html_escape`).

**Apply to:** Every dynamic string emitted into attribute or text context inside `render_detail_form` — labels, values, `edit_url`, `cancel_url`, `action_url`, every button label.

**Rule:** Without `html_escape`, a `"` in a caller-supplied URL breaks out of the `href` attribute (Pitfall 5 in RESEARCH). XSS tests (`render_detail_form_view_xss_escapes_*`) verify this.

### Pattern S-4: JsonSchema skipped for props containing ComponentNode

**Source:** `Tab` (`component.rs:442-450`), `TabsProps` (`:454-458`), `FormProps` (`:187-203`).

**Apply to:** `DetailFormProps` and `DetailField`.

**Rule:** The comment `// JsonSchema skipped: contains Vec<ComponentNode> — Component has custom Serialize/Deserialize` (or analogous) MUST appear directly above the struct. `#[derive(…)]` then OMITS `JsonSchema` but keeps `Debug, Clone, PartialEq, Serialize, Deserialize`.

### Pattern S-5: `#[serde(default, skip_serializing_if = "Option::is_none")]` on every `Option`

**Source:** Every `Option`-typed prop across `component.rs` (verified across `FormProps`, `InputProps`, `ButtonProps`, `KeyValueEditorProps`).

**Apply to:** Every `Option` field in `DetailFormProps` — `edit_label`, `save_label`, `cancel_label`, `method`.

**Rule:** No `Option` field serializes `null` — missing is the default state. Same attribute on every field; no exceptions.

---

## No Analog Found

*None.* Every file modified and every edit has a precedent in the existing ferro-json-ui crate or (for the MCP catalog entry) in ferro-mcp.

The one design question **without** a codebase analog is the empty-label Option-A rule (§9 of UI-SPEC). This is a new authoring convention introduced by this phase — RESEARCH recommends and the UI-SPEC ratifies it. The "copy from" source here is conceptual (the `render_input` label emission at `render.rs:1407-1412` determines *why* Option A is safe), not a direct pattern transfer.

---

## Insertion-Point Ordering Summary

| File | Section | Ordering convention | DetailForm insertion |
|------|---------|---------------------|----------------------|
| `component.rs` | `Component` enum (L949-991) | family grouping; `Plugin` last | after `KeyValueEditor`, before `Plugin` |
| `component.rs` | `Serialize` match (L1014-1058) | matches enum order | after `KeyValueEditor`, before `Plugin` |
| `component.rs` | `Deserialize` match (L1072-1203) | matches enum order | after `"KeyValueEditor"`, before `_ =>` fallback |
| `component.rs` | `ComponentNode` factories (L1223-…) | family grouping | after `ComponentNode::form` (L1245) |
| `component.rs` | struct definitions | family grouping | `EditMode` + `DetailField` + `DetailFormProps` together, near `FormProps` / `Tab` neighbourhood |
| `render.rs` | `collect_plugin_types_node` (L101-197) | containers first, leaf `|`-chain middle | container arm next to `Component::Form` (L114-118) |
| `render.rs` | `render_component` dispatch (L288-340) | family comment-banner groups | under `// Container components.`, after `Form` (L305) |
| `render.rs` | function definitions | family grouping | `fn render_detail_form` after `render_form` (L971-1031) |
| `render.rs` | `mod tests` | numbered banner sections (`// ── 19. Form ──`) | new numbered banner after the existing form/key-value banners |
| `resolve.rs` | `resolve_component_node` (L30-155) | containers first | after Form arm (L46-51) |
| `resolve.rs` | `collect_unresolved_node` (L204-328) | containers first | after Form arm (L219-224) |
| `resolve.rs` | `resolve_errors_node` (L377-476) | containers first | after Form arm (L399-403) |
| `resolve.rs` | `mod tests` | by-variant grouping | after `resolve_form_action` (L592-633) |
| `lib.rs` | `pub use component::{…}` (L59-71) | **strictly alphabetical** | `DetailField`, `DetailFormProps` between `DescriptionListProps` and `DropdownMenuAction`; `EditMode` between `EmptyStateProps` and `FormMaxWidth` |
| `lib.rs` | `COMPONENT_CATALOG` (L102-186) | loose family grouping | `### DetailForm` right after `### Form` (L116-117) |
| `ferro-mcp/.../json_ui_catalog.rs` | `build_component_catalog()` (L…-1031) | loose family grouping | near form-family entries |
| `ferro-mcp/.../json_ui_catalog.rs` | exhaustive-list test (L1114-1154) | unordered array, matched by name | append `"DetailForm"` (and backfill `"KeyValueEditor"`); update count L1107 to 41 |
| `docs/src/json-ui/components.md` | `###` sections | family grouping | after `### DescriptionList` (L415-471) |

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/tools/`, `docs/src/json-ui/`
**Files scanned:** `component.rs` (3700+ lines), `render.rs` (8700+ lines), `resolve.rs` (~640 lines), `lib.rs` (~190 lines), `action.rs` (~110 lines), `json_ui_catalog.rs` (1301 lines), `components.md`
**Pattern extraction date:** 2026-04-23
**Phase 146 precedent:** Same four-file playbook (component.rs + render.rs + resolve.rs + lib.rs). Phase 147 adds a fifth file (ferro-mcp catalog) because Pitfall 6 verified phase 146 left that gap open.

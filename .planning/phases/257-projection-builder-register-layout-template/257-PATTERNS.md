# Phase 257: Projection Builder — Register Layout Template - Pattern Map

**Mapped:** 2026-07-06
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/projection/builder.rs` | service (emit helper + match arm) | transform | Same file — `emit_kanban_root`, `emit_card_root`, `emit_datatable_root` | exact |
| `ferro-json-ui/src/projection/intent_layout.rs` | utility (template helper) | transform | Same file — `default_template` function body | exact |
| `ferro-json-ui/src/spec.rs` — SpecBuilder.fill_viewport | utility (builder setter) | transform | Same file — `SpecBuilder::layout`, `SpecBuilder::data` consuming setters | exact |
| `ferro-json-ui/src/spec.rs` — ElementBuilder.each | utility (builder setter) | transform | Same file — `ElementBuilder::prop`, `ElementBuilder::child`, `ElementBuilder::visible` consuming setters | exact |
| `ferro-json-ui/src/catalog.rs` — $each guard | utility (validation fix) | transform | Same file — `el.props.is_null()` early-continue guard (Stage 2, line 750) | role-match |
| `app/src/controllers/cassa.rs` — flip to projection | controller | request-response | `ferro-json-ui/src/projection/mod.rs` `JsonUiRenderer` usage in tests; `app/src/projections/order.rs` ServiceDef construction | role-match |
| `app/src/views/cassa.json` | — (deleted) | — | — | — |
| `app/src/routes.rs` — rimuovi deletion | config (route table edit) | request-response | Existing route table entries around line 24–25 | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/projection/builder.rs` — "Register" match arm + `emit_register_root`

**Analog:** Same file — `emit_kanban_root` (lines 403–499) and `emit_card_root` (lines 346–378) for the aux_elements idiom; `emit_datatable_root` (lines 292–331) for meaning-driven mapping and the `/data/{service.name}` convention.

**Match arm pattern** (lines 251–266 — the dispatch table the new arm extends):
```rust
let root = match layout {
    "DataTable" => emit_datatable_root(service),
    "Card" => emit_card_root(service, &template.slots, &mut aux_elements),
    "Form" => {
        return build_input_spec(service);  // short-circuits — Register must NOT do this
    }
    "KanbanBoard" => emit_kanban_root(service, ctx),
    "StatCard" => emit_statcard_root(service, &template.slots, &mut aux_elements),
    // NEW Register arm goes here:
    // "Register" => emit_register_root(service, &mut aux_elements)?,
    other => {
        return Err(ProjectionError::UnknownComponent {
            type_name: other.to_string(),
        });
    }
};
```

**Spec assembly path** (lines 272–278 — Register participates in this, unlike "Form"):
```rust
let mut builder = Spec::builder()
    .title(resolve_title(service))
    .element("root", root);
for (id, el) in aux_elements {
    builder = builder.element(id, el);
}
builder.build().map_err(ProjectionError::SpecBuild)
```

**Aux-elements pattern** (from `emit_card_root`, lines 346–378 — how children are registered in the flat element map):
```rust
fn emit_card_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    let mut children: Vec<String> = Vec::new();
    // ... emit child elements into aux, collect IDs into children ...
    let mut el = element_with_props("Card", props);
    for id in children {
        el = el.child(id);
    }
    el
}
```

**Meaning-driven field lookup** (from `emit_datatable_root`, lines 292–331):
```rust
fn emit_datatable_root(service: &ServiceDef) -> ElementBuilder {
    let columns: Vec<Column> = service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .filter(|f| lookup_meaning(&f.meaning).column.is_some())
        .map(build_column_for_field)
        .collect();
    // data_path follows the /data/{service.name} convention:
    let props = serde_json::to_value(DataTableProps {
        data_path: format!("/data/{}", service.name),
        // ...
    }).expect("DataTableProps serialization cannot fail");
    element_with_props("DataTable", props)
}
```

**Action derivation pattern** (from `emit_datatable_root`, lines 300–315 — confirm action selection analog):
```rust
let row_actions: Option<Vec<DropdownMenuAction>> = if service.actions.is_empty() {
    None
} else {
    Some(
        service.actions.iter()
            .map(|a| DropdownMenuAction {
                label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
                action: Action::new(format!("/{}/{{row_key}}/{}", service.name, a.name)),
                destructive: false,
                visible_if: None,
            })
            .collect(),
    )
};
```

**element_with_props helper** (lines 140–148 — used by every emit function to convert typed Props):
```rust
fn element_with_props(type_name: &str, props: serde_json::Value) -> ElementBuilder {
    let obj = props
        .as_object()
        .expect("typed Props must serialize to a JSON object");
    let mut el = Element::new(type_name);
    for (k, v) in obj {
        el = el.prop(k.clone(), v.clone());
    }
    el
}
```

**emit_register_root target element tree** (from `cassa.json` updated to 256 contract — two-grid structure satisfying all four register lint rules):
```
spec.root → "register_root": Grid(fill=true, columns=1)
              children: ["sale_form"]
"sale_form": Form(id="sale_form", action=<from first POST action in service.actions>)
              children: ["panes_grid"]
"panes_grid": Grid(fill=true, columns=1, md_columns=3, spans=[1,2])
               children: ["selection_pane", "tiles_pane"]
"selection_pane": SelectionPanel(form_id="sale_form")
                   children: ["confirm_btn"]
"confirm_btn": Button(label=<action display_name>, form="sale_form",
                      button_type=Submit, disable_on_submit=true)
"tiles_pane": TileGrid(form_id="sale_form", search=true,
                       data_path="/data/{service.name}")
               children: ["tile_tmpl"]
"tile_tmpl": Tile($each{path="/data/{service.name}", as_="p"},
                  item_id={"$data":"/p/{id_field.name}"},
                  name={"$data":"/p/{name_field.name}"},
                  price={"$data":"/p/{price_field.name}"},
                  price_cents={"$data":"/p/{price_field.name}_cents"},
                  field={"$data":"/p/field"})
```

**Critical: Register arm must set fill_viewport and layout at the Spec builder call site** (D-06, D-13). The Spec::builder call in `build_display_spec` becomes layout-conditional or the Register arm passes these flags through an out-parameter — the simplest approach mirrors the existing `build_display_spec` shared assembly path with conditionals added:
```rust
// Inside build_display_spec, after the match:
let mut builder = Spec::builder()
    .title(resolve_title(service));
if matches!(layout, "Register") {
    builder = builder.fill_viewport(true).layout("dashboard");
}
builder = builder.element("root", root);
// ...
```

**Test pattern — injected catalog (mandatory for projection tests)** (lines 828–829):
```rust
fn clean_catalog() -> Catalog {
    Catalog::build_builtins_only().expect("builtins-only catalog builds clean")
}

// In test body:
let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &clean_catalog())
    .expect("should project");
```

---

### `ferro-json-ui/src/projection/intent_layout.rs` — `register_template()` helper

**Analog:** Same file — `default_template` function (lines 56–120); the ThemeTemplates literal shape is demonstrated in `builder.rs` tests (lines 943–957).

**ThemeTemplates literal shape** (from `builder.rs` tests, lines 943–957):
```rust
let templates = ThemeTemplates {
    browse: Some(IntentModeTemplates {
        display: IntentSlotTemplate {
            slots: vec!["title".into(), "stats".into(), "metadata".into()],
            layout: Some("StatCard".into()),
        },
        input: IntentSlotTemplate::default(),
    }),
    focus: None,
    collect: None,
    process: None,
    summarize: None,
    analyze: None,
    track: None,
};
```

**register_template() function to add** (mirrors the shape above, targeting Collect):
```rust
/// Ready-made `ThemeTemplates` overriding Collect to use the Register layout.
///
/// Supply via `VisualContext.templates` to project a `ServiceDef` into a
/// Register composition (fill-viewport Grid + Form + SelectionPanel + TileGrid).
/// The slot list is informational; `emit_register_root` ignores slot granularity
/// (same convention as `emit_datatable_root`) and derives the full element tree
/// from the `ServiceDef` directly. The built-in `default_template(Collect)`
/// (Form layout) is unaffected.
pub fn register_template() -> ThemeTemplates {
    ThemeTemplates {
        collect: Some(IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["items".into(), "actions".into()],
                layout: Some("Register".into()),
            },
            input: IntentSlotTemplate::default(),
        }),
        browse: None,
        focus: None,
        process: None,
        summarize: None,
        analyze: None,
        track: None,
    }
}
```

**Default template test pattern** (lines 127–138 — mirror for register_template test):
```rust
#[test]
fn default_template_browse_uses_data_table() {
    let t = default_template(&Intent::Browse);
    assert_eq!(t.display.layout.as_deref(), Some("DataTable"));
    assert_eq!(
        t.display.slots,
        vec!["title".to_string(), "fields".to_string(), "pagination".to_string()]
    );
    assert!(t.input.slots.is_empty());
}
```

---

### `ferro-json-ui/src/spec.rs` — `SpecBuilder.fill_viewport(bool)` setter (D-13)

**Analog:** Same file — `SpecBuilder::layout` (lines 391–394) and `SpecBuilder::data` (lines 397–400) consuming setters.

**Existing consuming setter pattern** (lines 391–394):
```rust
pub fn layout(mut self, l: impl Into<String>) -> Self {
    self.layout = Some(l.into());
    self
}
```

**Current `build()` hardcode** (line 460 — the `false` to replace with `self.fill_viewport_`):
```rust
let spec = Spec {
    schema: SCHEMA_VERSION.to_string(),
    root,
    elements: self.elements,
    title: self.title,
    layout: self.layout,
    fill_viewport: false,   // ← replace with self.fill_viewport_
    data: self.data,
    design: None,
};
```

**New field and setter to add to SpecBuilder:**
```rust
// Add to SpecBuilder struct (after `layout: Option<String>`):
// fill_viewport_: bool,   (bool defaults to false via Default)

/// Enable fill-viewport mode. The root Grid must also have `fill: true`
/// and `spec.layout` must be an app-shell layout (`"dashboard"` or `"app"`)
/// for the ferro-fill CSS chain to activate.
pub fn fill_viewport(mut self, v: bool) -> Self {
    self.fill_viewport_ = v;
    self
}
```

---

### `ferro-json-ui/src/spec.rs` — `ElementBuilder.each(path, as_)` setter (D-12)

**Analog:** Same file — `ElementBuilder::prop`, `ElementBuilder::child`, `ElementBuilder::visible` (lines 482–521).

**Existing consuming setter pattern** (lines 488–491):
```rust
pub fn child(mut self, id: impl Into<String>) -> Self {
    self.children.push(id.into());
    self
}
```

**Existing private `each` field** (line 477):
```rust
each: Option<EachDirective>,   // already exists; only the public setter is missing
```

**EachDirective type** (lines 197–204):
```rust
pub struct EachDirective {
    pub path: String,
    #[serde(rename = "as")]
    pub as_: String,
}
```

**New setter to add:**
```rust
/// Set the `$each` iteration directive on this element.
///
/// `path` is a JSON-pointer to an array in `Spec.data`; `as_` is the
/// loop-variable name used inside this element's `$data` prop bindings.
/// Template elements have data-bound props that cannot be type-checked
/// before `$each` expansion — `Catalog::validate` skips per-element
/// Props validation for elements whose `each` is `Some`.
pub fn each(mut self, path: impl Into<String>, as_: impl Into<String>) -> Self {
    self.each = Some(EachDirective { path: path.into(), as_: as_.into() });
    self
}
```

---

### `ferro-json-ui/src/catalog.rs` — $each template element guard (Stage 2)

**Analog:** Same file — `el.props.is_null()` early-continue guard (lines 750–752).

**Existing null-props guard** (lines 750–752):
```rust
if el.props.is_null() {
    continue;
}
```

**New guard to add immediately after the null-props guard** (before the `strip_expr_objects` call at line 771):
```rust
// Template elements have data-bound props of any type; types cannot be
// validated before $each expansion. Skip per-element Props validation.
if el.each.is_some() {
    continue;
}
```

**Why it is needed — strip_expr_objects behavior** (lines 1162–1178):
```rust
fn strip_expr_objects(val: &Value) -> Value {
    match val {
        Value::Object(map) => {
            if map.len() == 1 && (map.contains_key("$data") || map.contains_key("$template")) {
                Value::String(String::new())  // becomes "" — fails anyOf[integer,null]
            } else { /* ... */ }
        }
        // ...
    }
}
```

`TileProps.price_cents` is `Option<u64>` (schema `anyOf[integer,null]`). After `strip_expr_objects`, any `{"$data":...}` binding becomes `""`, which matches neither `integer` nor `null`, causing Stage 2 validation to fail. The `el.each.is_some()` guard skips this check for template elements entirely.

---

### `app/src/controllers/cassa.rs` — flip to projection-derived spec

**Analog:** `ferro-json-ui/src/projection/mod.rs` lines 76–108 (`JsonUiRenderer` doctest + `Renderer` impl); `app/src/projections/order.rs` (ServiceDef construction with actions).

**Current handler to replace** (lines 1–59 of cassa.rs — the `render_file` call and data assembly):
```rust
use ferro::{handler, serde_json, JsonUi, Response};

#[handler]
pub async fn index() -> Response {
    // ... product data synthesis ...
    JsonUi::render_file("src/views/cassa.json", data)
}
```

**New handler shape:**
```rust
use ferro::{handler, serde_json, DataType, FieldMeaning, IntentHint, Intent, JsonUi, Response};
use ferro_projections::{derive_intents, ActionDef, ServiceDef};
use ferro_json_ui::{JsonUiRenderer, VisualContext, RenderMode};
use ferro_projections::render::{BaseContext, Renderer};
use ferro_json_ui::projection::intent_layout::register_template;

#[handler]
pub async fn index() -> Response {
    let service = ServiceDef::new("prodotti")
        .display_name("Cassa")
        // fields with meanings: Identifier, EntityName, Money + any additional
        // Italian display names are app-land (allowed per CLAUDE.md project-agnostic rule)
        .action(ActionDef::new("conferma").display_name("Conferma ordine"))
        .hint(IntentHint::Primary(Intent::Collect));  // D-11: force Collect primary

    let intents = derive_intents(&service);
    let ctx = VisualContext {
        base: BaseContext::default(),
        mode: RenderMode::Display,
        templates: Some(register_template()),
    };

    let spec = JsonUiRenderer.render(&service, &intents, &ctx)
        .map_err(|e| ferro::error_response!(500, "{e}"))?;

    // Synthesise product rows; must carry: id, nome, prezzo (display string),
    // price_cents (integer cents), field ("qty_{id}") per D-10 data contract.
    let prodotti: Vec<serde_json::Value> = /* ... */;
    let data = serde_json::json!({ "prodotti": prodotti });

    JsonUi::render(&spec, &data)
}
```

**JsonUiRenderer call shape** (from `projection/mod.rs` lines 101–108):
```rust
impl Renderer for JsonUiRenderer {
    type Output = Spec;
    type Context = VisualContext;

    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &VisualContext)
        -> Result<Spec, Error>
    {
        Spec::from_service_def(service, intents, ctx).map_err(|e| Error::Render(e.to_string()))
    }
}
```

**JsonUi::render signature** (from `framework/src/json_ui/mod.rs` line 74):
```rust
pub fn render(spec: &Spec, data: &serde_json::Value) -> Response
```

**ServiceDef fluent builder pattern** (from `app/src/projections/order.rs` lines 10–53):
```rust
ServiceDef::new("order")
    .display_name("Order")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer_name", DataType::String, FieldMeaning::EntityName)
    .action(ActionDef::new("submit").display_name("Submit"))
```

---

### `app/src/routes.rs` — rimuovi route deletion

**Analog:** Surrounding route entries (lines 23–25 in routes.rs).

**Line to remove** (line 25):
```rust
post!("/cassa/rimuovi/:id", controllers::cassa::rimuovi).name("cassa.rimuovi"),
```

**Lines to keep** (lines 23–24):
```rust
get!("/cassa", controllers::cassa::index).name("cassa.index"),
post!("/cassa/conferma", controllers::cassa::conferma).name("cassa.conferma"),
```

---

## Shared Patterns

### Consuming builder setters (house convention)
**Source:** `ferro-json-ui/src/spec.rs` — all `SpecBuilder` and `ElementBuilder` methods
**Apply to:** `fill_viewport` setter on SpecBuilder; `each` setter on ElementBuilder
```rust
pub fn setter_name(mut self, value: ValueType) -> Self {
    self.field_name = value;
    self
}
```

### Props serialization — infallible expect message
**Source:** `ferro-json-ui/src/projection/builder.rs` — every emit function
**Apply to:** every `serde_json::to_value(Props { ... })` call in `emit_register_root`
```rust
serde_json::to_value(GridProps { /* ... */ })
    .expect("GridProps serialization cannot fail")
```

### Injected-catalog test isolation (mandatory for all new projection tests)
**Source:** `ferro-json-ui/src/projection/builder.rs` lines 828–829, 839
**Apply to:** SC-1, D-05, D-14 tests
```rust
fn clean_catalog() -> Catalog {
    Catalog::build_builtins_only().expect("builtins-only catalog builds clean")
}
// Always use from_service_def_with_catalog, never from_service_def in tests:
let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &clean_catalog())
    .expect("...");
```

### Design lint assertion pattern
**Source:** `app/src/tests/design_lint.rs` lines 22–30
**Apply to:** D-05 integration test asserting the register spec is lint-clean
```rust
use ferro_json_ui::design::lint;
let findings = lint(&spec);
assert!(
    findings.is_empty(),
    "register spec must yield zero lint findings, got: {findings:#?}"
);
```

---

## Component Props Reference for `emit_register_root`

All structs in `ferro-json-ui/src/component.rs`:

| Component | Key Props | Required |
|-----------|-----------|----------|
| `GridProps` | `columns: u8`, `md_columns: Option<u8>`, `gap: GapSize`, `fill` (bool, via feature), `spans: Vec<u8>` | none (defaults) |
| `FormProps` | `action: Action`, `id: Option<String>`, `method: Option<HttpMethod>` | `action` |
| `SelectionPanelProps` | `form_id: String`, `currency: Option<String>`, `total_label: Option<String>`, `empty_message: Option<String>` | `form_id` |
| `ButtonProps` | `label: String`, `button_type: Option<ButtonType>`, `form: Option<String>`, `disable_on_submit: Option<bool>` | `label` |
| `TileGridProps` | `data_path: String`, `form_id: String`, `search: Option<bool>` | `data_path`, `form_id` |
| `TileProps` | `item_id: String`, `name: String`, `price: String`, `field: String`, `price_cents: Option<u64>` | `item_id`, `name`, `price`, `field` |

**TileProps.price_cents note:** `Option<u64>` with a `$data` binding becomes `""` after `strip_expr_objects`, which fails `anyOf[integer,null]`. The catalog.rs `el.each.is_some()` guard (above) must be in place before `emit_register_root` can produce a catalog-valid spec with this field.

---

## No Analog Found

None — all files have close analogs in the existing codebase.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/projection/`, `ferro-json-ui/src/`, `app/src/controllers/`, `app/src/projections/`, `app/src/tests/`, `app/src/views/`
**Files scanned:** 18
**Pattern extraction date:** 2026-07-06

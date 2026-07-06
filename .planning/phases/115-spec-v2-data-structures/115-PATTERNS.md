# Phase 115: Spec v2 Data Structures - Pattern Map

**Mapped:** 2026-04-18
**Files analyzed:** 8
**Analogs found:** 7 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/spec.rs` (NEW) | data-model + builder + validator | transform (parse → validate) | `ferro-json-ui/src/view.rs` (v1 top-level type) + `ferro-projections/src/service.rs` (builder) + `ferro-projections/src/error.rs` (thiserror enum) | exact (composite) |
| `ferro-json-ui/src/component.rs` (REWRITE) | data-model (Props structs) | transform | current `component.rs` minus `Component`/`ComponentNode`/custom ser/de | exact (self-reduction) |
| `ferro-json-ui/src/render.rs` (REWRITE as placeholder) | renderer (placeholder) | transform (Spec → HTML) | current `render.rs` (structure + `RenderResult` shape) | role-match |
| `ferro-json-ui/src/projection/mod.rs` (update Output) | renderer impl (Renderer trait) | transform (ServiceDef → Spec) | current `projection/mod.rs` + `ferro-projections/src/render/mod.rs` trait | exact (shape swap) |
| `ferro-json-ui/src/lib.rs` (REWRITE re-exports) | crate-root re-export manifest | config | current `lib.rs` | exact (self-reduction) |
| `framework/src/json_ui/mod.rs` (REWRITE caller) | framework integration | request-response | current `JsonUi` impl (lines 33–232) | exact (self-reduction) |
| `ferro-json-ui/tests/round_trip.rs` (NEW) | integration test | fixture-driven IO | `view.rs` tests (`round_trip_build_to_json_from_json`, `from_json_full_example`) | role-match |
| `ferro-json-ui/tests/reject.rs` (NEW) | integration test | fixture-driven IO | none — new test category (validation failure assertions) | no direct analog |

## Pattern Assignments

### `ferro-json-ui/src/spec.rs` (NEW)

Single file holding `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SpecError`, `SCHEMA_VERSION`, `MAX_NESTING_DEPTH`, and the validation pipeline. Combines three analogs: top-level-struct shape from `view.rs`, consuming-builder ergonomics from `service.rs`, and `thiserror` enum convention from `projections/error.rs`.

#### Pattern A — Top-level struct shape with `$schema` rename and optional-field skip rules

**Analog:** `ferro-json-ui/src/view.rs` lines 33–47

```rust
// JsonSchema skipped: contains Vec<ComponentNode> — Component has custom Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonUiView {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Vec<String>>>,
    pub components: Vec<ComponentNode>,
}
```

**What to copy verbatim:**
- `#[serde(rename = "$schema")] pub schema: String`
- `#[serde(default, skip_serializing_if = "Option::is_none")]` on `layout` and `title`
- `#[serde(default, skip_serializing_if = "serde_json::Value::is_null")]` on `data`
- `Debug, Clone, PartialEq, Serialize, Deserialize` derive list

**What must change (D-06):**
- Replace `components: Vec<ComponentNode>` with `elements: HashMap<String, Element>` (required, not skipped)
- Add `root: String` (required)
- **DELETE** `errors: Option<HashMap<String, Vec<String>>>` — errors move to render context per D-06
- Remove "JsonSchema skipped" comment — after v2 strip, `JsonSchema` can be added to derive list if needed for Phase 117 (not required in 115)
- Constant `SCHEMA_VERSION = "ferro-json-ui/v2"` replaces the v1 string

#### Pattern B — `Element` shape (keyword-collision-safe rename)

**Analog:** no single analog — assembled from `view.rs` (schema rename trick) and `component.rs` lines 147–158 (`CardProps` skip rules).

```rust
// Source: CONTEXT.md D-07 verbatim (mirrors serde conventions already in view.rs/component.rs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub props: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Visibility>,
}
```

**Load-bearing attributes (do not omit):**
- `#[serde(rename = "type")] pub type_name: String` — avoids Rust keyword collision while preserving wire format
- `#[serde(default, skip_serializing_if = "Vec::is_empty")]` on `children` — Pitfall 6 in RESEARCH.md; without this every leaf emits `"children": []`
- `Option<Action>` / `Option<Visibility>` — these types derive cleanly today, no change required

#### Pattern C — Consuming builder with `mut self → Self`

**Analog:** `ferro-projections/src/service.rs` lines 82–127

```rust
impl ServiceDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            fields: Vec::new(),
            actions: Vec::new(),
            // ...
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef { /* ... */ });
        self
    }
}
```

**Patterns to replicate for `SpecBuilder` and `ElementBuilder`:**
- `mut self → Self` consuming pattern (CLAUDE.md convention, restated in ferro-projections/CLAUDE.md)
- `impl Into<String>` for string parameters to accept both `&str` and `String`
- Builder stored as an owned struct; terminal method (`build()`) consumes and returns `Result<Spec, SpecError>`

**What must differ from ServiceDef's pattern:**
- ServiceDef is infallible — its constructor returns `Self` directly. `SpecBuilder::build()` **must** return `Result<Spec, SpecError>` because it runs the same structural validation as `from_json()` (D-24). `ServiceDef::new()` is the wrong analog for the fallible terminal; the shape of the intermediate `with_*` calls is identical.
- Also analog-match with v1's `JsonUiView::new().title().component()` from `view.rs` lines 49–96 — same ergonomic chain, fallible terminal is new.

#### Pattern D — `thiserror`-derived error enum with structured variants

**Analog:** `ferro-projections/src/error.rs` (full file, 13 lines)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("service definition error: {0}")]
    Definition(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**What to copy:**
- `use thiserror::Error;` + `#[derive(Error, Debug)]`
- `#[error("…")]` attribute on every variant
- `#[from] serde_json::Error` for the JSON parse pass-through variant

**What must differ (CONTEXT.md D-11):**
- Variants carry **structured** payloads, not formatted strings (paths are `Vec<String>`, not strings):
  - `DuplicateId(String)`
  - `RootMissing(String)`
  - `DanglingChild { element: String, child: String }`
  - `Cycle { path: Vec<String> }` with format string `{}` using `path.join(" -> ")`
  - `DepthExceeded { max: usize, found: usize, path: Vec<String> }`
  - `InvalidId(String)`
  - `Json(#[from] serde_json::Error)` — same pattern as `Serialization` in the analog
- Must add `thiserror = "1.0"` to `ferro-json-ui/Cargo.toml` under `[dependencies]` (not currently listed — verified in RESEARCH.md A2)

---

### `ferro-json-ui/src/component.rs` (REWRITE — self-reduction)

**Analog:** itself (current file at lines 140–320 and 972–1177). The rewrite is a deletion-only operation plus field-type narrowing. No cross-file pattern import needed.

#### Pattern E — Props struct with `Vec<ComponentNode>` fields stripped

**Analog before:** `ferro-json-ui/src/component.rs` lines 147–158 (`CardProps`)

```rust
/// Props for Card component.
// JsonSchema skipped: contains Vec<ComponentNode> — Component has custom Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<ComponentNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
}
```

**Pattern after (Phase 115 rewrite):**
- Delete `children: Vec<ComponentNode>` and `footer: Vec<ComponentNode>` (children live on `Element.children: Vec<String>` now)
- Delete the `// JsonSchema skipped: …` comment
- Add `JsonSchema` to the derive list: `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]`
- Keep `title`, `description`, `max_width` — these are genuine Card-specific props

**Apply the same transformation to** (per RESEARCH.md Props Struct Audit, lines 895–925):
- `CardProps` (line 147) — strip `children`, `footer`
- `FormProps` (line 189) — strip `fields`
- `ModalProps` (line 309) — strip `children`, `footer`
- `Tab` (line 408) — strip `children`
- `GridProps` (line 648) — strip `children`
- `CollapsibleProps` (line 676) — strip `children`
- `FormSectionProps` (line 708) — strip `children`
- `PageHeaderProps` (line 722) — strip `actions: Vec<ComponentNode>`
- `ButtonGroupProps` (line 733) — strip `buttons: Vec<ComponentNode>`
- `KanbanColumnProps` (line 774) — strip `children`
- `SwitchProps` (line 358) — remove stale "JsonSchema skipped" comment, add `JsonSchema` to derive (comment is cargo-cult per RESEARCH.md)
- `DropdownMenuAction` (line 740) — add `JsonSchema` (cargo-cult skip)
- `DropdownMenuProps` (line 749) — add `JsonSchema` (follow-on)
- `DataTableProps` (line 760) — add `JsonSchema` (follow-on)

**Delete entirely:**
- `Component` enum (line 909) + custom `Serialize`/`Deserialize` block (lines 972–1160 per CONTEXT.md)
- `ComponentNode` (line 1168)
- `PluginProps` (line 858) — no longer needed; type_name string IS the plugin indicator

---

### `ferro-json-ui/src/render.rs` (REWRITE as placeholder)

**Analog:** current `render.rs` lines 29–89 (shape of `render_to_html` / `render_to_html_with_plugins` / `RenderResult`)

#### Pattern F — Renderer public surface + `RenderResult` struct

**Analog:**

```rust
// ferro-json-ui/src/render.rs lines 37-89 (excerpts)

pub fn render_to_html(view: &JsonUiView, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
    );
    for node in &view.components {
        html.push_str(&render_node(node, data));
    }
    html.push_str("</div>");
    html
}

pub struct RenderResult {
    pub html: String,
    pub css_head: String,
    pub scripts: String,
}

pub fn render_to_html_with_plugins(view: &JsonUiView, data: &Value) -> RenderResult {
    let html = render_to_html(view, data);
    let plugin_types = collect_plugin_types(view);
    if plugin_types.is_empty() {
        return RenderResult { html, css_head: String::new(), scripts: String::new() };
    }
    // ... plugin asset collection ...
}
```

**What to keep (public API surface must remain callable from `framework/src/json_ui/mod.rs`):**
- Two public functions named `render_spec_to_html` and `render_spec_to_html_with_plugins` (renamed from v1 `render_to_html*` per RESEARCH.md §8).
- `RenderResult { html, css_head, scripts }` struct — unchanged shape; the framework wrapper at `framework/src/json_ui/mod.rs` line 118 expects these three fields.

**What must change (Phase 115 placeholder, per RESEARCH.md §8 lines 829–874):**
- First arg changes: `&JsonUiView` → `&Spec`
- Body changes: no component walk, no plugin collection. Emit pretty JSON inside a `<pre>` block:
  ```rust
  pub fn render_spec_to_html(spec: &Spec, _data: &Value) -> String {
      let pretty = serde_json::to_string_pretty(spec)
          .unwrap_or_else(|e| format!("{{\"error\": \"serialize failed: {e}\"}}"));
      let escaped = html_escape(&pretty);
      format!(
          "<!-- ferro-json-ui v2 render pipeline arrives in Phase 116 -->\n\
           <div class=\"ferro-json-ui\" data-spec-version=\"v2\">\n\
           <pre style=\"font-family:monospace;white-space:pre-wrap;\"><code>{}</code></pre>\n\
           </div>",
          escaped
      )
  }
  ```
- `render_spec_to_html_with_plugins` returns `RenderResult { html, css_head: "", scripts: "" }` — no plugin walk in Phase 115
- Keep the local `html_escape` helper (present in `framework/src/json_ui/mod.rs` lines 239–245 as a pattern reference)
- **Delete** `collect_plugin_types`, `render_node`, every per-Component dispatch arm. Phase 116 re-introduces a real walker.

---

### `ferro-json-ui/src/projection/mod.rs` (UPDATE Output type)

**Analog:** current `projection/mod.rs` lines 107–165 + `ferro-projections/src/render/mod.rs` lines 33–54 (Renderer trait)

#### Pattern G — Renderer trait with associated `Output` type

**Renderer trait analog:** `ferro-projections/src/render/mod.rs` lines 33–54

```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &Self::Context,
    ) -> Result<Self::Output, Error>;
}
```

**Current impl analog:** `ferro-json-ui/src/projection/mod.rs` lines 107–125

```rust
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = serde_json::Value;
    type Context = VisualContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Value, Error> {
        let intent_score = intents.get(ctx.intent_index).ok_or_else(|| {
            Error::Render(format!(/* ... */))
        })?;
        // ... intent dispatch, returns serde_json::Value matching JsonUiView shape
    }
}
```

**What must change (D-20, RESEARCH.md A4):**
- `type Output = serde_json::Value` → `type Output = Spec`
- Return type of `fn render` changes from `Result<Value, Error>` to `Result<Spec, Error>`
- Internal helpers (`render_browse`, `render_focus`, `render_collect`, `render_process`, etc.) currently build `serde_json::Value` trees shaped like `JsonUiView`. They must build `Spec` values instead. Per D-20: **mapping stays naive** — emit one Spec per service with a root element (pick a sensible `type_name` like `"DataTable"` or `"Form"` based on intent) and whatever flat props the current code constructs.
- The rustdoc example at lines 87–106 must update its assertions from `json["$schema"] == "ferro-json-ui/v1"` and `json["components"]` checks to `spec.schema == "ferro-json-ui/v2"` and `spec.root` / `spec.elements` checks.

**What stays identical:**
- `Renderer` trait impl structure (single `fn render` method)
- `VisualContext` (associated `Context` type) — the visual fields `intent_index`, `current_state`, `mode`, `templates` are preserved unchanged
- Intent dispatch `match` block at lines 136–165 — same intents, same mode switching. Only the inner helper return types swap.

---

### `ferro-json-ui/src/lib.rs` (REWRITE re-exports)

**Analog:** current `lib.rs` lines 42–88

#### Pattern H — Module declarations + flat re-exports

**Analog (lines 42–88):**

```rust
pub mod action;
pub mod component;
pub mod config;
pub mod data;
pub mod layout;
pub mod plugin;
pub mod plugins;
pub mod render;
pub mod resolve;
pub mod view;
pub mod visibility;

pub(crate) mod runtime;

pub use action::{Action, ActionOutcome, ConfirmDialog, DialogVariant, HttpMethod, NotifyVariant};
pub use component::{
    ActionCardProps, /* ~40 Props types */ TextElement, TextProps, ToastProps, ToastVariant,
};
pub use config::JsonUiConfig;
pub use layout::{/* ... */};
pub use plugin::{/* ... */};
pub use plugins::{register_built_in_plugins, MapPlugin};
pub use render::{render_to_html, render_to_html_with_plugins, RenderResult};
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
pub use view::{JsonUiView, SCHEMA_VERSION};
pub use visibility::{Visibility, VisibilityCondition, VisibilityOperator};

#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, RenderMode, VisualContext};
```

**What to change:**
- Replace `pub mod view;` with `pub mod spec;`
- Delete `pub use view::{JsonUiView, SCHEMA_VERSION};`
- Add `pub use spec::{Spec, Element, SpecBuilder, ElementBuilder, SpecError, SCHEMA_VERSION, MAX_NESTING_DEPTH};`
- In the `component::{…}` re-export block: **remove** `Component`, `ComponentNode`, `PluginProps` (all three deleted). Keep all remaining Props structs and enums.
- In the `render::{…}` re-export: rename `render_to_html, render_to_html_with_plugins` → `render_spec_to_html, render_spec_to_html_with_plugins`. Keep `RenderResult`.
- Update top-of-file rustdoc example (lines 20–40): replace `JsonUiView::new().title().component(…)` with `Spec::builder().title().element("root", Element::new("Text").prop("content", "Hi")).build().unwrap()`.
- Update the `COMPONENT_CATALOG` const string at lines 100–180: replace `"Vec<ComponentNode>"` occurrences with `"Vec<String>"` (child-ID refs). The broader COMPONENT_CATALOG rewrite is Phase 117; Phase 115 does the minimum to keep MCP tooling compiling.

---

### `framework/src/json_ui/mod.rs` (REWRITE caller)

**Analog:** itself — `framework/src/json_ui/mod.rs` lines 33–232 is the entire public `JsonUi` surface.

#### Pattern I — Renderer facade with `render` / `render_with_config` / `render_with_errors`

**Analog (lines 33–70):**

```rust
use ferro_json_ui::{
    render_layout, render_to_html_with_plugins, resolve_actions, resolve_errors, JsonUiConfig,
    JsonUiView, LayoutContext,
};

pub struct JsonUi;

impl JsonUi {
    fn resolve(view: &JsonUiView) -> JsonUiView {
        let mut resolved = view.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolved
    }

    pub fn render(view: &JsonUiView, data: &serde_json::Value) -> Response {
        Self::render_with_config(view, data, &JsonUiConfig::new())
    }

    pub fn render_with_config(
        view: &JsonUiView,
        data: &serde_json::Value,
        config: &JsonUiConfig,
    ) -> Response {
        let resolved = Self::resolve(view);
        Self::build_response(&resolved, data, config)
    }
    // ...
}
```

**Analog (build_response body, lines 77–143):** — the head-building / layout-dispatching / response-construction logic is the load-bearing part. It reads `view.title`, `view.layout`, serializes view + data into `data-view` / `data-props` attributes, and calls `render_to_html_with_plugins`.

**What must change (D-17):**
- Import swap: `render_to_html_with_plugins`, `JsonUiView`, `resolve_actions`, `resolve_errors` → `render_spec_to_html_with_plugins`, `Spec` (keep `JsonUiConfig`, `LayoutContext` unchanged)
- Method signatures: every `&JsonUiView` → `&Spec` (6 public methods)
- `Self::resolve(view)` body: `resolve_actions(&mut spec, …)` — the walker is part of `resolve.rs` which must also update to walk `spec.elements.values_mut()` instead of the recursive tree (covered by D-28/D-26 scope: `resolve.rs` is marked REWRITE-lite per RESEARCH.md migration blast radius §971). Phase 115 version: iterate `spec.elements.values_mut()` and resolve each `el.action` if present.
- `resolve_with_errors` body at lines 161–167: **delete** the line `resolved.errors = Some(errors.clone());` — `Spec` has no `errors` field (D-06). The errors map flows side-channel; for Phase 115 placeholder it can be embedded in the `data-props` JSON next to data, or simply dropped with a TODO(Phase 116) comment.
- Line 118 `render_to_html_with_plugins(view, data)` → `render_spec_to_html_with_plugins(spec, data)`
- Test fixtures at lines 268–1019: mechanical rewrite from `JsonUiView::new().component(ComponentNode{ … Component::Card(…) … })` to `Spec::builder().element("root", Element::new("Card").prop(…)).build().unwrap()`. Per RESEARCH.md: two plugin-specific tests (`test_plugin_component_renders_in_full_page` and theme tests creating sample views with Leaflet assertions) must be tagged `#[ignore]` with a `// TODO(Phase 116): placeholder renderer does not collect plugin assets` comment — the placeholder emits pretty JSON, not Leaflet CSS/JS.

---

### `ferro-json-ui/tests/round_trip.rs` (NEW integration test)

**Analog:** `ferro-json-ui/src/view.rs` tests module lines 127–412 (inline `#[cfg(test)] mod tests`)

#### Pattern J — Round-trip equality assertion

**Analog (view.rs lines 161–188):**

```rust
#[test]
fn round_trip_build_to_json_from_json() {
    let original = JsonUiView::new()
        .title("Dashboard")
        .layout("app")
        .component(ComponentNode {
            key: "alert".to_string(),
            component: Component::Alert(AlertProps {
                message: "Welcome".to_string(),
                variant: AlertVariant::Success,
                title: None,
            }),
            action: None,
            visibility: None,
        });

    let json = original.to_json().unwrap();
    let parsed = JsonUiView::from_json(&json).unwrap();
    assert_eq!(original, parsed);
}
```

**Analog (view.rs lines 191–262):** fixture-from-string parse + field-by-field assert.

**What to copy:**
- `serde_json::to_string` / `Spec::from_json` round-trip pattern
- `assert_eq!(original, parsed)` — works because `Spec` derives `PartialEq` (same as `JsonUiView` today)
- Multi-field assertions on the parsed result (`assert_eq!(parsed.schema, SCHEMA_VERSION)` etc.)

**What must change (Phase 115 D-29, D-31, RESEARCH.md Test Fixtures):**
- Tests live under `ferro-json-ui/tests/round_trip.rs` (integration test, not inline module) — matches RESEARCH.md §Wave 0 Gaps
- Fixtures loaded from `ferro-json-ui/tests/fixtures/ok/*.json` via a helper:
  ```rust
  fn fixture(path: &str) -> String {
      std::fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
  }
  ```
- Seven fixtures (per D-29 and RESEARCH.md table):
  - `minimal_single_element.json`, `three_level_nested.json`, `with_actions.json`, `with_visibility.json`, `with_plugin_named_type.json`, `with_data_payload.json`, `omitted_optional_fields.json`
- For each fixture: parse → serialize → reparse → assert `spec1 == spec2`.
- Builder parity (D-31): for each fixture, construct the equivalent via `Spec::builder()` and assert `spec_from_json == spec_from_builder`. Reuse the v1 builder test style (`builder_produces_valid_json` at view.rs lines 136–158) adapted to the new API.
- JsonSchema smoke tests (D-32): port view.rs lines 416–479 pattern verbatim — each surviving `*Props` struct gets a `schemars::schema_for!(T)` assertion. Test name template: `test_json_schema_for_<snake_name>_generates`.

---

### `ferro-json-ui/tests/reject.rs` (NEW — no direct analog)

**Analog:** none in ferro-json-ui. Closest pattern for fixture-driven variant-assertion is `ferro-projections/tests/generate_schemas.rs` for fixture loading, and RESEARCH.md §Test Fixtures §reject lines 1027–1055 for the test body structure.

#### Pattern K — Fixture-driven rejection with variant-specific assertion

**Source (RESEARCH.md lines 1036–1055, distilled into copyable form):**

```rust
// tests/reject.rs
use std::fs;
use ferro_json_ui::{Spec, SpecError};

fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
}

#[test]
fn reject_missing_root_gives_specific_variant() {
    let json = fixture("reject/missing_root.json");
    match Spec::from_json(&json) {
        Err(SpecError::RootMissing(id)) => assert_eq!(id, "nope"),
        other => panic!("expected RootMissing, got {other:?}"),
    }
}

#[test]
fn reject_dangling_child_gives_specific_variant() {
    let json = fixture("reject/dangling_child.json");
    match Spec::from_json(&json) {
        Err(SpecError::DanglingChild { element, child }) => {
            assert_eq!(element, "root");
            assert_eq!(child, "ghost");
        }
        other => panic!("expected DanglingChild, got {other:?}"),
    }
}
// ... one test per reject fixture
```

**Eleven fixtures (per D-30 and RESEARCH.md table lines 1011–1023):**

| Fixture | Expected `SpecError` variant |
|---------|------------------------------|
| `missing_root.json` | `RootMissing("nope")` |
| `dangling_child.json` | `DanglingChild { element, child }` |
| `simple_cycle.json` | `Cycle { path }` with ≥ 3 elements |
| `self_cycle.json` | `Cycle { path: ["A", "A"] }` |
| `four_level_nesting.json` | `DepthExceeded { max: 3, found, path }` |
| `invalid_id_space.json` | `InvalidId("user form")` |
| `invalid_id_empty.json` | `InvalidId("")` |
| `invalid_id_digit_start.json` | `InvalidId("1form")` |
| `invalid_id_too_long.json` | `InvalidId(_)` (129-char key) |
| `invalid_child_ref_format.json` | `InvalidId("user form")` (ref, not key) |
| `duplicate_id.json` | `DuplicateId("a")` (raw JSON with repeated key) |

**Pattern choices:**
- **Table-driven not proptest:** RESEARCH.md §Validation Architecture settles this — 11 reject fixtures × variant match = 11 assertions covers the structural contract completely.
- **No helper framework:** direct `match` + `panic!` on wrong variant. Matches the idiomatic Rust test style already used across the workspace.
- **JSON files in `tests/fixtures/reject/`:** hand-written so the test data is human-auditable; path-relative loading matches `ferro-projections/tests/generate_schemas.rs` pattern (lines 14–21 of that file use `CARGO_MANIFEST_DIR`; since tests/ run from crate root, `tests/fixtures/…` works unprefixed).

---

## Shared Patterns

### S-1: `#[serde(rename = "$key")]` convention

**Source:** `ferro-json-ui/src/view.rs` line 36 (`#[serde(rename = "$schema")]`)

**Apply to:**
- `Spec.schema` (rename `$schema`) — same as v1
- `Element.type_name` (rename `type`) — keyword collision

```rust
#[serde(rename = "$schema")]
pub schema: String,

#[serde(rename = "type")]
pub type_name: String,
```

### S-2: `skip_serializing_if` on optional / defaultable fields

**Source:** every struct in `ferro-json-ui/src/component.rs` uses this. Representative example `CardProps` (lines 147–158):

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub description: Option<String>,
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub children: Vec<ComponentNode>,
```

**Apply to:**
- Every `Option<T>` field on `Spec` and `Element` → `skip_serializing_if = "Option::is_none"`
- `Element.children: Vec<String>` → `skip_serializing_if = "Vec::is_empty"`
- `Spec.data: serde_json::Value` / `Element.props: serde_json::Value` → `skip_serializing_if = "serde_json::Value::is_null"` (Pitfall 6 in RESEARCH.md)
- `#[serde(default)]` is the partner — always pair them so missing-from-JSON parses and `skip_serializing_if` output match round-trip.

### S-3: Consuming builder convention

**Source:** CLAUDE.md ("Builder pattern: `with_*` methods taking `mut self` → `Self` (consuming)") + `ferro-projections/src/service.rs` lines 99–107

**Apply to:** `SpecBuilder`, `ElementBuilder`, every `with_*` / `.title()` / `.prop()` / `.child()` / `.action()` / `.visible()` method in the new API.

```rust
pub fn display_name(mut self, name: impl Into<String>) -> Self {
    self.display_name = Some(name.into());
    self
}
```

### S-4: `thiserror` error enum with `#[from]` for wrapped crate errors

**Source:** `ferro-projections/src/error.rs` line 12 (`Serialization(#[from] serde_json::Error)`)

**Apply to:** `SpecError::Json(#[from] serde_json::Error)` — the one variant that chains an underlying error type. All other variants carry structured fields (`path: Vec<String>`, etc.), not wrapped errors.

### S-5: `impl Into<String>` on builder method string parameters

**Source:** `ferro-projections/src/service.rs` lines 84, 99, 105 (every `impl Into<String>` on `name` / description params)

**Apply to:** every `SpecBuilder` method that takes a label, ID, or content string. Accepts both `&str` and `String` for author ergonomics.

### S-6: Integration test fixture loader helper

**Source:** RESEARCH.md §Test Fixtures §lines 1028–1035

```rust
fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
}
```

**Apply to:** `tests/round_trip.rs` and `tests/reject.rs`. Path-relative loading works because `cargo test` runs from the crate root (`ferro-json-ui/`), so `tests/fixtures/ok/foo.json` resolves correctly without `CARGO_MANIFEST_DIR` gymnastics.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-json-ui/tests/reject.rs` | integration test | fixture-driven validation failure | No precedent in ferro-json-ui or elsewhere in the workspace for "load fixture, assert specific error enum variant". Pattern assembled directly from RESEARCH.md §Test Fixtures. The shape is mechanical: `match Spec::from_json(fixture) { Err(Variant{…}) => asserts, other => panic!() }`. |

## Metadata

**Analog search scope:**
- `ferro-json-ui/src/` (full crate)
- `ferro-projections/src/` (builder + error + trait patterns)
- `framework/src/json_ui/mod.rs` (caller)
- `framework/tests/`, `ferro-projections/tests/` (integration test layout)
- `ferro-projections/CLAUDE.md` (crate boundary rules)

**Files scanned:** 12 (view.rs, lib.rs, render.rs, projection/mod.rs, component.rs, resolve.rs, action.rs, Cargo.toml; service.rs, render/mod.rs, error.rs; framework/json_ui/mod.rs)

**Pattern extraction date:** 2026-04-18

**Key patterns identified:**
- Every new piece of Phase 115 surface has at least one concrete Rust analog in the existing workspace — no pattern is invented from scratch.
- The biggest self-reduction target is `component.rs`: mechanical deletion of one enum + one wrapper + ~200 LoC of custom ser/de, plus field-type narrowing on 10 Props structs. No external analog needed.
- `SpecBuilder` blends three analogs: `JsonUiView::new().title().component()` ergonomics (v1 `view.rs`), `mut self → Self` style (`ServiceDef` in `service.rs`), fallible terminal (`build() -> Result<Spec, SpecError>`) is new but mechanically trivial.
- Placeholder renderer (`render.rs`) replaces an 8000-line tree walker with ~40 lines of pretty-JSON-in-`<pre>`. The only pattern requirement is keeping the `RenderResult { html, css_head, scripts }` shape so `framework/src/json_ui/mod.rs` line 118 keeps compiling.

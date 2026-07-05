# Phase 254: Props Contracts + Touch Foundation + Design Rules - Pattern Map

**Mapped:** 2026-07-05
**Files analyzed:** 7 (6 Rust source, 1 Markdown doc)
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/render/classes.rs` | utility | transform | same file — `INTERACTIVE_BASE` constant + `interactive_base_is_motion_fast_plus_focus_ring` test | exact |
| `ferro-json-ui/src/component.rs` — `ProductTileProps` extension | model | transform | `GridProps.spans` (`:904`) for serde conventions; `ProductTileProps` itself (`:1345`) for field order rule | exact |
| `ferro-json-ui/src/component.rs` — 5 new `*Props` structs | model | transform | `EmptyStateProps` (`:929`) + `CollapsibleProps` (`:921`) for struct shape; `schema_smoke_tests::assert_schema_nonempty_object` (`:1395`) for smoke test pattern | exact |
| `ferro-json-ui/src/component.rs` — `GridProps.row_weights` | model | transform | `GridProps.spans` (`:904`) | exact |
| `ferro-json-ui/src/render/atoms.rs` — `render_product_tile` | utility | transform | same function (`:1357`) | exact |
| `ferro-json-ui/src/design/rules.rs` — 4 new rules + 12 fixtures | middleware | transform | `check_page_header` (`:94`) for internal-gate pattern; `check_prefer_data_table` (`:130`) for element-scan pattern; test fixtures from `tests` module (`:413`) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` — `RULE_COMPONENTS` | config | request-response | existing `RULE_COMPONENTS` static (`:81`) | exact |
| `docs/src/design-system/patterns.md` — 4 new rule sections | config/doc | N/A | existing `## \`page-header\`` section (`:9-54`) | exact |
| `ferro-json-ui/assets/input.css` — `@utility pos-tap-highlight` | config | transform | `@utility duration-fast` (`:94`) | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/render/classes.rs` (utility, transform)

**Analog:** Same file — existing constants block and `#[cfg(test)] mod tests`.

**Existing constants pattern** (lines 9-38):
```rust
pub(crate) const FOCUS_RING: &str =
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";

pub(crate) const MOTION_FAST: &str = "transition-colors duration-fast ease-base";

pub(crate) const INTERACTIVE_BASE: &str = concat!(
    "transition-colors duration-fast ease-base ",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
);
```

**Rustdoc convention** (lines 9-38 — every constant has a doc comment stating token sourcing and when to use it):
```rust
/// Fast-tier interactive base = fast motion + focus ring, for buttons/links/nav.
pub(crate) const INTERACTIVE_BASE: &str = concat!(...);
```

**Composition test pattern** (lines 47-49):
```rust
#[test]
fn interactive_base_is_motion_fast_plus_focus_ring() {
    assert_eq!(INTERACTIVE_BASE, format!("{MOTION_FAST} {FOCUS_RING}"));
}
```

**New drift-guard test pattern** (from RESEARCH.md Finding 5 — mirrors `design/mod.rs:326`):
```rust
#[test]
fn pos_render_functions_use_constants_not_literals() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let render_dir = std::path::Path::new(&manifest_dir).join("src/render");
    let guarded_literals = [
        "touch-manipulation",
        "min-h-[44px] min-w-[44px]",
    ];
    for entry in std::fs::read_dir(&render_dir).expect("src/render readable") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        if filename == "classes.rs" { continue; } // source, not consumer
        let source = std::fs::read_to_string(&path).unwrap();
        for literal in &guarded_literals {
            assert!(
                !source.contains(literal),
                "{filename}: raw POS literal {literal:?} — import from render::classes instead"
            );
        }
    }
}
```

**Token-compliance test pattern** (lines 52-58):
```rust
#[test]
fn fragments_use_token_utilities() {
    assert!(FOCUS_RING.contains("focus-visible:ring-ring"));
    assert!(MOTION_FAST.contains("duration-fast") && MOTION_FAST.contains("ease-base"));
}
```

**Five new constants to add** (immediately after the existing INTERACTIVE_BASE block):
```rust
/// Enables touch-manipulation scroll optimisation on POS tap targets.
/// Full class literal: scanner-visible to Tailwind @source.
pub(crate) const POS_TOUCH_ACTION: &str = "touch-manipulation";

/// Minimum 44px hit target for POS interactive elements (WCAG 2.5.5).
pub(crate) const POS_HIT_TARGET_MIN: &str = "min-h-[44px] min-w-[44px]";

/// Press-state feedback for POS tap surfaces: scale-down + border tint.
/// Uses semantic tokens only (active:bg-border = --color-border).
pub(crate) const POS_PRESS_ACTIVE: &str = "active:scale-95 active:bg-border";

/// Prevents scroll bounce from escaping a POS pane boundary.
pub(crate) const POS_OVERSCROLL_CONTAIN: &str = "overscroll-contain";

/// Removes the default iOS tap highlight rectangle on POS touch targets.
/// Defined via @utility in input.css (Path B — guaranteed CSS generation).
pub(crate) const POS_TAP_HIGHLIGHT: &str = "pos-tap-highlight";
```

---

### `ferro-json-ui/src/component.rs` — `ProductTileProps` extension (model, transform)

**Analog:** `ProductTileProps` itself (lines 1340-1352) for field order; `GridProps.spans` (lines 903-905) for serde conventions.

**Current struct** (lines 1340-1352 — new optional fields go AFTER these 5):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProductTileProps {
    pub product_id: String,
    pub name: String,
    pub price: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_quantity: Option<u32>,
}
```

**Serde convention for Vec<String> optional field** (mirrors `spans` at line 904):
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub spans: Vec<u8>,
```

**Serde convention for Option<String> optional field** (mirrors `md_columns` at line 887):
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub md_columns: Option<u8>,
```

**Backward-compat round-trip test pattern** (new test — add to `schema_smoke_tests` module or a separate `tests` module in component.rs):
```rust
#[test]
fn product_tile_legacy_json_round_trips_unchanged() {
    let legacy = r#"{"product_id":"p1","name":"Widget","price":"€10,00","field":"qty_p1"}"#;
    let props: ProductTileProps = serde_json::from_str(legacy).unwrap();
    assert!(props.categories.is_empty());
    assert!(props.image_url.is_none());
    assert!(props.color.is_none());
    assert!(props.stock_badge.is_none());
    let re_serialized = serde_json::to_string(&props).unwrap();
    assert!(!re_serialized.contains("categories"));
    assert!(!re_serialized.contains("image_url"));
    assert!(!re_serialized.contains("color"));
    assert!(!re_serialized.contains("stock_badge"));
}
```

---

### `ferro-json-ui/src/component.rs` — `GridProps.row_weights` addition (model, transform)

**Analog:** `GridProps.spans` field (lines 900-905) — exact mirror.

**Spans field** (lines 900-905 — `row_weights` follows this pattern identically):
```rust
/// Per-child column spans, aligned positionally with `children` (missing
/// entries default to 1). A child with span N occupies N tracks — e.g.
/// `columns: 1, md_columns: 3, spans: [2, 1]` renders a 2/3 + 1/3 row.
/// Supported spans: 2–4 on the base grid, 2–3 at the `md` breakpoint.
/// Ignored in `scrollable` mode.
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub spans: Vec<u8>,
```

**Schema smoke test for GridProps** (line 1565 referenced in RESEARCH.md — add `row_weights` round-trip assertion alongside):
```rust
#[test]
fn schema_for_grid_props_generates() {
    assert_schema_nonempty_object::<GridProps>("GridProps");
}
```

---

### `ferro-json-ui/src/component.rs` — 5 new `*Props` structs (model, transform)

**Analog:** `EmptyStateProps` (lines 929-938) and `CollapsibleProps` (lines 921-926) for derive set, rustdoc style, and struct shape.

**Derive set** (same as all other Props in the file):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
```

**Enum with derive set** (for `NumpadMode` — mirrors `FormSectionLayout` at lines 940-947):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FormSectionLayout {
    #[default]
    Stacked,
    TwoColumn,
}
```

**Schema smoke test pattern** (lines 1395-1411 — one test per new struct):
```rust
fn assert_schema_nonempty_object<T: schemars::JsonSchema>(type_label: &str) {
    let schema = schemars::schema_for!(T);
    let value = serde_json::to_value(&schema).expect("schema serializes to JSON");
    assert!(value.is_object(), "{type_label}: schema must be a JSON object");
    let props = value
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|o| !o.is_empty())
        .unwrap_or(false);
    assert!(props, "{type_label}: schema must have a non-empty `properties` field");
}

#[test]
fn schema_for_product_grid_props_generates() {
    assert_schema_nonempty_object::<ProductGridProps>("ProductGridProps");
}
// ... repeat for CartPanelProps, CategoryNavProps, QuantityStepperProps, NumpadProps
```

---

### `ferro-json-ui/src/render/atoms.rs` — `render_product_tile` modification (utility, transform)

**Analog:** The existing function itself (lines 1357-1390).

**Current function signature and props decode** (lines 1357-1370 — unchanged):
```rust
pub(crate) fn render_product_tile(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: ProductTileProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("ProductTile", e),
    };
    let name = html_escape(&props.name);
    let price = html_escape(&props.price);
    let field = html_escape(&props.field);
    let qty = props.default_quantity.unwrap_or(0);
```

**Constant migration** — replace the three inline literal occurrences at lines 1373, 1380, 1384:

| Line | Current | Replacement |
|------|---------|-------------|
| 1373 | `"... touch-manipulation"` in the outer div class | replace `touch-manipulation` with `{POS_TOUCH_ACTION}` in the format! string |
| 1380 | `"min-h-[44px] min-w-[44px] ..."` in dec-button class | `{POS_HIT_TARGET_MIN} ...` |
| 1384 | `"min-h-[44px] min-w-[44px] ..."` in inc-button class | `{POS_HIT_TARGET_MIN} ...` |

**`data-product-categories` conditional emission pattern** (add before the format! call):
```rust
let categories_attr = if props.categories.is_empty() {
    String::new()
} else {
    format!(
        " data-product-categories=\"{}\"",
        html_escape(&props.categories.join(" "))
    )
};
```

Then add `{categories_attr}` to the outer div in the format! string.

**Render equality test for backward-compat** (add in a `#[cfg(test)]` block or in the spec):
```rust
// Legacy spec without new fields must produce byte-identical HTML.
// Verify by rendering a ProductTileProps with empty categories/None options.
```

---

### `ferro-json-ui/src/design/rules.rs` — 4 new rules + 12 fixtures (middleware, transform)

**Analog:** `check_page_header` (lines 94-128) for internal-gate + all-intents pattern; test fixtures from `tests` module (lines 413+).

**Rule entry shape** (lines 7-13 — copy exactly, change fields):
```rust
DesignRule {
    id: "page-header",
    title: "Dashboard pages start with a PageHeader",
    rationale: "A PageHeader gives every app page a consistent title, breadcrumb, and action-button slot.",
    intents: &[], // all intents — layout gate is inside check_page_header
    check: check_page_header,
},
```

**Internal-gate pattern** (lines 90-97 — copy `is_app_shell_layout` for POS type presence):
```rust
fn is_app_shell_layout(spec: &Spec) -> bool {
    matches!(spec.layout.as_deref(), Some("dashboard") | Some("app"))
}

fn check_page_header(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !is_app_shell_layout(spec) {
        return vec![]; // gate: early exit when condition not met
    }
    // ... check logic
}
```

**Element-presence gate pattern** (for `pos-fill-viewport` — mirrors `check_page_header` early-return):
```rust
fn check_pos_fill_viewport(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    const POS_TRIGGER_TYPES: &[&str] = &["ProductGrid", "CartPanel", "Numpad"];
    let has_pos = spec.elements.values()
        .any(|el| POS_TRIGGER_TYPES.contains(&el.type_name.as_str()));
    if !has_pos || spec.fill_viewport {
        return vec![];
    }
    vec![Finding { ... }]
}
```

**Single-finding return pattern** (lines 104-111 — for rules that emit at most one finding):
```rust
vec![Finding {
    rule: "page-header",
    element_id: None,
    severity: Severity::Warning,
    message: "Dashboard-family layout has no PageHeader element.".into(),
    suggestion: "Add a PageHeader element (with a `title` prop) as the first child of root.".into(),
}]
```

**`fill-viewport-layout-unknown` supported-layout set** — use `is_app_shell_layout` helper exactly (RESEARCH.md Finding 1):
```rust
// Supported set: "app" and "dashboard" only. is_app_shell_layout() already encodes this.
fn check_fill_viewport_layout_unknown(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !spec.fill_viewport {
        return vec![];
    }
    if is_app_shell_layout(spec) {
        return vec![];
    }
    vec![Finding {
        rule: "fill-viewport-layout-unknown",
        element_id: None,
        severity: Severity::Warning,
        message: "fill_viewport is set but the layout is not in the supported set (\"app\", \"dashboard\").".into(),
        suggestion: "Use layout: \"app\" or \"dashboard\"; fill_viewport silently degrades to whole-page scroll on other layouts.".into(),
    }]
}
```

**Test helper** (lines 417-421 — copy into the Phase 254 tests block):
```rust
fn findings_for(all: Vec<Finding>, rule: &str) -> Vec<Finding> {
    all.into_iter().filter(|f| f.rule == rule).collect()
}
```

**Three-fixture test structure** (violating/conforming/data-bound — from 252-PATTERNS.md mandate + D-12):

*Violating fixture* (mirrors `page_header_violating_dashboard_no_header` at line 426):
```rust
#[test]
fn pos_fill_viewport_violating_product_grid_no_fill_viewport() {
    let spec = Spec::from_json(r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "r",
        "elements": {"r": {"type": "ProductGrid"}},
        "design": {}
    }"#).unwrap();
    let findings = findings_for(lint(&spec), "pos-fill-viewport");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Warning);
}
```

*Conforming fixture* (mirrors `page_header_conforming_dashboard_with_titled_header` at line 447):
```rust
#[test]
fn pos_fill_viewport_conforming_fill_viewport_set() {
    let spec = Spec::from_json(r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "r",
        "fill_viewport": true,
        "elements": {"r": {"type": "ProductGrid"}},
        "design": {}
    }"#).unwrap();
    let findings = findings_for(lint(&spec), "pos-fill-viewport");
    assert!(findings.is_empty());
}
```

*Data-bound no-misfire fixture* (mirrors `list_empty_state_conforming_data_bound_empty_message` at line 574 — mandatory per D-12):
```rust
#[test]
fn pos_fill_viewport_data_bound_no_misfire() {
    // A spec with $data bindings but no POS component type names → 0 findings.
    let spec = Spec::from_json(r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "r",
        "fill_viewport": false,
        "elements": {"r": {"type": "DataTable", "props": {"data_path": {"$data": "/products"}}}},
        "design": {}
    }"#).unwrap();
    let findings = findings_for(lint(&spec), "pos-fill-viewport");
    assert!(findings.is_empty(), "spec without POS type names must not misfire: {findings:#?}");
}
```

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` — `RULE_COMPONENTS` additions (config, request-response)

**Analog:** Existing `RULE_COMPONENTS` static (lines 81-96).

**Existing entries** (lines 81-96):
```rust
static RULE_COMPONENTS: &[(&str, &[&str])] = &[
    ("page-header", &["PageHeader"]),
    ("prefer-data-table", &["Table", "DataTable"]),
    ("list-empty-state", &["DataTable", "MediaCardGrid", "EmptyState"]),
    ("row-actions-grouped", &["ActionGroup", "Button"]),
    ("breadcrumb-on-subpages", &["Breadcrumb", "PageHeader"]),
    ("process-kanban", &["KanbanBoard"]),
    ("card-actions-in-menu", &["KanbanBoard", "ActionGroup"]),
    ("create-separate-page", &["Modal", "Form"]),
    ("form-default-values", &["Form", "Input", "Select"]),
    ("destructive-confirmation", &["Button"]),
    ("prefer-components", &["RawHtml"]),
];
```

**Four new entries to append** (RESEARCH.md Finding 7):
```rust
    ("pos-fill-viewport", &["Grid"]),          // register-root concern; Grid is the root
    ("pos-grid-fill", &["Grid"]),              // directly about Grid.fill property
    ("pos-cart-present", &["Grid"]),           // register composition concern
    ("fill-viewport-layout-unknown", &[]),    // layout concern; no component; passes Direction 3
```

**Direction 3 empty-slice** (verified from lines 757-763): `&[]` passes Direction 3 because the inner loop `for &c in *comps` does not execute — semantically correct for `fill-viewport-layout-unknown`.

---

### `docs/src/design-system/patterns.md` — 4 new rule sections (config/doc)

**Analog:** Existing `## \`page-header\`` section (lines 9-54).

**Exact section format required** (drift guard reads `## \`rule-id\`` prefix — lines 347-352 of `design/mod.rs`):
```markdown
## `pos-fill-viewport`

**Title:** POS register pages must fill the viewport

**Rationale:** A ProductGrid or CartPanel outside a fill_viewport spec causes silent
whole-page scroll, breaking the kiosk feel.

**Intents:** all (applies to any spec containing POS component types)

### Conforming example

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "r",
  "fill_viewport": true,
  "layout": "app",
  "elements": {
    "r": { "type": "Grid", "props": { "fill": true } }
  }
}
```

### Violating example

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "r",
  "elements": {
    "r": { "type": "ProductGrid" }
  }
}
```

### How to allow

Add `"allow": ["pos-fill-viewport"]` to the `design` object when the spec is
intentionally not fill-mode (e.g., a product browse page, not a register):

```json
{ "design": { "allow": ["pos-fill-viewport"] } }
```
```

The four section headers required (exact strings the drift-guard checks):
- `## \`pos-fill-viewport\``
- `## \`pos-grid-fill\``
- `## \`pos-cart-present\``
- `## \`fill-viewport-layout-unknown\``

---

### `ferro-json-ui/assets/input.css` — `@utility pos-tap-highlight` (config, transform)

**Analog:** `@utility duration-fast` at line 94.

**Existing @utility pattern** (lines 94-96):
```css
@utility duration-fast {
  transition-duration: var(--motion-duration-fast, 120ms);
}
```

**New @utility to add** (Path B — zero CSS generation risk, per RESEARCH.md Finding 4):
```css
@utility pos-tap-highlight {
  -webkit-tap-highlight-color: transparent;
}
```

Add after the `@utility duration-slow` block (line 102), before the `@media (prefers-reduced-motion)` block. This makes `POS_TAP_HIGHLIGHT = "pos-tap-highlight"` a guaranteed generated utility — no `@source inline()` safelist needed, full literal in Rust source.

---

## Shared Patterns

### Serde optional-field conventions
**Source:** `ferro-json-ui/src/component.rs:887-913` (multiple fields across `GridProps`)
**Apply to:** All new optional fields in `ProductTileProps` extension, all 5 new `*Props` structs, `GridProps.row_weights`
```rust
// Vec optional — omit when empty (never serializes as [])
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub field_name: Vec<T>,

// Option optional — omit when None
#[serde(default, skip_serializing_if = "Option::is_none")]
pub field_name: Option<T>,
```

### Struct derive set
**Source:** `ferro-json-ui/src/component.rs:881` (`GridProps`) and every other Props struct
**Apply to:** All 5 new `*Props` structs; `NumpadMode` enum gets `Eq` added
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MyProps { ... }
```

### Drift-guard test using `CARGO_MANIFEST_DIR` + `read_dir`/`read_to_string`
**Source:** `ferro-json-ui/src/design/mod.rs:326-357`
**Apply to:** The new POS constants drift-guard test in `render/classes.rs`
```rust
let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
let path = std::path::Path::new(&manifest_dir).join("src/render");
// or: .join("../docs/src/design-system/patterns.md")
```

### Design rule `intents: &[]` + internal presence gate
**Source:** `ferro-json-ui/src/design/rules.rs:11` (`page-header`) + `rules.rs:94` (`is_app_shell_layout`)
**Apply to:** All 4 new POS rules (D-11)
```rust
intents: &[], // all-intents; internal gate is inside the check function
```

### Three-fixture test structure (violating / conforming / data-bound)
**Source:** `ferro-json-ui/src/design/rules.rs:574-592` (`list_empty_state_conforming_data_bound_empty_message` — the mandatory data-bound no-misfire fixture from 252-PATTERNS.md)
**Apply to:** All 4 new POS rules (D-12 requires this fixture class per rule)

The data-bound fixture must use `$data.*`-scoped props on a spec that lacks POS type names, asserting the rule does not misfire on data-binding patterns.

---

## No Analog Found

No files in this phase lack a close codebase match. All patterns are extensions of existing modules.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/render/`, `ferro-json-ui/src/component.rs`, `ferro-json-ui/src/design/`, `ferro-mcp/src/tools/`, `ferro-json-ui/assets/`, `docs/src/design-system/`
**Files scanned:** 9 source files read directly
**Pattern extraction date:** 2026-07-05

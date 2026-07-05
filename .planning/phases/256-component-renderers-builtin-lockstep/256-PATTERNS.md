# Phase 256: Component Renderers + BUILTIN Lockstep — Pattern Map

**Mapped:** 2026-07-06
**Files analyzed:** 10
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/render/atoms.rs` | renderer (leaf) | request-response (SSR) | self: `render_tile`, `render_action_card`, `render_empty_state` | exact |
| `ferro-json-ui/src/render/containers.rs` | renderer (container) | request-response (SSR) | self: `render_grid`, `render_collapsible` | exact |
| `ferro-json-ui/src/render/mod.rs` | dispatch registry | config | self: existing `BUILTIN_TYPES` + dispatch match | exact |
| `ferro-json-ui/src/catalog.rs` | component catalog | config | self: existing `BUILTIN_SPECS` + count guard | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | MCP mirror catalog | config | self: existing count guard + `RULE_COMPONENTS` | exact |
| `ferro-json-ui/src/runtime/selection.rs` | browser runtime (new) | event-driven | `ferro-json-ui/src/runtime/filters.rs`, `tiles.rs` | role-match |
| `ferro-json-ui/src/runtime/mod.rs` | bundle assembly | config | self: existing LazyLock + dispatcher + 2 drift tests | exact |
| `ferro-json-ui/src/runtime/tiles.rs` | browser runtime (extend) | event-driven | self: `initQtyButton` | exact |
| `ferro-json-ui/src/component.rs` | Props structs | config | self: `TileProps`, `SelectionPanelProps`, etc. | exact |
| `docs/src/json-ui/components.md` | documentation | — | self: `## Component rename migration (v16.6)` table | exact |

---

## Pattern Assignments

---

### `ferro-json-ui/src/render/atoms.rs` — render_tile redesign + 3 new atoms

**Analog:** `render_tile` (lines 1363–1413), `render_action_card` (lines 1303–1358), `render_empty_state` (lines 659–703)

**Imports pattern** (lines 9–25) — add new Props to the `use crate::component::` import list:
```rust
use crate::component::{
    // existing entries...
    FilterTabsProps, NumpadProps, NumpadMode, QuantityStepperProps,
    // (TileGridProps + SelectionPanelProps go in containers.rs)
};
use super::classes::{
    DISABLED_BASE, FOCUS_RING, HIT_TARGET_MIN, HIT_TARGET_NUMPAD,
    INTERACTIVE_BASE, MOTION_BASE, MOTION_FAST,
    PRESS_ACTIVE, TAP_HIGHLIGHT, TOAST_TONE_DESTRUCTIVE,
    TOAST_TONE_NEUTRAL, TOAST_TONE_SUCCESS, TOAST_TONE_WARNING,
    TOUCH_ACTION,
};
```
Note: `HIT_TARGET_NUMPAD`, `PRESS_ACTIVE`, `TAP_HIGHLIGHT` are not currently imported in atoms.rs — they must be added. They are already declared in `render/classes.rs` lines 47–58.

**Render function signature pattern** (uniform across all atoms, e.g. line 1365):
```rust
pub(crate) fn render_tile(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: TileProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Tile", e),
    };
    // ... html_escape every prop before interpolation
    let name = html_escape(&props.name);
    // ... build HTML string
}
```
For new atoms that do NOT recurse children, use `_spec`, `_data`, `_depth` (prefixed underscores). Containers that recurse via `render_element` remove the underscores.

**render_tile redesign — tap-to-add pattern** (replaces lines 1395–1412):
```rust
// Outer wrapper: carries data-filter-text + data-filter-tokens + data-unit-price
// The whole inner button is the tap surface; hidden input is a sibling of the button
// (inputs inside <button> are invalid HTML — D-01 note)
format!(
    "<div class=\"... {TOUCH_ACTION}\" \
     data-filter-text=\"{name}\"\
     {categories_attr}{unit_price_attr}>\
     <button type=\"button\" data-qty-inc=\"{field}\" \
       class=\"{HIT_TARGET_MIN} {TOUCH_ACTION} {PRESS_ACTIVE} {TAP_HIGHLIGHT} ...\"\
       aria-label=\"Add {name}\">\
       <!-- image / name / price / stock_badge markup -->\
     </button>\
     <input type=\"hidden\" name=\"{field}\" data-qty-input=\"{field}\" value=\"{qty}\">\
     </div>"
)
```
Key: `data-filter-text` and `data-unit-price` on the outer `<div>`; `data-qty-inc` on the `<button>`; hidden input adjacent to the button (sibling, not child). `initQtyButton` finds the input document-wide by `[data-qty-input="{field}"]` — no scoping needed.

**Tone exhaustive match pattern** (copy from `render_action_card` lines 1317–1322):
```rust
let border_class = match props.tone {
    Tone::Neutral     => "border-l-border",
    Tone::Success     => "border-l-success",
    Tone::Warning     => "border-l-warning",
    Tone::Destructive => "border-l-destructive",
};
```
For tile `color: Option<Tone>` — same exhaustive match, different class vocabulary (D-03). If `Tone` remains `Option<String>` in component.rs rather than `Option<Tone>`, parse the string to a Tone or drop the color rendering rather than using `format!("bg-{}", color)` (SC-3 absolute prohibition).

**EmptyState markup vocabulary** (lines 675–703, for SelectionPanel inline empty state):
```rust
// Full bordered card variant (use inside panel as a lightweight div with data-selection-empty):
"<div class=\"rounded-lg border border-border bg-card min-h-40 py-8 px-6 flex items-center justify-center\">\
 <div class=\"text-center max-w-md\">"
// Title:       class="text-base font-semibold text-text mb-2"
// Description: class="text-sm text-text-muted"
```
The panel's EmptyState needs `data-selection-empty` so the runtime can toggle it. Wrap the inner structure and add the attribute on the outer div.

**render_quantity_stepper pattern** — modeled on the old `render_tile` stepper section (lines 1402–1410), but self-contained: dec button + display span + inc button + own hidden input. All three `data-qty-*` attributes, ≥44px buttons. The stepper emits `data-qty-min/-max/-step` on the inc/dec buttons when set (D-22 hook for runtime extension):
```rust
let min_attr  = props.min.map(|v| format!(" data-qty-min=\"{}\"", v)).unwrap_or_default();
let max_attr  = props.max.map(|v| format!(" data-qty-max=\"{}\"", v)).unwrap_or_default();
let step_attr = props.step.map(|v| format!(" data-qty-step=\"{}\"", v)).unwrap_or_default();
// Emit on both inc and dec buttons
```

**render_numpad pattern** — emits the exact Phase 255 attribute contract (D-23). Container `data-numpad data-numpad-target="{field}"`; optional `data-numpad-mode="price"` when `props.mode == NumpadMode::Price`; display `data-numpad-display`; 3×4 key grid with `data-numpad-key`; hidden input `name="{field}" data-numpad-input="{field}"` adjacent:
```rust
let mode_attr = match props.mode {
    NumpadMode::Price    => " data-numpad-mode=\"price\"",
    NumpadMode::Quantity => "",
};
// keys: 1..=9, clear, 0, backspace — all as data-numpad-key="..."
// Key classes: {HIT_TARGET_NUMPAD} {TOUCH_ACTION} {PRESS_ACTIVE} {TAP_HIGHLIGHT}
```

**render_filter_tabs pattern** — emits a tab strip inside a `data-filter-scope` wrapper (when standalone). Active tab initial state: `border-primary text-primary font-semibold aria-selected="true"`. Inactive initial state: `border-transparent text-text-muted hover:text-text aria-selected="false"`. These exact class sets are what `updateFilterTabClasses` in `runtime/filters.rs` lines 75–84 toggles — they MUST match. `all_label` default is "All" (correcting the 254 rustdoc "Tutte" — D-28):
```rust
let all_label = props.all_label.as_deref().unwrap_or("All");
```

**Test pattern** (atoms.rs tests module, lines 1465–1478):
```rust
fn spec_with_root(el: crate::spec::ElementBuilder) -> Spec {
    Spec::builder().element("root", el).build().expect("trivial spec builds")
}

#[test]
fn tile_tap_to_add_emits_qty_inc_button() {
    let spec = spec_with_root(
        Element::new("Tile")
            .prop("item_id", "p1").prop("name", "Coffee").prop("price", "€2.00").prop("field", "p1"),
    );
    let el = spec.elements.get("root").unwrap();
    let html = render_tile(el, &spec, &json!({}), 1);
    assert!(html.contains("data-qty-inc=\"p1\""), "got: {html}");
    assert!(html.contains("data-qty-input=\"p1\""), "got: {html}");
    assert!(!html.contains("data-qty-display"), "tap-to-add: no on-tile qty display; got: {html}");
    assert!(!html.contains("data-qty-dec"), "tap-to-add: no dec button on tile; got: {html}");
    assert!(html.contains("Add Coffee"), "neutral English aria-label; got: {html}");
}
```

---

### `ferro-json-ui/src/render/containers.rs` — render_tile_grid, render_selection_panel, render_grid extension

**Analog:** `render_grid` (lines 798–870), `render_collapsible` (lines 883–), `render_card` / `render_page_header` for multi-slot patterns

**Imports pattern** (lines 14–26) — add new Props to the `use crate::component::` import:
```rust
use crate::component::{
    // existing entries...
    SelectionPanelProps, TileGridProps,
    // FilterTabsProps may also be needed if render_filter_tab_strip is a crate-level helper
};
use super::classes::{
    DISABLED_BASE, INTERACTIVE_BASE, MOTION_FAST,
    HIT_TARGET_MIN, OVERSCROLL_CONTAIN, PRESS_ACTIVE, TAP_HIGHLIGHT, TOUCH_ACTION,
};
```
Note: `HIT_TARGET_MIN`, `OVERSCROLL_CONTAIN`, `PRESS_ACTIVE`, `TAP_HIGHLIGHT`, `TOUCH_ACTION` are not currently imported in containers.rs — they must be added for the new render functions to pass the `render_functions_use_constants_not_literals` drift-guard test in `classes.rs` lines 83–108.

**Container render function signature** (line 798):
```rust
pub(crate) fn render_grid(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: GridProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => return format!("<!-- ferro-json-ui: failed to decode Grid props: {} -->", html_escape(&e.to_string())),
    };
    // ...
    let body: String = el.children.iter()
        .enumerate()
        .map(|(i, cid)| {
            let rendered = render_element(cid, spec, data, depth + 1);
            // optional per-child wrapper
            rendered
        })
        .collect();
    // emit wrapper div
}
```
Note: containers use `serde_json::from_value(el.props.clone())` (NOT `decode_props`). Atoms use `decode_props` (a crate-internal helper that emits the diagnostic comment). Either works, but follow the existing convention per file.

**Child-render pipeline pattern** (lines 824–849) — copy for render_tile_grid:
```rust
let body: String = el.children
    .iter()
    .map(|cid| render_element(cid, spec, data, depth + 1))
    .collect();
```
Simpler than Grid's (no span wrapper needed for TileGrid children — tiles are uniform).

**Grid row_weights extension pattern** (D-24, lines 865–870):
```rust
// BEFORE the final format!(...):
let row_style = if fill && !props.row_weights.is_empty() {
    let rows = props.row_weights
        .iter()
        .map(|w| format!("{}fr", w))
        .collect::<Vec<_>>()
        .join(" ");
    format!(" style=\"grid-template-rows: {}\"", rows)
} else {
    String::new()
};
// Change the final format! from:
//   format!("<div class=\"grid w-full {col_classes} {gap}\">{body}</div>")
// to:
//   format!("<div class=\"grid w-full {col_classes} {gap}\"{row_style}>{body}</div>")
```
Attribute-order note: `class` before `style` (matches kanban inline style pattern at lines ~486–488).

**Inline style pattern** (kanban, lines ~486–488):
```rust
html.push_str(
    "<div class=\"hidden md:block overflow-x-auto\" \
     style=\"margin-left: -1.5rem; margin-right: -1.5rem; padding-left: 1.5rem; padding-right: 1.5rem;\">",
);
```

**render_selection_panel layout contract** (D-15) — pinned-scrollable pane using `OVERSCROLL_CONTAIN`:
```rust
// Panel root: data-selection-panel + data-selection-form="{form_id}"
// Outer: fill_viewport / height + overflow structure (mirrors Grid fill path):
//   "flex flex-col h-full min-h-0 {OVERSCROLL_CONTAIN}"
// Lines container (scrollable region): "flex-1 overflow-y-auto min-h-0"
// Header + total + confirm slot: "flex-shrink-0" (pinned)
```
Children of SelectionPanel (the confirm Button) are rendered into the confirm slot via:
```rust
let confirm_slot: String = el.children.iter()
    .map(|cid| render_element(cid, spec, data, depth + 1))
    .collect();
```

**Template pattern for selection lines** (D-08) — `<template>` element emitted in Rust:
```rust
html.push_str(
    "<template data-selection-line-template>\
     <div data-selection-line class=\"flex items-center gap-2 py-2 border-b border-border\">\
       <span data-selection-line-name class=\"flex-1 text-sm text-text\"></span>\
       <!-- dec/inc buttons with data-selection-dec/inc, remove with data-selection-remove -->\
       <span data-selection-line-total class=\"text-sm font-semibold text-text\"></span>\
     </div>\
     </template>"
);
```

**containers.rs test helper** (lines 1415–1426):
```rust
fn build_spec(elements: Vec<(&str, ElementBuilder)>) -> Spec {
    let mut b = Spec::builder();
    for (id, el) in elements { b = b.element(id, el); }
    b.build().expect("ok")
}
```

---

### `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES + dispatch (5 additions)

**Analog:** self — lines 44–96 (BUILTIN_TYPES) and lines 177–231 (dispatch match)

**BUILTIN_TYPES extension pattern** (lines 44–96):
```rust
pub(crate) const BUILTIN_TYPES: &[&str] = &[
    // Leaves (atoms.rs)
    // ... existing 23 entries ...
    "TileGrid",          // containers.rs (add after existing atoms section)
    "FilterTabs",        // atoms.rs
    "QuantityStepper",   // atoms.rs
    "Numpad",            // atoms.rs
    // Containers (containers.rs)
    // ... existing container entries ...
    "SelectionPanel",    // containers.rs
];
```
Order in the array must match order in the dispatch match below (comment says "Order matches the dispatch match below for reviewability"). Add one entry per commit (D-25).

**Dispatch match extension pattern** (lines 177–231):
```rust
// In the atoms section:
"TileGrid"         => containers::render_tile_grid(el, spec, data, depth),
"FilterTabs"       => atoms::render_filter_tabs(el, spec, data, depth),
"QuantityStepper"  => atoms::render_quantity_stepper(el, spec, data, depth),
"Numpad"           => atoms::render_numpad(el, spec, data, depth),
// In the containers section:
"SelectionPanel"   => containers::render_selection_panel(el, spec, data, depth),
```

---

### `ferro-json-ui/src/catalog.rs` — BUILTIN_SPECS + count guard + History comment

**Analog:** self — lines 124–270 (BUILTIN_SPECS), lines 1211–1219 (count guard + History)

**BUILTIN_SPECS entry pattern** (atoms — no slot fields, lines 253–257):
```rust
(
    "Tile",
    "Touch-friendly tile with name, price, and +/- quantity controls.",
    || to_value(schema_for!(TileProps)).unwrap(),
    &[],
),
```
For containers with child-slot fields (line 275):
```rust
(
    "Card",
    "Content container with title, description, optional badge and subtitle, body children, and optional footer slot.",
    || to_value(schema_for!(CardProps)).unwrap(),
    &["footer"],
),
```
The five new components are all `&[]` (no named slot fields — children are positional or there are no slot-ID props that need the slot-field metadata).

**BUILTIN_SPECS description pattern:** One sentence, neutral English, names the key props/behavior.

**Count guard + History comment pattern** (lines 1211–1219):
```rust
// History: 39 → 40 (CheckboxList) → ... → 47 (DropdownMenu replaced by ActionGroup).
assert_eq!(crate::render::BUILTIN_TYPES.len(), 47);
```
On each component addition: increment the number AND append `// → 48 (TileGrid).` to the History comment line. Do NOT skip the History append — it is the audit trail (D-25).

**Drift guard (structural — not the count)**: `catalog.rs` line 576:
```rust
if BUILTIN_SPECS.len() != crate::render::BUILTIN_TYPES.len() {
    return Err(CatalogError::BuildFailed(...));
}
```
This fires at catalog-build time when lengths diverge. BOTH arrays must be updated in the same commit.

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` — mirror count + expected names + RULE_COMPONENTS

**Analog:** self — lines 81–103 (RULE_COMPONENTS), lines 396–461 (count guard + expected names)

**Mirror count pattern** (lines 396–407) — bump from 47 to 52 one step per component addition:
```rust
assert_eq!(
    catalog.components.len(),
    47,   // ← bump each time; final value is 52
    "Catalog should contain all 47 built-in components (...), got {}",
    catalog.components.len()
);
```
Update the error message string to match the new count and name the new components.

**Expected names array pattern** (lines 410–458) — append each new name:
```rust
let expected = [
    // ... existing 47 names ...
    "TileGrid",
    "FilterTabs",
    "QuantityStepper",
    "Numpad",
    "SelectionPanel",
];
```

**RULE_COMPONENTS extension pattern** (lines 81–103):
```rust
static RULE_COMPONENTS: &[(&str, &[&str])] = &[
    // ... existing entries ...
    // Extend these in the commit that registers TileGrid:
    ("register-fill-viewport", &["Grid", "TileGrid"]),
    ("register-grid-fill",     &["Grid", "TileGrid"]),
    // Extend this in the commit that registers SelectionPanel:
    ("register-selection-present", &["Grid", "SelectionPanel"]),
    // Numpad: add to register-selection-present alongside SelectionPanel (D-26)
    // OR leave unmapped — verify component_rule_mapping_is_exhaustive semantics first.
    ("fill-viewport-layout-unknown", &[]),  // unchanged
];
```
The bidirectional test at line ~530 (`component_rule_mapping_is_exhaustive`) requires: (a) every rule id in `RULE_COMPONENTS` exists in `design::rules()`; (b) every component name in a mapping is a real builtin. Never add a rule id not already in `design::rules()`. Never add a component name not yet in BUILTIN_TYPES.

---

### `ferro-json-ui/src/runtime/selection.rs` — new module setupSelection (ES5, delegated)

**Analog:** `ferro-json-ui/src/runtime/filters.rs` (setup pattern), `tiles.rs` (qty input contract), `form_guards.rs` (form= attribute resolution + delegated events)

**Module file shape** (copy from tiles.rs line 1 or filters.rs line 6):
```rust
pub(super) const SOURCE: &str = r#"
    // ── Selection panel — live cart view ────────────────────────────────────
    //
    // Attribute contract (D-06..D-15):
    //   [data-selection-panel]                   — panel root
    //   [data-selection-form="{form_id}"]        — scope isolator
    //   [data-selection-line-template]           — <template> for line markup
    //   [data-selection-lines]                   — lines container (scrollable)
    //   [data-selection-empty]                   — EmptyState (toggled)
    //   [data-selection-total]                   — running total display
    //   [data-selection-inc="{field}"]           — per-line inc button (delegated)
    //   [data-selection-dec="{field}"]           — per-line dec button (delegated)
    //   [data-selection-remove="{field}"]        — per-line remove button (delegated)
    //   [data-filter-text]                       — tile root (name + unit price source)
    //   [data-unit-price]                        — integer cents on tile root
    //   [data-qty-input="{field}"]               — hidden form input

    function setupSelection() {
        var panels = document.querySelectorAll('[data-selection-panel]');
        if (panels.length === 0) return;
        for (var i = 0; i < panels.length; i++) {
            initSelectionPanel(panels[i]);
        }
    }
"#;
```

**ES5 style rules** (from research + all existing runtime modules):
- `var` only — no `const`/`let`
- `function foo() {}` — no arrow functions
- String concatenation `'a' + x + 'b'` — no template literals
- `for (var i = 0; i < n; i++)` — no `forEach` with arrow
- `document.querySelectorAll`, `getAttribute`, `addEventListener`, `dispatchEvent`

**setup pattern** (from setupFilters in filters.rs lines 31–37):
```javascript
function setupFilters() {
    var scopes = document.querySelectorAll('[data-filter-scope]');
    if (scopes.length === 0) return;  // no-op when absent
    for (var i = 0; i < scopes.length; i++) {
        initFilterScope(scopes[i]);
    }
}
```

**Form scope resolution pattern** (from form_guards.rs lines 29–34 — resolves `form=` attr):
```javascript
var form = btn.closest('form');
if (!form && btn.getAttribute('form')) {
    form = document.getElementById(btn.getAttribute('form'));
}
if (!form) return;
```
For selection.rs: resolve the panel's `data-selection-form` attribute to get the form element (D-11 scoping).

**Delegated click pattern** (D-10 — for post-load template-cloned lines):
```javascript
panel.addEventListener('click', function(e) {
    var incBtn = e.target.closest('[data-selection-inc]');
    if (incBtn) {
        var field = incBtn.getAttribute('data-selection-inc');
        if (field) field = field.replace(/["\\\]]/g, '');  // sanitize (copy from tiles.rs line 21)
        var input = form.querySelector('[data-qty-input="' + field + '"]');
        if (input) {
            input.value = parseInt(input.value, 10) + 1;
            input.dispatchEvent(new Event('input', { bubbles: true }));
        }
    }
    // similar for dec (clamp at 0 — reconcile removes on 0) and remove (set to 0)
});
```

**Input-event delegation pattern** (D-07 — reconcile on any hidden-input change):
```javascript
form.addEventListener('input', function(e) {
    if (e.target && e.target.getAttribute('data-qty-input')) {
        reconcile();
    }
});
```

**Template clone pattern** (D-08 — ES5 compatible):
```javascript
var tmpl = panel.querySelector('[data-selection-line-template]');
var clone = tmpl.content.cloneNode(true);  // DocumentFragment deep clone
clone.querySelector('[data-selection-line-name]').textContent = name;
clone.querySelector('[data-selection-line-total]').textContent = lineTotal;
linesEl.appendChild(clone);
```

**Integer-cents money pattern** (from numpad.rs `numpadPriceDisplay`):
```javascript
// Integer arithmetic only — never float (PITFALLS.md)
// Format cents as two-decimal display:
function formatCents(cents) {
    var n = parseInt(cents, 10) || 0;
    return (n / 100).toFixed(2);  // display only; input stays integer cents
}
```

**Semantic token classes** (enforced by `variant_classes_use_semantic_tokens` test in mod.rs lines 84–101): use `bg-surface`, `text-text`, `text-text-muted`, `border-border` — never raw palette. The scan asserts no `bg-blue-500` / `bg-green-500` / `bg-yellow-500` / `bg-red-500` anywhere in the bundle.

---

### `ferro-json-ui/src/runtime/mod.rs` — bundle assembly extension

**Analog:** self — lines 8–77 (LazyLock + dispatcher), lines 193–258 (drift tests)

**Module declaration pattern** (lines 8–22):
```rust
mod dismissibles;
// ... existing 14 modules ...
mod selection;  // ADD: after existing mods, alphabetical or by concern
```

**LazyLock push pattern** (lines 29–76):
```rust
pub static FERRO_RUNTIME_JS: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::with_capacity(8 * 1024);
    s.push_str("(function() {\n    'use strict';\n");
    // ... existing push_str calls ...
    s.push_str(selection::SOURCE);  // ADD before the ferroRuntime() dispatcher string
    s.push_str(
        "\n    function ferroRuntime() {\n\
         \x20       var setups = [\n\
         // ... existing setup names ...
         \x20           setupSelection\n\  // ADD
         \x20       ];\n\
         // ..."
    );
    s
});
```

**bundle_contains_all_setup_functions test pattern** (lines 193–217):
```rust
#[test]
fn bundle_contains_all_setup_functions() {
    for fn_name in [
        // ... existing 15 names ...
        "setupSelection",  // ADD
    ] {
        assert!(FERRO_RUNTIME_JS.contains(fn_name), "bundle missing {fn_name}");
    }
}
```

**dispatcher_invokes_every_setup test pattern** (lines 228–258):
```rust
#[test]
fn dispatcher_invokes_every_setup() {
    let js: &str = FERRO_RUNTIME_JS.as_str();
    let dispatcher_start = js.find("function ferroRuntime()").unwrap();
    let dispatcher = &js[dispatcher_start..];
    for name in [
        // ... existing 15 names ...
        "setupSelection",  // ADD
    ] {
        assert!(dispatcher.contains(name), "dispatcher setups array missing {name}");
    }
}
```

---

### `ferro-json-ui/src/runtime/tiles.rs` — initQtyButton min/max/step extension

**Analog:** self — lines 15–33 (full `initQtyButton` implementation)

**Current pattern** (lines 15–33):
```rust
pub(super) const SOURCE: &str = r#"
    function initQtyButton(btn, delta) {
        btn.addEventListener('click', function() {
            var field = btn.getAttribute(delta > 0 ? 'data-qty-inc' : 'data-qty-dec');
            if (field) field = field.replace(/["\\\]]/g, '');
            var display = document.querySelector('[data-qty-display="' + field + '"]');
            var input = document.querySelector('[data-qty-input="' + field + '"]');
            if (!display || !input) return;
            var current = parseInt(input.value, 10) || 0;
            var next = current + delta;
            if (next < 0) next = 0;
            input.value = next;
            display.textContent = next;
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
    }
"#;
```

**D-22 extension — insert after `var current = ...` line**:
```javascript
var step = parseInt(btn.getAttribute('data-qty-step'), 10) || 1;
var min  = parseInt(btn.getAttribute('data-qty-min'),  10) || 0;
var rawMax = btn.getAttribute('data-qty-max');
var max  = rawMax !== null ? parseInt(rawMax, 10) : Infinity;
var next = Math.min(Math.max(current + delta * step, min), max);
// remove old: var next = current + delta; if (next < 0) next = 0;
```
Note: `display` may be null when the tile-root-as-button form is used (no `data-qty-display` on tap-to-add tiles — D-02). The `if (!display || !input) return;` guard currently returns early when display is missing. Relax to `if (!input) return;` then conditionally update display: `if (display) display.textContent = next;`. This ensures tile-tap (no display) still updates the hidden input.

---

### `ferro-json-ui/src/component.rs` — props additions + rustdoc fixes

**Analog:** self — `TileProps` (lines 1358–1388), `SelectionPanelProps` (lines 1411–1420), `FilterTabsProps` (lines 1422–1435)

**Additive optional prop pattern** (lines 1380–1387):
```rust
/// Optional item image URL. Declared here for the Phase 256 tile visual;
/// not rendered in Phase 254 (D-03).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub image_url: Option<String>,
```
For `TileProps.price_cents` (D-04):
```rust
/// Machine-readable unit price in integer cents. Rendered as
/// `data-unit-price="{cents}"` on the tile root. The client-computed
/// running total reads this attribute; `price` is a display string and
/// cannot be parsed. Both fields are expected to agree — the Phase 257
/// projector emits both from one source. Missing attribute is treated as
/// 0 cents by the runtime. Integer cents only — never float.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub price_cents: Option<u64>,
```

**Rustdoc fix pattern** (FilterTabsProps line 1432–1434):
```rust
// BEFORE (line 1432 — incorrect, predates vocabulary-neutralization):
/// Label for the "show all" tab (Phase 256 render default is "Tutte").
// AFTER (D-28 fix):
/// Label for the "show all" tab. Phase 256 render default is "All".
/// Pass `all_label: "Tutte"` or any locale string from the consumer.
```

**Serde enum pattern** for `NumpadMode` (lines 1454–1463) — already correct:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumpadMode {
    #[default]
    Quantity,
    Price,
}
```
Follow this for any new enum in component.rs.

---

### `docs/src/json-ui/components.md` — migration table note

**Analog:** self — lines 103–112 (Component rename migration table, `## Component rename migration (v16.6)` section)

**Migration table row pattern** (lines 107–111):
```markdown
| Renamed in v16.6 | New surface | Migration action |
|------------------|-------------|------------------|
| Commerce tile builtin (formerly the product-prefixed type string) | `Tile` | Change `"type"` in every spec to `"Tile"` |
```

**D-02 addition** — append ONE row to the existing table (or a sub-note below it):
```markdown
| Tile interaction model (on-tile +/- stepper markup, Phase 255 and earlier) | Tile (tap-to-add) | On-tile quantity stepper markup replaced in v16.6 (Phase 256). The tile root is now a `<button type="button" data-qty-inc>`. Selection feedback and per-line quantity editing moved to the SelectionPanel. |
```
Do NOT write full component docs here — that is Phase 258. One migration note only.

---

## Shared Patterns

### Prop decode pattern — atoms vs containers
**Atoms** (`render_tile`, `render_action_card`, etc.) use the `decode_props` helper (defined at atoms.rs line 30+):
```rust
let props: TileProps = match decode_props(&el.props) {
    Ok(p) => p,
    Err(e) => return decode_diagnostic("Tile", e),
};
```
**Containers** (`render_grid`, `render_collapsible`, etc.) use `serde_json::from_value` directly:
```rust
let props: GridProps = match serde_json::from_value(el.props.clone()) {
    Ok(p) => p,
    Err(e) => return format!("<!-- ferro-json-ui: failed to decode Grid props: {} -->", html_escape(&e.to_string())),
};
```
Follow the convention of the file you're editing. New atoms go in atoms.rs → use `decode_props`. New containers go in containers.rs → use `serde_json::from_value`.

### HTML escape rule
Every `props.*` string interpolated into an HTML attribute or text node MUST go through `html_escape(...)`. Numeric types (`u32`, `u64`) can be formatted directly. Example:
```rust
let name = html_escape(&props.name);
let qty  = props.default_quantity.unwrap_or(0);  // u32 — no escape needed
format!("... {name} ... value=\"{qty}\"")
```

### Class constant import rule
The `render_functions_use_constants_not_literals` test (classes.rs lines 83–108) asserts that NO `.rs` file in `src/render/` (except `classes.rs` itself) contains the raw string literals `"touch-manipulation"`, `"min-h-[44px] min-w-[44px]"`, or `"min-h-[56px] min-w-[56px]"`. Import and reference the constants:
```rust
use super::classes::{TOUCH_ACTION, HIT_TARGET_MIN, HIT_TARGET_NUMPAD, PRESS_ACTIVE, TAP_HIGHLIGHT, INTERACTIVE_BASE, OVERSCROLL_CONTAIN};
```

### BUILTIN lockstep — commit checklist
Every component addition requires ALL of these in ONE commit:
1. `render/{atoms,containers}.rs`: add `pub(crate) fn render_{name}(...) -> String`
2. `render/mod.rs` line 44+: add type name string to `BUILTIN_TYPES`
3. `render/mod.rs` line 177+: add dispatch match arm
4. `catalog.rs` line 124+: add entry to `BUILTIN_SPECS`
5. `catalog.rs` line 1219: bump count + append History comment line
6. `ferro-mcp/src/tools/json_ui_catalog.rs` line 402: bump mirror count
7. `ferro-mcp/src/tools/json_ui_catalog.rs` line 410+: add name to `expected` array
8. (where applicable) `ferro-mcp/src/tools/json_ui_catalog.rs` line 81+: extend `RULE_COMPONENTS`

### ES5-only runtime constraint
All `runtime/*.rs` `SOURCE` constants are ES5 strict-mode JavaScript. No arrow functions, no template literals, no `const`/`let`, no destructuring assignment, no `Promise`/`async`. The `bundle_is_single_iife` test (mod.rs line 219) enforces the IIFE wrapper. The `variant_classes_use_semantic_tokens` test (mod.rs line 84) enforces semantic token classes.

### Field sanitization in runtime JS (security)
Copy the field-name sanitization from tiles.rs line 21 into ANY JS code that builds an attribute selector from a field name:
```javascript
if (field) field = field.replace(/["\\\]]/g, '');
```
This prevents `querySelector` SyntaxError from malformed field names.

### Schema export churn
Running `cargo test --all-features` regenerates `docs/protocol/schemas/*.json`. For Phase 256, these should have NO real content changes (new props are `ferro-json-ui` types, not `ferro-projections` types — see RESEARCH.md Q12). After the full gate run: `git diff docs/protocol/schemas/` — if only whitespace/ordering churn, `git checkout docs/protocol/schemas/`; if real content changes, commit them (D-30 clause).

---

## No Analog Found

All files have close analogs within the codebase. No files lack pattern coverage.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/render/`, `ferro-json-ui/src/runtime/`, `ferro-json-ui/src/catalog.rs`, `ferro-json-ui/src/component.rs`, `ferro-mcp/src/tools/json_ui_catalog.rs`, `docs/src/json-ui/components.md`
**Files scanned:** 13 source files read directly
**Pattern extraction date:** 2026-07-06

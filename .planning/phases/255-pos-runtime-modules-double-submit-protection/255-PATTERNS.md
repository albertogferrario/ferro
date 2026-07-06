# Phase 255: POS Runtime Modules + Double-Submit Protection — Pattern Map

**Mapped:** 2026-07-05
**Files analyzed:** 17 (2 new, 15 modified)
**Analogs found:** 17 / 17

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/runtime/numpad.rs` | middleware (JS runtime) | event-driven | `runtime/product_tiles.rs` (input dispatch) + `runtime/kanban.rs` (delegation) | role-match |
| `ferro-json-ui/src/runtime/filters.rs` | middleware (JS runtime) | event-driven | `runtime/tabs.rs` (active-class toggle) + `runtime/product_tiles.rs` (querySelectorAll loops) | role-match |
| `ferro-json-ui/src/runtime/form_guards.rs` | middleware (JS runtime) | event-driven | itself — `initNumberGuard` + `findGuardedSubmit` + disabled-state vocab | exact |
| `ferro-json-ui/src/runtime/mod.rs` | config (bundle assembly) | request-response | itself — existing concat list, dispatcher, two drift-list arrays | exact |
| `ferro-json-ui/src/runtime/tiles.rs` (renamed) | middleware (JS runtime) | event-driven | `runtime/product_tiles.rs` — pure rename | exact |
| `ferro-json-ui/src/component.rs` | model (data types) | transform | itself — existing `ButtonProps` `disabled`/`form` optional fields | exact |
| `ferro-json-ui/src/render/atoms.rs` | renderer (atoms) | request-response | itself — `render_product_tile` body + `render_button_inner` optional-attr pattern | exact |
| `ferro-json-ui/src/render/classes.rs` | utility (constants) | N/A | itself — existing `POS_*` constants | exact |
| `ferro-json-ui/src/render/mod.rs` | router (dispatch) | request-response | itself — existing dispatch arm pattern | exact |
| `ferro-json-ui/src/catalog.rs` | config (catalog) | transform | itself — existing `BUILTIN_SPECS` entry + count assertion | exact |
| `ferro-json-ui/src/lib.rs` | config (re-exports) | N/A | itself — existing `pub use component::{...}` list | exact |
| `ferro-json-ui/src/design/rules.rs` | utility (lint) | transform | itself — existing `POS_TRIGGER_TYPES` + rule id strings | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | utility (MCP catalog) | transform | itself — `RULE_COMPONENTS` + `test_all_components_present` array | exact |
| `app/src/views/cassa.json` | config (view spec) | N/A | itself — existing element props pattern | exact |
| `docs/src/json-ui/components.md` | documentation | N/A | itself — existing migration table at lines 72–93 | exact |
| `docs/src/design-system/patterns.md` | documentation | N/A | itself — existing `pos-fill-viewport` section format | exact |
| `docs/src/features/write-kernel.md` | documentation | N/A | itself — existing `##` section + prose + code-block format | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/runtime/numpad.rs` (new — middleware, event-driven)

**Primary analog:** `ferro-json-ui/src/runtime/product_tiles.rs`
**Secondary analog:** `ferro-json-ui/src/runtime/kanban.rs` (for `closest()` delegation style)

**Module shape pattern** (`product_tiles.rs` lines 1–30 — the entire file):
```rust
pub(super) const SOURCE: &str = r#"
    // ── [Concern] ─────────────────────────────────────────────────────────

    function setupConcern() {
        var items = document.querySelectorAll('[data-concern]');
        if (items.length === 0) return;   // no-op guard per D-06
        for (var i = 0; i < items.length; i++) {
            initConcern(items[i]);
        }
    }

    function initConcern(el) {
        el.addEventListener('click', function() { ... });
    }
"#;
```

Rules extracted from `product_tiles.rs`:
- `pub(super) const SOURCE: &str = r#"..."#` — exact visibility and type
- No IIFE, no `DOMContentLoaded` — those are in `mod.rs`
- ES5 only: `var`, `function`, no `=>`, no template literals (`` ` ``), no `let`/`const`
- One `setup*()` function per module + one or more `init*()` helpers
- No-op guard: `if (items.length === 0) return;` at the top of `setup*()`

**Input event dispatch pattern** (`product_tiles.rs` line 27):
```javascript
input.dispatchEvent(new Event('input', { bubbles: true }));
```
Copy this verbatim for every numpad key-tap write (D-04). `{ bubbles: true }` is required so `initNumberGuard` listeners on ancestor elements receive the event.

**Event delegation pattern** (`kanban.rs` lines 30–end, conceptually):
```javascript
function initNumpad(container) {
    container.addEventListener('click', function(e) {
        var key = e.target.closest('[data-numpad-key]');
        if (!key) return;
        // handle key.getAttribute('data-numpad-key')
    });
}
```
`closest()` is a DOM Level 4 API, not ES6 syntax — valid in the ES5 runtime. No existing module uses `closest()`; numpad is the first. The `e.target` is the clicked element; `closest()` walks up from there.

---

### `ferro-json-ui/src/runtime/filters.rs` (new — middleware, event-driven)

**Primary analog:** `ferro-json-ui/src/runtime/tabs.rs`
**Secondary analog:** `ferro-json-ui/src/runtime/product_tiles.rs`

**Active-tab class toggle pattern** (`tabs.rs` lines 64–72):
```javascript
// Active tab:
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
t.classList.add('border-primary', 'text-primary', 'font-semibold');
t.setAttribute('aria-selected', 'true');
// Inactive tab:
t.classList.remove('border-primary', 'text-primary', 'font-semibold');
t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
t.setAttribute('aria-selected', 'false');
```
Filter tabs MUST mirror this exact pattern. All six class strings are semantic-token, full-literal; `variant_classes_use_semantic_tokens` already passes for these strings (from `tabs.rs`). No CSS regen needed for these class strings.

**querySelectorAll + for-loop iteration pattern** (`product_tiles.rs` lines 5–12):
```javascript
function setupProductTiles() {
    var incBtns = document.querySelectorAll('[data-qty-inc]');
    for (var i = 0; i < incBtns.length; i++) {
        initQtyButton(incBtns[i], 1);
    }
    var decBtns = document.querySelectorAll('[data-qty-dec]');
    for (var j = 0; j < decBtns.length; j++) {
        initQtyButton(decBtns[j], -1);
    }
}
```
Use named index variables (`i`, `j`, `k`) per scope level, not reusing `i`. For `setupFilters`, iterate `[data-filter-scope]` containers first, then within each scope find tabs and tiles.

**No-op guard** (`tabs.rs` line 14):
```javascript
if (triggers.length === 0) return;
```
Apply at the scope level: `if (scopes.length === 0) return;` (D-12).

**hide/show mechanism** (D-11 — do NOT use `hidden` attribute or Tailwind class):
```javascript
tile.style.display = 'none';  // hide
tile.style.display = '';      // show (restores default)
```

---

### `ferro-json-ui/src/runtime/form_guards.rs` (modified — middleware, event-driven)

**Analog:** itself — exact code excerpts to extend

**`findGuardedSubmit` helper** (lines 14–23, read-only reference for inverse pattern):
```javascript
function findGuardedSubmit(form) {
    var inside = form.querySelector('button[type="submit"]');
    if (inside) return inside;
    if (form.id) {
        return document.querySelector(
            'button[type="submit"][form="' + form.id + '"]'
        );
    }
    return null;
}
```
The double-submit guard needs the inverse: given a `button[data-disable-on-submit]`, find its form. Copy the pattern in reverse: `btn.closest('form')` first, then `btn.getAttribute('form')` + `document.getElementById(...)` fallback (D-14).

**initNumberGuard input collection** (lines 54–60) — extend by adding third NodeList:
```javascript
function initNumberGuard(form) {
    var numberInputs = form.querySelectorAll('input[type="number"]');
    var qtyInputs = form.querySelectorAll('input[data-qty-input]');
    // ADD:
    var numpadInputs = form.querySelectorAll('input[data-numpad-input]');
    var inputs = [];
    for (var n = 0; n < numberInputs.length; n++) inputs.push(numberInputs[n]);
    for (var q = 0; q < qtyInputs.length; q++) inputs.push(qtyInputs[q]);
    // ADD:
    for (var m = 0; m < numpadInputs.length; m++) inputs.push(numpadInputs[m]);
    // ... rest unchanged
```
Also update the comment on line 63: `// Skip ProductTile +/- controls` → `// Skip Tile +/- controls`.

**Disabled-state vocabulary** (lines 43–46 and 82–85):
```javascript
// Disable:
submitBtn.setAttribute('disabled', 'disabled');
submitBtn.classList.add('opacity-50', 'cursor-not-allowed');
// Enable:
submitBtn.removeAttribute('disabled');
submitBtn.classList.remove('opacity-50', 'cursor-not-allowed');
```
The double-submit guard uses these exact two strings for D-14. They are already in the bundle so `variant_classes_use_semantic_tokens` passes without change.

**Double-submit guard block** — goes inside `setupFormGuards()` after the existing guard loop (D-13):
```javascript
    // ── Double-submit guard ───────────────────────────────────────────────
    var disableBtns = document.querySelectorAll('button[data-disable-on-submit]');
    for (var d = 0; d < disableBtns.length; d++) {
        initDisableOnSubmit(disableBtns[d]);
    }
    // bfcache recovery (D-15): iPad Safari restores the DOM from cache on back-
    // navigation; re-enable all guarded buttons so the register is usable again.
    window.addEventListener('pageshow', function(e) {
        if (!e.persisted) return;
        for (var r = 0; r < disableBtns.length; r++) {
            disableBtns[r].removeAttribute('disabled');
            disableBtns[r].classList.remove('opacity-50', 'cursor-not-allowed');
        }
    });

    function initDisableOnSubmit(btn) {
        var form = btn.closest('form');
        if (!form && btn.getAttribute('form')) {
            form = document.getElementById(btn.getAttribute('form'));
        }
        if (!form) return;
        var submitted = false;
        form.addEventListener('submit', function(e) {
            if (submitted) { e.preventDefault(); return; }
            submitted = true;
            btn.setAttribute('disabled', 'disabled');
            btn.classList.add('opacity-50', 'cursor-not-allowed');
        });
    }
```
Bind on `form.submit` — never on `button.click` (click-time disable races with form submission; D-14).

---

### `ferro-json-ui/src/runtime/mod.rs` (modified — config, request-response)

**Analog:** itself — all three patterns excerpted verbatim

**Module declaration block** (lines 8–20):
```rust
mod dismissibles;
mod dropdowns;
mod form_guards;
// ...
mod product_tiles;   // → rename to `mod tiles;`
// ADD:
mod numpad;
mod filters;
```

**FERRO_RUNTIME_JS concat list** (lines 27–62):
```rust
pub static FERRO_RUNTIME_JS: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::with_capacity(8 * 1024);
    s.push_str("(function() {\n    'use strict';\n");
    s.push_str(sse::SOURCE);
    // ...
    s.push_str(product_tiles::SOURCE);  // → tiles::SOURCE
    // ADD after tiles:
    s.push_str(numpad::SOURCE);
    s.push_str(filters::SOURCE);
    // ...dispatcher string follows
```

**Dispatcher body** (lines 43–61):
```rust
    s.push_str(
        "\n    function ferroRuntime() {\n\
         \x20       setupScrollPreserve();\n\
         // ...
         \x20       setupFormGuards();\n\
         \x20       setupProductTiles();\n\  // → setupTiles();
         // ADD:
         \x20       setupNumpad();\n\
         \x20       setupFilters();\n\
         // ...
         \x20   }\n\
         \x20   document.addEventListener('DOMContentLoaded', ferroRuntime);\n\
         })();\n",
    );
```

**`bundle_contains_all_setup_functions` drift array** (lines 180–201):
```rust
#[test]
fn bundle_contains_all_setup_functions() {
    for fn_name in [
        "setupSSE",
        "setupTabs",
        // ...
        "setupFormGuards",
        "setupProductTiles",  // → "setupTiles"
        // ADD:
        "setupNumpad",
        "setupFilters",
        // ...
    ] {
        assert!(FERRO_RUNTIME_JS.contains(fn_name), "bundle missing {fn_name}");
    }
}
```

**`dispatcher_invokes_every_setup` drift array** (lines 210–231):
```rust
#[test]
fn dispatcher_invokes_every_setup() {
    let js: &str = FERRO_RUNTIME_JS.as_str();
    let dispatcher_start = js.find("function ferroRuntime()").unwrap();
    let dispatcher = &js[dispatcher_start..];
    for call in [
        "setupSSE();",
        // ...
        "setupFormGuards();",
        "setupProductTiles();",  // → "setupTiles();"
        // ADD:
        "setupNumpad();",
        "setupFilters();",
        // ...
    ] {
        assert!(dispatcher.contains(call), "dispatcher missing {call}");
    }
}
```

Both arrays must be updated in the same edit as the concat list and dispatcher — they are separate from each other (Pitfall 2).

---

### `ferro-json-ui/src/component.rs` (modified — model, transform)

**Analog:** itself — `ButtonProps` existing optional field pattern (lines 305–318)

**Existing `Option<bool>` field pattern to mirror** (`ButtonProps` lines 305–317):
```rust
pub struct ButtonProps {
    pub label: String,
    #[serde(default)]
    pub variant: Variant,
    #[serde(default)]
    pub size: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    // ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
}
```

**New `disable_on_submit` field** (goes after `form: Option<String>`):
```rust
/// When `true`, emits `data-disable-on-submit` on the rendered button; the
/// runtime guard disables this button after the first form submission to
/// prevent double-posting (D-16). Pairs with a per-render `idempotency_key`
/// hidden input for server-side deduplication (see `dispatch_write` step 2).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub disable_on_submit: Option<bool>,
```
The `#[serde(default, skip_serializing_if = "Option::is_none")]` pair is the standard additive-prop convention — identical to `disabled` and `form` above it.

**`ProductTileProps` → `TileProps` rename** (lines 1352–1382): rename struct, rename `product_id` → `item_id`, update rustdoc. Other fields (`name`, `price`, `field`, `default_quantity`, `categories`, `image_url`, `color`, `stock_badge`) unchanged.

**`CartPanelProps` → `SelectionPanelProps`** (lines 1407–1420): rename struct; REMOVE `show_staff: Option<bool>` and `show_people: Option<bool>`. No `#[serde(deny_unknown_fields)]` is used in this codebase (confirmed by cascade inventory) — removing fields is backward-compatible for deserialization of existing specs that include the old props.

---

### `ferro-json-ui/src/render/atoms.rs` (modified — renderer, request-response)

**Analog:** itself — `render_button_inner` for optional-attribute emission; `render_product_tile` for rename + new attribute addition

**Optional attribute emission pattern** (`render_button_inner` lines 167–204):
```rust
// Option<bool> → conditional attribute string (disabled pattern):
let disabled_attr = if props.disabled == Some(true) {
    " disabled aria-disabled=\"true\""
} else {
    ""
};

// Option<String> → conditional attribute string (form pattern):
let form_attr = match props.form.as_deref() {
    Some(f) => format!(" form=\"{}\"", html_escape(f)),
    None => String::new(),
};

// Both appear in the format! string:
format!(
    "<button{type_attr}{form_attr} class=\"...\"{disabled_attr}>{content}</button>"
)
```

**`disable_on_submit` attribute emission** — add to `render_button_inner` following the same pattern:
```rust
let disable_on_submit_attr = if props.disable_on_submit == Some(true) {
    " data-disable-on-submit"
} else {
    ""
};
// Add {disable_on_submit_attr} to the format! string
```

**`render_tile` additions** (post-rename from `render_product_tile`, lines 1371–1391): the existing `categories_attr` block emits `data-product-categories` conditionally → rename to `data-filter-tokens`. Additionally, `data-filter-text` must be emitted UNCONDITIONALLY (D-08 — it is the universal tile marker AND search source):
```rust
// After rename and attribute changes:
let filter_text_attr = format!(" data-filter-text=\"{}\"", html_escape(&props.name));
// categories_attr (now data-filter-tokens) remains conditional:
let filter_tokens_attr = if props.categories.is_empty() {
    String::new()
} else {
    format!(" data-filter-tokens=\"{}\"", html_escape(&props.categories.iter()
        .map(|c| c.replace(' ', "-")).collect::<Vec<_>>().join(" ")))
};
// Both appear in the top-level div:
format!("<div ...{filter_tokens_attr}{filter_text_attr}>...")
```

**Test pattern** (`make_product_tile` helper, lines 2548–2561):
```rust
fn make_product_tile(categories: Vec<&str>) -> (crate::spec::Element, Spec) {
    use crate::spec::Element as SpecElement;
    let mut el_builder = SpecElement::new("ProductTile")  // → "Tile"
        .prop("product_id", "p1")   // → "item_id"
        .prop("name", "Espresso")
        .prop("price", "€2,50")
        .prop("field", "qty_espresso");
    if !categories.is_empty() {
        el_builder = el_builder.prop("categories", categories);
    }
    let spec = spec_with_root(el_builder);
    let el = spec.elements.get("root").unwrap().clone();
    (el, spec)
}
```
Post-rename: `"Tile"` + `"item_id"`.

**`tile_legacy_render_is_byte_identical` extensions** (post-rename from `product_tile_legacy_render_is_byte_identical`, lines 2563–2581):
```rust
// EXISTING assertions (update attribute names):
assert!(!html.contains("data-filter-tokens"), "legacy tile must not emit data-filter-tokens");
// NEW assertions per D-08:
assert!(html.contains("data-filter-text=\"Espresso\""),
    "data-filter-text must always be emitted (universal tile marker); got: {html}");
// New escaping test for data-filter-text (mirrors product_tile_escapes_categories):
// build a tile with name containing '"', assert HTML-escaped in data-filter-text
```

---

### `ferro-json-ui/src/render/classes.rs` (modified — utility, N/A)

**Analog:** itself — existing `POS_*` constants (lines 40–58)

**Current constants to rename** (lines 40–58):
```rust
pub const POS_TOUCH_ACTION: &str = "touch-manipulation";       // → TOUCH_ACTION
pub const POS_HIT_TARGET_MIN: &str = "min-h-[44px] min-w-[44px]"; // → HIT_TARGET_MIN
pub const POS_HIT_TARGET_NUMPAD: &str = "min-h-[56px] min-w-[56px]"; // → HIT_TARGET_NUMPAD
pub const POS_PRESS_ACTIVE: &str = "active:scale-95 active:bg-border"; // → PRESS_ACTIVE
pub const POS_OVERSCROLL_CONTAIN: &str = "overscroll-contain";  // → OVERSCROLL_CONTAIN
pub const POS_TAP_HIGHLIGHT: &str = "pos-tap-highlight";        // → TAP_HIGHLIGHT
```
Class VALUE strings are unchanged — no CSS regen needed from this rename alone.

**Test to update** (lines 111–125 — `pos_constants_are_full_literals_and_token_compliant`):
```rust
#[test]
fn pos_constants_are_full_literals_and_token_compliant() {
    assert_eq!(POS_TOUCH_ACTION, "touch-manipulation");   // → TOUCH_ACTION
    // ... update all six constant names, keep the value assertions identical
}
```

**`pos_render_functions_use_constants_not_literals` auto-coverage** (lines 83–108): this test iterates `src/render/*.rs` automatically — it will auto-cover renamed `atoms.rs` after the rename. No manual update needed, but the guarded literal list (`"touch-manipulation"`, `"min-h-[44px] min-w-[44px]"`, `"min-h-[56px] min-w-[56px]"`) stays the same since VALUES are unchanged.

All call sites of `POS_TOUCH_ACTION` etc. in `atoms.rs` must be updated to the new constant names in the same edit.

---

### `ferro-json-ui/src/design/rules.rs` (modified — utility, transform)

**Analog:** itself — existing rule ids and `POS_TRIGGER_TYPES` constant (lines 84–112, 443–510)

**Rule id strings** (lines 85, 92, 99):
```rust
DesignRule { id: "pos-fill-viewport", ... }  // → "register-fill-viewport"
DesignRule { id: "pos-grid-fill", ... }       // → "register-grid-fill"
DesignRule { id: "pos-cart-present", ... }    // → "register-selection-present"
```

**Trigger types constant** (line 443):
```rust
const POS_TRIGGER_TYPES: &[&str] = &["ProductGrid", "CartPanel", "Numpad"];
// →
const REGISTER_TRIGGER_TYPES: &[&str] = &["TileGrid", "SelectionPanel", "Numpad"];
```
Update all usages of `POS_TRIGGER_TYPES` to `REGISTER_TRIGGER_TYPES` (lines 449, and in `check_pos_fill_viewport`). Update all `"ProductGrid"` / `"CartPanel"` type string comparisons (lines 499–500) to `"TileGrid"` / `"SelectionPanel"`. Update all `rule:` assertion strings in tests.

---

### `ferro-json-ui/src/render/mod.rs` (modified — router, request-response)

**Analog:** itself — existing BUILTIN_TYPES array and dispatch match arm

**BUILTIN_TYPES entry** (line 67):
```rust
"ProductTile",  // → "Tile"
```

**Dispatch arm** (line 200):
```rust
"ProductTile" => atoms::render_product_tile(el, spec, data, depth),
// →
"Tile" => atoms::render_tile(el, spec, data, depth),
```

---

### `ferro-json-ui/src/catalog.rs` (modified — config, transform)

**Analog:** itself — existing BUILTIN_SPECS entry pattern (lines 252–257)

**Entry to update** (lines 252–257):
```rust
(
    "ProductTile",     // → "Tile"
    "Touch-friendly POS tile with name, price, and +/- quantity controls.",
    || to_value(schema_for!(ProductTileProps)).unwrap(),  // → TileProps
    &[],
),
```
Count assertion at line 1219 remains `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47)` — unchanged.

---

### `ferro-json-ui/src/lib.rs` (modified — config, N/A)

**Analog:** itself — existing `pub use component::{...}` list (lines 50–63)

**Re-export to update** (line 58):
```rust
// current:
ProductTileProps,
// →
TileProps,
```
Also add any new public types that come from this phase (e.g. `TileGridProps`, `SelectionPanelProps`, `FilterTabsProps` if not already exported — check the existing pub use list; the cascade inventory confirms `ProductTileProps` is at line 58).

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` (modified — utility, transform)

**Analog:** itself — `RULE_COMPONENTS` array and `test_all_components_present` expected list

**`RULE_COMPONENTS` entries** (lines 99–101):
```rust
("pos-fill-viewport", &["Grid"]),    // → ("register-fill-viewport", &["Grid"])
("pos-grid-fill", &["Grid"]),        // → ("register-grid-fill", &["Grid"])
("pos-cart-present", &["Grid"]),     // → ("register-selection-present", &["Grid"])
```

**`test_all_components_present` expected array** (line 450):
```rust
"ProductTile",  // → "Tile"
```
Count assertion (line 402–406) stays at 47; update the message text only:
```rust
assert_eq!(
    catalog.components.len(),
    47,
    "Catalog should contain all 47 built-in components ..., got {}",
    catalog.components.len()
);
```

---

### `app/src/views/cassa.json` (modified — config, N/A)

**Analog:** itself — existing element props pattern (lines 70–79)

**Current state** (lines 70–79):
```json
"tile": {
  "type": "ProductTile",
  "$each": { "path": "/prodotti", "as": "p" },
  "props": {
    "product_id": { "$data": "/p/id" },
    "name": { "$data": "/p/nome" },
    "price": { "$data": "/p/prezzo" },
    "field": { "$data": "/p/field" }
  }
}
```

**After changes**:
```json
"tile": {
  "type": "Tile",
  "$each": { "path": "/prodotti", "as": "p" },
  "props": {
    "item_id": { "$data": "/p/id" },
    "name": { "$data": "/p/nome" },
    "price": { "$data": "/p/prezzo" },
    "field": { "$data": "/p/field" }
  }
}
```
Also find `btn_confirm` element and add `"disable_on_submit": true` to its props (D-16; the exact element key in `cassa.json` needs verification by reading the full file — the RESEARCH confirms this element exists).

---

### `docs/src/json-ui/components.md` (modified — documentation, N/A)

**Analog:** itself — existing migration table format (lines 72–93)

**Existing migration table format to extend** (lines 72–93):
```markdown
## Component vocabulary migration

The canonical `variant`/`tone`/`size` vocabulary replaced the per-component enums.
...

| Component | Old prop | Old value | New prop | New value |
|-----------|----------|-----------|----------|-----------|
| Button, ActionGroup item | `variant` | `default` | `variant` | `primary` |
...
```

**New rows to add** (V-07 — add to the migration table):

| Component | Old | — | New | — |
|-----------|-----|---|-----|---|
| ProductTile (component type) | `"type": "ProductTile"` | → | `"type": "Tile"` | — |
| Tile | `product_id` | — | `item_id` | — |
| Tile | `data-product-categories` (JS attr) | — | `data-filter-tokens` | — |

Table column mapping for this phase (component-rename flavor, slightly different from prop-value migration format — model after the existing table's spirit but adjust columns to fit a rename-only migration):

```markdown
| Old name | New name | Notes |
|----------|----------|-------|
| `ProductTile` (type string) | `Tile` | Published break; rename in all JSON specs |
| `product_id` prop | `item_id` prop | On the Tile component only |
| `data-product-categories` (data attribute) | `data-filter-tokens` | JS/HTML contract; update any custom runtime code |
```

**`### ProductTile` section** (lines 1400–1424): rename heading to `### Tile`, update props table (`product_id` → `item_id`), update example JSON, update prose.

---

### `docs/src/design-system/patterns.md` (modified — documentation, N/A)

**Analog:** itself — existing `pos-fill-viewport` section format (lines 522–564)

**Existing section format** (lines 522–563):
```markdown
## `pos-fill-viewport`

**Title:** POS register pages must fill the viewport

**Rationale:** A ProductGrid, CartPanel, or Numpad outside a fill_viewport spec ...

**Intents:** all (applies to any spec containing POS component types)

### Conforming example
...
### Violating example
...
### How to allow

Add `"allow": ["pos-fill-viewport"]` to the `design` object ...
```

Apply the same structure to all three sections, updating:
- `## \`pos-fill-viewport\`` → `## \`register-fill-viewport\``
- `## \`pos-grid-fill\`` → `## \`register-grid-fill\``
- `## \`pos-cart-present\`` → `## \`register-selection-present\``
- All component name strings in prose and fixture JSON: `ProductGrid` → `TileGrid`, `CartPanel` → `SelectionPanel`
- All allow-list strings: `"pos-fill-viewport"` → `"register-fill-viewport"`, etc.

---

### `docs/src/features/write-kernel.md` (modified — documentation, N/A)

**Analog:** itself — existing `##` section + prose + code-block format (lines 1–60)

**Existing section format to mirror**:
```markdown
## Guard re-evaluation is server-side and fail-closed

Authorization at call time uses **live database state**, never a cached ...

```rust
use ferro::write::WriteDispatcher;
...
```

**New section to add** (D-18 — after the existing pipeline steps section):
```markdown
## Double-submit protection for forms

POS-style selection forms require layered protection:

1. **Client guard** — add `disable_on_submit: true` to the confirm Button's props.
   The runtime guard (`data-disable-on-submit`) disables the button after the
   first submission, preventing accidental double-tap on iOS Safari.

2. **Server dedupe** — include a per-render UUID in a hidden input named
   `idempotency_key` in the selection-mutation form. `dispatch_write` step 2
   checks `inputs["idempotency_key"]` against `(tenant_id, key)` and returns
   the stored result without re-executing.

3. **PRG** — redirect after POST so browser back/refresh does not re-POST.
```

The step numbers 2 and 5 referenced in D-17 correspond to the existing pipeline steps documented in the `## \`dispatch_write\`` section (step 2 = idempotency check, step 5 = seal idempotency).

---

## Shared Patterns

### ES5 Runtime Module Convention
**Source:** `ferro-json-ui/src/runtime/product_tiles.rs` (entire file, lines 1–30)
**Apply to:** `runtime/numpad.rs`, `runtime/filters.rs`
```rust
pub(super) const SOURCE: &str = r#"
    // ── [Concern] ─────────────────────────────────────────────────────────
    function setup[Concern]() {
        var items = document.querySelectorAll('[data-...]');
        if (items.length === 0) return;
        for (var i = 0; i < items.length; i++) { init[Concern](items[i]); }
    }
    function init[Concern](el) { ... }
"#;
```
Rules: `var` only, `function` declarations, no `=>`, no `` ` ``, no `let`/`const`, no destructuring, manual for-loops (not `.forEach`/`.map`/`.some`).

### Disabled-State Visual Vocabulary
**Source:** `ferro-json-ui/src/runtime/form_guards.rs` lines 43–46 and 82–85
**Apply to:** `runtime/form_guards.rs` (double-submit guard)
```javascript
btn.setAttribute('disabled', 'disabled');
btn.classList.add('opacity-50', 'cursor-not-allowed');
// to undo:
btn.removeAttribute('disabled');
btn.classList.remove('opacity-50', 'cursor-not-allowed');
```

### Semantic-Token Class Strings
**Source:** `ferro-json-ui/src/runtime/tabs.rs` lines 65–71
**Apply to:** `runtime/filters.rs` (active-tab state)
```javascript
// active: border-primary text-primary font-semibold
// inactive: border-transparent text-text-muted hover:text-text
```
No raw palette classes (`bg-blue-500`, `text-gray-500`, etc.). All class strings must be full, unsplit literals (no string concatenation that creates partial matches).

### Optional Prop → Conditional HTML Attribute
**Source:** `ferro-json-ui/src/render/atoms.rs` `render_button_inner` lines 167–204
**Apply to:** `render/atoms.rs` (`disable_on_submit` emission in `render_button_inner`)
```rust
let some_attr = if props.option_bool_field == Some(true) {
    " data-attribute-name"
} else {
    ""
};
// appears in the format! string as {some_attr}
```

### Input Event Dispatch
**Source:** `ferro-json-ui/src/runtime/product_tiles.rs` line 27
**Apply to:** `runtime/numpad.rs` (every key tap)
```javascript
input.dispatchEvent(new Event('input', { bubbles: true }));
```

---

## No Analog Found

None — all files have clear analogs in the codebase.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/`, `app/src/`, `docs/src/`
**Files scanned:** 17 target files + runtime module siblings (`kanban.rs`, `tabs.rs`, `modals.rs`)
**Pattern extraction date:** 2026-07-05

**Notes for planner:**
- The vocabulary rename (Part A) is a mechanical cascade — the RESEARCH.md cascade inventory is exhaustive with line numbers. The planner should treat each file in that inventory as a separate action with the rename table as the action spec.
- The runtime modules (Part B) have no compile-time verification until wired in `mod.rs`; wire both modules in the same commit as their implementation (Pitfall 2 — drift lists must match the concat list atomically).
- The SC-0 grep gate is the exit criterion for Part A: `grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` must return zero hits.
- `docs/protocol/schemas/*.json` will NOT be dirtied by this phase (V-07 clarification from RESEARCH.md — those files contain ferro-projections protocol schemas, not component props).

# Phase 256: Component Renderers + BUILTIN Lockstep — Research

**Researched:** 2026-07-06
**Domain:** ferro-json-ui builtin component pipeline; vanilla-JS runtime; CSS regen
**Confidence:** HIGH — all findings drawn from direct code reads of the current branch

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Five new builtins (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad) are first-class catalog members; count 47 → 52.
- Tile tap-to-add redesign: tile root is a `<button>` carrying `data-qty-inc`; hidden input moves adjacent; NO on-tile qty display or stepper.
- SelectionPanel is a live client-side view of form state; new `runtime/selection.rs` module with `setupSelection()`; `<template>` for line markup.
- Reconciliation is input-event-driven; delegated click for panel line controls (`data-selection-*`).
- `Grid.row_weights` emits fractional `grid-template-rows` inline style in fill mode.
- One commit per component registration; both count guards bumped in same commit; History comment as audit trail.
- RULE_COMPONENTS extended in same commit that registers TileGrid/SelectionPanel/Numpad.
- Locale-neutral defaults: `FilterTabsProps.all_label` render default is **"All"** (correcting the 254 "Tutte" rustdoc), new aria-labels in neutral English.
- `gen-ferro-base-css.sh` runs ONCE after all five renderers land; commit changed `ferro-base.css`.
- CI-exact gate before every commit: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus `cargo doc` clean.

### Claude's Discretion
- Exact tile markup structure (button root vs wrapper for valid HTML around hidden input).
- Exact `data-selection-*` attribute names; panel template internals; JS money-format helper naming; SelectionPanel display-format prop names.
- Registration order of five components; BUILTIN_SPECS example content.
- Responsive column-class ladder for TileGrid `columns`.
- Aria-label wording (neutral English); backspace/clear key glyphs.
- Whether `design/infer.rs` gains a TileGrid → collect inference branch.

### Deferred Ideas (OUT OF SCOPE)
- Per-line extra columns generic mechanism.
- Sibling FilterTabs↔TileGrid pairing (`data-filter-for`).
- "Uncategorized" virtual sentinel tab.
- Qty badge / picked-state ring on tiles.
- `row_weights` validation lint.
- Barcode keyboard-wedge, payment flow, receipts, shift close.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| POS-01 | TileGrid builtin with responsive tile grid, client-side search, tap-to-add-only tiles | D-16/D-17/D-19 decisions; render_grid child-pipeline pattern; filters.rs contract |
| POS-03 | FilterTabs standalone builtin filtering visible tiles by token, ≥44px targets | D-17 shared helper; filters.rs `updateFilterTabClasses` exact class sets |
| POS-04 | SelectionPanel live view of form state — lines appear as tiles are tapped, per-line stepper+remove, running total, EmptyState | D-06..D-15 decisions; form_guards.rs `form=` attr support verified |
| POS-05 | QuantityStepper standalone builtin on hidden-input contract | D-21/D-22; initQtyButton document-wide lookup pattern |
| POS-06 | Numpad tap-surface keypad ≥56px keys, never native input | D-23; numpad.rs shipped contract to emit against |
| POS-09 | Grid `row_weights` fractional fill-row weighting | D-24; render_grid fill path at containers.rs line 865-870 |
</phase_requirements>

---

## Summary

Phase 256 is implementation-only: all five POS builtin Props structs already exist in `component.rs` (lines 1359–1475), the runtime attribute contracts are locked by Phase 255, and the design rules are locked by Phase 254. The work is writing render functions and wiring them into the BUILTIN_TYPES/dispatch/catalog lockstep.

The central technical challenge is the SelectionPanel live view (POS-04). Its `setupSelection()` runtime must be ES5-only, delegated-event-driven (for template-cloned lines), and safe to wire in alongside the existing 15-module bundle. The tile tap-to-add redesign (D-01) requires resolving a valid-HTML structure: a `<button>` root cannot wrap a `<input type="hidden">`, so the planner needs a wrapper-div structure. All other render functions are straightforward composition of existing class constants.

The Grid `row_weights` inline-style path (D-24) requires conditional emission of `style="grid-template-rows: …"` inside the existing `render_grid`, which already uses inline styles in child cell wrappers. CSS precedence: inline `style` beats the class-applied `auto-rows-fr`. Existing tests for Grid without `row_weights` serve as regression coverage.

**Primary recommendation:** Implement in the D-25 commit-per-component order (TileGrid → FilterTabs → QuantityStepper → Numpad → SelectionPanel); this puts the simpler registrations first and leaves the runtime-heavy SelectionPanel last, where it can depend on tested tile emission.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tile tap-to-add (add-path) | Browser Client | — | Reuses `initQtyButton` document-wide lookup; no server round-trip per tap |
| Tile filter / search | Browser Client | — | `setupFilters` runs entirely in the browser against `data-filter-*` attrs |
| SelectionPanel reconciliation | Browser Client | — | Input-event delegation; panel is a pure view of hidden-input state |
| Running total arithmetic | Browser Client | — | Integer-cents arithmetic on `data-unit-price`; display only, no server |
| Form POST / confirm | API / Backend | — | Single confirm POST; hidden inputs carry final qty state |
| HTML render functions | Frontend Server (SSR) | — | ferro-json-ui render fns emit all markup at server render time |
| Grid row_weights layout | Frontend Server (SSR) | — | Inline style emitted at render time; no JS needed for layout |
| BUILTIN lockstep guards | Frontend Server (SSR) | — | Compile-time count assertions in catalog.rs and ferro-mcp tests |

---

## Q1: Grid `fill: true` row sizing + `row_weights` inline style

**Source:** `ferro-json-ui/src/render/containers.rs` lines 823–870 [VERIFIED: code read]

Current fill-mode path in `render_grid`:
```rust
let fill = props.fill == Some(true) && props.scrollable != Some(true);
// ...
if fill {
    // child wrapper: "min-h-0 h-full overflow-y-auto"
    classes.push("min-h-0 h-full overflow-y-auto".to_string());
}
// ...
if fill {
    col_classes.push_str(" h-full min-h-0 auto-rows-fr");
}
format!("<div class=\"grid w-full {col_classes} {gap}\">{body}</div>")
```

The grid emits `auto-rows-fr` (Tailwind: `grid-auto-rows: 1fr`) giving equal-height rows. `row_weights` is already declared on `GridProps` (component.rs line 919: `pub row_weights: Vec<u8>`) with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.

**Inline style pattern:** `render_kanban_board` (containers.rs lines ~486-488) already emits inline `style=` attributes:
```rust
html.push_str(
    "<div class=\"hidden md:block overflow-x-auto\" \
     style=\"margin-left: -1.5rem; margin-right: -1.5rem; ...\">",
);
```

**D-24 implementation approach for render_grid:**
1. After building `col_classes` (in the fill branch), check `props.row_weights`.
2. If non-empty, build `style="grid-template-rows: 2fr 1fr"` by joining weights as `"{n}fr"`.
3. Emit it as the `style` attribute on the grid `<div>`.
4. `auto-rows-fr` coexists safely — it applies to auto-placed rows beyond the explicit `grid-template-rows` count, and explicit rows take the template values.

**CSS precedence:** Inline `style` beats all stylesheet rules including `auto-rows-fr`. The inline `grid-template-rows` is an explicit property that supersedes the implicit `grid-auto-rows` for explicitly-placed grid items. [VERIFIED: CSS cascade specification behavior]

**Regression guard:** Existing `grid_recurses_children` test (containers.rs line 1429) verifies `grid-cols-2` is present; a new test should assert: (a) with `row_weights: [2,1]` + `fill: true` → `style="grid-template-rows: 2fr 1fr"` present; (b) without `row_weights` → no `style` attr on the outer div.

---

## Q2: Child-render pipeline for TileGrid

**Source:** `ferro-json-ui/src/render/containers.rs` lines 823–849; `render/mod.rs` lines 149–175 [VERIFIED: code read]

Every container renders `el.children` via the standard pipeline:
```rust
let body: String = el
    .children
    .iter()
    .map(|(i, cid)| {
        let rendered = render_element(cid, spec, data, depth + 1);
        // optional per-child wrapper for spans/fill
        rendered
    })
    .collect();
```

`render_element` is the single recursive dispatch function. It handles ID lookup, visibility, and per-type dispatch. **`$each` expansion happens upstream** in the data-binding layer before `render_tile_grid` is called — by the time the renderer runs, `el.children` contains fully-expanded element IDs (one per item). Phase 257 will wire `$each` via `ElementBuilder.each()`; for Phase 256 catalog specs use static children.

**TileGrid render function:** Treats children exactly like Grid does — iterates `el.children`, calls `render_element` on each. The children are expected to be Tile elements (already resolved). The TileGrid wraps this `body` in its outer `data-filter-scope` div, with optional integrated filter strip and search input prepended. New container render functions go in `containers.rs` per the file organization comment (line 1–11).

---

## Q3: BUILTIN registration touchpoints — exact locations

**Source:** multiple files [VERIFIED: code read]

### File 1: `ferro-json-ui/src/render/mod.rs`
- `BUILTIN_TYPES` const at **line 44**: array of `&str` type names, currently 47 entries ending with `"MediaCardGrid"`.
- Dispatch match in `render_element` starting at **line 177**: add one arm per new component.

### File 2: `ferro-json-ui/src/catalog.rs`
- `BUILTIN_SPECS` static array starts at **line ~124** (approximately; search for the array definition).
- Drift guard test at **line 1219**: `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47);`
- History comment at line 1216–1217: `// → 47 (DropdownMenu replaced by ActionGroup).`
- When bumping: update the number AND append `// → 48 (ComponentName)` to the History comment.

### File 3: `ferro-mcp/src/tools/json_ui_catalog.rs`
- `RULE_COMPONENTS` static at **line 81**: currently has `("register-fill-viewport", &["Grid"])`, `("register-grid-fill", &["Grid"])`, `("register-selection-present", &["Grid"])`.
- D-26: extend `register-fill-viewport` to include `"TileGrid"`, `register-grid-fill` to include `"TileGrid"`, `register-selection-present` to include `"SelectionPanel"`, plus `"Numpad"` added to an appropriate rule. Do this in the same commit as registering TileGrid/SelectionPanel/Numpad.
- Count guard at **line 402**: `assert_eq!(catalog.components.len(), 47, ...)`.
- Expected names array at **lines 410–458**: add each new component name to this array.
- Note: the `component_rule_mapping_is_exhaustive` test (line ~530) checks bidirectional: every rule id in `design::rules()` must be in `RULE_COMPONENTS`, and every component name in a mapping must be a real builtin. Adding new builtins to rule mappings is safe; do NOT invent new rule ids.

---

## Q4: HTML assertion test patterns

**Source:** `ferro-json-ui/src/render/atoms.rs` tests module at line 1465 [VERIFIED: code read]

Pattern for a new atom:
```rust
#[test]
fn tile_grid_renders_filter_scope() {
    let spec = spec_with_root(
        Element::new("TileGrid")
            .prop("data_path", "/items")
            .prop("form_id", "cart"),
    );
    let el = spec.elements.get("root").unwrap();
    let html = render_tile_grid(el, &spec, &json!({}), 1);
    assert!(html.contains("data-filter-scope"), "got: {html}");
    assert!(html.contains("grid"), "got: {html}");
    assert!(html.contains("min-h-[44px]"), "HIT_TARGET_MIN; got: {html}");
}
```

For a new container, use `containers.rs` test module's `build_spec` helper (line 1420):
```rust
fn build_spec(elements: Vec<(&str, ElementBuilder)>) -> Spec { ... }
```

**Italian aria-labels to remove** (D-28): current `render_tile` at lines 1404/1408:
- `aria-label=\"Diminuisci quantit\u{00E0} {name}\"`
- `aria-label=\"Aumenta quantit\u{00E0} {name}\"`

These must be replaced with neutral English (e.g. `"Add {name}"`) on the new tap-to-add button.

**The existing `product_tile_legacy_render_is_byte_identical` test** referenced in D-02 of CONTEXT.md: search for it in atoms.rs and delete it as part of the tap-to-add redesign. The test name suggests byte-for-byte comparison of the old markup.

**Catalog render-smoke coverage**: the `BUILTIN_SPECS` render smoke in catalog.rs automatically exercises every new component's example spec through `render_spec_to_html`, so a bogus render function that panics would be caught without writing separate smoke tests.

---

## Q5: `runtime/mod.rs` wiring pattern for `selection.rs`

**Source:** `ferro-json-ui/src/runtime/mod.rs` [VERIFIED: code read]

Current module count: 16 modules (`dismissibles`, `dropdowns`, `filters`, `form_guards`, `hero_lazy`, `kanban`, `modals`, `notifications`, `numpad`, `scroll_preserve`, `sidebar`, `sse`, `tabs`, `tiles`, `toasts`).

Wiring additions for `selection.rs`:
1. `mod selection;` at the top of mod.rs alongside the other `mod` declarations.
2. `s.push_str(selection::SOURCE);` in the `FERRO_RUNTIME_JS` LazyLock body.
3. `setupSelection,` added to the `setups` array inside the `ferroRuntime()` dispatcher string.
4. `"setupSelection"` added to the `bundle_contains_all_setup_functions` test (line ~195) and `dispatcher_invokes_every_setup` test (line ~229).

**ES5 style requirements:** `var`/`function` declarations, no arrow functions `() => {}`, no template literals `` `${x}` ``, no `const`/`let`, no destructuring. `document.querySelectorAll`, `addEventListener`, `getAttribute` patterns only.

**`variant_classes_use_semantic_tokens` scan** (runtime/mod.rs line 84): asserts:
- `FERRO_RUNTIME_JS.contains("bg-primary")` — must be true somewhere in the bundle.
- Does NOT contain `"bg-blue-500"`, `"bg-green-500"`, `"bg-yellow-500"`, `"bg-red-500"`.
- Does NOT contain `"duration-300"`, `"duration-150"`.
- Does contain `"duration-base"`, `"transitionend"`.

Selection.rs must use semantic token classes only. Any CSS class string in selection.rs (e.g. for active line styling) must use `bg-surface`, `text-text`, `border-border` etc. — never raw palette colors.

---

## Q6: `<template>` element — ES5 usage and pitfalls

**Source:** code survey of all runtime modules [VERIFIED: no existing module uses `<template>`]; DOM specification [ASSUMED for browser support]

**ES5-compatible template usage:**
```javascript
var tmpl = document.querySelector('[data-selection-line-template]');
var clone = tmpl.content.cloneNode(true);
// Fill in data:
clone.querySelector('[data-selection-line-name]').textContent = name;
clone.querySelector('[data-selection-line-total]').textContent = lineTotal;
linesContainer.appendChild(clone);
```

`tmpl.content` returns a `DocumentFragment`. `.cloneNode(true)` deep-clones it. `.querySelector` on a `DocumentFragment` works. All of this is property access and method calls — no ES6 syntax needed.

**Key pitfall — template content is inert:** The `<template>` content is NOT in the live DOM tree at load time. `setupTiles()` calls `document.querySelectorAll('[data-qty-inc]')` at load — cloned lines that appear post-load are NOT bound. This is exactly why D-10 uses **delegated** click handling on the panel root instead of per-button binding. Delegation works correctly for post-load elements.

**`<template>` element** is DOM Level 4, supported in all current browsers including iOS Safari 8+. No polyfill needed. [ASSUMED: training knowledge, not separately verified in this session]

---

## Q7: `initQtyButton` — min/max/step extension (D-22)

**Source:** `ferro-json-ui/src/runtime/tiles.rs` [VERIFIED: code read]

Current `initQtyButton(btn, delta)`:
- Reads `field` from `data-qty-inc` or `data-qty-dec` attribute on `btn`.
- Document-wide query: `document.querySelector('[data-qty-display="' + field + '"]')` and `document.querySelector('[data-qty-input="' + field + '"]')`.
- Clamps at 0 (`if (next < 0) next = 0;`). No max. No step.

**D-22 extension:** Emit `data-qty-min/-max/-step` on the inc/dec buttons (or on the QuantityStepper wrapper). `initQtyButton` reads them from `btn`:
```javascript
var step = parseInt(btn.getAttribute('data-qty-step'), 10) || 1;
var min = parseInt(btn.getAttribute('data-qty-min'), 10) || 0;
var rawMax = btn.getAttribute('data-qty-max');
var max = rawMax !== null ? parseInt(rawMax, 10) : Infinity;
var next = Math.min(Math.max(current + delta * step, min), max);
```

**Tile root as `data-qty-inc`:** The tile root in the tap-to-add redesign IS a `<button>` carrying `data-qty-inc`. `setupTiles` does `document.querySelectorAll('[data-qty-inc]')` — this matches ANY element with the attribute, not just `<button>`. So a `<button>` or a `<div>` both work with the current selector. The tap-to-add `<button>` works correctly. [VERIFIED: tiles.rs code — `querySelector` is attribute-based, not tag-restricted]

---

## Q8: Valid-HTML tile structure for tap-to-add

**Source:** D-01 text in 256-CONTEXT.md; HTML specification [ASSUMED: training knowledge]; atoms.rs existing tile at line 1395 [VERIFIED: code read]

Current render_tile (line 1395) uses a `<div>` root. D-01 says to make the root a `<button>` carrying `data-qty-inc`, but notes inputs inside buttons are invalid HTML and says the hidden input "moves adjacent within a wrapper".

**Recommended concrete structure:**
```html
<div
  class="..."
  data-filter-text="{name}"
  data-unit-price="{price_cents}"
  data-filter-tokens="{tokens}"
>
  <button
    type="button"
    data-qty-inc="{field}"
    class="{HIT_TARGET_MIN} {TOUCH_ACTION} {PRESS_ACTIVE} {TAP_HIGHLIGHT} rounded-lg w-full h-full flex flex-col gap-2 p-3"
    aria-label="Add {name}"
  >
    <!-- image_url rendering if present -->
    <!-- name + price display -->
    <!-- stock_badge chip if present -->
  </button>
  <input type="hidden" name="{field}" data-qty-input="{field}" value="{qty}">
</div>
```

**Why this structure works:**
- `data-filter-text` on the outer `<div>` → filters.rs `applyFilter` selects `[data-filter-text]` and sets `style.display` on the div. The whole tile hides/shows correctly.
- `data-qty-inc` on the `<button>` → `initQtyButton` binds to it via `document.querySelectorAll('[data-qty-inc]')`.
- `data-qty-input` on the sibling `<input>` → `initQtyButton` finds it via `document.querySelector('[data-qty-input="' + field + '"]')` (document-wide, not scoped to button).
- `data-unit-price` on the outer div → selection.rs reads via `closest('[data-filter-text]').getAttribute('data-unit-price')` — but D-09 says "the runtime resolves the event's input → tile root (`closest`), reads... `data-unit-price`". The tile root here is the outer `<div>` which has `data-filter-text`. The selection runtime should do `input.closest('[data-filter-text]').getAttribute('data-unit-price')` — requires the hidden input to be INSIDE the wrapper div (which it is). ✓
- Hidden input is NOT inside the `<button>` (valid HTML). ✓
- D-04: `TileProps.price_cents: Option<u64>` already declared in component.rs. Rendered as `data-unit-price="{cents}"` on the tile root div.

---

## Q9: EmptyState, Badge, image class vocabulary; `Tone` enum

**Source:** `ferro-json-ui/src/render/atoms.rs` lines 659–703 (EmptyState), atoms.rs imports (Tone, Badge); `ferro-json-ui/src/component.rs` [VERIFIED: code read]

**EmptyState classes:**
```
Outer: "rounded-lg border border-border bg-card min-h-40 py-8 px-6 flex items-center justify-center"
Inner: "text-center max-w-md"
Title: "text-base font-semibold text-text mb-2"
Desc:  "text-sm text-text-muted"
CTA:   "mt-4 inline-flex items-center justify-center rounded-md border border-border bg-card text-text px-4 py-2 text-sm font-medium hover:bg-surface {INTERACTIVE_BASE}"
```

For the panel EmptyState, D-13 says "reuse the existing EmptyState markup vocabulary." The panel needs a lightweight inline toggle (not a full bordered card). Use `render_empty_state` pattern but without the outer card shell — or emit a `<div data-selection-empty>` that wraps the same inner `text-center` structure. Runtime toggles visibility via `style.display`.

**`Tone` enum** (component.rs): `Neutral | Success | Warning | Destructive`. D-03 color mapping uses an exhaustive match on `Tone` → full-literal accent classes. Tone values map to accent color classes:
```rust
match color_tone {
    Tone::Neutral  => "",                          // default border, no accent
    Tone::Success  => "border-success bg-success/10",
    Tone::Warning  => "border-warning bg-warning/10",
    Tone::Destructive => "border-destructive bg-destructive/10",
}
```
If planning determines `Tone` is wrong for tile accents (D-03 fallback): drop `color` rendering this phase.

**Stock badge chip:** Use Badge-style vocabulary. From `render_badge` output patterns (atoms.rs): `inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium`. Badge tone classes:
- Neutral: `bg-secondary/10 text-secondary-foreground`
- Success: `bg-success/10 text-success`
- Warning: `bg-warning/10 text-warning`
- Destructive: `bg-destructive/10 text-destructive`

Stock badge is a label string on a chip overlay. Position: absolute or top-right of the image area (planner's call).

**Image handling:** `render_image` uses `object-cover`. For tile image: render `<img src="{url}" alt="{name}" class="w-full aspect-square object-cover rounded-t-lg" loading="lazy">` (aspect planner's call). Absent `image_url` → text-only layout.

---

## Q10: `form_guards.rs` — ButtonProps.form + disable_on_submit outside form

**Source:** `ferro-json-ui/src/runtime/form_guards.rs` lines 29–46 [VERIFIED: code read]

`initDisableOnSubmit(btn)`:
```javascript
var form = btn.closest('form');
if (!form && btn.getAttribute('form')) {
    form = document.getElementById(btn.getAttribute('form'));
}
if (!form) return;
```

**Confirmed:** A `<button data-disable-on-submit form="{form_id}">` placed OUTSIDE the form (e.g. in the panel footer) IS correctly wired. `btn.closest('form')` returns null, then fallback to `document.getElementById(btn.getAttribute('form'))` finds the form. ✓

`findGuardedSubmit(form)` (lines 52–61):
```javascript
function findGuardedSubmit(form) {
    var inside = form.querySelector('button[type="submit"]');
    if (inside) return inside;
    if (form.id) {
        return document.querySelector('button[type="submit"][form="' + form.id + '"]');
    }
    return null;
}
```

**Confirmed:** For `number-gt-0` guard on the tile grid form, `findGuardedSubmit` finds the confirm `<button type="submit" form="{form_id}">` outside the form via the second branch. ✓

**`render_button` for form attribute** (atoms.rs lines 196–200):
```rust
let form_attr = match props.form.as_deref() {
    Some(f) => format!(" form=\"{}\"", html_escape(f)),
    None => String::new(),
};
```
And `disable_on_submit_attr` at lines 204–208 emits `" data-disable-on-submit"` when `disable_on_submit == Some(true)`. Both confirmed working. ✓

---

## Q11: `scripts/gen-ferro-base-css.sh` mechanics; new arbitrary classes in input.css

**Source:** `scripts/gen-ferro-base-css.sh` [VERIFIED: code read]; `ferro-json-ui/assets/input.css` [VERIFIED: code read]

The script runs:
```bash
.tooling/bin/tailwindcss -i ferro-json-ui/assets/input.css -o ferro-json-ui/assets/ferro-base.css --minify
```

The Tailwind scanner sources `ferro-json-ui/src/**/*.rs` (via implicit config or `@source`). Full-string literals in Rust source are scanned automatically.

**Phase 254/255 classes already in input.css or classes.rs:**
- `pos-tap-highlight` → `@utility pos-tap-highlight { -webkit-tap-highlight-color: transparent; }` in input.css line 103.
- `min-h-[44px] min-w-[44px]` → `HIT_TARGET_MIN` constant in classes.rs (scanner picks it up as a literal string value).
- `min-h-[56px] min-w-[56px]` → `HIT_TARGET_NUMPAD` in classes.rs.
- `active:scale-95`, `active:bg-border` → `PRESS_ACTIVE` in classes.rs.
- `overscroll-contain` → `OVERSCROLL_CONTAIN` in classes.rs.
- `touch-manipulation` → `TOUCH_ACTION` in classes.rs.

**Phase 256 new arbitrary classes to verify:**
- `grid-template-rows: 2fr 1fr` — emitted as an inline `style=` attribute, NOT a Tailwind class. No scanner concern. ✓
- TileGrid column classes (e.g. `grid-cols-2`, `grid-cols-3`, `grid-cols-4`): if emitted via full-literal exhaustive match arms, the scanner picks them up. If via `format!("grid-cols-{}", n)`, they need `@source inline()` in input.css. **Use exhaustive match** (already in the grid-cols `@source inline` line if needed, but full literals are cleaner).
- Selection.rs CSS class strings: any class in a JS string literal in selection.rs must be a complete full string (e.g. `'text-text-muted'`, `'border-border'`) that the scanner picks up from the Rust source file (since SOURCE is a `&str` constant in a `.rs` file).
- `aspect-square` for tile image: full literal in atoms.rs, scanner picks up. ✓

**Regen trigger (D-29):** Run `scripts/gen-ferro-base-css.sh` once after all five renderers are in tree. Commit changed `ferro-base.css`. Classes that appear as full-string literals in Rust source do NOT need safelist entries.

---

## Q12: Schema export churn — D-30 analysis

**Source:** `ferro-projections/tests/generate_schemas.rs` [VERIFIED: code read]; D-30 in 256-CONTEXT.md

`ferro-projections/tests/generate_schemas.rs` generates `docs/protocol/schemas/*.json` from `ferro-projections` types: `ServiceDef`, `FieldDef`, `DataType`, `FieldMeaning`, `StateMachine`, `StateDef`, `Transition`, `Warning`, `ActionDef`, `InputDef`, `GuardDef`, `RelationshipDef`, `Cardinality`, `NavigationHint`, `Intent`, `IntentScore`, `IntentHint`.

**Phase 256 changes:** All new props (`TileProps.price_cents`, `SelectionPanelProps` display props, `TileGridProps`, `FilterTabsProps`, `QuantityStepperProps`, `NumpadProps`) are `ferro-json-ui` types. They are NOT `ferro-projections` types. Therefore:
- `docs/protocol/schemas/*.json` should have NO real content changes from Phase 256 work.
- D-30 saying "regenerate with REAL changes (new props: price_cents, panel display props)" appears to be an overstated expectation — those props are in ferro-json-ui, not ferro-projections.

**Practical implication:** Run `cargo test --all-features`. If `docs/protocol/schemas/*.json` has churn (content changes), check whether the diff is real (different schema content) or just whitespace/ordering. If NO real content changes: apply the usual discard rule (`git checkout docs/protocol/schemas/`). If real changes: commit them. Verify before committing.

**Risk:** LOW — the schema export behavior is well-understood from prior phases (MEMORY.md `project_schema_export_test_dirties_tree.md`).

---

## Standard Stack

### Core (ferro-json-ui only — no new crates)

| File | Role | Key Constants/Functions |
|------|------|------------------------|
| `ferro-json-ui/src/component.rs` | Props structs for all 5 new components | TileGridProps:1394, SelectionPanelProps:1413, FilterTabsProps:1426, QuantityStepperProps:1439, NumpadProps:1465 |
| `ferro-json-ui/src/render/atoms.rs` | Leaf renderers | `render_tile_grid` (new), `render_filter_tabs` (new), `render_quantity_stepper` (new), `render_numpad` (new); existing `render_tile` redesigned |
| `ferro-json-ui/src/render/containers.rs` | Container renderers | `render_selection_panel` (new); `render_grid` extended for row_weights |
| `ferro-json-ui/src/render/classes.rs` | Touch constants | `TOUCH_ACTION`, `HIT_TARGET_MIN`, `HIT_TARGET_NUMPAD`, `PRESS_ACTIVE`, `OVERSCROLL_CONTAIN`, `TAP_HIGHLIGHT`, `INTERACTIVE_BASE` |
| `ferro-json-ui/src/render/mod.rs` | Dispatch + BUILTIN_TYPES | Line 44: BUILTIN_TYPES; line 177: dispatch match |
| `ferro-json-ui/src/catalog.rs` | BUILTIN_SPECS + count guard | Line ~1219: count assertion; line 1216: History comment |
| `ferro-json-ui/src/runtime/tiles.rs` | qty button runtime | `initQtyButton` extended with min/max/step |
| `ferro-json-ui/src/runtime/selection.rs` | NEW: selection panel runtime | `setupSelection()` |
| `ferro-json-ui/src/runtime/mod.rs` | Bundle assembly | LazyLock + dispatcher + 2 drift tests |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | Mirror count + RULE_COMPONENTS | Line 81: RULE_COMPONENTS; line 402: count; line 410: names array |

**Installation:** No new dependencies. All work within the existing workspace.

---

## Architecture Patterns

### System Architecture: Selection Panel Data Flow

```
Tile DOM (data-filter-text / data-unit-price)
    ↓ tap
<button data-qty-inc="{field}"> [tile]
    ↓ initQtyButton (click → increment hidden input)
<input data-qty-input="{field}" value="N">
    ↓ dispatchEvent('input', {bubbles:true})
setupSelection() delegated listener on [data-form-id]
    ↓
applySelection(): for each [data-qty-input] in form
    → qty > 0: ensure line exists, update qty + line-total
    → qty = 0: remove line
    → always: recompute running total from all lines
    → toggle EmptyState visibility
    ↓
SelectionPanel DOM updated (line items + total display)
    ↓ (user taps confirm)
<button type="submit" form="{form_id}" data-disable-on-submit>
    ↓ form submit (all hidden inputs included)
POST /cassa/conferma
```

### Recommended Project Structure Changes

```
ferro-json-ui/src/
├── render/
│   ├── atoms.rs         # + render_tile_grid, render_filter_tabs,
│   │                    #   render_quantity_stepper, render_numpad
│   │                    #   (render_tile REDESIGNED)
│   ├── containers.rs    # + render_selection_panel, render_grid EXTENDED
│   └── classes.rs       # (unchanged — constants already present)
└── runtime/
    ├── selection.rs     # NEW: setupSelection()
    ├── tiles.rs         # EXTENDED: initQtyButton + min/max/step
    └── mod.rs           # EXTENDED: +selection module, +dispatcher entry, +test entries
```

**Where new render functions go:**
- `render_tile_grid`, `render_filter_tabs`, `render_quantity_stepper`, `render_numpad` → `atoms.rs` (they are leaves: no child element rendering via `render_element`). TileGrid renders children via `render_element` making it a container, but may live in atoms.rs with a child loop — or containers.rs per the file organization. Given TileGrid IS a container (iterates children), `containers.rs` is more correct.
- `render_selection_panel` → `containers.rs` (renders children into the confirm slot).

### Pattern: BUILTIN Registration Commit Sequence (D-25)

Each component addition requires exactly these changes in ONE commit:
1. `component.rs`: Props struct exists (all 5 already done in Phase 254/255). Phase 256 adds `TileProps.price_cents` field.
2. `render/{atoms,containers}.rs`: Add `pub(crate) fn render_{name}(el, spec, data, depth) -> String`.
3. `render/mod.rs`: Add type name to `BUILTIN_TYPES` array AND add match arm in dispatch.
4. `catalog.rs`: Add import to `use crate::component::{...}` AND add entry to `BUILTIN_SPECS` static array AND bump count in line 1219 AND append to History comment.
5. `ferro-mcp/src/tools/json_ui_catalog.rs`: Bump count AND add name to `expected` array AND extend RULE_COMPONENTS if applicable.

### Pattern: ES5 Runtime Module

```rust
// ferro-json-ui/src/runtime/selection.rs
pub(super) const SOURCE: &str = r#"
    // ── Selection panel — live cart view ──────────────────────────────────
    //
    // Attribute contract (D-06..D-15):
    //   [data-selection-panel]                  — panel root
    //   [data-selection-form="{form_id}"]       — scope isolator
    //   [data-selection-line-template]          — <template> for line markup
    //   [data-selection-lines]                  — lines container (scrollable)
    //   [data-selection-empty]                  — EmptyState (toggled)
    //   [data-selection-total]                  — running total display
    //   [data-selection-inc="{field}"]          — per-line inc button
    //   [data-selection-dec="{field}"]          — per-line dec button
    //   [data-selection-remove="{field}"]       — per-line remove button
    //   [data-filter-text]                      — tile root (for name + price)
    //   [data-unit-price]                       — cents on the tile root
    //   [data-qty-input="{field}"]              — form hidden input

    function setupSelection() {
        var panels = document.querySelectorAll('[data-selection-panel]');
        if (panels.length === 0) return;
        for (var i = 0; i < panels.length; i++) {
            initSelectionPanel(panels[i]);
        }
    }
    // ... (implementation follows D-07..D-15)
"#;
```

### Anti-Patterns to Avoid

- **`format!("grid-cols-{}", n)` in render functions:** Full-literal exhaustive match only. Use `match props.columns { Some(2) => "grid-cols-2", Some(3) => "grid-cols-3", _ => "grid-cols-2" }`.
- **`format!("bg-{}", color)` for tile color:** Exhaustive match on `Tone` → full-literal classes. If Tone is wrong, drop the feature.
- **`data-qty-inc/dec` on cloned selection lines:** These bind per-element at load time via `setupTiles`. Post-load clones won't be bound. Use `data-selection-inc/dec` with panel-level delegation.
- **Bumping only one count guard:** Both `catalog.rs:1219` and `ferro-mcp/src/tools/json_ui_catalog.rs:402` must be bumped in the SAME commit. Never split.
- **Inline Italian strings in ferro-json-ui renderers:** All user-visible defaults must be neutral English or prop-configurable (D-28).
- **Inputs inside `<button>`:** Invalid HTML. Hidden input goes adjacent, outside the button.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tab/filter active class management | Custom CSS toggle JS | `updateFilterTabClasses` in `filters.rs` (already shipped) | Already matches the exact class set the tab-strip helper must emit |
| Qty button event binding | New click listener for selection lines | `data-selection-*` delegated events + `initQtyButton` for tile tap | `initQtyButton` already handles document-wide hidden input lookup; selection lines need delegation |
| Integer-cents money display | Float math | Integer arithmetic + `numpadPriceDisplay` pattern from `numpad.rs` | Float money is forbidden (PITFALLS.md); the pattern is already implemented |
| Tailwind scan of new classes | `@source inline()` entries | Full-literal match arms in Rust | Scanner picks up literal strings in `.rs` files; safelist only for `format!()`-built classes |
| Focus ring / motion | Per-component constants | `INTERACTIVE_BASE`, `MOTION_FAST`, `FOCUS_RING` from `classes.rs` | Composition drift-guard test (`render_functions_use_constants_not_literals`) enforces this |

---

## Common Pitfalls

### Pitfall 1: Bumping Only One Count Guard
**What goes wrong:** `catalog.rs:1219` passes but `ferro-mcp/src/tools/json_ui_catalog.rs:402` fails (or vice versa). Both tests fail independently; CI fails on the MCP crate.
**Prevention:** Both count bumps in the SAME commit per D-25. Never batch multiple component additions into one count bump.
**Warning signs:** A green `cargo test --all-features` on `ferro-json-ui` while failing on `ferro-mcp`.

### Pitfall 2: `data-qty-inc/dec` on Template-Cloned Lines
**What goes wrong:** `setupTiles()` runs at DOMContentLoaded and binds to all `[data-qty-inc]` elements present at that time. Template-cloned lines inserted post-load have no binding → stepper buttons are silent.
**Prevention:** Use `data-selection-inc/dec` with panel-root delegation (D-10). These never use `data-qty-inc/dec`. The tile itself has `data-qty-inc` (bound at load); the panel line has `data-selection-inc/dec` (delegated).

### Pitfall 3: Dynamic Class Construction in render functions
**What goes wrong:** `format!("grid-cols-{}", columns)` → class absent from `ferro-base.css` → silent layout break in production (the Phase 253 WR-01 col-span bug class).
**Prevention:** Exhaustive match on bounded u8 range → full-literal class strings. Audit with `grep -rn 'format!(".*-{}' ferro-json-ui/src/'` → zero unaccounted matches (SC-3 success criterion).

### Pitfall 4: Tile root carries `data-filter-text`; selection runtime needs it for name
**What goes wrong:** If the filter-text container and the tile button are the same element, hiding the tile (via `style.display = 'none'`) also hides the button. But with the wrapper-div structure, `data-filter-text` is on the outer div — so the selection runtime can do `input.closest('[data-filter-text]')` to get the tile root and read its name and unit price.
**Prevention:** Ensure `data-filter-text` and `data-unit-price` are both on the outer wrapper div (which is what the filter runtime hides/shows). The hidden input must be inside this wrapper (sibling to the button) so `closest('[data-filter-text]')` from the input finds the right tile.

### Pitfall 5: FilterTabs all_label "Tutte" vs "All" (D-28)
**What goes wrong:** The FilterTabsProps rustdoc (component.rs line 1434) still says `Phase 256 render default is "Tutte"`. This was a pre-neutralization note. The actual render default must be **"All"** per D-28.
**Prevention:** When writing `render_filter_tabs`, emit `props.all_label.as_deref().unwrap_or("All")`. Also correct the rustdoc comment.

### Pitfall 6: Forgetting RULE_COMPONENTS same-commit rule (D-26)
**What goes wrong:** BUILTIN_TYPES added (count passes) but RULE_COMPONENTS not extended → `component_rule_mapping_is_exhaustive` test fails because mapped component names must be real builtins and a real builtin may now be missing from a relevant rule.
**Prevention:** In the commit that registers TileGrid: extend `register-fill-viewport` and `register-grid-fill` to include `"TileGrid"`. In the commit that registers SelectionPanel: extend `register-selection-present` to include `"SelectionPanel"`. Check what `REGISTER_TRIGGER_TYPES` in `design/rules.rs` already names.

---

## Code Examples

### Grid row_weights emission (D-24)

```rust
// containers.rs render_grid — in the fill branch
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
format!("<div class=\"grid w-full {col_classes} {gap}\"{row_style}>{body}</div>")
```

### Filter tab strip shared helper (D-17)

The tab-strip markup must emit the exact inactive-state classes that `updateFilterTabClasses` removes on activation:
- Inactive: `border-transparent text-text-muted hover:text-text` ← must be the initial state
- Active: `border-primary text-primary font-semibold` ← applied by JS on click

```rust
// Rendered by the shared tab-strip helper called from both render_filter_tabs
// and the integrated strip inside render_tile_grid
fn render_filter_tab_strip(items: &[String], all_label: &str) -> String {
    let mut html = String::from(
        "<div class=\"flex overflow-x-auto\" role=\"tablist\">"
    );
    // "All" tab first (empty token value)
    html.push_str(&format!(
        "<button type=\"button\" role=\"tab\" data-filter-tab=\"\" \
         class=\"{HIT_TARGET_MIN} px-4 text-sm font-medium border-b-2 \
                border-primary text-primary font-semibold {TOUCH_ACTION} {INTERACTIVE_BASE}\" \
         aria-selected=\"true\">{}</button>",
        html_escape(all_label)
    ));
    for item in items {
        let token = item.replace(' ', "-");
        html.push_str(&format!(
            "<button type=\"button\" role=\"tab\" data-filter-tab=\"{}\" \
             class=\"{HIT_TARGET_MIN} px-4 text-sm font-medium border-b-2 \
                    border-transparent text-text-muted hover:text-text {TOUCH_ACTION} {INTERACTIVE_BASE}\" \
             aria-selected=\"false\">{}</button>",
            html_escape(&token),
            html_escape(item)
        ));
    }
    html.push_str("</div>");
    html
}
```

### Selection runtime initial reconciliation (D-07)

```javascript
function setupSelection() {
    var panels = document.querySelectorAll('[data-selection-panel]');
    if (panels.length === 0) return;
    for (var i = 0; i < panels.length; i++) {
        initSelectionPanel(panels[i]);
    }
}

function initSelectionPanel(panel) {
    var formId = panel.getAttribute('data-selection-form');
    var form = formId ? document.getElementById(formId) : document;
    if (!form) form = document;
    var tmpl = panel.querySelector('[data-selection-line-template]');
    var linesEl = panel.querySelector('[data-selection-lines]');
    var emptyEl = panel.querySelector('[data-selection-empty]');
    var totalEl = panel.querySelector('[data-selection-total]');

    // D-07: listen for input events bubbling from any [data-qty-input] in form scope
    form.addEventListener('input', function(e) {
        if (e.target && e.target.getAttribute('data-qty-input')) {
            reconcile();
        }
    });

    // D-10: delegated click for per-line controls
    panel.addEventListener('click', function(e) {
        var incBtn = e.target.closest('[data-selection-inc]');
        var decBtn = e.target.closest('[data-selection-dec]');
        var remBtn = e.target.closest('[data-selection-remove]');
        if (incBtn) {
            var field = incBtn.getAttribute('data-selection-inc');
            var input = form.querySelector('[data-qty-input="' + field + '"]');
            if (input) { input.value = parseInt(input.value, 10) + 1; input.dispatchEvent(new Event('input', {bubbles: true})); }
        }
        // (similar for dec/remove — dec to 0 triggers remove via reconcile)
    });

    // D-07: initial pass
    reconcile();

    function reconcile() { /* ... */ }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ProductTile` with on-tile +/- stepper | `Tile` as button (tap-to-add) + SelectionPanel for quantity editing | Phase 256 | Simpler tile render; panel is the single quantity-edit surface |
| Static BUILTIN_TYPES count hardcoded in multiple places | Canonical count in catalog.rs + documented mirror in ferro-mcp | Phase 253 consolidation | Single bump per addition; History comment is audit trail |
| Italian aria-labels in render_tile ("Diminuisci/Aumenta quantità") | Neutral English defaults (D-28) | Phase 256 | Project-agnostic crates principle |
| Tile with qty display overlay | No qty display on tile (on-tile qty badge DEFERRED) | Phase 256 | Operator interaction model change 2026-07-05 |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `<template>` element `.content.cloneNode(true)` works in ES5 syntax | Q6 | Low — DOM Level 4 property access is not ES6 syntax |
| A2 | CSS inline `style` beats `auto-rows-fr` class for grid-template-rows | Q1 | Low — CSS cascade is unambiguous: inline style wins over stylesheet |
| A3 | Phase 256 changes do NOT affect `docs/protocol/schemas/*.json` content | Q12 | Medium — if D-30 is correct and something I missed causes real changes, the discard rule would miss them |
| A4 | Tailwind scanner picks up full-literal string values inside `const SOURCE: &str` in `.rs` files | Q11 | Medium — if the scanner doesn't source `.rs` files in the runtime/ dir, classes in JS strings need safelist entries |

**Note on A4:** If the Tailwind v4 scanner has a `.rs` source configured, it will pick up string literals in `runtime/*.rs`. The existing `filters.rs` classes (e.g. `'border-primary'`, `'text-text-muted'`) are already in the generated CSS, confirming the scanner does source `.rs` files. So selection.rs class literals will also be scanned. [HIGH confidence based on existing runtime modules working]

---

## Open Questions (RESOLVED)

1. **D-03: Is `Tone` the right vocabulary for tile accent color?** — RESOLVED: `TileProps.color` becomes `Option<Tone>` with an exhaustive match to full-literal accent classes (Plan 01 Tasks 1–2).
   - What we know: `TileProps.color: Option<String>` is declared as an arbitrary string, not a Tone. An exhaustive match would need to parse the string to a Tone value.
   - What's unclear: should the type be `Option<Tone>` (enforce the enum at the type boundary) or remain `Option<String>` with a runtime match?
   - Recommendation: Change `TileProps.color` to `Option<Tone>` in the same commit as writing `render_tile` (no breaking change since not published). If planning determines Tone is wrong, drop `color` rendering.

2. **TileGrid: atoms.rs or containers.rs?** — RESOLVED: `render_tile_grid` lives in `containers.rs` (recurses children); `render_filter_tabs`/`render_quantity_stepper`/`render_numpad` live in `atoms.rs` (Plans 02–03).
   - What we know: TileGrid iterates `el.children` via `render_element` → it IS a container.
   - What's unclear: the file organization comment says atoms = leaves (no recursive children).
   - Recommendation: Put `render_tile_grid` in `containers.rs`. Put `render_filter_tabs`, `render_quantity_stepper`, `render_numpad` in `atoms.rs` (they don't recurse children).

3. **D-26: Which rule should Numpad be added to?** — RESOLVED: Numpad is appended to `register-selection-present` in its registering commit (Plan 03 Task 2).
   - What we know: `RULE_COMPONENTS` has `register-fill-viewport`, `register-grid-fill`, `register-selection-present`. The Numpad is used inside a `SelectionPanel` or standalone for price entry.
   - Recommendation: `register-selection-present` maps to the overall register composition; Numpad could be added there alongside SelectionPanel. Or leave Numpad unmapped (no rule references it). Verify `component_rule_mapping_is_exhaustive` test semantics — it may only require that MAPPED component names be real builtins, not that every builtin is mapped.

---

## Environment Availability

> Phase has no new external dependencies — purely code/config changes within the existing workspace. Step 2.6: SKIPPED.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config | No separate config file; `[profile.test]` in workspace Cargo.toml |
| Quick run | `cargo test -p ferro-json-ui -- --test-thread=1 2>&1 \| head -50` |
| Full suite | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | File |
|--------|----------|-----------|------|
| POS-01 | TileGrid emits data-filter-scope, search input | unit | `ferro-json-ui/src/render/containers.rs` tests |
| POS-01 | TileGrid renders children via render_element | unit | containers.rs tests |
| POS-03 | FilterTabs emits `data-filter-tab=""` (All) + item tabs | unit | atoms.rs tests |
| POS-03 | FilterTabs tab inactive-state class set matches updateFilterTabClasses | unit | atoms.rs tests |
| POS-04 | SelectionPanel emits data-selection-panel, data-selection-form | unit | atoms.rs or containers.rs tests |
| POS-04 | SelectionPanel emits `<template data-selection-line-template>` | unit | atoms.rs or containers.rs tests |
| POS-04 | setupSelection wired in FERRO_RUNTIME_JS | unit | runtime/mod.rs `bundle_contains_all_setup_functions` |
| POS-04 | setupSelection in dispatcher | unit | runtime/mod.rs `dispatcher_invokes_every_setup` |
| POS-05 | QuantityStepper emits dec/display/inc/input with correct attrs | unit | atoms.rs tests |
| POS-05 | min-h-[44px] on stepper buttons | unit | HTML assertion in test |
| POS-06 | Numpad emits data-numpad, data-numpad-target, 12 keys | unit | atoms.rs tests |
| POS-06 | Numpad keys min-h-[56px] | unit | HTML assertion in test |
| SC-1 | BUILTIN_TYPES.len() == 52 | unit | catalog.rs `builtin_types_count_drift_guard` |
| SC-1 | ferro-mcp catalog count == 52 | unit | json_ui_catalog.rs `test_all_components_present` |
| SC-2 | All POS interactive elements have min-h-[44px] | unit | per-component HTML assertions |
| SC-3 | grep of format!() dynamic classes returns zero | manual | pre-commit check |
| SC-4 | row_weights emits fractional grid-template-rows | unit | containers.rs regression test |
| SC-5 | FilterTabs + TileGrid categories_path composition | unit | containers.rs integration test |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui --all-features 2>&1 | tail -5`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full CI-exact gate green (fmt + clippy --all-features + test --all-features + doc) before `/gsd-verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. New tests are written inline in the wave that implements each component.

---

## Security Domain

> This phase adds render functions (HTML generation) and vanilla JS. Applicable ASVS categories:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation / Output Encoding | YES | `html_escape()` on all props values (existing pattern) |
| V2 Authentication | No | No new auth surface |
| V3 Session Management | No | No session changes |
| V4 Access Control | No | No new access control |
| V6 Cryptography | No | No new crypto |

### Known Threat Patterns for HTML rendering

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via unescaped prop values | Tampering | `html_escape()` on every `props.*` string interpolated into HTML (enforced by atoms.rs pattern: `decode_diagnostic` + `html_escape` on every field) |
| XSS via `data-unit-price` if price_cents is not u64 | Tampering | `price_cents: Option<u64>` — u64 serializes as a number, `html_escape` on the formatted string |
| Double-submit via rapid tap | Tampering | `data-disable-on-submit` + `setupFormGuards` (shipped in Phase 255) |

---

## Sources

### Primary (HIGH confidence — direct code reads)
- `ferro-json-ui/src/render/containers.rs` lines 798–870 — Grid fill-mode, row sizing, child-render pipeline
- `ferro-json-ui/src/render/mod.rs` lines 44–96, 177–232 — BUILTIN_TYPES, dispatch
- `ferro-json-ui/src/render/classes.rs` — all POS touch constants + composition drift-guard
- `ferro-json-ui/src/render/atoms.rs` lines 1363–1413 — current render_tile; lines 1465+ — test patterns
- `ferro-json-ui/src/runtime/tiles.rs` — `initQtyButton` full implementation
- `ferro-json-ui/src/runtime/filters.rs` — `setupFilters` + `updateFilterTabClasses` full implementation
- `ferro-json-ui/src/runtime/numpad.rs` — numpad attribute contract
- `ferro-json-ui/src/runtime/form_guards.rs` — `initDisableOnSubmit` + `findGuardedSubmit` (form= attr support)
- `ferro-json-ui/src/runtime/mod.rs` — LazyLock + dispatcher + 2 drift tests
- `ferro-json-ui/src/catalog.rs` lines 1211–1219 — count guard + History comment
- `ferro-json-ui/src/component.rs` lines 1359–1475 — all 5 new Props structs + TileProps (with image_url/color/stock_badge/categories already present)
- `ferro-mcp/src/tools/json_ui_catalog.rs` lines 81–103, 391–462 — RULE_COMPONENTS + count guard + expected names
- `ferro-json-ui/assets/input.css` lines 70–119 — safelist + @utility declarations
- `scripts/gen-ferro-base-css.sh` — regen script mechanics
- `ferro-projections/tests/generate_schemas.rs` — schema export scope (ferro-projections types only)

### Secondary (MEDIUM confidence)
- `.planning/research/STACK.md` — cart runtime module design, hit target standards, vanilla-JS patterns
- `.planning/research/PITFALLS.md` — count lockstep, safelist drift, token bypass patterns
- `.planning/research/ARCHITECTURE.md` — lockstep checklist, integration points
- `256-CONTEXT.md` — all decisions (D-01 through D-31)

### Tertiary (LOW confidence)
- D-30 claim about `docs/protocol/schemas/*.json` changes — assessed as likely incorrect; needs runtime verification

---

## Metadata

**Confidence breakdown:**
- BUILTIN registration touchpoints: HIGH — verified from code with exact line numbers
- render_grid row_weights: HIGH — fill mode code fully read, inline style pattern confirmed in kanban
- selection.rs runtime design: HIGH on structure (ES5, delegation, template); MEDIUM on exact JS (not yet written)
- form= button outside form: HIGH — form_guards.rs code confirms the fallback branch
- Schema export churn (D-30): LOW — likely no real changes, but D-30 contradicts code evidence

**Research date:** 2026-07-06
**Valid until:** 2026-08-06 (stable codebase; no external library changes)

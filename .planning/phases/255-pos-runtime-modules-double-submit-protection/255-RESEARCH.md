# Phase 255: POS Runtime Modules + Double-Submit Protection — Research

**Researched:** 2026-07-05
**Domain:** ferro-json-ui vocabulary rename + JavaScript runtime modules + ButtonProps extension
**Confidence:** HIGH — all findings verified by direct source read

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Vocabulary neutralization (V-01..V-08 — operator decision 2026-07-05, LOCKED):**
- V-01: `ProductTileProps` → `TileProps`; field `product_id` → `item_id`; `ProductGridProps` → `TileGridProps`; `CartPanelProps` → `SelectionPanelProps` (remove `show_staff`/`show_people`); `CategoryNavProps` → `FilterTabsProps`; `QuantityStepperProps`/`NumpadProps`/`NumpadMode` unchanged.
- V-02: `render_product_tile` → `render_tile`; `runtime/product_tiles.rs` → `runtime/tiles.rs`; `setupProductTiles` → `setupTiles` (dispatcher + BOTH drift-list tests updated same commit).
- V-03: `data-product-categories` → `data-filter-tokens`; planned search attribute is `data-filter-text` (not `data-product-name`); space→hyphen normalization contract unchanged.
- V-04: `pos-fill-viewport` → `register-fill-viewport`, `pos-grid-fill` → `register-grid-fill`, `pos-cart-present` → `register-selection-present`; `POS_TRIGGER_TYPES` → `REGISTER_TRIGGER_TYPES = ["TileGrid", "SelectionPanel", "Numpad"]`.
- V-05: All five `POS_*` constants in `render/classes.rs` lose the `POS_` prefix; class VALUE strings unchanged; test in classes.rs updated, not weakened.
- V-06: SC-0 grep gate: `grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` must return zero hits.
- V-07: Docs ship in-phase; schema export artifacts (`docs/protocol/schemas/*.json`) regenerate with real changes and ARE committed.
- V-08: Planning vocabulary (`POS-xx` IDs, milestone name, `.planning/` files) is NOT renamed.

**Numpad runtime (D-01..D-06 — LOCKED):**
- D-01: Container `data-numpad` + `data-numpad-target="{field}"`; display `data-numpad-display`; keys `data-numpad-key="0".."9"|"backspace"|"clear"`; hidden input `data-numpad-input="{field}"`. Event delegation via `event.target.closest('[data-numpad-key]')`.
- D-02: Quantity mode: leading-zero collapse, backspace, clear, empty = "0".
- D-03: Price mode: cents-shift entry; hidden field carries raw integer cents; display formats with decimal separator.
- D-04: Every key tap dispatches `new Event('input', { bubbles: true })` on the hidden input.
- D-05: `initNumberGuard` extended to collect `input[data-numpad-input]`.
- D-06: Max-length cap (exact bound planner's discretion). No-op when no `[data-numpad]` exists.

**Filter runtime (D-07..D-12 — LOCKED):**
- D-07: `[data-filter-scope]` containers; `[data-filter-tab]` (value = token; empty = All); `[data-filter-search]`; tiles identified by `[data-filter-text]`.
- D-08: `render_tile` gains always-emitted `data-filter-text="{name}"`. Phase 254 test extended post-rename to assert new attribute is present; new escaping assertion added.
- D-09: AND intersection semantics; case-insensitive substring for search; verbatim case-insensitive token matching.
- D-10: Untokened tiles (no `data-filter-tokens`) visible under All, hidden under specific filter tab.
- D-11: `el.style.display = 'none'` / `''` (not `hidden` attribute, not Tailwind class).
- D-12: Active-tab visual state uses semantic-token classes only; `setupFilters()` no-op when no `[data-filter-scope]`.

**Double-submit guard (D-13..D-16 — LOCKED):**
- D-13: Guard lives inside `setupFormGuards()`, NOT a new setup function.
- D-14: Bind on form's `submit` event (not click). Resolve form via `closest('form')` + `form="<id>"` fallback. On second submit: `preventDefault()`. Visual: `opacity-50 cursor-not-allowed`. Same vocab as existing guards.
- D-15: `pageshow` with `event.persisted`: reset submitted flag, re-enable button.
- D-16: Additive `disable_on_submit: Option<bool>` on `ButtonProps`; `render_button` emits `data-disable-on-submit` when true; `/cassa` confirm button gets the prop.

**Idempotency docs (D-17..D-18 — LOCKED):**
- D-17: No new mechanism — attach to existing `framework::write` idempotency hook.
- D-18: New section in `docs/src/features/write-kernel.md`; documents layered pattern: (1) client guard, (2) per-render UUID hidden input `idempotency_key`, (3) PRG.

**Module organization + wiring (D-19..D-22 — LOCKED):**
- D-19: Two new files: `runtime/numpad.rs` and `runtime/filters.rs`; ES5 style (var, function, no arrow, no template literals). Wired into FERRO_RUNTIME_JS concatenation and `ferroRuntime()` dispatcher.
- D-20: BOTH drift lists extended in same commit. `setupProductTiles` entry becomes `setupTiles`.
- D-21: Inline-source inspection tests per SC-3/SC-4; HTML attribute assertions for `data-filter-text`, `data-disable-on-submit`.
- D-22: CI-exact gate: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus `cargo doc` clean. Run `scripts/gen-ferro-base-css.sh` and diff-check.

### Claude's Discretion

- Exact neutral names for the V-05 constants (no `POS_` prefix).
- Price-mode display separator character; exact max-length entry cap (D-03/D-06).
- Exact active-tab class strings (token-compliant, full literals) for `setupFilters` (D-12).
- Whether `/cassa` demo handler demonstrates the idempotency hidden field (D-18).
- Internal JS naming, helper factoring inside the two new modules; diacritic handling in search.

### Deferred Ideas (OUT OF SCOPE)

- CartRuntime (live per-tap selection updates, client-computed totals).
- Barcode keyboard-wedge module.
- `register-text-input-position` lint rule.
- "Uncategorized" virtual sentinel tab (Phase 256 render decision).
- `TileProps.price` naming beyond the `product_id`→`item_id` rename.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| POS-08 | POS forms are double-submit protected — a `data-disable-on-submit` runtime guard plus the documented idempotency-key pattern on the existing `framework::write` idempotency hook. | `form_guards.rs` `setupFormGuards()` is the integration point; `write-kernel.md` idempotency steps 2/5 documented; `ButtonProps` additive `disable_on_submit: Option<bool>` prop; cassa.json confirm button is the live demonstration. |
| SC-0 | Vocabulary neutralization with zero-hits grep gate across ferro-json-ui/src, ferro-mcp/src, app/src, docs/src. | Full cascade inventory documented below — 15 file sites across 6 directories. |

</phase_requirements>

---

## Summary

Phase 255 has two parts: (A) a vocabulary rename across 15 source sites renaming all POS-domain identifiers to structural-neutral names before any runtime is written, and (B) two new ES5 runtime modules (`numpad.rs`, `filters.rs`) plus a double-submit guard extension inside the existing `form_guards.rs`.

Part (A) is a pure rename with no behavioral change. The risk is a missed site that prevents SC-0's zero-hits grep from passing. This research documents every site exhaustively. The `docs/protocol/schemas/*.json` files (ferro-projections protocol) contain no component props and are unaffected by this rename — that concern in V-07 likely refers to the schema smoke tests in component.rs which run inline and do not write to disk; verify before committing.

Part (B) extends the existing bundle assembly and dispatcher pattern. The `product_tiles.rs` idiom (`dispatchEvent(new Event('input', { bubbles: true }))`) is the canonical pattern to copy for numpad. The `tabs.rs` active-state class pattern (`border-primary`/`text-primary`/`font-semibold`) is the canonical reference for filter tab active classes. The `form_guards.rs` `findGuardedSubmit` helper and `opacity-50 cursor-not-allowed` disabled vocabulary carry over directly to the double-submit guard.

**Primary recommendation:** rename FIRST in one atomic commit (SC-0 gate), then layer the runtime modules on top so they are written against the final attribute vocabulary.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Vocabulary rename (types, fns, rule ids, constants) | ferro-json-ui Rust (component.rs, render/atoms.rs, render/classes.rs, design/rules.rs, catalog.rs, runtime/mod.rs) | ferro-mcp (json_ui_catalog.rs), app sample (cassa.json), docs | Rename is in the compiled library surface; MCP mirror and docs are downstream reflections |
| Runtime modules (numpad, filters) | ferro-json-ui JS bundle (runtime/*.rs as `const SOURCE: &str`) | Browser DOM | Embedded IIFE served inline; no external HTTP requests |
| Double-submit guard | ferro-json-ui JS bundle (form_guards.rs) | Browser form submit event | Lives alongside existing form guards, same module, same dispatcher setup function |
| `ButtonProps.disable_on_submit` emission | ferro-json-ui Rust (component.rs + render/atoms.rs `render_button`) | JSON spec authors | Additive prop; emission follows existing Option<bool> pattern |
| Idempotency-key documentation | docs/src/features/write-kernel.md | framework::write (read-only reference) | Documentation only; no new Rust mechanism |

---

## Standard Stack

### Core (all verified by direct source read)

| File | Current State | Phase 255 Action |
|------|---------------|------------------|
| `ferro-json-ui/src/component.rs` | Defines `ProductTileProps`, `ProductGridProps`, `CartPanelProps`, `CategoryNavProps`, `ButtonProps` | Rename structs, field `product_id`→`item_id`, remove `show_staff`/`show_people`, add `disable_on_submit: Option<bool>` to ButtonProps |
| `ferro-json-ui/src/catalog.rs` | `BUILTIN_SPECS[253] = ("ProductTile", ...)`, count assert `== 47` at line 1219 | Change to `"Tile"`, update import |
| `ferro-json-ui/src/render/mod.rs` | `BUILTIN_TYPES[67] = "ProductTile"`, dispatch at line 200 | Rename to `"Tile"`, update dispatch arm |
| `ferro-json-ui/src/render/atoms.rs` | `render_product_tile`, emits `data-product-categories` | Rename fn, emit `data-filter-tokens` + `data-filter-text`, update all tests |
| `ferro-json-ui/src/render/classes.rs` | 5 constants: `POS_TOUCH_ACTION`, `POS_HIT_TARGET_MIN`, `POS_HIT_TARGET_NUMPAD`, `POS_PRESS_ACTIVE`, `POS_OVERSCROLL_CONTAIN`, `POS_TAP_HIGHLIGHT` | Drop `POS_` prefix from all; class VALUE strings unchanged |
| `ferro-json-ui/src/design/rules.rs` | 3 rule ids `pos-*`, `POS_TRIGGER_TYPES = ["ProductGrid", "CartPanel", "Numpad"]` | Rename rule ids, rename constant, update trigger type strings |
| `ferro-json-ui/src/runtime/product_tiles.rs` | `function setupProductTiles()` | Rename file to `tiles.rs`, rename function |
| `ferro-json-ui/src/runtime/form_guards.rs` | `setupFormGuards()`, `initNumberGuard` collects `input[type="number"]` + `input[data-qty-input]` | Extend number guard with `input[data-numpad-input]`; add double-submit guard block; update "ProductTile" comment |
| `ferro-json-ui/src/runtime/mod.rs` | 14 setup functions, two drift-list arrays | Add `mod tiles/numpad/filters`, update concat, dispatcher, both drift lists |
| `ferro-json-ui/src/lib.rs` | `pub use component::{...ProductTileProps...}` at line 58 | Update re-export to `TileProps` |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | `RULE_COMPONENTS` has `"pos-fill-viewport"` etc.; `test_all_components_present` expected list includes `"ProductTile"` at line 450, count 47 | Rename rule id strings; rename `"ProductTile"` → `"Tile"` in expected list; count stays 47 |
| `app/src/views/cassa.json` | `"type": "ProductTile"`, `"product_id": {$data}`, Button confirm with no disable_on_submit | Rename type + field; add `"disable_on_submit": true` on `btn_confirm` |
| `docs/src/json-ui/components.md` | `### ProductTile` section with props table; category row in component index | Rename section; update props table; add migration table |
| `docs/src/design-system/patterns.md` | Three rule sections with `pos-*` ids; fixture JSON uses `"ProductGrid"`, `"CartPanel"` | Rename all three sections; update fixture type strings and allow strings |
| `docs/src/features/write-kernel.md` | Existing idempotency doc steps 2/5 | Add "Double-submit protection for forms" section |

### New Files

| File | Content |
|------|---------|
| `ferro-json-ui/src/runtime/numpad.rs` | `pub(super) const SOURCE: &str = r#"...#"` — ES5 `setupNumpad()`, `initNumpad()` helpers |
| `ferro-json-ui/src/runtime/filters.rs` | `pub(super) const SOURCE: &str = r#"...#"` — ES5 `setupFilters()`, `initFilterScope()` helpers |

---

## Rename Cascade — Exhaustive Inventory (SC-0)

The SC-0 grep gate checks:
```
grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid'
  ferro-json-ui/src ferro-mcp/src app/src docs/src
```

Every current hit is listed below with its file, line, and the required change.

### `ferro-json-ui/src/component.rs` [VERIFIED]

| Line(s) | Current | New |
|---------|---------|-----|
| 1353 | `pub struct ProductTileProps` | `pub struct TileProps` |
| 1354 | `pub product_id: String` | `pub item_id: String` |
| 1361-1363 | rustdoc: `render_product_tile`, `data-product-categories`, `setupPosFilter` | Update to `render_tile`, `data-filter-tokens` |
| 1384-1386 | rustdoc: `ProductGrid POS builtin` ... `ProductTile children` | Update to `TileGrid`, `Tile children` |
| 1388 | `pub struct ProductGridProps` | `pub struct TileGridProps` |
| 1391 | rustdoc: `CartPanel` | `SelectionPanel` |
| 1405-1406 | `/// CartPanel POS builtin` | `/// SelectionPanel POS builtin` |
| 1408 | `pub struct CartPanelProps` | `pub struct SelectionPanelProps` |
| 1409 | rustdoc: `ProductGrid` | `TileGrid` |
| 1415-1419 | fields `show_staff: Option<bool>`, `show_people: Option<bool>` | REMOVE both |
| 1422-1424 | `/// CategoryNav POS builtin` / `data-product-categories` | `/// FilterTabs POS builtin` / `data-filter-tokens` |
| 1426 | `pub struct CategoryNavProps` | `pub struct FilterTabsProps` |
| 1428-1429 | rustdoc: `data-product-categories`, `ProductTileProps::categories` | `data-filter-tokens`, `TileProps::categories` |
| 1438 | rustdoc: `ProductTile contract` | `Tile contract` |
| 1758-1759 | `fn schema_for_product_tile_props_generates` / `ProductTileProps` | rename fn, use `TileProps` |
| 1974 | `ProductGridProps` | `TileGridProps` |
| 1979 | `CartPanelProps` | `SelectionPanelProps` |
| 1984 | `CategoryNavProps` | `FilterTabsProps` |
| 2403 | `mod product_tile_contract_tests` | `mod tile_contract_tests` |
| 2404 | `//! ...ProductTileProps...` | `TileProps` |
| 2410 | `/// Legacy ProductTile JSON...` | `Tile` |
| 2413 | `fn product_tile_legacy_json_round_trips_unchanged` | rename |
| 2415 | `let tile: ProductTileProps` | `TileProps` |
| 2446-2449 | `/// ProductTileProps with categories...`, `fn product_tile_with_categories_serializes`, `let tile = ProductTileProps {` | rename |

### `ferro-json-ui/src/catalog.rs` [VERIFIED]

| Line(s) | Current | New |
|---------|---------|-----|
| 35 | `ProductTileProps` in import | `TileProps` |
| 253 | `"ProductTile"` | `"Tile"` |
| 255 | `schema_for!(ProductTileProps)` | `schema_for!(TileProps)` |

### `ferro-json-ui/src/lib.rs` [VERIFIED]

| Line | Current | New |
|------|---------|-----|
| 58 | `ProductTileProps` in pub use | `TileProps` |

### `ferro-json-ui/src/render/mod.rs` [VERIFIED]

| Line | Current | New |
|------|---------|-----|
| 67 | `"ProductTile"` in BUILTIN_TYPES | `"Tile"` |
| 200 | `"ProductTile" => atoms::render_product_tile(...)` | `"Tile" => atoms::render_tile(...)` |

### `ferro-json-ui/src/render/atoms.rs` [VERIFIED]

| Line(s) | Current | New |
|---------|---------|-----|
| 16 | `ProductTileProps` import | `TileProps` |
| 1356 | `// ── 23. ProductTile` | `// ── 23. Tile` |
| 1358 | `pub(crate) fn render_product_tile` | `pub(crate) fn render_tile` |
| 1364 | `let props: ProductTileProps` | `TileProps` |
| 1366 | `decode_diagnostic("ProductTile", e)` | `"Tile"` |
| 1371-1390 | `categories_attr` block emits `data-product-categories` | Emit `data-filter-tokens` (same logic) + always emit `data-filter-text="{name}"` |
| 2271 | `// ── 23. ProductTile` | `// ── 23. Tile` |
| 2273 | `/// INT-pass (251-02): ProductTile +/- buttons...` | `Tile` |
| 2275 | `fn product_tile_buttons_carry_token_focus_ring` | rename |
| 2277 | `Element::new("ProductTile")` | `Element::new("Tile")` |
| 2278-2281 | `.prop("product_id", "1")` | `.prop("item_id", "1")` |
| 2296-2308 | `fn product_tile_emits_name_and_price` / `Element::new("ProductTile")` / `.prop("product_id", ...)` | rename fn + element type + field |
| 2548-2559 | `make_product_tile` helper: `SpecElement::new("ProductTile")`, `.prop("product_id", "p1")` | `"Tile"`, `"item_id"` |
| 2564-2580 | `fn product_tile_legacy_render_is_byte_identical` / `!html.contains("data-product-categories")` | rename fn; assert `!html.contains("data-filter-tokens")`; also assert `html.contains("data-filter-text")` (D-08: always emitted) |
| 2583-2595 | `fn product_tile_emits_data_product_categories` / `data-product-categories` | rename fn to `tile_emits_data_filter_tokens`; update assertions |
| 2598-2607 | `fn product_tile_normalizes_spaces_in_category_names` / `data-product-categories` | rename fn; update assertions |
| 2610-2622 | `fn product_tile_escapes_categories` | rename fn |

**Important test extension (D-08):** After rename, `product_tile_legacy_render_is_byte_identical` (renamed `tile_legacy_render_is_byte_identical`) must gain TWO new assertions: (1) `html.contains("data-filter-text=\"Espresso\"")` — always-emitted `data-filter-text` with raw name, (2) a new escaping test mirroring the categories-escape test but for `data-filter-text` with an XSS-candidate name.

### `ferro-json-ui/src/render/classes.rs` [VERIFIED]

Current POS_ constants (lines 41, 44, 47, 51, 54, 58):
```
POS_TOUCH_ACTION = "touch-manipulation"
POS_HIT_TARGET_MIN = "min-h-[44px] min-w-[44px]"
POS_HIT_TARGET_NUMPAD = "min-h-[56px] min-w-[56px]"
POS_PRESS_ACTIVE = "active:scale-95 active:bg-border"
POS_OVERSCROLL_CONTAIN = "overscroll-contain"
POS_TAP_HIGHLIGHT = "pos-tap-highlight"
```

All usages outside `classes.rs` itself: the `pos_render_functions_use_constants_not_literals` test guards `render/` directory (auto-covers renamed file). Rename the constants — class VALUE strings are unchanged, so no CSS regen needed for this specific change. Tests in `classes.rs` lines 112-124 update constant names accordingly.

Suggested neutral names (planner's final call per Claude's Discretion):
- `TOUCH_ACTION` / `HIT_TARGET_MIN` / `HIT_TARGET_NUMPAD` / `PRESS_ACTIVE` / `OVERSCROLL_CONTAIN` / `TAP_HIGHLIGHT`

All call sites using `POS_TOUCH_ACTION` etc. are in `render/atoms.rs`. Must be updated there too.

### `ferro-json-ui/src/design/rules.rs` [VERIFIED]

| Line(s) | Current | New |
|---------|---------|-----|
| 85 | `id: "pos-fill-viewport"` | `"register-fill-viewport"` |
| 87 | rationale: `"ProductGrid, CartPanel, or Numpad"` | `"TileGrid, SelectionPanel, or Numpad"` |
| 92 | `id: "pos-grid-fill"` | `"register-grid-fill"` |
| 99 | `id: "pos-cart-present"` | `"register-selection-present"` |
| 100 | `title: "A ProductGrid register needs a CartPanel"` | `"A TileGrid register needs a SelectionPanel"` |
| 101 | `rationale: "A ProductGrid with no CartPanel..."` | `"A TileGrid with no SelectionPanel..."` |
| 443 | `const POS_TRIGGER_TYPES: &[&str] = &["ProductGrid", "CartPanel", "Numpad"]` | `const REGISTER_TRIGGER_TYPES: &[&str] = &["TileGrid", "SelectionPanel", "Numpad"]` |
| 449 | `POS_TRIGGER_TYPES.contains(...)` | `REGISTER_TRIGGER_TYPES.contains(...)` |
| 454 | `rule: "pos-fill-viewport"` | `"register-fill-viewport"` |
| 485 | `rule: "pos-grid-fill"` | `"register-grid-fill"` |
| 499 | `el.type_name == "ProductGrid"` | `"TileGrid"` |
| 500 | `el.type_name == "CartPanel"` | `"SelectionPanel"` |
| 505 | `rule: "pos-cart-present"` | `"register-selection-present"` |
| 508-509 | message/suggestion mentioning `ProductGrid`, `CartPanel` | Updated names |
| All test fixtures | `"type": "ProductGrid"`, `"type": "CartPanel"` | `"TileGrid"`, `"SelectionPanel"` |
| Test fn names | `pos_fill_viewport_*`, `pos_grid_fill_*`, `pos_cart_present_*` | `register_fill_viewport_*`, etc. |
| Test assertion strings | `"pos-fill-viewport"`, `"pos-grid-fill"`, `"pos-cart-present"` | new ids |

### `ferro-json-ui/src/runtime/product_tiles.rs` → `tiles.rs` [VERIFIED]

File rename + single function rename: `setupProductTiles()` → `setupTiles()`. Content otherwise unchanged.

### `ferro-json-ui/src/runtime/form_guards.rs` [VERIFIED]

| Line | Current | Action |
|------|---------|--------|
| 63 | `// Skip ProductTile +/- controls` | `// Skip Tile +/- controls` |
| 55-60 | `initNumberGuard` merges `input[type="number"]` + `input[data-qty-input]` | Add third merge: `input[data-numpad-input]` (same pattern as qtyInputs loop) |
| (new block inside setupFormGuards) | — | Add double-submit guard: iterate `button[data-disable-on-submit]`, bind `form.submit` handler, add `window.pageshow` bfcache reset |

### `ferro-json-ui/src/runtime/mod.rs` [VERIFIED]

| Line | Current | New |
|------|---------|-----|
| 15 | `mod product_tiles;` | `mod tiles;` |
| (new) | — | `mod numpad;` |
| (new) | — | `mod filters;` |
| 39 | `s.push_str(product_tiles::SOURCE);` | `s.push_str(tiles::SOURCE);` |
| (new) | — | `s.push_str(numpad::SOURCE);` |
| (new) | — | `s.push_str(filters::SOURCE);` |
| 54 | `setupProductTiles();\n\` | `setupTiles();\n\` |
| (new dispatcher entries) | — | `setupNumpad();\n\`, `setupFilters();\n\` |
| 191 | `"setupProductTiles"` | `"setupTiles"` |
| (new drift entries) | — | `"setupNumpad"`, `"setupFilters"` |
| 224 | `"setupProductTiles();"` | `"setupTiles();"` |
| (new drift entries) | — | `"setupNumpad();"`, `"setupFilters();"` |

### `ferro-mcp/src/tools/json_ui_catalog.rs` [VERIFIED]

| Line(s) | Current | New |
|---------|---------|-----|
| 99 | `("pos-fill-viewport", &["Grid"])` | `("register-fill-viewport", &["Grid"])` |
| 100 | `("pos-grid-fill", &["Grid"])` | `("register-grid-fill", &["Grid"])` |
| 101 | `("pos-cart-present", &["Grid"])` | `("register-selection-present", &["Grid"])` |
| 402-407 | `assert_eq!(catalog.components.len(), 47, ...)` message text | Update message text (count stays 47) |
| 450 | `"ProductTile"` in expected array | `"Tile"` |

### `app/src/views/cassa.json` [VERIFIED]

| Location | Current | New |
|----------|---------|-----|
| Line 71 | `"type": "ProductTile"` | `"type": "Tile"` |
| Line 74 | `"product_id": { "$data": "/p/id" }` | `"item_id": { "$data": "/p/id" }` |
| `btn_confirm` element props | `{"label": "Conferma ordine", "variant": "primary"}` | Add `"disable_on_submit": true` |

No changes needed to `app/src/controllers/cassa.rs` — it has no component type strings, only data construction.

### `docs/src/json-ui/components.md` [VERIFIED]

| Location | Current | New |
|----------|---------|-----|
| Line 34 | `\| **Commerce** \| ProductTile \|` | `\| **Commerce** \| Tile \|` |
| Line 1400 | `### ProductTile` | `### Tile` |
| Line 1402 | description text | Update: neutral "touch-friendly tile" language |
| Lines 1404-1410 | props table with `product_id` | Replace `product_id` with `item_id`; add additive props (`categories`, `image_url`, `color`, `stock_badge`) |
| Lines 1413-1421 | example JSON with `"type": "ProductTile"`, `"product_id"` | Update type + field name |
| Line 1424 | "Place ProductTile elements..." | "Place Tile elements..." |
| Migration table (after the Phase 251 precedent at line 72) | Existing variant/tone migration table | Add rows: `ProductTile`→`Tile`, `product_id`→`item_id`, `data-product-categories`→`data-filter-tokens` |

### `docs/src/design-system/patterns.md` [VERIFIED]

The three rule sections at lines 522-654 need updating:

| Section | Current | New |
|---------|---------|-----|
| `## \`pos-fill-viewport\`` heading (line 522) | `pos-fill-viewport` | `register-fill-viewport` |
| Rule title/rationale | mentions `ProductGrid, CartPanel, or Numpad` | `TileGrid, SelectionPanel, or Numpad` |
| Violating example | `"type": "ProductGrid"` | `"type": "TileGrid"` |
| Allow string | `["pos-fill-viewport"]` | `["register-fill-viewport"]` |
| `## \`pos-grid-fill\`` heading (line 567) | `pos-grid-fill` | `register-grid-fill` |
| Allow string | `["pos-grid-fill"]` | `["register-grid-fill"]` |
| `## \`pos-cart-present\`` heading (line 611) | `pos-cart-present` | `register-selection-present` |
| Title (line 613) | "A ProductGrid register needs a CartPanel" | "A TileGrid register needs a SelectionPanel" |
| Rationale (line 615) | "ProductGrid with no CartPanel" | "TileGrid with no SelectionPanel" |
| Intents line (line 617) | "containing a ProductGrid" | "containing a TileGrid" |
| Conforming example | `"type": "ProductGrid"`, `"type": "CartPanel"` | `"TileGrid"`, `"SelectionPanel"` |
| Violating example | `"type": "ProductGrid"` | `"type": "TileGrid"` |
| Allow description | "ProductGrid...non-register" | "TileGrid...non-register" |
| Allow string | `["pos-cart-present"]` | `["register-selection-present"]` |

---

## Architecture Patterns

### Runtime Module Pattern (verified from `product_tiles.rs` + `tabs.rs` + `form_guards.rs`)

Each module contributes exactly one `pub(super) const SOURCE: &str = r#"..."#` containing a single `setup*()` function and any helpers. The outer IIFE wrapper and dispatcher are in `mod.rs` — modules contribute only their function bodies (no IIFE, no DOMContentLoaded).

```rust
// Source: ferro-json-ui/src/runtime/product_tiles.rs (existing, verified)
pub(super) const SOURCE: &str = r#"
    // ── [Concern] ────────────────────────────────────────────────────────────

    function setupConcern() {
        var items = document.querySelectorAll('[data-concern]');
        if (items.length === 0) return;  // no-op guard per D-06/D-12
        for (var i = 0; i < items.length; i++) {
            initConcern(items[i]);
        }
    }

    function initConcern(el) {
        el.addEventListener('click', function() { ... });
    }
"#;
```

**ES5 constraints confirmed:** `var` only, `function` declarations, no arrow functions (`=>`), no template literals (`` ` ``), no `let`/`const`, no destructuring.

### Event Delegation Pattern for Numpad

`closest()` is a DOM Level 4 method (not ES6 syntax) — valid in the ES5 runtime. The existing runtime uses per-element listeners; numpad uses delegation as per D-01:

```javascript
// Source: CONTEXT.md D-01 contract
function initNumpad(container) {
    container.addEventListener('click', function(e) {
        var key = e.target.closest('[data-numpad-key]');
        if (!key) return;
        // ... handle key
    });
}
```

No existing module currently uses `closest()` — numpad is the first. This is fine as a pattern — it is a DOM API, not JS syntax, and is fully ES5-compatible.

### Input Event Dispatch Pattern (verified from `product_tiles.rs`)

```javascript
// Source: ferro-json-ui/src/runtime/product_tiles.rs (verified)
input.dispatchEvent(new Event('input', { bubbles: true }));
```

This is the canonical pattern to copy for numpad key taps (D-04). The `{ bubbles: true }` is required so form-guard listeners on ancestor elements receive the event.

### Active-Tab Class Pattern (verified from `tabs.rs`)

```javascript
// Source: ferro-json-ui/src/runtime/tabs.rs (verified)
// Active:
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
t.classList.add('border-primary', 'text-primary', 'font-semibold');
// Inactive:
t.classList.remove('border-primary', 'text-primary', 'font-semibold');
t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
```

Filter tabs MUST mirror this pattern with semantic-token classes only. These classes are already in the bundle (tabs.rs) so `variant_classes_use_semantic_tokens` scan will pass without adding new CSS. No CSS regen needed for the active-tab state strings.

### Form Guard Disabled-State Pattern (verified from `form_guards.rs`)

```javascript
// Source: ferro-json-ui/src/runtime/form_guards.rs (verified)
// Disable:
submitBtn.setAttribute('disabled', 'disabled');
submitBtn.classList.add('opacity-50', 'cursor-not-allowed');
// Enable:
submitBtn.removeAttribute('disabled');
submitBtn.classList.remove('opacity-50', 'cursor-not-allowed');
```

The double-submit guard uses exactly this vocabulary for D-14.

### Bundle Assembly Pattern (verified from `mod.rs`)

```rust
// Source: ferro-json-ui/src/runtime/mod.rs (verified)
pub static FERRO_RUNTIME_JS: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::with_capacity(8 * 1024);
    s.push_str("(function() {\n    'use strict';\n");
    s.push_str(sse::SOURCE);
    // ... other modules
    s.push_str(product_tiles::SOURCE);  // becomes tiles::SOURCE
    // + numpad::SOURCE and filters::SOURCE go here
    s.push_str("\n    function ferroRuntime() {\n\
         \x20       setupProductTiles();\n\  // becomes setupTiles();
         // + setupNumpad(); and setupFilters(); go here
         ...");
});
```

### `findGuardedSubmit` — Existing Helper for Double-Submit Guard (verified from `form_guards.rs`)

```javascript
// Source: ferro-json-ui/src/runtime/form_guards.rs (verified, lines 14-23)
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

D-14 needs the INVERSE: given a `button[data-disable-on-submit]`, find its form. Use `button.closest('form')` first, then `button.getAttribute('form')` fallback + `document.getElementById(...)`.

### ButtonProps Additive Prop Pattern (verified from component.rs existing props)

```rust
// Source: ferro-json-ui/src/component.rs lines 305-318 (verified — ButtonProps existing structure)
#[serde(default, skip_serializing_if = "Option::is_none")]
pub disabled: Option<bool>,
// ...
#[serde(default, skip_serializing_if = "Option::is_none")]
pub form: Option<String>,
// New:
/// When `true`, emits `data-disable-on-submit` on the rendered button; the
/// runtime guard disables this button after the first form submission to
/// prevent double-posting (D-16). Pairs with a per-render `idempotency_key`
/// hidden input for server-side deduplication (see `dispatch_write` step 2).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub disable_on_submit: Option<bool>,
```

Emission in `render_button_inner`:
```rust
// Analogous to the `form_attr` and `disabled_attr` patterns already in render_button_inner
let disable_on_submit_attr = if props.disable_on_submit == Some(true) {
    " data-disable-on-submit"
} else {
    ""
};
// Add to format! string alongside existing attrs
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Active-tab visual state | Custom CSS class strings | Mirror `tabs.rs` `border-primary`/`text-primary`/`font-semibold` | Already in bundle; variant_classes scan already passes; no CSS regen needed |
| Form resolution for double-submit | Custom traversal | `closest('form')` + `form="<id>"` fallback pattern (mirrors `findGuardedSubmit` inverse) | Edge case: externally mounted buttons (PageHeader actions slot) |
| Browser history state on back | Page reload detection | `pageshow` event with `event.persisted` check (D-15) | bfcache on iOS Safari: back-navigation restores DOM with disabled state intact unless reset |
| Money arithmetic in price mode | Float arithmetic | Integer cents — display-only decimal formatting | Float rounding errors on `0.1 + 0.2 ≠ 0.3` in POS context — confirmed PITFALLS.md |
| New runtime setup function for double-submit | New `setupDisableOnSubmit()` entry in dispatcher | Extend `setupFormGuards()` (D-13) | SC-1/SC-2 name exactly two new setups; a third would contradict the phase's own success criteria |

---

## Common Pitfalls

### Pitfall 1: Partial rename — grep gate fails
**What goes wrong:** A test helper or fixture JSON uses the old string; SC-0 grep returns a hit; phase does not pass verification.
**Why it happens:** Test module comments, `mod` names, helper function names are easy to miss in a global rename.
**How to avoid:** Use the cascade table above as a checklist. After each file is edited, run the grep command and verify zero new hits for that file's tokens.
**Warning signs:** `cargo test --all-features` passes (wrong type string in fixture becomes a "component not found" at render time, not a compile error).

### Pitfall 2: Drift-list tests not updated atomically
**What goes wrong:** `bundle_contains_all_setup_functions` or `dispatcher_invokes_every_setup` fails because `setupProductTiles` was removed but `setupTiles`/`setupNumpad`/`setupFilters` were not added.
**Why it happens:** The two drift arrays in `mod.rs` are separate from the bundle assembly code; they can be edited without editing each other.
**How to avoid:** Update BOTH arrays in the same edit as the `mod.rs` dispatcher/concat changes. The two arrays are at lines ~180 and ~210 of `mod.rs`.

### Pitfall 3: `data-filter-text` missing on legacy tiles
**What goes wrong:** `setupFilters` iterates `[data-filter-text]` but legacy tiles (no categories) have no such attribute → they are invisible to the filter runtime even under All.
**Why it happens:** D-08 says `data-filter-text` is ALWAYS emitted (not conditional like `data-filter-tokens`). This is intentional: it is the universal tile marker AND the search source.
**How to avoid:** Emit `data-filter-text` unconditionally in `render_tile` (outside the `categories_attr` block). The test update in `tile_legacy_render_is_byte_identical` explicitly asserts this attribute IS present.

### Pitfall 4: Double-submit guard in a new dispatcher entry (violates D-13)
**What goes wrong:** Creating `setupDisableOnSubmit()` and adding it to the dispatcher as a third new entry contradicts SC-1/SC-2 (which names exactly `setupNumpad` and `setupFilters` as the two new setups).
**Why it happens:** Intuition says "one concern = one setup function."
**How to avoid:** The double-submit guard is a FORM guard and belongs inside `setupFormGuards()` as an additional init block, per D-13.

### Pitfall 5: `show_staff`/`show_people` removal breaks serde deserialization of existing specs
**What goes wrong:** A spec with `show_staff: true` in JSON fails to deserialize into `SelectionPanelProps` because the field no longer exists on the struct.
**Why it happens:** `serde` by default errors on unknown fields.
**How to avoid:** Use `#[serde(deny_unknown_fields)]`-free structs. `SelectionPanelProps` does NOT derive `deny_unknown_fields` (standard pattern for this codebase — unknown fields are silently ignored). Confirmed by checking existing component props pattern: no component uses `deny_unknown_fields`. So removing the fields is backward-compatible at deserialization.

### Pitfall 6: `closest()` on the CONTAINER vs on event.target
**What goes wrong:** `container.closest('[data-numpad-key]')` on the container itself (which has `data-numpad`) rather than on `event.target` (the clicked child).
**Why it happens:** Confusing the delegation target with the listener target.
**How to avoid:** The handler is: `container.addEventListener('click', function(e) { var key = e.target.closest('[data-numpad-key]'); ... })`. `e.target` is the element that was actually clicked; `closest()` walks up from there to find the key element.

### Pitfall 7: Token comparison in filter — normalization mismatch
**What goes wrong:** Filter tab sends label "Bevande calde"; tile has token "Bevande-calde"; match fails.
**Why it happens:** Category labels in FilterTabs props are raw strings; tokens in `data-filter-tokens` are already space→hyphen normalized (from render time).
**How to avoid:** In `setupFilters`, normalize the tab's `data-filter-tab` value the same way: `.replace(/ /g, '-')` before comparing. OR render FilterTabs already-normalized (Phase 256 render decision). Document the normalization contract in `runtime/filters.rs` comments.

---

## Code Examples

### D-05: Extending `initNumberGuard` to include `input[data-numpad-input]`

```javascript
// Source: pattern from ferro-json-ui/src/runtime/form_guards.rs lines 54-92 (verified)
function initNumberGuard(form) {
    var numberInputs = form.querySelectorAll('input[type="number"]');
    var qtyInputs = form.querySelectorAll('input[data-qty-input]');
    var numpadInputs = form.querySelectorAll('input[data-numpad-input]');  // NEW
    var inputs = [];
    for (var n = 0; n < numberInputs.length; n++) inputs.push(numberInputs[n]);
    for (var q = 0; q < qtyInputs.length; q++) inputs.push(qtyInputs[q]);
    for (var m = 0; m < numpadInputs.length; m++) inputs.push(numpadInputs[m]);  // NEW
    // ... rest unchanged
}
```

### D-14: Double-submit guard block inside `setupFormGuards`

```javascript
// Source: design from CONTEXT.md D-13/D-14/D-15
// (goes inside setupFormGuards(), after the existing guard loop)

    // ── Double-submit guard ────────────────────────────────────────────────
    var disableBtns = document.querySelectorAll('button[data-disable-on-submit]');
    for (var d = 0; d < disableBtns.length; d++) {
        initDisableOnSubmit(disableBtns[d]);
    }

    // bfcache reset: when browser restores page from back/forward cache,
    // re-enable all disable-on-submit buttons so the register is usable again.
    window.addEventListener('pageshow', function(e) {
        if (!e.persisted) return;
        for (var r = 0; r < disableBtns.length; r++) {
            disableBtns[r].removeAttribute('disabled');
            disableBtns[r].classList.remove('opacity-50', 'cursor-not-allowed');
        }
        // reset submitted flags — the next submit should go through
    });

    function initDisableOnSubmit(btn) {
        var form = btn.closest('form');
        if (!form && btn.getAttribute('form')) {
            form = document.getElementById(btn.getAttribute('form'));
        }
        if (!form) return;
        var submitted = false;
        form.addEventListener('submit', function(e) {
            if (submitted) {
                e.preventDefault();
                return;
            }
            submitted = true;
            btn.setAttribute('disabled', 'disabled');
            btn.classList.add('opacity-50', 'cursor-not-allowed');
        });
    }
```

### D-09: Filter matching logic skeleton

```javascript
// Source: design from CONTEXT.md D-09 + D-10
function applyFilter(scope, activeToken, searchText) {
    var tiles = scope.querySelectorAll('[data-filter-text]');
    for (var i = 0; i < tiles.length; i++) {
        var tile = tiles[i];
        var tileText = (tile.getAttribute('data-filter-text') || '').toLowerCase();
        var tileTokens = tile.getAttribute('data-filter-tokens') || '';
        var tokenMatch = (activeToken === '') ||
            tileTokens.split(' ').some(function(t) {
                return t.toLowerCase() === activeToken.toLowerCase();
            });
        // D-10: tiles with no data-filter-tokens are visible under All only
        if (activeToken !== '' && tileTokens === '') tokenMatch = false;
        var searchMatch = searchText === '' ||
            tileText.indexOf(searchText.toLowerCase()) !== -1;
        tile.style.display = (tokenMatch && searchMatch) ? '' : 'none';
    }
}
```

Note: `Array.prototype.some` is ES5-compatible. Alternatively use a manual for-loop if strict ES5 is required.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Individual element click listeners (per-button) | Event delegation via `closest()` for numpad | This phase (new) | Single listener per container — handles dynamically rendered keys |
| POS-domain naming on public crate surface | Domain-neutral structural naming | This phase (rename) | Crate is project-agnostic; consumers not locked to gestiscilo vocabulary |
| `data-product-categories` as filter token source | `data-filter-tokens` + `data-filter-text` separation | This phase | Clear separation: tokens for category filtering; text for full-text search; both on every tile |

---

## Schema Export Artifacts (V-07 clarification)

**Finding:** `docs/protocol/schemas/*.json` files contain ferro-projections protocol schemas (ServiceDef, ActionDef, etc.). None of these files reference `ProductTileProps`, `TileProps`, or any component props struct — confirmed by grep returning zero hits. [VERIFIED]

**Implication:** The V-07 statement "schema export artifacts regenerate with REAL changes this time" does NOT mean `docs/protocol/schemas/*.json` will change. These files are regenerated by the ferro-projections Phase 94 export test and are unaffected by component props renames in ferro-json-ui.

**Planner action:** Remove the schema churn discard step from plan — no schema files will be dirtied by this rename. The component.rs schema smoke tests (`assert_schema_nonempty_object::<TileProps>`) run entirely in-memory and write nothing to disk.

---

## CSS Regen Assessment (D-22)

`scripts/gen-ferro-base-css.sh` runs Tailwind v4 CLI scanning the Rust source. Changes this phase makes:

1. **POS_ constant rename** — class VALUE strings unchanged (`"touch-manipulation"` etc. are unchanged). No new CSS. [VERIFIED]
2. **Active-tab strings in `filters.rs`** — uses `border-primary`, `text-primary`, `font-semibold`, `border-transparent`, `text-text-muted`, `hover:text-text`. All already in the bundle from `tabs.rs`. No new CSS. [VERIFIED by reading tabs.rs]
3. **Double-submit guard strings** — uses `opacity-50`, `cursor-not-allowed`. Already in bundle from existing form guards. No new CSS. [VERIFIED]

**Conclusion:** `scripts/gen-ferro-base-css.sh` regen will produce identical output. D-22 still requires running it and diff-checking before concluding (low risk but required by the locked decision).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `SelectionPanelProps` (formerly `CartPanelProps`) does not use `#[serde(deny_unknown_fields)]`, so removing `show_staff`/`show_people` fields is backward-compatible for deserialization of existing specs that include those props | Pitfall 5 | Existing specs with `show_staff` would fail to deserialize — check component struct derive attrs; none found in the codebase grep |
| A2 | `Array.prototype.some` is acceptable for the filter matching loop in the ES5 bundle, OR a manual for-loop is substituted | Code Examples | Strict ES5 compatibility issue — use for-loop if in doubt |
| A3 | V-07's mention of schema export artifacts refers to an expected but non-existent change; no component schema files are written to disk by any test in this workspace | Schema Export section | A test I did not find writes component schema JSON to disk — verify with `grep -rn "write\|fs::write" ferro-json-ui/src/` before concluding |

---

## Open Questions (RESOLVED)

1. **Exact neutral names for V-05 constants**
   - What we know: `POS_` prefix must go; class VALUE strings are unchanged
   - What's unclear: Preferred naming convention — option A: drop prefix entirely (`TOUCH_ACTION`, `HIT_TARGET_MIN`); option B: domain prefix `TOUCH_` (`TOUCH_ACTION_MANIPULATION`, `TOUCH_HIT_TARGET_MIN`)
   - RESOLVED: Plan 255-01 Task 2 locks the exact names — `TOUCH_ACTION`, `HIT_TARGET_MIN`, `HIT_TARGET_NUMPAD`, `PRESS_ACTIVE`, `OVERSCROLL_CONTAIN`, `TAP_HIGHLIGHT` (within the V-05 Claude's-Discretion grant in CONTEXT.md)

2. **`data-filter-tab` token normalization**
   - What we know: Phase 256 will render FilterTabs; this phase only defines the runtime contract
   - What's unclear: Should `setupFilters` normalize tab token values (`.replace(/ /g, '-')`) or assume Phase 256 will pre-normalize?
   - RESOLVED: CONTEXT.md D-09 — tokens are space→hyphen normalized at render time; the runtime compares verbatim, case-insensitively. Plan 255-04 Task 3 comments the normalization contract in the module source

3. **`Array.prototype.some` vs. for-loop in filter runtime**
   - What we know: All other runtime modules use only `for` loops (no array methods)
   - RESOLVED: manual `for` loop per ES5 house style; Plan 255-04 verify blocks grep-reject `.some(`

---

## Validation Architecture

Nyquist validation is enabled (no `workflow.nyquist_validation` key in config.json — absent = enabled).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | Cargo.toml (workspace) |
| Quick run command | `cargo test -p ferro-json-ui --all-features` |
| Full suite command | `cargo test --all-features` |

### SC-0..SC-4 → Test Map

| SC | Behavior | Test Type | Command | Notes |
|----|----------|-----------|---------|-------|
| SC-0 | Zero hits on vocabulary grep gate | shell | `grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` | Must return empty output |
| SC-0 | Count stays 47 | unit | `cargo test -p ferro-json-ui --all-features -- builtin_types_count_drift_guard` | catalog.rs line 1219 |
| SC-0 | BUILTIN_SPECS names match BUILTIN_TYPES | unit | `cargo test -p ferro-json-ui --all-features -- builtin_specs_names_match_dispatch` | catalog.rs line 1423 |
| SC-0 | ferro-mcp mirror count stays 47 | unit | `cargo test -p ferro-mcp --all-features -- test_all_components_present` | json_ui_catalog.rs line 396 |
| SC-1 | setupNumpad + setupFilters in bundle | unit | `cargo test -p ferro-json-ui --all-features -- bundle_contains_all_setup_functions` | mod.rs line 180 |
| SC-2 | dispatcher calls both + no-op when absent | unit | `cargo test -p ferro-json-ui --all-features -- dispatcher_invokes_every_setup` | mod.rs line 210 |
| SC-3 | Numpad writes target field + dispatches input event | unit (inline-source) | `cargo test -p ferro-json-ui --all-features` — new test in mod.rs asserting `FERRO_RUNTIME_JS.contains("data-numpad-key")`, `.contains("bubbles: true")`, `.contains("data-filter-tokens")`, `.contains("data-filter-search")`, `.contains("data-filter-text")` | D-21 |
| SC-3 | `data-filter-text` on rendered tile | unit (HTML assertion) | test in atoms.rs: `tile_legacy_render_is_byte_identical` extended to assert `data-filter-text` present | D-08 |
| SC-4 | confirm button emits `data-disable-on-submit` | unit (HTML assertion) | new test in atoms.rs: `render_button` with `disable_on_submit: Some(true)` → HTML contains `data-disable-on-submit`; without prop → attribute absent | D-21 |
| SC-4 | runtime wires `data-disable-on-submit` | unit (inline-source) | new test in mod.rs: `FERRO_RUNTIME_JS.contains("data-disable-on-submit")` | D-21 |
| SC-4 | docs section exists | build | `cargo doc --no-deps` exits 0; mdBook build `cd docs && mdbook build` exits 0 | D-18, D-22 |
| D-22 | CI-exact gate | CI parity | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | Must run all three, in this order |

### Wave 0 Gaps

None — all test files exist. New test assertions extend existing test functions in `mod.rs` and `atoms.rs`. No new test files need creation.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/config changes only within the existing Rust workspace. No external tools, services, databases, or CLI utilities beyond the Rust toolchain (already confirmed working — workspace compiles at 0.2.86). The `scripts/gen-ferro-base-css.sh` requires Tailwind v4 CLI (auto-installed into `.tooling/bin/` on first run — documented in the script itself).

---

## Security Domain

No new authentication, authorization, session management, or cryptography in this phase. The double-submit guard is a UX protection, not a security boundary — the server-side idempotency key (step 2 of `dispatch_write`) is the actual security control. No ASVS categories directly applicable; the documentation section (D-18) reminds implementors to use the existing `framework::write` idempotency hook rather than building custom deduplication.

---

## Sources

### Primary (HIGH confidence — verified by direct source read)

- `ferro-json-ui/src/runtime/mod.rs` — bundle assembly, dispatcher, two drift-list test arrays (lines 180, 210); exact current state of all setup function names
- `ferro-json-ui/src/runtime/product_tiles.rs` — input event dispatch idiom; function to rename
- `ferro-json-ui/src/runtime/form_guards.rs` — `findGuardedSubmit`, `initNumberGuard` input collection, `opacity-50 cursor-not-allowed` disabled vocabulary
- `ferro-json-ui/src/runtime/tabs.rs` — active-tab class toggle pattern: `border-primary`/`text-primary`/`font-semibold`/`border-transparent`/`text-text-muted`/`hover:text-text`
- `ferro-json-ui/src/component.rs` — current `ProductTileProps`/`ProductGridProps`/`CartPanelProps`/`CategoryNavProps`/`ButtonProps` struct shapes with exact field names and line numbers
- `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` entry at line 253; count assertion at line 1219
- `ferro-json-ui/src/render/atoms.rs` — `render_product_tile` body, `data-product-categories` emission, test module `product_tile_contract_tests`, `make_product_tile` helper, all test fn names
- `ferro-json-ui/src/render/classes.rs` — all five POS_ constants with exact names and values
- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` at line 67; dispatch at line 200
- `ferro-json-ui/src/lib.rs` — `ProductTileProps` public re-export at line 58
- `ferro-json-ui/src/design/rules.rs` — rule ids, `POS_TRIGGER_TYPES`, fixture JSON using `ProductGrid`/`CartPanel`, `RULE_COMPONENTS` in ferro-mcp
- `ferro-mcp/src/tools/json_ui_catalog.rs` — `test_all_components_present` expected list; `RULE_COMPONENTS` mapping; count assertion at line 402
- `app/src/views/cassa.json` — `"type": "ProductTile"`, `"product_id"` field, confirm Button element without `disable_on_submit`
- `app/src/controllers/cassa.rs` — no component type strings; data-only handler
- `docs/src/json-ui/components.md` — `### ProductTile` section structure; Phase 251 migration table precedent at line 72
- `docs/src/design-system/patterns.md` — three POS rule sections with exact content and fixture JSON
- `docs/src/features/write-kernel.md` — existing idempotency documentation; steps 2/5 detail
- `docs/src/SUMMARY.md` — write-kernel.md already listed; no new SUMMARY entries needed
- `docs/protocol/schemas/*.json` — confirmed no ProductTile/component props content
- `.planning/config.json` — no `workflow.nyquist_validation` key → validation enabled by default
- `scripts/gen-ferro-base-css.sh` — Tailwind v4 CLI invocation only; no source scanning override

---

## Metadata

**Confidence breakdown:**
- Cascade inventory: HIGH — every file read directly; line numbers provided
- Runtime patterns: HIGH — verified from tabs.rs, product_tiles.rs, form_guards.rs source
- Test structure: HIGH — drift-list arrays verified at exact line numbers
- CSS regen impact: HIGH — all new class strings already present in existing modules
- Schema export artifacts: HIGH — grep confirmed docs/protocol/schemas/ has no component props

**Research date:** 2026-07-05
**Valid until:** 2026-08-05 (stable codebase; no external dependencies)

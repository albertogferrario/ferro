# Feature Landscape: Touch-First POS Sale Screen Components

**Domain:** Point-of-sale sale screen — product selection + cart building only
**Researched:** 2026-07-04
**Confidence:** HIGH (gestiscilo picker is the primary evidence source — ~1100 lines of battle-tested production code); MEDIUM (real POS product survey — Square, Shopify, Loyverse, Odoo; patterns consistent across sources); LOW (numpad and open-price entry — not represented in the gestiscilo codebase)
**Milestone boundary:** Sale screen only. Payment flow, receipt rendering, and shift/session close are explicitly out of scope.

---

## Primary Evidence Source

The gestiscilo `build_product_picker_html()` function (`app/src/controllers/helpers.rs:423`) is the battle-tested requirements spec. It is ~1100 lines of Rust string-building that produces four HTML+JS fragments for two live production pages (`cassa/orders_nuovo` and `calendario/booking_new`). The function signature is the requirements surface:

```
fn build_product_picker_html(
    products,           // product catalog slice
    categories,         // category label list
    product_categories, // per-product M2M category membership
    existing_qty,       // restore qty state (validation failure re-render)
    existing_people,    // restore people stepper state
    people_ui,          // whether to show people stepper column
    form_id,            // scope isolator for the runtime JS
    eligible_staff_per_product, // optional per-product staff assignment
    initial_cart_staff_state,   // restore staff selection state
) -> (cart_html, search_html, products_html, cart_runtime_html)
```

The four fragments it returns map directly onto the four catalog components this milestone must produce.

---

## Existing Components That Already Cover Part of the Problem

These ferro-json-ui builtins are sufficient for their piece of the sale screen and do NOT need new components:

| Existing Component | Role in Sale Screen | Gap (if any) |
|-------------------|---------------------|--------------|
| `Grid { fill: true, spans: [1,2] }` + `fill_viewport` | Two-pane viewport layout (cart left 1/3, products right 2/3) | None — Phase 253 shipped this specifically for POS |
| `StatCard` | Running total display | None |
| `EmptyState` | Empty cart placeholder | None |
| `Form` + `Button` | Order submission, cancel | None |
| `Input { input_type: Search }` | Search field rendering | Lacks client-side JS filtering hook |
| `DataTable` | Cart line item list (read-only snapshot view) | Cannot do inline qty +/- controls per row with live total sync; mobile cards are touch-friendly but too tall for compact cart density |
| `SegmentedControl` | Tab navigation | URL-based only; does not do client-side product filtering |
| `Tabs` | Panel switching | Switches DOM panels, not card visibility |
| `ProductTile` | Individual product card with +/- controls | Missing: category tokens, qty badge, picked-ring, cart synchronization data attributes |

---

## Table Stakes

Features the sale screen cannot work without. Missing any of these produces a screen that staff cannot use under time pressure.

| Feature | Why Expected | Complexity | Composes From | New Component Needed |
|---------|--------------|------------|---------------|----------------------|
| **Product tap-grid** | Staff must be able to tap a product to add it to the cart in one interaction | HIGH | `ProductTile` shape + Grid layout; cart runtime JS | `ProductGrid` (orchestrates tiles + category filter + search + cart sync) |
| **Cart panel with line items** | Staff must see what is in the current order; without a cart the screen is a picker with no confirmation path | HIGH | None — DataTable cannot do inline qty controls + live total | `CartPanel` (scrollable line items, qty +/− per row, remove-on-zero, empty state, running total header) |
| **Category filter strip** | Any catalog with more than 12-15 products requires filtering to stay usable; tab strip is the universal convention | MEDIUM | `SegmentedControl` shape, but client-side JS filter semantics not URL navigation | `CategoryStrip` OR integrated into `ProductGrid` as a `categories` prop |
| **Client-side product search** | Name search avoids page reload under time pressure; all surveyed POS products include it | LOW | `Input { input_type: Search }` + runtime JS; no new component if integrated into `ProductGrid` | Integrated into `ProductGrid` |
| **Qty badge on product tile** | When a product is already in the cart, its tile must show the current quantity; otherwise staff lose track of what was added | LOW | `ProductTile` extension (add `data-qty-badge` overlay via runtime JS) | Extend `ProductTile` via runtime |
| **Picked-state ring on product tile** | Visually distinguishes in-cart products from unchosen ones; reduces re-add errors | LOW | `ProductTile` extension (CSS inset shadow via `data-picked="true"`) | Extend `ProductTile` via runtime |
| **Cart ↔ tile synchronization** | Qty changes in the cart must propagate to the tile badge and vice-versa; hidden `<input name="qty_{id}">` is the integration contract with the form | HIGH | Cart runtime JS (new `runtime/cart.rs` in ferro-json-ui) | Runtime only, no new component |
| **Running total** | Staff and customers must see the subtotal at all times; recomputation on every qty change | LOW | Cart runtime JS reads `data-row-cents` from cart rows | Runtime only |
| **Remove-on-zero** | Decrementing a cart line to 0 removes it; empty state appears when last line is removed | LOW | Cart runtime JS | Runtime only |
| **Empty cart state** | Clear affordance when no products are selected yet ("Tap a product to add it") | LOW | `EmptyState` or inline empty-state pattern in `CartPanel` | None |
| **Form integration** | Hidden `qty_{id}` inputs per product; `form guard: "number-gt-0"` disabling submit on empty cart | LOW | Existing `Form.guard` mechanism | None |
| **Touch hit targets ≥ 44px** | POS screens are operated by fingers at counter speed; undersized targets cause mis-taps under time pressure | LOW | CSS min-width/min-height; already enforced in current `ProductTile` +/- buttons | Enforce consistently in all new components |

---

## Differentiators

Features that add value but are not blocking for a working sale screen.

| Feature | Value Proposition | Complexity | Dependency |
|---------|-------------------|------------|------------|
| **Numpad component** | Qty and open-price entry without a system keyboard; essential for open-price items (e.g. "custom amount"); used in Square/Shopify for price override and custom qty | MEDIUM | New `Numpad` component; no existing analog |
| **Product image support** | Visual recognition of products; reduces cognitive load when catalog is large; Loyverse defaults to grid-with-images on tablets | LOW | Extend `ProductTile` with `image_url: Option<String>`; add thumbnail rendering with aspect-ratio skeleton |
| **Color-coded tile** | Category-color association as a visual shorthand when images are absent; common in hospitality/bar POS | LOW | Extend `ProductTile` with `color: Option<String>`; render as `background-color` on the tile |
| **Stock badge on tile** | Shows low-stock or out-of-stock state on the tile; prevents adding unavailable items | LOW | Extend `ProductTile` with `stock_badge: Option<String>` (label text, tone-coded) |
| **Favorites / top-sellers pinned page** | A "Preferiti" virtual category tab shows highest-velocity items without navigating; reduces average tap path by 1 for the most common orders | LOW | Data: controller marks items as favorites; category label "★ Preferiti" becomes a regular category tab in `CategoryStrip` |
| **Mobile row weighting** | On phones the product pane should be taller than the cart (253-FRICTION.md gap: currently equal-height rows in fill mode) | MEDIUM | CSS grid `grid-template-rows` proportional sizing; possible `ProductGrid.phone_weight` prop or `CartPanel` compact mode |
| **DataTable compact density** | Cart line items in `DataTable` mobile card mode are too tall for a dense cart; a `density: compact` option would give tighter rows | LOW | Extend `DataTable` with `density` prop; applies `py-1 px-2` cell padding variant |
| **"Uncategorized" virtual tab** | Products with no category membership are unreachable when a category filter is active; a sentinel tab surfacing them is what the gestiscilo picker already implements (`data-tab=""`) | LOW | Logic in `CategoryStrip` or `ProductGrid` |

---

## Anti-Features

Capabilities to explicitly not build in this milestone. Including these would violate the milestone boundary or create premature complexity.

| Anti-Feature | Why Avoid | What Covers It Instead |
|--------------|-----------|------------------------|
| **Payment tender screen** (cash/card/split) | Explicitly out of scope by milestone boundary; payment flow is a separate, subsequent milestone | Milestone boundary |
| **Receipt / fiscal document rendering** | Out of scope; separate milestone or consumer-app concern | Milestone boundary |
| **Shift / session open-close** | Out of scope by milestone boundary | Milestone boundary |
| **Per-line discount entry** | Adds a second input mode to every cart row; complex pricing rules belong in the application layer, not the cart runtime | Application layer (form field + handler) |
| **Custom product modifiers / extras** | Restaurant "add onions", "extra cheese" — complex menu item mutation; not present in gestiscilo's model | Application layer (separate `build_booking_picker_html` variant already handles the staffing dimension) |
| **Barcode scanner integration** | Requires browser hardware API (`navigator.hid` or serial port); outside the hypertext server-rendered model | Plugin system (future `ferro-json-ui` plugin if needed) |
| **Customer lookup / loyalty** | A customer-name field is already in the gestiscilo order form as a plain `Input`; a full customer search widget is scope creep | Existing `Input` + `datalist` for autocomplete |
| **Offline / service worker mode** | Architectural departure; the framework is server-rendered; offline is a v2.0+ direction | Framework direction |
| **Multi-currency** | Not present in gestiscilo; all prices are EUR cents; a formatter `fmt(cents)` is sufficient | Application layer |
| **Inventory management** | Stock levels are read-only display data on the tile (stock_badge differentiator); write-path stock management is a domain concern | Application layer |

---

## Feature Dependencies

```
fill_viewport + Grid { fill, spans }     — layout shell (existing, Phase 253)
    └── CartPanel                         — left pane (NEW)
          ├── CartRuntime (JS)             — synchronization kernel (NEW)
          │     └── ProductGrid            — right pane (NEW)
          │           ├── CategoryStrip    — client-side filter (NEW or integrated)
          │           ├── Search input    — client-side filter (existing Input, runtime hook)
          │           └── ProductTile ×N  — individual tiles (existing, EXTEND)
          ├── StatCard                    — running total (existing)
          └── EmptyState / inline         — empty cart state (existing or inline)

Form + Button                             — submit path (existing)
    └── hidden qty_{id} inputs            — owned by ProductGrid runtime
```

The `CartRuntime` JS is the load-bearing dependency. All visual components are thin wrappers over data attributes; the runtime is what makes them behave as a coherent POS unit.

---

## Candidate Component List

### 1. `ProductGrid` (NEW — headline component)

The promoted form of `build_product_picker_html`'s `products_html` + `search_html` fragments.

**What it must own:**
- Responsive grid rendering of product cards from `data_path` (responsive columns: 2 mobile / 3 sm / 4 md+)
- Each card: name, price, optional image or color swatch, optional stock badge, qty badge overlay driven by runtime, picked-state ring driven by runtime
- Search input (client-side substring filter on product name, lowercase)
- Category tab strip (scrollable, client-side filter; "All" tab always present; first category default-active)
- Emits hidden `<input name="qty_{id}" data-qty-input="{id}" value="N">` per product
- Data attributes for runtime: `data-product-card`, `data-product-id`, `data-product-price-cents`, `data-product-categories`, `data-product-name-lc`

**Props (minimum viable):**
- `data_path: String` — JSON pointer to product array
- `form_id: String` — scope isolator linking to CartPanel
- `categories_path: Option<String>` — JSON pointer to category string array; when absent, no category strip
- `columns: Option<u8>` — override base grid columns (default 2)
- `search: Option<bool>` — enable search input (default true when categories or many items)

**Complexity:** HIGH — owns the most JS and the most data-attribute surface

**Dependency:** new `runtime/cart.rs`; extends `ProductTile` data-attribute model

---

### 2. `CartPanel` (NEW)

The promoted form of `build_product_picker_html`'s `cart_html` fragment.

**What it must own:**
- Fixed-height scrollable body (height: 200px desktop, up to 320px mobile)
- Header: "Prodotti selezionati" label + running total (data-cart-total), item count
- Empty state: visible when no items (`data-cart-empty`)
- Line-item table: Product | Price | Qty (columns: name, line total, qty +/− controls)
- Qty +/− buttons in each row with `data-row-pid` linkage back to `ProductGrid` hidden inputs
- Remove-on-zero: qty reaches 0 → row removed → empty state shown
- Mobile: table collapses to stacked cards with `data-label` attributes

**Props (minimum viable):**
- `form_id: String` — scope isolator matching `ProductGrid.form_id`
- `empty_message: Option<String>` — placeholder text; defaults to "Tap a product to add it."
- `show_staff: Option<bool>` — whether the Staff column is visible (gestiscilo booking mode; false for cassa)
- `show_people: Option<bool>` — whether the People stepper column is visible

**Complexity:** HIGH — owns significant JS interaction surface and must stay synchronized with `ProductGrid`

**Dependency:** `CartRuntime` JS; no new ferro component dependency

---

### 3. `CategoryStrip` (NEW or integrated into `ProductGrid`)

The promoted form of `build_product_picker_html`'s category tab section within `search_html`.

**Decision point:** Can be a standalone component or a prop on `ProductGrid`. Standalone is cleaner for composition (e.g., category tabs above a manually-composed grid); integrated is simpler for the common case where grid + strip always appear together. Recommend: implement as part of `ProductGrid` (via `categories_path` + `search` props), but expose the strip's rendering as an internal sub-renderer that can be tested independently.

**What it must own:**
- Horizontally scrollable tab strip (`overflow-x: auto`, `scrollbar-width: none`, `scroll-snap-type: x proximity`)
- `role="tablist"` with `aria-selected` on active tab
- Each tab: `data-tab="{category}"`, active class vs inactive class
- "All" tab (Italian: "Tutte") always last, active only when no category tabs exist
- Optional "Uncategorized" sentinel tab for products with no category membership
- Client-side JS: tab click → toggle `data-tab` active classes + filter `[data-product-categories]` cards by substring match

**Complexity:** MEDIUM — small JS filter, but accessibility (ARIA) requires care

**Dependency:** `ProductGrid` runtime JS (filter logic lives there)

---

### 4. `Numpad` (NEW — differentiator tier)

A 3×4 touch-optimized numeric entry grid. Drives a target input's value.

**What it must own:**
- 12 buttons: 1 2 3 / 4 5 6 / 7 8 9 / ← 0 .
- Backspace button clears last character
- Updates a target input field identified by `target_field`
- Touch targets minimum 60px (POS numpad buttons are larger than standard 44px)
- Optional `mode: "quantity"` (integer only, no decimal) vs `"price"` (two decimal places)

**Props:**
- `target_field: String` — name of the hidden/text input this numpad drives
- `mode: Option<NumpadMode>` — `quantity` or `price` (default quantity)

**When to use:** open-price items ("custom amount"), quantity override for items where the +/- stepper is too slow (e.g., entering "24" for a bulk order)

**Complexity:** MEDIUM — pure client-side; no server round-trip; runtime JS is self-contained

**Dependency:** None; standalone component

---

### 5. `ProductTile` extensions (EXTEND existing component)

The existing `ProductTile` renders name + price + +/- buttons with a hidden input. It needs extensions to participate in a `ProductGrid`:

- `categories: Vec<String>` — emitted as `data-product-categories="{joined}"` for CategoryStrip filter
- `image_url: Option<String>` — thumbnail rendering with aspect-ratio skeleton on load failure
- `color: Option<String>` — tile background color swatch (CSS color string) for category color coding
- `stock_badge: Option<String>` — label text for a tone-coded stock availability badge

When used standalone (outside a `ProductGrid`), `ProductTile` continues to work as today. The `data-qty-badge` and picked-ring behavior activate only when a `CartRuntime` is present on the page.

**Complexity:** LOW — additive props, backward-compatible

---

## MVP Recommendation

Prioritize in this order:

1. **`ProductGrid`** (with integrated CategoryStrip and search) — without this, the RawHtml escape hatch cannot be closed; it is the reason the milestone exists
2. **`CartPanel`** — without this, `ProductGrid` has nowhere to show the cart state; the two are co-dependent
3. **`CartRuntime` JS** — the kernel that makes components 1 and 2 behave as a coherent unit
4. **`ProductTile` extensions** (categories, image_url, color, stock_badge) — unlocks category filtering in `ProductGrid`
5. **`Numpad`** — differentiator; implement after the table stakes work

Defer: DataTable `density: compact` prop (minor; defers to the next design-system iteration)

---

## Projection Derivation Note

The existing seven-intent vocabulary derives a sale screen without a new intent:

- **Browse** intent → product catalog → `ProductGrid` rendering target
- **Collect** intent → order creation form → `CartPanel` + `Form` + `Button`

A `ServiceDef` with a products model (Browse) and an orders model (Collect) can derive a working two-pane sale screen within the existing derivation surface. The projection → register path is: `ServiceDef` with products/orders → `derive_intents()` → Browse + Collect → `JsonUiRenderer` → `fill_viewport` + `Grid { fill, spans }` + `ProductGrid` + `CartPanel`.

No new intent is needed. The composition is the POS interpretation of the Browse/Collect pairing.

---

## Sources

- gestiscilo `app/src/controllers/helpers.rs:423` — `build_product_picker_html()` (~1100 lines, primary evidence)
- gestiscilo `app/src/views/cassa/orders_nuovo.json` — production spec showing 4 RawHtml elements
- ferro `app/src/views/cassa.json` — Phase 253 demo spec showing the viewport-fill + Grid + ProductTile composition
- ferro `ferro-json-ui/src/component.rs` — existing component surface (ProductTile at line 1340)
- ferro `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — consolidation audit + POS component suite gap analysis
- [Square POS item grid docs](https://squareup.com/help/us/en/article/8334-set-up-item-grid) — category tabs, tap-to-add grid convention
- [Shopify POS UI design principles](https://www.shopify.com/blog/pos-ui) — smart grid, consistency across devices, accessibility baseline
- [Loyverse home sale screen layouts](https://help.loyverse.com/help/home-sale-screen-layouts) — grid vs list view toggle, image-emphasis grid default on tablets
- [Odoo POS grid/list view switcher](https://apps.odoo.com/apps/modules/19.0/codusic_pos_product_view_switcher) — category position flexibility
- [POS system design principles (hashmato.com)](https://hashmato.com/point-of-sale-system-design-principles-tactics/) — three-tap rule, touch ergonomics, visual hierarchy
- [Shopify POS UI Extensions — Tile component](https://shopify.dev/docs/api/pos-ui-extensions/2024-04/components/tile) — smart grid tile patterns

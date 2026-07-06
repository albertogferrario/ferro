---
phase: 256-component-renderers-builtin-lockstep
plan: "01"
subsystem: ui
tags: [ferro-json-ui, render, props, tile, grid, pos, touch, tailwind]

requires:
  - phase: 255-pos-runtime-modules-double-submit-protection
    provides: "initQtyButton tile runtime, data-qty-input/qty-inc/qty-display attribute contract, TOUCH_ACTION/HIT_TARGET_MIN/PRESS_ACTIVE/TAP_HIGHLIGHT constants in classes.rs"
  - phase: 254-props-contracts-touch-foundation-design-rules
    provides: "TileProps.image_url/color/stock_badge additive props, GridProps.row_weights, Tone enum, render/classes.rs POS constants, composition drift-guard"

provides:
  - "TileProps.price_cents: Option<u64> — integer-cents machine-readable unit price emitted as data-unit-price on the tile wrapper"
  - "TileProps.color: Option<Tone> — exhaustive-match enum replacing Option<String>; closes SC-3 dynamic-class injection vector"
  - "SelectionPanelProps.currency: Option<String> — currency symbol for running-total display"
  - "FilterTabsProps.all_label rustdoc corrected to neutral English 'All' (D-28)"
  - "render_tile redesigned as tap-to-add: outer <div> wrapper + inner <button data-qty-inc> + sibling hidden input; Tone border match; lazy image + badge chip; neutral English aria-label"
  - "initQtyButton display-null relaxation: missing data-qty-display no longer blocks tap-to-add tiles"
  - "render_grid row_weights: fill+non-empty weights emit inline style='grid-template-rows: Nfr ...'"
  - "Round-trip tests for price_cents, color Tone enum (incl. negative 'blue'), SelectionPanel.currency, Grid row_weights"

affects:
  - "256-02 through 256-05 (downstream render plans targeting same files)"
  - "Phase 257 projection-builder consuming TileGrid/SelectionPanel registration and data-filter-text/data-unit-price/data-qty-input attributes"
  - "Phase 258 MCP catalog docs consuming the props contracts"

tech-stack:
  added: []
  patterns:
    - "Tile-as-tap-surface: outer <div> wrapper carrying data attrs + inner <button data-qty-inc> + sibling hidden input (inputs inside <button> are invalid HTML)"
    - "Tone exhaustive match to full-literal border classes — never dynamic format!(\"border-{}\", color)"
    - "price_cents integer-only machine-readable money alongside display price string"
    - "initQtyButton display-optional: only input is required for tap-to-add tiles"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/runtime/tiles.rs

key-decisions:
  - "Tile HTML structure: <div wrapper> + <button data-qty-inc> (tap surface) + <input type=hidden> sibling — valid HTML, single tap path"
  - "Tone enum (not Option<String>) for TileProps.color: closes SC-3 dynamic class injection; unknown values impossible at serde parse"
  - "price_cents integer cents only (u64) — never float; display price string kept alongside for UI rendering"
  - "initQtyButton relaxed to display-optional so the same runtime handles both tap-to-add tiles (no display) and standalone QuantityStepper (has display)"
  - "row_style injected as HTML attribute string between class and > — class before style, matching kanban inline-style ordering"

patterns-established:
  - "Tap-to-add tile pattern: wrapper div with data attrs, button as full-surface tap target, hidden input as sibling"
  - "Tone → border class via exhaustive match — no dynamic string formatting for CSS classes"

requirements-completed: [POS-01, POS-09]

duration: ~30min
completed: "2026-07-06"
---

# Phase 256 Plan 01: Props Substrate + Tile Tap-to-Add + Grid row_weights Summary

**TileProps type-safe money+color props, render_tile redesigned as a single-button tap-to-add surface with Tone border exhaustive match, and render_grid fractional row sizing via row_weights inline style**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-07-06T23:45:00Z
- **Completed:** 2026-07-06T00:19:49Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Props substrate locked: `TileProps.price_cents` (integer cents, `data-unit-price`), `TileProps.color` narrowed from `Option<String>` to `Option<Tone>` (SC-3 fix), `SelectionPanelProps.currency`, `FilterTabsProps.all_label` rustdoc neutralized to English "All"
- `render_tile` redesigned: the tile is now a `<div>` wrapper carrying `data-filter-text` + `data-unit-price`, containing a `<button type="button" data-qty-inc>` (full tap surface) + a sibling `<input type="hidden" data-qty-input>`. No on-tile qty display, no dec button, no Italian aria-labels; optional lazy image and stock badge chip rendered
- `initQtyButton` in `runtime/tiles.rs` relaxed: `display` null is now tolerated — only `input` is required; `if (display) display.textContent = next` guards the update. Tap-to-add tiles (no `data-qty-display`) now increment correctly
- `render_grid` extended with `row_style`: in fill mode with non-empty `row_weights`, emits `style="grid-template-rows: Nfr ..."` as a sibling attribute to `class`; empty or scrollable → no style attribute (existing specs render byte-identically)
- 7 new tests added; `tile_legacy_render_is_byte_identical` deleted (superseded by tap-to-add design)

## Task Commits

All three tasks landed as a single commit (per plan instruction):

1. **Tasks 1–3 (all Plan 01 changes)** — `f21c1d55` (feat(256))

## Tile Markup Structure (for Plan 04 reference)

The exact emitted structure (field = `"qty_espresso"`, name = `"Espresso"`):

```html
<div class="border border-border bg-card rounded-lg touch-manipulation"
     data-filter-text="Espresso"
     data-unit-price="250">          <!-- only when price_cents is Some -->
  <button type="button" data-qty-inc="qty_espresso"
          class="min-h-[44px] min-w-[44px] touch-manipulation active:scale-95 active:bg-border pos-tap-highlight ... w-full flex flex-col gap-2 p-3 rounded-lg text-left"
          aria-label="Add Espresso">
    <img src="..." alt="Espresso" loading="lazy" ...>  <!-- only when image_url is Some -->
    <span class="text-sm font-semibold text-text">Espresso</span>
    <span class="text-sm text-text-muted">€2,50</span>
    <span class="...badge...">Low</span>               <!-- only when stock_badge is Some -->
  </button>
  <input type="hidden" name="qty_espresso" data-qty-input="qty_espresso" value="0">
</div>
```

**Key attributes Plan 04 (`selection.rs` reconciler) targets:**
- `data-filter-text` — tile name (on wrapper div)
- `data-unit-price` — integer cents (on wrapper div, when price_cents is set)
- `data-qty-input="{field}"` — hidden form input (sibling of button)
- `data-qty-inc="{field}"` — tap-to-add button (bound by setupTiles at load)

**Tone → border class map:**
- `None` / `Tone::Neutral` → `"border border-border"`
- `Tone::Success` → `"border border-success"`
- `Tone::Warning` → `"border border-warning"`
- `Tone::Destructive` → `"border border-destructive"`

**tiles.rs relaxation:** `if (!input) return;` + `if (display) display.textContent = next;` — missing display is no longer a bail-out condition.

## Files Created/Modified

- `ferro-json-ui/src/component.rs` — `TileProps.price_cents` added; `TileProps.color` changed to `Option<Tone>`; `SelectionPanelProps.currency` added; `FilterTabsProps.all_label` rustdoc corrected; 4 new round-trip tests
- `ferro-json-ui/src/render/atoms.rs` — `render_tile` redesigned (tap-to-add); imports extended with `PRESS_ACTIVE`, `TAP_HIGHLIGHT`; `tile_legacy_render_is_byte_identical` deleted; 3 new tests added
- `ferro-json-ui/src/render/containers.rs` — `render_grid` extended with `row_style` fractional row sizing; 3 new tests added
- `ferro-json-ui/src/runtime/tiles.rs` — `initQtyButton` display-null guard relaxed

## Decisions Made

- **Tile HTML**: `<div>` wrapper (not `<button>` root) so the hidden input can be a valid sibling; `<button>` inside is the tap surface. Valid HTML, clean attribute separation.
- **`Option<Tone>` for color**: Breaking change from `Option<String>` but justified by SC-3 — enum enforcement makes dynamic class injection impossible at parse time; no consumer was using the field (only added in Phase 254, never rendered).
- **Inline format arg fix**: Clippy `uninlined_format_args` caught `format!("{}fr", w)` and `format!("..{}", rows)` — inlined to `format!("{w}fr")` and `format!("..{rows}")` to satisfy `-D warnings`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy uninlined_format_args in row_style**
- **Found during:** Task 3 CI gate (clippy pass)
- **Issue:** `format!("{}fr", w)` and `format!("..{}", rows)` triggered `-D warnings` with `uninlined_format_args`
- **Fix:** Changed to `format!("{w}fr")` and `format!(" style=\"grid-template-rows: {rows}\"")`
- **Files modified:** `ferro-json-ui/src/render/containers.rs`
- **Verification:** `cargo clippy --all --all-targets --all-features -- -D warnings` clean after fix

**2. [Rule 1 - Bug] Missing `price_cents` in tile_with_categories_serializes struct literal**
- **Found during:** Task 3 CI gate (clippy/test compile)
- **Issue:** `tile_with_categories_serializes` constructed `TileProps { ... }` without the new `price_cents` field, causing E0063
- **Fix:** Added `price_cents: None` to the struct literal
- **Files modified:** `ferro-json-ui/src/component.rs`
- **Verification:** Test compiles and passes

---

**Total deviations:** 2 auto-fixed (both Rule 1 — caught at CI gate before commit)
**Impact on plan:** Necessary corrections only; no scope change.

## Known Stubs

None — all plan-scoped fields are wired (price_cents → data-unit-price, color → border class, currency on SelectionPanelProps awaits render_selection_panel in a later plan, row_weights → grid-template-rows inline style).

Note: `SelectionPanelProps.currency` is declared and round-trip tested but `render_selection_panel` does not exist yet — it is authored in a Wave 2 plan of this phase. The field is not a stub: it has a type, serde contract, and test; the render function that reads it is a later plan's work.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. All T-256-01 mitigations applied: `html_escape()` on every string prop (name, price, field, image_url, stock_badge); `price_cents`/`qty` are numeric and formatted directly. T-256-02 closed: `Option<Tone>` exhaustive match eliminates dynamic class construction. T-256-03 unchanged (field sanitization untouched). T-256-04 confirmed: `u8` weights formatted as `{w}fr`, no user string reaches the style attribute.

## Next Phase Readiness

- Wave 2 plans (02–05): props substrate is stable; `render_tile` emits the attribute contract (`data-filter-text`, `data-unit-price`, `data-qty-input`, `data-qty-inc`) that Plan 04's `selection.rs` reconciler targets
- Plan 04 (SelectionPanel): can safely target `data-filter-text` and `data-unit-price` on the tile wrapper `div`, `data-qty-input` as a form input (sibling of button)
- `render_selection_panel` must emit `data-selection-currency` reading from `SelectionPanelProps.currency`

---
*Phase: 256-component-renderers-builtin-lockstep*
*Completed: 2026-07-06*

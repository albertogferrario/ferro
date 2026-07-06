---
phase: 256-component-renderers-builtin-lockstep
verified: 2026-07-06T10:00:00Z
status: human_needed
score: 8/9
overrides_applied: 0
human_verification:
  - test: "Open the register page in a browser; tap a product tile; verify a line appears in the SelectionPanel with the correct name and price, and the running total updates in integer cents."
    expected: "One tap on a tile increments the hidden input; the reconciler creates a line in the panel with the tile name (from data-filter-text) and unit price (from data-unit-price); the running total reflects qty × price_cents."
    why_human: "setupSelection is a JS IIFE dispatched at page load — its reconcile() function clones template nodes at runtime. Static analysis confirms the attribute contract and wiring, but correct DOM manipulation and event-delegation behavior requires a live browser."
  - test: "Tap a category filter tab; verify only tiles with that category token are visible; tap All and verify all tiles reappear."
    expected: "data-filter-tab click triggers updateFilterTabClasses active/inactive toggle and setupFilters show/hide logic against data-filter-tokens on tile wrappers."
    why_human: "Client-side filter behavior (setupFilters) requires a browser with JS execution."
  - test: "Focus the TileGrid search input on an iOS device or simulator; verify the keyboard appears without the viewport zooming."
    expected: "Input carries text-base (16px) — no iOS Safari auto-zoom on focus."
    why_human: "iOS viewport zoom behavior requires device/simulator testing."
---

# Phase 256: Component Renderers + BUILTIN Lockstep — Verification Report

**Phase Goal:** All five new POS builtins are first-class catalog members; a spec author can compose a complete sale screen from `TileGrid`, `SelectionPanel`, `FilterTabs`, `QuantityStepper`, and `Numpad`; `Grid` `row_weights` renders asymmetric layout.
**Verified:** 2026-07-06T10:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | builtin_specs_names_match_dispatch passes at 52; both count guards agree at 52; per-component commits exist | VERIFIED | catalog.rs:1252 `assert_eq!(BUILTIN_TYPES.len(), 52)`; ferro-mcp:405 `52`; History comment catalog.rs:1248-1251 (48→49→50→51→52); commits 912eacf0 / 3af7dfee / fc802fc6 / 3616b2e3 / e4914e01 confirmed in git log |
| 2 | Every interactive element in TileGrid, FilterTabs, SelectionPanel, QuantityStepper, Numpad renders with min-h-[44px]; Numpad keys min-h-[56px] | PARTIAL | Code correctly enforces HIT_TARGET_MIN for all elements (atoms.rs:1446 tile button, 1473 FilterTabs tabs, 1612 QuantityStepper buttons, 1661 Numpad keys; containers.rs:924 TileGrid search, 1572/1598 SelectionPanel line template buttons). Explicit HTML-assertion tests exist for FilterTabs (atoms.rs:3075), QuantityStepper (atoms.rs:3143), and Numpad (atoms.rs:3268). Tile button and SelectionPanel line template buttons lack explicit 44px assertion tests (code is correct; `render_functions_use_constants_not_literals` drift guard prevents raw-literal bypass). |
| 3 | No raw palette class in POS render functions; variant_classes_use_semantic_tokens passes | VERIFIED | Tone exhaustive match atoms.rs:1405-1410 (border-success/warning/destructive — never format!("border-{}")); column class ladder containers.rs uses exhaustive full-literal match; `variant_classes_use_semantic_tokens` test in runtime/mod.rs:87; CI confirmed clean |
| 4 | Grid row_weights emits fractional grid-template-rows; existing Grid specs without row_weights unaffected | VERIFIED | containers.rs:877-888 `row_style` guard: `fill && !props.row_weights.is_empty()`; tests `grid_row_weights_emits_fractional_rows`, `grid_without_row_weights_emits_no_style`, `grid_scrollable_ignores_row_weights` all present; CI green |
| 5 | FilterTabs renders ≥44px targets and filters client-side via data-filter-tokens; TileGrid categories_path populates the filter strip | VERIFIED | `render_filter_tab_strip` emits `data-filter-tab` on buttons with `HIT_TARGET_MIN`; `render_tile_grid` resolves categories_path → `render_filter_tab_strip`; test `tile_grid_categories_path_populates_strip` (containers.rs:3194) asserts data-filter-tab="Drinks" / data-filter-tab="Food" |
| 6 | Tap-to-add-only tile: no on-tile steppers or qty display; tile root is a single tap surface | VERIFIED | render_tile (atoms.rs:1437-1456): button with data-qty-inc; sibling hidden input; no data-qty-display, no data-qty-dec; test `tile_tap_to_add_emits_qty_inc_button` (atoms.rs:2841) asserts no data-qty-display and no data-qty-dec; hidden input confirmed outside </button> |
| 7 | SelectionPanel reconciler (selection.rs) wired into bundle, dispatcher, and both drift-list tests | VERIFIED | runtime/mod.rs:18 `mod selection;`; mod.rs:45 `selection::SOURCE` push; mod.rs:67 setupSelection in dispatcher; mod.rs:211/250 setupSelection in both `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` drift tests; commit 77c8e50c |
| 8 | Integer-cents client total; form-state single source of truth (no data-cart-target props hook) | VERIFIED | selection.rs:5-6 comment + line 126 "T-256-15: integer-cents arithmetic"; formatMoney only uses (n/100).toFixed(2) for display; no `data-cart-target` anywhere in ferro-json-ui/src/ (grep confirmed zero hits) |
| 9 | All five WR-01..WR-05 review fixes present in code | VERIFIED | WR-01: containers.rs:970-973 data-selection-form on TileGrid root; WR-02: selection.rs:79/87 NaN guard `|| 0`; WR-03: json_ui_catalog.rs:100-101 TileGrid in register-selection-present; WR-04: component.rs all_label on TileGridProps + total_label on SelectionPanelProps; WR-05: components.md Tile section rewritten to tap-to-add contract |

**Score:** 8/9 truths fully verified (SC-2 implementation correct, test coverage partial)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/component.rs` | TileProps.price_cents, TileProps.color:Option<Tone>, SelectionPanelProps.currency, TileGridProps.all_label, SelectionPanelProps.total_label | VERIFIED | Lines 1386/1398/1449/1429/1451/1454/1468/1471 confirmed |
| `ferro-json-ui/src/render/atoms.rs` | render_tile (tap-to-add), render_filter_tab_strip, render_filter_tabs, render_quantity_stepper, render_numpad | VERIFIED | Lines 1365, 1461/1471, 1500/1504, 1580, 1646 confirmed |
| `ferro-json-ui/src/render/containers.rs` | render_tile_grid, render_selection_panel, render_grid (row_weights) | VERIFIED | Lines 908, 1538, 877 confirmed |
| `ferro-json-ui/src/render/mod.rs` | BUILTIN_TYPES with all 5 POS components; dispatch arms | VERIFIED | Lines 44-88 (52 entries) and 206-226 (dispatch arms) confirmed |
| `ferro-json-ui/src/catalog.rs` | BUILTIN_SPECS for all 5; count guard at 52; History comment | VERIFIED | Lines 261/279/285/334/376 (specs); 1252 (assert 52); 1248-1251 (History) |
| `ferro-json-ui/src/runtime/selection.rs` | setupSelection reconciler, template-clone lines, integer-cents total | VERIFIED | File created; setupSelection at line 40; reconcile loop; formatMoney at line 175 |
| `ferro-json-ui/src/runtime/mod.rs` | selection module wired + both drift-list tests updated | VERIFIED | mod selection:18; SOURCE push:45; dispatcher:67; drift tests:211/250 |
| `ferro-json-ui/src/runtime/tiles.rs` | initQtyButton display-optional + min/max/step bounds | VERIFIED | Display-null guard at line 26 (`if (!input) return`), display guard at line 35 (`if (display)`); bounds (min/max/step) in initQtyButton |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | Mirror count 52; all 5 POS names in expected list; TileGrid in register-selection-present | VERIFIED | Count at line 405; names at lines 441/453/461-463; register-selection-present at lines 100-101 |
| `ferro-json-ui/assets/ferro-base.css` | Regenerated with all Phase 256 class literals | VERIFIED | aspect-square, object-cover, border-success, overscroll-contain all confirmed present |
| `docs/src/json-ui/components.md` | Tile section updated to tap-to-add contract | VERIFIED | Lines 1413+ rewritten; v16.6 migration table entry at line 112 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| render_tile `<button data-qty-inc>` | initQtyButton in tiles.rs | `document.querySelectorAll('[data-qty-inc]')` at load | WIRED | tiles.rs display-null relaxation means tap-to-add tiles (no data-qty-display) still increment; confirmed in tiles.rs:26/35 |
| render_tile wrapper `data-unit-price` | selection.rs reconciler | `tile.getAttribute('data-unit-price')` in reconcile() | WIRED | selection.rs:125 reads the attribute; atoms.rs:1397-1400 emits it when price_cents is set |
| render_tile_grid `data-filter-scope` | runtime/filters.rs setupFilters | `document.querySelectorAll('[data-filter-scope]')` | WIRED | containers.rs:973 emits data-filter-scope; filter runtime binds to it |
| render_filter_tab_strip `data-filter-tab` | runtime/filters.rs updateFilterTabClasses | data-filter-tab="token" attribute | WIRED | atoms.rs:1476/1487 emits data-filter-tab; class literals match exactly what updateFilterTabClasses toggles (D-12 lockstep verified in REVIEW) |
| SelectionPanel `data-selection-form` | selection.rs setupSelection | `panel.getAttribute('data-selection-form')` | WIRED | containers.rs:1585 emits data-selection-form; selection.rs:50 reads it for form scope |
| setupSelection | FERRO_RUNTIME_JS bundle | runtime/mod.rs SOURCE concatenation + dispatcher | WIRED | mod.rs:45/67; both drift-list tests confirm |
| TileGrid `data-selection-form` (WR-01) | selection.rs form-scope query | sibling markup contract | WIRED | containers.rs:970-973; test tile_grid_emits_selection_form_scope at containers.rs:3157 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| render_tile | name, price, field | TileProps (serde-decoded props) | Spec author-supplied, passed through html_escape | FLOWING |
| render_tile_grid | categories | categories_path resolved via resolve_path(data, p) | Real data-bound array from spec data context | FLOWING |
| selection.rs reconcile() | qty, name, unit_price | form `[data-qty-input]` values + tile DOM attributes | Input events from user taps (live browser only) | FLOWING — browser required |
| render_selection_panel | form_id, empty_message, currency, total_label | SelectionPanelProps (serde-decoded props) | Spec author-supplied | FLOWING |
| render_grid (row_weights) | row_weights | GridProps.row_weights (Vec<u8>) | Spec author-supplied; u8 → "{n}fr" in style attr | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| BUILTIN_TYPES length == 52 | `grep -c "BUILTIN_TYPES.len(), 52" catalog.rs` | Line 1252 found | PASS |
| render_tile emits data-qty-inc button | `grep -c "data-qty-inc" render/atoms.rs` | Lines 1445, 2582, 2854 found | PASS |
| render_tile has no data-qty-dec or data-qty-display | `grep -c "data-qty-dec\|data-qty-display" render/atoms.rs` (outside quantity_stepper scope) | None in render_tile scope (2862/2866 are test assertions) | PASS |
| Grid row_style only in fill mode with non-empty weights | `grep -n "row_style" render/containers.rs` | Lines 875-888 show conditional guard | PASS |
| selection.rs in runtime bundle | `grep -c "setupSelection" runtime/mod.rs` | Lines 67/211/250 (dispatcher + both drift tests) | PASS |
| WR-02 NaN guard present | `grep -c "|| 0) + 1" runtime/selection.rs` | Line 79 found | PASS |
| WR-03 TileGrid in register-selection-present | `grep -c "TileGrid" json_ui_catalog.rs` (rule section) | Line 101 found | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| POS-01 | 256-01-PLAN.md | TileGrid builtin; tap-to-add tiles; text search | SATISFIED | render_tile_grid in BUILTIN_TYPES; tap-to-add render_tile; data-filter-search in TileGrid |
| POS-03 | 256-02-PLAN.md | FilterTabs standalone builtin; ≥44px touch targets | SATISFIED | render_filter_tabs in BUILTIN_TYPES; HIT_TARGET_MIN on tab buttons; filter_tabs_min_44px test |
| POS-04 | 256-03/04-PLAN.md | SelectionPanel live client view; CartRuntime slice | SATISFIED | render_selection_panel + selection.rs reconciler; both wired and tested |
| POS-05 | 256-03-PLAN.md | QuantityStepper builtin; +/- with bounds | SATISFIED | render_quantity_stepper in BUILTIN_TYPES; min/max/step bounds in initQtyButton |
| POS-06 | 256-03-PLAN.md | Numpad builtin; ≥56px keys; never native input | SATISFIED | render_numpad in BUILTIN_TYPES; HIT_TARGET_NUMPAD on keys; hidden input only |
| POS-09 | 256-01-PLAN.md | Grid row_weights asymmetric fill-row weighting | SATISFIED | render_grid row_style path; three tests covering all cases |

**Note on REQUIREMENTS.md tracking table:** The tracking table at lines 355-363 still shows "Not started" for POS-01/03/04/05/06/09. The requirement body text (lines 258-296) has `[x]` checkboxes showing completion. The tracking table was not updated as part of this phase — documentation inconsistency only, not a code issue.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-json-ui/src/render/atoms.rs` | 2841 (`tile_tap_to_add_emits_qty_inc_button`) | Test verifies tap-to-add structure but does not assert HIT_TARGET_MIN on the button class | Warning | Regression detection gap: if HIT_TARGET_MIN is removed from tile button class string, no test fails |
| `ferro-json-ui/src/render/containers.rs` | 3290 (`selection_panel_emits_contract`) | SelectionPanel contract test verifies attribute structure but does not assert HIT_TARGET_MIN on dec/inc/remove buttons in the `<template>` | Warning | Same regression detection gap for SelectionPanel line template buttons |
| `.planning/REQUIREMENTS.md` | 355-363 | Tracking table shows "Not started" for POS-01/03/04/05/06/09 despite implementation being complete | Info | Documentation drift only; requirement body checkboxes correctly show `[x]` |

### Human Verification Required

**SC-2 deferred to browser:** The render code correctly enforces HIT_TARGET_MIN for all interactive elements (code-level enforcement is solid). The missing assertion tests (tile button, SelectionPanel template buttons) are a test coverage gap, not a functional gap — but they were cited as required by SC-2.

#### 1. Live SelectionPanel Reconciler

**Test:** Start the dev server; open a register page that uses TileGrid + SelectionPanel composition. Tap several product tiles.
**Expected:** Each tap increments that tile's hidden input; the reconciler immediately creates (or updates) a line in the SelectionPanel showing the product name and running subtotal; the total at the panel bottom reflects the sum in integer cents formatted as "N.NN". Decrement a line stepper in the panel to zero — the line should disappear (remove-on-zero behavior).
**Why human:** JS template-cloning, event delegation, and DOM manipulation cannot be verified from static code or HTML string assertions. The entire reconcile() loop runs in the browser.

#### 2. Filter Tab Client-Side Filtering

**Test:** On the register page with category data bound via categories_path, click a category tab (e.g. "Drinks"). Verify non-matching tiles are hidden. Click "All" and verify all tiles reappear.
**Expected:** setupFilters toggles visibility based on data-filter-tokens vs data-filter-tab value; updateFilterTabClasses applies active/inactive border classes.
**Why human:** Client-side filter behavior is JS-driven; no server-rendered assertion covers it.

#### 3. iOS 16px Search Input (No Zoom)

**Test:** On iOS Safari (or Simulator), focus the TileGrid search input.
**Expected:** No automatic viewport zoom — the input carries `text-base` (16px minimum font), which prevents the iOS Safari auto-zoom pitfall.
**Why human:** Requires iOS device or simulator; cannot be verified from HTML output alone.

### Gaps Summary

No functional gaps blocking the phase goal. The phase delivers all five POS builtins as first-class catalog members, the tap-to-add tile, the live SelectionPanel reconciler, Grid row_weights, all five review fixes, and a regenerated ferro-base.css.

The only outstanding items are:
1. **Test coverage gap (SC-2):** Tile button and SelectionPanel line template buttons lack explicit HTML-assertion tests for min-h-[44px]. The render code correctly enforces the constraint via HIT_TARGET_MIN constant; the `render_functions_use_constants_not_literals` drift guard prevents raw-literal bypass. This is a regression-test gap, not a functional issue.
2. **REQUIREMENTS.md tracking table** shows "Not started" for completed requirements — stale documentation, not a code issue.
3. **Three human verification items** require live browser testing (JS reconciler, filter behavior, iOS zoom).

---

_Verified: 2026-07-06T10:00:00Z_
_Verifier: Claude (gsd-verifier)_

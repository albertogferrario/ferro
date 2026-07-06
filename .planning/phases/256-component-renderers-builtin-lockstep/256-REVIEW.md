---
phase: 256-component-renderers-builtin-lockstep
reviewed: 2026-07-06T02:02:09Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/runtime/selection.rs
  - ferro-json-ui/src/runtime/tiles.rs
  - ferro-json-ui/src/runtime/mod.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - docs/src/json-ui/components.md
  - ferro-json-ui/assets/ferro-base.css
findings:
  critical: 0
  warning: 5
  info: 4
  total: 9
status: issues_found
---

# Phase 256: Code Review Report

**Reviewed:** 2026-07-06T02:02:09Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Reviewed the Phase 256 POS component work (diff base `f21c1d55^`): five new render functions (`render_tile_grid`, `render_selection_panel`, `render_filter_tabs`, `render_quantity_stepper`, `render_numpad`), the tap-to-add `render_tile` redesign, the new `runtime/selection.rs` reconciler, the `initQtyButton` bounds extension in `runtime/tiles.rs`, the BUILTIN lockstep registrations, and the Grid `row_weights` inline-style path.

**The security posture of the new code is solid.** All prop-derived interpolations in the five new renderers and the redesigned tile pass through `html_escape` (name, price, field, categories, image_url, stock_badge, form_id, empty_message, currency, all_label, tab items); numeric props (`price_cents`, `qty`, `min`/`max`/`step`, `row_weights`) are typed integers interpolated directly, which is injection-safe. The JS reconciler sanitizes field names (stripping `"`, `\`, `]`) before every attribute-selector interpolation, and the sanitized value is used consistently for both `setAttribute` writes and subsequent lookups, so canonicalization is self-consistent. Template-cloned line content is written exclusively via `textContent` (selection.rs:146, 156-157) — no `innerHTML` anywhere in the new runtime. All money arithmetic is integer cents; the only float is the presentational `(n/100).toFixed(2)` in `formatMoney`, never stored or POSTed. No double-binding exists between `setupTiles` (per-element listeners on `data-qty-inc`/`data-qty-dec`) and `setupSelection` (delegation on `data-selection-inc`/`dec`/`remove`): the attribute namespaces are disjoint, `<template>` content is not reachable by `document.querySelectorAll` at `setupTiles` time, and each tile tap dispatches exactly one bubbling `input` event that triggers exactly one reconcile. No raw palette classes or dynamic class construction in the new code — Tone borders and TileGrid columns use exhaustive full-literal matches, and `row_weights` deliberately uses an inline `grid-template-rows` style from `u8` values instead of synthesized class names. The filter-tab active/inactive class literals in `render_filter_tab_strip` match `updateFilterTabClasses` in `runtime/filters.rs` exactly (D-12 lockstep verified).

Lockstep registrations check out: `BUILTIN_TYPES` contains exactly 52 entries (27 atoms + 15 containers + 7 form + 3 data) with dispatch arms for all five new types; `catalog.rs` adds the five `BUILTIN_SPECS` entries with a relational length guard and the count pin updated to 52; `ferro-mcp` mirrors 52 with all five names in its expected list. `ferro-base.css` is plausibly regenerated — it contains the new utilities (`56px` numpad hit target, `2ch` qty width, `pos-tap-highlight`, `aspect-square`, `auto-rows-fr`, `overscroll-contain`, `grid-cols-3/4`).

Five warnings: a required-but-unused `TileGrid.form_id` prop (contract/implementation mismatch), a missing NaN guard on the panel's inc/dec handlers, a missing `TileGrid` entry in the MCP `register-selection-present` rule mapping, non-overridable user-visible English strings ("All", "Total") in a project-agnostic crate, and stale/incomplete component documentation.

## Warnings

### WR-01: `TileGridProps.form_id` is required but never used by the renderer

**File:** `ferro-json-ui/src/component.rs:1408-1409`, `ferro-json-ui/src/render/containers.rs:905-970`
**Issue:** `form_id` is a required (non-optional) prop documented as "Scope isolator that links this grid's hidden inputs to a sibling SelectionPanel", but `render_tile_grid` decodes it and never reads it — no `form="{form_id}"` attribute is emitted on the tile hidden inputs and no scoping markup is produced. The linkage only works if the TileGrid happens to be a DOM descendant of `<form id="{form_id}">`. A spec author who trusts the prop doc and places the TileGrid outside the form (as a sibling of the SelectionPanel, which the doc invites) gets tiles whose taps mutate inputs that neither submit with the form nor appear in the panel's `form.querySelectorAll('[data-qty-input]')` — a silent no-op register. Meanwhile, omitting the prop entirely is a hard decode error even though it has no effect.
**Fix:** Either (a) make the prop functional — thread `form_id` into the tile hidden inputs as the HTML `form` attribute (`<input type="hidden" form="{form_id}" ...>`), which makes sibling placement work; or (b) make the prop optional, correct its rustdoc to state that the TileGrid must be a descendant of the target form, and add a design-lint rule that flags a TileGrid whose `form_id` doesn't match an enclosing Form `id`.

### WR-02: SelectionPanel inc/dec handlers lack the NaN guard used everywhere else

**File:** `ferro-json-ui/src/runtime/selection.rs:79, 87`
**Issue:** The delegated per-line handlers parse without a fallback:
```js
input.value = parseInt(input.value, 10) + 1;                 // line 79
input.value = Math.max(0, parseInt(input.value, 10) - 1);    // line 87
```
If `input.value` is ever non-numeric (empty string, external mutation), `parseInt` yields `NaN`, `NaN + 1` is `NaN`, and `Math.max(0, NaN)` is also `NaN` — the hidden input is set to the literal string `"NaN"`, which then POSTs as the field value on confirm (the reconciler's own `|| 0` masks the corruption visually, showing qty 0 while the form carries `"NaN"`). The sibling code in `runtime/tiles.rs:27` (`parseInt(input.value, 10) || 0`) and the reconciler itself (selection.rs:118) both guard this; only the panel's inc/dec paths omit it.
**Fix:**
```js
input.value = (parseInt(input.value, 10) || 0) + 1;
// ...
input.value = Math.max(0, (parseInt(input.value, 10) || 0) - 1);
```

### WR-03: MCP `register-selection-present` rule mapping omits `TileGrid`

**File:** `ferro-mcp/src/tools/json_ui_catalog.rs:99-102`
**Issue:** The rule is titled "A TileGrid register needs a SelectionPanel" and its check (`check_pos_cart_present` in `ferro-json-ui/src/design/rules.rs:495-511`) gates exclusively on `TileGrid` presence. Yet `RULE_COMPONENTS` maps it to `&["Grid", "Numpad", "SelectionPanel"]` — the one component that actually triggers the rule is absent. An agent fetching the TileGrid catalog entry's `component_guidance` will not see the most important composition rule for that component, while `Numpad` (which the check never inspects) is listed. The other two POS rules were correctly extended with `TileGrid` in this same commit, so this looks like an oversight rather than a decision.
**Fix:**
```rust
(
    "register-selection-present",
    &["Grid", "TileGrid", "Numpad", "SelectionPanel"],
),
```

### WR-04: User-visible English strings without override in a project-agnostic crate

**File:** `ferro-json-ui/src/render/containers.rs:943` (TileGrid "All"), `ferro-json-ui/src/render/containers.rs:1601` (SelectionPanel "Total")
**Issue:** Two visible-text strings are hardcoded with no prop override:
1. `render_tile_grid`'s integrated category strip calls `render_filter_tab_strip(&categories, "All")` with a literal. The standalone `FilterTabs` exposes `all_label` precisely for this ("Pass `all_label: "Tutte"` or any locale string from the consumer" — component.rs:1449-1453), but `TileGridProps` has no equivalent, so the integrated strip — the primary register path — cannot be localized. The Phase 257 consumer is Italian-locale; this surfaces immediately.
2. `render_selection_panel` renders a visible `<span>Total</span>` with no override, while the same component's `empty_message` is overridable. Inconsistent within the same renderer.
**Fix:** Add `all_label: Option<String>` to `TileGridProps` (default "All", passed through to `render_filter_tab_strip`) and `total_label: Option<String>` to `SelectionPanelProps` (default "Total"), mirroring the existing `empty_message`/`all_label` pattern.

### WR-05: components.md is stale — five new builtins undocumented, Tile section contradicts its own migration row

**File:** `docs/src/json-ui/components.md:19, 23-36, 1409-1437`
**Issue:** The doc states "The sections below document every built-in component" (line 19), but TileGrid, SelectionPanel, FilterTabs, QuantityStepper, and Numpad appear in neither the Component Overview table (lines 23-36, Commerce row still lists only "Tile") nor the body sections. Worse, the Tile section (lines 1411-1435) still documents the retired interaction — "+/− buttons that drive a hidden form input" — and its props table omits all six Phase 254/256 props (`categories`, `image_url`, `color`, `stock_badge`, `price_cents`), directly contradicting the v16.6 migration row added at line 112 in this same phase ("the tile root is now a tap-to-add button... quantity editing moved to the SelectionPanel"). Project instructions require docs to reflect current features, and doc accuracy is held to the same bar as the Rust API.
**Fix:** Update the Tile section body and props table to the tap-to-add contract, add the five new components to the overview table and body sections (Commerce: TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad), and add `numpad_mode` (`"quantity"` | `"price"`) to the Component-Specific Enum Values list.

## Info

### IN-01: Reconciler double-counts the total when two inputs share a field

**File:** `ferro-json-ui/src/runtime/selection.rs:112-128`
**Issue:** `reconcile()` accumulates `totalCents` per `[data-qty-input]` element, not per unique field. If a spec composes two inputs with the same `field` in one form scope (e.g. a Tile plus a standalone QuantityStepper bound to the same field — both emit their own hidden input), the line renders once but the total counts it twice, and the form POSTs duplicate field entries.
**Fix:** Track seen fields in the loop (`if (seen[field]) continue; seen[field] = true;`), or document the one-input-per-field invariant and add a design-lint check.

### IN-02: Panel inc/dec bypasses declared `data-qty-min`/`max` bounds

**File:** `ferro-json-ui/src/runtime/selection.rs:73-98` vs `ferro-json-ui/src/runtime/tiles.rs:29-33`
**Issue:** Bounds enforcement (D-22) lives on the tile/stepper buttons and is honored only in `initQtyButton`. The SelectionPanel's delegated inc/dec writes the input directly with a hardcoded floor of 0 and no ceiling — if a future tile or stepper declares `max` (e.g. stock limits in Phase 257), incrementing from the panel silently exceeds it.
**Fix:** When resolving the input in the panel handlers, also read bounds from the associated stepper buttons (or store them as data attributes on the input itself at render time) and clamp identically to `initQtyButton`.

### IN-03: Non-localizable English aria-labels in the new renderers

**File:** `ferro-json-ui/src/render/atoms.rs:1448` ("Add {name}"), `ferro-json-ui/src/render/atoms.rs:1620,1624` ("Decrease/Increase {field}"), `ferro-json-ui/src/render/containers.rs:1580-1588` ("Decrease"/"Increase"/"Remove"), Numpad "Clear"/"Backspace"
**Issue:** Screen-reader labels are hardcoded English with no override, and the stepper labels interpolate the machine field name (e.g. "Increase qty_p1"), which is meaningless to assistive-tech users. Lower stakes than WR-04 (not visible), but the same project-agnostic principle applies.
**Fix:** Acceptable as neutral defaults for now; consider optional label props or an item-name interpolation when the Phase 257 projector wires real product names through.

### IN-04: `QuantityStepper` `step: 0` silently coerced to 1 by the runtime

**File:** `ferro-json-ui/src/runtime/tiles.rs:29`, `ferro-json-ui/src/component.rs:1468-1470`
**Issue:** `parseInt(btn.getAttribute('data-qty-step'), 10) || 1` maps a declared `step: 0` (valid `u32`) to 1 with no diagnostic. Harmless, but the schema permits a value the runtime reinterprets.
**Fix:** Note the minimum in the prop rustdoc ("step ≥ 1; 0 is treated as 1"), or reject 0 at the schema level.

---

_Reviewed: 2026-07-06T02:02:09Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

---
status: complete
phase: 257-projection-builder-register-layout-template
source: [257-VERIFICATION.md]
started: 2026-07-06T13:20:00Z
updated: 2026-07-06T16:45:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Tablet visual quality of the projection-derived /cassa register

expected: Open `/cassa` in Chrome DevTools MCP at a tablet viewport (e.g.
1024×768). The product tile grid renders with names/prices (e.g. "Caffè").
Tapping tiles updates the SelectionPanel live view with lines and a running
total. The cart pane and product grid scroll independently inside their
panes; the document body itself never scrolls (fill-viewport register feel).
result: issue
reported: "Verified live via Chrome DevTools MCP at 1024×768: tiles render
  (24, names+prices), taps update the panel (2×Caffè + Aperol → lines with
  steppers, Total 8.40 integer-cents correct), remove works (Total 2.40),
  search filters (only Toast visible for 'toast'), EmptyState toggles, body
  never scrolls. BUT the SelectionPanel Total row and 'Conferma ordine'
  confirm button sit at y≈1032–1125 in a 746px viewport — off-screen; the
  panel does not pin, the whole workspace (both panes together) scrolls
  inside the outer fill-grid cell instead of each pane scrolling
  independently. The 256 D-15 pinned header/total/confirm contract is broken
  on the projection-derived page."
severity: major

### 2. Live geometry re-verify — SelectionPanel footer in-viewport after 257-04 fix

expected: Re-open `/cassa` in Chrome DevTools MCP at 1024×768. (a) The
SelectionPanel Total row and "Conferma ordine" button are fully visible
without scrolling (y < 746px); (b) the tiles pane and the cart lines each
scroll independently inside their panes; (c) the document body does not
scroll. Compare against the pre-fix screenshot at
`app/tmp/257-cassa-uat-tablet.png`. The 257-04 fix emits
`flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0` on the sale_form
so it constrains to the outer fill-grid cell height instead of rendering
content-sized.
result: pass
reported: "Verified live via Chrome DevTools MCP at 1024×746 (post-fix,
  commit 156150e0): form#sale_form now 673px with classes 'flex flex-col
  h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0' (was 1076px content-sized).
  Total row at y=652-672 and 'Conferma ordine' at y=686-722 — both fully
  in the 746px viewport with 10 cart lines present (Total 29.50, integer-
  cents correct). Two independent scrollers: tiles pane
  ('md:col-span-2 min-h-0 h-full overflow-y-auto', 673/1076) and cart
  lines ('flex-1 overflow-y-auto min-h-0', 573/610), neither containing
  the confirm button — footer pins outside both. documentElement not
  scrollable. Screenshot: app/tmp/257-cassa-uat-tablet-postfix.png"

### 3. Register visual quality (user-reported after test 2)

expected: No visible scrollbars on the register panes; SelectionPanel line
elements aligned between rows; product names legible; no unlabeled controls.
result: pass (after fixes)
reported: "User feedback on the test-2 state: visible scrollbars, cart line
  elements misaligned between rows, product names unreadable, and an
  unidentifiable empty block above the tile grid (the search input rendered
  with an empty placeholder). Fixed in commit d9eb4ea3: scrollbar-none
  utility on both register scrollers; SelectionPanel line template
  restructured to a two-deck row (name text-base font-medium + tabular-nums
  total on top, stepper + remove below) — the old single-row layout collapsed
  names to zero width and clipped totals at tablet pane width;
  TileGridProps.search_placeholder with neutral 'Search' default. Re-verified
  live at 1024×746 and 1366×768: one x-position per control column across all
  rows (2-digit qty included), scrollbar gutters 0px, names 16px/500 visible,
  footer pinned, document never scrolls. Screenshots:
  app/tmp/257-cassa-uat-tablet-visualfix4.png (final),
  visualfix2/visualfix3 (intermediate), 257-cassa-uat-empty.png (EmptyState)."

## Summary

total: 3
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

Note: test 1's original issue (footer off-viewport) is resolved — closed by
gap plan 257-04 and re-verified live as test 2. Test 3's visual issues are
resolved by commit d9eb4ea3 and re-verified live. Localization note: the
panel EmptyState ("No items selected"), total label ("Total"), and search
placeholder ("Search") are D-28 neutral English defaults — the sample app's
projection can pass Italian overrides via SelectionPanelProps/TileGridProps
if desired (candidate Phase 258 docs example).

## Gaps

```yaml
- truth: "Under fill_viewport, the SelectionPanel pins with header/total/confirm visible while its lines container and the tiles pane scroll independently (256 D-15); the operator can always see and reach the running total and confirm button"
  status: resolved
  reason: "User-observed (live browser, 1024×768): Total row at y=1032-1081 and confirm button at y=1089-1125 are outside the 746px viewport; body overflow hidden so they are only reachable by scrolling the ENTIRE workspace (both panes move together) inside the outer fill-grid cell. FIXED by gap-closure plan 257-04 (commits eef721b9, 156150e0): FormProps.fill + fill-aware render_form + emit_register_root fill:true — verified mechanistically (class-chain assertions at 3 layers), live geometry re-check pending as test 2"
  severity: major
  test: 1
  artifacts:
    - "ferro-json-ui/src/projection/builder.rs (emit_register_root — composes root Grid(fill:true) → Form#sale_form → inner Grid(h-full min-h-0) → panes)"
    - "ferro-json-ui/src/render (render_form emits 'flex flex-wrap' with NO height-chain classes)"
    - "app/tmp/257-cassa-uat-tablet.png (screenshot showing panel footer off-viewport)"
  missing:
    - "Height-chain propagation through the Form layer: the outer fill-Grid cell is correctly constrained (673px, overflow-y-auto) but FORM#sale_form renders content-sized (1076px, 'flex flex-wrap', no h-full/min-h-0), so the inner panes Grid's h-full resolves against 1076px instead of 673px"
    - "Root cause: the 256 D-11 Form-as-common-ancestor composition puts the Form INSIDE the fill-height chain — the hand-authored cassa.json never had this (its confirm form was a small leaf inside the cart pane). render_form has no way to emit fill/height classes"
    - "Candidate fix (planner to confirm): additive FormProps.fill: Option<bool> (serde default, skip_serializing_if) — render_form emits the height-chain classes (h-full min-h-0 + a stretching display, e.g. grid or flex flex-col with child stretch) when true; emit_register_root sets fill: true on sale_form; regression coverage via HTML class assertions + an app-side geometry-representative assertion; consider a lint follow-up for Form-in-fill-chain"
```

---
phase: 255-pos-runtime-modules-double-submit-protection
plan: 01
subsystem: ui
tags: [ferro-json-ui, vocabulary-rename, POS, touch-foundation, SC-0]

# Dependency graph
requires:
  - phase: 254-props-contracts-touch-foundation-design-rules
    provides: ProductTileProps with categories, POS_ touch constants, design rules

provides:
  - TileProps (renamed from ProductTileProps; field item_id replaces product_id)
  - TileGridProps, SelectionPanelProps (show_staff/show_people removed), FilterTabsProps
  - render_tile emitting data-filter-tokens (HTML-escaped) and always-emitted data-filter-text
  - Six neutral touch constants (TOUCH_ACTION, HIT_TARGET_MIN, HIT_TARGET_NUMPAD, PRESS_ACTIVE, OVERSCROLL_CONTAIN, TAP_HIGHLIGHT)
  - Catalog entry Tile (was ProductTile); BUILTIN_TYPES count stays 47
affects:
  - 255-02 (design/rules.rs rename; uses REGISTER_TRIGGER_TYPES)
  - 255-03 (runtime/mod.rs, ferro-mcp, docs, app/views/cassa.json)
  - 256 (Phase 256 renderers target TileGridProps/SelectionPanelProps/FilterTabsProps)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Vocabulary neutralization compile-atomicity: rename types+render fn+constants together so crate compiles again
    - data-filter-text always-emitted on Tile (universal tile marker + search source for setupFilters)
    - data-filter-tokens conditional (non-empty categories only); data-filter-text unconditional

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/classes.rs

key-decisions:
  - "make_product_tile helper renamed to make_tile to pass SC-0 grep gate (product_tile substring matches)"
  - "data-filter-text emitted unconditionally in render_tile (D-08: every tile must be findable by setupFilters even without categories)"

patterns-established:
  - "Rename compile-atomically: Tasks 1+2 can be committed mid-rename because per-task verification is grep-based; Task 3 consumes both renames and is the compile + test gate"

requirements-completed: [SC-0]

# Metrics
duration: 15min
completed: 2026-07-05
---

# Phase 255 Plan 01: Vocabulary Neutralization — Props, Constants, Render Fn Summary

**Domain-neutral rename across six ferro-json-ui source files: TileProps/render_tile/TOUCH_ACTION replace POS-named identifiers; data-filter-tokens replaces data-product-categories; data-filter-text always emitted; 748 tests green.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-05T12:30:00Z
- **Completed:** 2026-07-05T12:47:09Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- ProductTileProps → TileProps (field product_id → item_id); ProductGridProps → TileGridProps; CartPanelProps → SelectionPanelProps (show_staff/show_people removed); CategoryNavProps → FilterTabsProps — all in component.rs, lib.rs, catalog.rs
- Six POS_ touch constants renamed without prefix in render/classes.rs; class VALUE strings byte-identical; render/mod.rs dispatch arm updated to Tile/render_tile
- render_tile in atoms.rs: data-product-categories → data-filter-tokens; added always-emitted data-filter-text (D-08); renamed make_product_tile → make_tile to pass SC-0 grep gate; T-255-01 tile_escapes_categories XSS guard preserved
- cargo test -p ferro-json-ui --all-features: 748 tests pass (0 failures); cargo fmt clean; cargo clippy clean

## Task Commits

1. **Task 1: Rename Props structs and fields in component.rs, lib.rs, catalog.rs** - `8bb6dc46` (feat)
2. **Task 2: Rename POS_ touch constants in classes.rs; update render/mod.rs dispatch** - `effa19e0` (feat)
3. **Task 3: Rename render_product_tile and all atoms.rs sites; compile and test** - `b0a37a88` (feat)

## Files Created/Modified
- `ferro-json-ui/src/component.rs` — TileProps/TileGridProps/SelectionPanelProps/FilterTabsProps; test module tile_contract_tests
- `ferro-json-ui/src/lib.rs` — pub use TileProps (alphabetical reorder by cargo fmt)
- `ferro-json-ui/src/catalog.rs` — BUILTIN_SPECS entry Tile/schema_for!(TileProps); count 47 untouched
- `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES["Tile"]; dispatch arm render_tile
- `ferro-json-ui/src/render/atoms.rs` — render_tile; data-filter-tokens; data-filter-text; renamed tests; make_tile helper
- `ferro-json-ui/src/render/classes.rs` — six neutral constants; test renamed touch_constants_are_full_literals_and_token_compliant

## Decisions Made
- Renamed `make_product_tile` helper to `make_tile` (not mentioned as optional in plan task text, but required to pass SC-0 grep which matches `product_tile` substring)
- `data-filter-text` emitted as plain HTML attribute on the outer tile div, unconditionally — placed before `{categories_attr}` in the format string

## Deviations from Plan

None — plan executed exactly as written. The `make_product_tile` → `make_tile` rename was the plan-stated option ("keep helper name make_product_tile, or rename to make_tile and update all callers") — chose rename to satisfy SC-0 grep gate.

## Issues Encountered
- cargo fmt reformatted import lists (TileProps alphabetical ordering) and condensed render_tile function signature to one line — applied fmt and re-verified all acceptance criteria held.

## Known Stubs

None — this plan is a pure rename; no new UI rendering stubs introduced.

## Threat Flags

No new attack surface. T-255-01 (XSS on data-filter-tokens) mitigated: html_escape preserved in categories_attr block and tile_escapes_categories test kept. T-255-02 (show_staff/show_people removal) accepted: no serde deny_unknown_fields on any component struct.

## Next Phase Readiness
- Plan 255-02 can now rename design/rules.rs (POS_ rule ids → register-*, POS_TRIGGER_TYPES → REGISTER_TRIGGER_TYPES)
- Plans 255-03+ can rename runtime/product_tiles.rs → tiles.rs, update ferro-mcp, docs, app/views/cassa.json
- Phase 256 renderers can target TileGridProps/SelectionPanelProps/FilterTabsProps without vocabulary drift risk

## Self-Check

### Files exist:

- `ferro-json-ui/src/component.rs` with `pub struct TileProps` ✓
- `ferro-json-ui/src/render/atoms.rs` with `pub(crate) fn render_tile` ✓
- `ferro-json-ui/src/render/classes.rs` with `pub const TOUCH_ACTION` ✓

### Commits exist:

- `8bb6dc46` ✓
- `effa19e0` ✓
- `b0a37a88` ✓

## Self-Check: PASSED

---
*Phase: 255-pos-runtime-modules-double-submit-protection*
*Completed: 2026-07-05*

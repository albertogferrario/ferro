---
phase: 255-pos-runtime-modules-double-submit-protection
plan: 03
subsystem: ui
tags: [ferro-mcp, docs, vocabulary-rename, POS, SC-0, cassa]

# Dependency graph
requires:
  - phase: 255-01
    provides: TileProps, BUILTIN_TYPES["Tile"], renamed render_tile
  - phase: 255-02
    provides: register-* rule ids in design/rules.rs; patterns.md already updated

provides:
  - ferro-mcp RULE_COMPONENTS with register-* ids (register-fill-viewport/register-grid-fill/register-selection-present)
  - ferro-mcp test_all_components_present expects "Tile" (count 47 unchanged)
  - app/src/views/cassa.json tile element: type Tile, prop item_id (no disable_on_submit)
  - docs/src/json-ui/components.md: Tile section + grep-clean v16.6 rename migration note
  - SC-0 global grep gate: zero hits across ferro-json-ui/src, ferro-mcp/src, app/src, docs/src

affects:
  - 255-04 (numpad runtime module; builds on closed SC-0 vocabulary)
  - 255-05 (disable_on_submit: SC-4; adds ButtonProps field then wires cassa.json)
  - 256 (renderers target TileGrid/SelectionPanel component types)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - SC-0 grep gate closure pattern: retire token everywhere (Rust, JSON specs, MCP mirror, docs) in one wave of plans (255-01/02/03)
    - Migration note authoring: descriptive wording ("formerly the product-prefixed ...") avoids re-spelling retired compound identifiers while still documenting the break for consumers

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - app/src/views/cassa.json
    - docs/src/json-ui/components.md

key-decisions:
  - "Migration note uses descriptive prose (no retired compound substrings) — satisfies both SC-0 zero-hits gate and V-07 consumer documentation requirement simultaneously"
  - "patterns.md already updated in 255-02 (drift guard forced it); no duplicate edit needed in 255-03"

patterns-established:
  - "docs/src/design-system/patterns.md: the patterns_md_matches_rule_registry drift guard in ferro-json-ui enforces parity at test time — patterns.md cannot lag rule renames"

requirements-completed: [SC-0]

# Metrics
duration: ~15min
completed: 2026-07-05
---

# Phase 255 Plan 03: ferro-mcp Mirror + cassa.json + Docs Rename; SC-0 Closed Summary

**Downstream mirror updates close SC-0 to global zero hits: ferro-mcp RULE_COMPONENTS register-* ids + Tile name, cassa.json type/field rename, docs Tile section with grep-clean v16.6 migration note. CI-exact gate green.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-05T14:59:38Z
- **Completed:** 2026-07-05T15:15:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- ferro-mcp RULE_COMPONENTS: three POS rule ids renamed to register-fill-viewport / register-grid-fill / register-selection-present; comment updated from ProductGrid/CartPanel to TileGrid/SelectionPanel
- ferro-mcp test_all_components_present: "ProductTile" → "Tile" in expected array; count 47 unchanged; test passes
- cassa.json tile element: `"type": "ProductTile"` → `"type": "Tile"`, `"product_id"` → `"item_id"`; disable_on_submit intentionally absent (Plan 255-05)
- docs/src/json-ui/components.md: Commerce index row Tile; § Commerce Components / ### Tile with item_id prop, neutral touch-tile language; v16.6 rename migration note added (descriptive wording — no retired compound substrings)
- docs/src/design-system/patterns.md: already clean from Plan 255-02; no changes needed
- Global SC-0 grep gate: `grep -rn 'ProductTile|product_tile|setupProductTiles|data-product-|CartPanel|CategoryNav|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` → zero hits
- CI-exact gate green: fmt + clippy --all-targets -D warnings + test --all-features

## Task Commits

1. **Task 1: Update ferro-mcp catalog mirror — register-* rule ids + Tile name** — `16b1c785` (feat)
2. **Task 2: Rename cassa.json tile element — type Tile, prop item_id** — `65131b03` (feat)
3. **Task 3: docs — Tile section + v16.6 rename migration note; close SC-0** — `7c8a610b` (feat)

## Files Created/Modified

- `ferro-mcp/src/tools/json_ui_catalog.rs` — RULE_COMPONENTS register-* ids; comment TileGrid/SelectionPanel; expected array "Tile"
- `app/src/views/cassa.json` — tile element: type Tile, item_id
- `docs/src/json-ui/components.md` — Commerce index Tile; ### Tile section; v16.6 migration table

## Decisions Made

- Migration note authored with descriptive wording ("formerly the product-prefixed type string / prop name / data attribute") rather than spelling out retired compound identifiers — satisfies SC-0 zero-hits gate while fully documenting the break for consumers.
- `docs/src/design-system/patterns.md` required no changes: the `patterns_md_matches_rule_registry` drift guard forced 255-02 to update it atomically with rules.rs, so it was already clean.

## Deviations from Plan

None — plan executed exactly as written. Task 3 noted that patterns.md was already updated in 255-02 (drift-guard enforcement); the plan's Task 3 action for patterns.md was still satisfied because 255-02 performed the identical edits (register-* headings, TileGrid/SelectionPanel, allow-list strings).

## Known Stubs

None — this plan is a pure rename across mirrors and docs; no new rendering stubs introduced.

## Threat Flags

No new attack surface. T-255-05 (ferro-mcp catalog name drift) fully mitigated: test_all_components_present asserts "Tile" against the live catalog, count 47 guarded. T-255-06 (docs migration note repudiation) accepted per plan threat model.

## Next Phase Readiness

- Plan 255-04 (numpad runtime module) can proceed against the fully closed SC-0 vocabulary
- Plan 255-05 (disable_on_submit / SC-4) adds ButtonProps field then wires cassa.json btn_confirm
- Phase 256 renderers target TileGrid/SelectionPanel without vocabulary drift risk

## Self-Check

### Files exist:

- `ferro-mcp/src/tools/json_ui_catalog.rs` with `register-fill-viewport` ✓
- `ferro-mcp/src/tools/json_ui_catalog.rs` with `"Tile"` in expected array ✓
- `app/src/views/cassa.json` with `"type": "Tile"` and `"item_id"` ✓
- `docs/src/json-ui/components.md` with `### Tile` ✓

### Commits exist:

- `16b1c785` ✓
- `65131b03` ✓
- `7c8a610b` ✓

## Self-Check: PASSED

---
*Phase: 255-pos-runtime-modules-double-submit-protection*
*Completed: 2026-07-05*

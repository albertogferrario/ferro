---
phase: 255-pos-runtime-modules-double-submit-protection
plan: 02
subsystem: ui
tags: [ferro-json-ui, vocabulary-rename, POS, runtime, design-rules, SC-0]

# Dependency graph
requires:
  - phase: 255-01
    provides: TileProps, TileGridProps, SelectionPanelProps, FilterTabsProps (compile-complete rename)

provides:
  - runtime/tiles.rs with setupTiles (product_tiles.rs removed)
  - runtime/mod.rs wired: mod tiles; concat tiles::SOURCE; dispatcher setupTiles(); both drift lists updated
  - form_guards.rs comment neutralized (Skip ProductTile -> Skip Tile)
  - design/rules.rs: register-fill-viewport, register-grid-fill, register-selection-present rule ids
  - REGISTER_TRIGGER_TYPES = [TileGrid, SelectionPanel, Numpad]
  - docs/design-system/patterns.md: three register-* rule sections updated

affects:
  - 255-03 (runtime/mod.rs, ferro-mcp, docs, app/views/cassa.json — builds on setupTiles)
  - 256 (Phase 256 renderers target TileGrid/SelectionPanel component types)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Drift guard enforcement: patterns_md_matches_rule_registry blocks rule-id rename without doc update
    - cargo fmt reorders alphabetical mod declarations on rename (tiles after tabs, before toasts)

key-files:
  created: []
  modified:
    - ferro-json-ui/src/runtime/tiles.rs (renamed from product_tiles.rs; setupProductTiles -> setupTiles)
    - ferro-json-ui/src/runtime/mod.rs (mod tiles; concat; dispatcher; both drift arrays)
    - ferro-json-ui/src/runtime/form_guards.rs (comment neutralized)
    - ferro-json-ui/src/design/rules.rs (rule ids, REGISTER_TRIGGER_TYPES, type comparisons, tests)
    - docs/src/design-system/patterns.md (three register-* sections)

key-decisions:
  - "patterns_md_matches_rule_registry drift guard treated as Rule 1 fix: updated patterns.md in same commit as rules.rs"
  - "check_pos_cart_present has_grid chain reformatted to single line by cargo fmt (SelectionPanel chain kept multi-line)"

patterns-established:
  - "Design-rule rename requires three sites: registry entry + check function + patterns.md section (drift guard enforces the third)"

requirements-completed: [SC-0]

# Metrics
duration: ~8min
completed: 2026-07-05
---

# Phase 255 Plan 02: Runtime Module Rename + Lint Rule-ID Rename Summary

**Runtime module product_tiles.rs renamed to tiles.rs with setupTiles; three POS lint rule ids renamed to register-* with REGISTER_TRIGGER_TYPES = [TileGrid, SelectionPanel, Numpad]; 748 tests green.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-07-05T12:47:30Z
- **Completed:** 2026-07-05T12:55:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `git mv product_tiles.rs tiles.rs`; `setupProductTiles` → `setupTiles` in JS source string
- `runtime/mod.rs`: `mod tiles;` (alphabetical after tabs), `s.push_str(tiles::SOURCE)`, dispatcher `setupTiles()`, both drift arrays (`bundle_contains_all_setup_functions` + `dispatcher_invokes_every_setup`) updated
- `form_guards.rs` comment: "Skip ProductTile +/- controls" → "Skip Tile +/- controls"
- `design/rules.rs`: three rule ids (`pos-fill-viewport`→`register-fill-viewport`, `pos-grid-fill`→`register-grid-fill`, `pos-cart-present`→`register-selection-present`); `POS_TRIGGER_TYPES`→`REGISTER_TRIGGER_TYPES = ["TileGrid", "SelectionPanel", "Numpad"]`; type comparisons `ProductGrid`→`TileGrid`, `CartPanel`→`SelectionPanel`; test function names and fixture JSON updated; section comment neutralized
- `docs/src/design-system/patterns.md`: three `## pos-*` sections updated to `## register-*` with matching titles, rationales, example JSON types, and allow snippets
- cargo fmt + clippy clean; 748 tests pass (0 failures)

## Task Commits

1. **Task 1: Rename runtime/product_tiles.rs to tiles.rs; setupTiles wired; form_guards comment** - `7d460b19` (feat)
2. **Task 2: Rename POS lint rule ids and trigger types to register-* vocabulary** - `726062be` (feat)

## Files Created/Modified

- `ferro-json-ui/src/runtime/tiles.rs` — renamed from product_tiles.rs; JS function setupTiles
- `ferro-json-ui/src/runtime/mod.rs` — mod tiles (alphabetical); concat; dispatcher; drift arrays
- `ferro-json-ui/src/runtime/form_guards.rs` — comment: Skip Tile +/- controls
- `ferro-json-ui/src/design/rules.rs` — register-* rule ids; REGISTER_TRIGGER_TYPES; test renames
- `docs/src/design-system/patterns.md` — three register-* rule sections

## Decisions Made

- Updated `patterns.md` in the same Task 2 commit: the `patterns_md_matches_rule_registry` drift guard in `ferro-json-ui/src/design/mod.rs` enforces that every rule id in the registry has a section in that doc file. This is a correctness requirement (Rule 2), not a separate deliverable.
- `cargo fmt` reordered `mod tiles;` to be alphabetical (after `mod tabs;`, before `mod toasts;`). Applied before committing.
- `has_grid` single-line form in `check_pos_cart_present` was required by `cargo fmt` (chain fits under the line length limit).

## Deviations from Plan

**1. [Rule 2 - Missing critical functionality] patterns.md updated alongside rules.rs rename**
- **Found during:** Task 2 test run
- **Issue:** `patterns_md_matches_rule_registry` drift guard failed with "patterns.md is missing rule id `register-fill-viewport`" — the plan listed only `ferro-json-ui/src/design/rules.rs` in `files`, but the drift guard enforces doc parity
- **Fix:** Updated the three `## pos-*` sections in `docs/src/design-system/patterns.md` to match the new register-* ids and neutral type names (TileGrid/SelectionPanel)
- **Files modified:** `docs/src/design-system/patterns.md`
- **Commit:** `726062be`

## Known Stubs

None — pure rename; no new rendering or UI stubs introduced.

## Threat Flags

No new attack surface. T-255-03 and T-255-04 accepted per plan threat model: rules are diagnostics-only pre-render; bundle is a static LazyLock string.

## Next Phase Readiness

- Plan 255-03 can rename ferro-mcp tool descriptions, docs, and `app/views/cassa.json` against the final `setupTiles` / `register-*` vocabulary
- Phase 256 renderers can target `TileGrid`/`SelectionPanel` component type names without vocabulary drift risk

## Self-Check

### Files exist:

- `ferro-json-ui/src/runtime/tiles.rs` with `function setupTiles` ✓
- `ferro-json-ui/src/runtime/product_tiles.rs` does NOT exist ✓
- `ferro-json-ui/src/design/rules.rs` with `register-fill-viewport` ✓
- `ferro-json-ui/src/design/rules.rs` with `REGISTER_TRIGGER_TYPES` ✓
- `docs/src/design-system/patterns.md` with `register-fill-viewport` section ✓

### Commits exist:

- `7d460b19` ✓
- `726062be` ✓

## Self-Check: PASSED

---
*Phase: 255-pos-runtime-modules-double-submit-protection*
*Completed: 2026-07-05*

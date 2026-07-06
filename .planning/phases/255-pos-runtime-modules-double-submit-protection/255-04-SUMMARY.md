---
phase: 255-pos-runtime-modules-double-submit-protection
plan: 04
subsystem: ui
tags: [ferro-json-ui, runtime, numpad, filters, POS, SC-1, SC-2, SC-3, ES5]

# Dependency graph
requires:
  - phase: 255-01
    provides: TileProps, renamed render_tile with data-filter-tokens, data-filter-text already emitted
  - phase: 255-02
    provides: runtime/tiles.rs module shape (renamed from product_tiles.rs, setupTiles in drift lists)
  - phase: 255-03
    provides: SC-0 fully closed (zero ProductTile/product_tile hits)

provides:
  - ferro-json-ui/src/runtime/numpad.rs: ES5 setupNumpad + initNumpad (quantity + price cents-shift)
  - ferro-json-ui/src/runtime/filters.rs: ES5 setupFilters + initFilterScope (token+text AND matching)
  - ferro-json-ui/src/render/atoms.rs: tile_escapes_filter_text XSS guard test (T-255-07)
  - ferro-json-ui/src/runtime/mod.rs: mod numpad/filters; concat; dispatcher; both drift lists extended; SC-3 test

affects:
  - 255-05 (disable_on_submit guard — builds on the same mod.rs wiring pattern)
  - 256 (render functions target data-filter-text/data-filter-tokens/data-numpad-* contracts)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - ES5 runtime module: pub(super) const SOURCE, var/function, no arrows/template literals/let/const
    - Event delegation with e.target.closest() — DOM Level 4 (not ES6); first use in this codebase
    - No-op guard pattern: if (items.length === 0) return; at top of setup*() functions
    - Integer-cents price entry: hidden field carries raw digit string, display is presentational only
    - Style-display hide/show: el.style.display = 'none'/''; not hidden attribute or Tailwind class
    - AND-intersection filter matching: tokenMatch && searchMatch per tile
    - Space->hyphen tab-token normalization before token comparison (matches render_tile emit contract)
    - Drift-list atomicity: both bundle_contains_all_setup_functions and dispatcher_invokes_every_setup updated in same commit as concat list and dispatcher

key-files:
  created:
    - ferro-json-ui/src/runtime/numpad.rs
    - ferro-json-ui/src/runtime/filters.rs
  modified:
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/runtime/mod.rs

key-decisions:
  - "data-filter-text attribute was already emitted in render_tile from Plan 255-01; Task 1 added only the tile_escapes_filter_text XSS guard test"
  - "filters.rs uses inline closure (function initFilterTab) for shared activeToken/searchText mutable state — valid ES5 closure pattern, not ES6"
  - "Rust const SOURCE declaration triggers false positive in plan's ES5 grep check (\\bconst\\b) — affects all runtime modules; JS content is ES5-clean, verified by sed-skipping the declaration line"
  - "No form_guards.rs changes in this plan — D-05 (data-numpad-input in number guard) is part of Plan 255-05 per files_modified scope"

patterns-established:
  - "e.target.closest('[data-numpad-key]') — first use of closest() in the runtime codebase; DOM Level 4, valid ES5"
  - "Price mode integer-cents contract: hidden field = raw digit string, server re-validates"

requirements-completed: [POS-08]

# Metrics
duration: ~5min
completed: 2026-07-05
---

# Phase 255 Plan 04: POS Runtime Modules (numpad + filters) + SC-1/2/3 Summary

**ES5 setupNumpad (tap-surface keypad, quantity + price cents-shift) and setupFilters (token/text AND-intersection tile visibility) wired into the bundle with both drift lists extended and SC-3 inline-source test green. The data-attribute contract is stable for Phase 256 render functions.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-07-05T13:18:02Z
- **Completed:** 2026-07-05T13:22:50Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

**Task 1 — tile_escapes_filter_text test (atoms.rs):**
- `data-filter-text` was already unconditionally emitted in `render_tile` (from Plan 255-01) and `tile_legacy_render_is_byte_identical` already asserted its presence
- Added `tile_escapes_filter_text` test: builds a Tile with `name="Ba"r <item>"`, asserts HTML-escaped in `data-filter-text` (double-quote → `&quot;`, less-than → `&lt;`)
- T-255-07 XSS guard now in place for the data-filter-text emission path

**Task 2 — runtime/numpad.rs:**
- `setupNumpad()` with no-op guard when `[data-numpad]` absent (SC-2/D-06)
- Event delegation: one `click` listener per container, `e.target.closest('[data-numpad-key]')` (D-01)
- Quantity mode: digit append, leading-zero collapse, backspace, clear, max 9 digits (D-02/D-06)
- Price mode: cents-shift entry; hidden field carries integer cents string (e.g. "125" for €1.25); display formatted via `numpadPriceDisplay` (D-03)
- Every key tap: `input.dispatchEvent(new Event('input', { bubbles: true }))` (D-04)
- Module-level comment documents integer-cents contract

**Task 3 — runtime/filters.rs + mod.rs wiring:**
- `setupFilters()` with no-op guard when `[data-filter-scope]` absent (SC-2/D-12)
- `initFilterScope`: tab click listeners + optional search input listener per scope
- `applyFilter`: iterates `[data-filter-text]` tiles; sets `tile.style.display` (D-11)
- `filterTokenMatch`: D-09/D-10 — empty activeToken = All; specific token requires `data-filter-tokens` containing it; untokened tiles hidden under specific tabs
- `filterSearchMatch`: case-insensitive `indexOf` on `data-filter-text`
- Space→hyphen tab token normalization before comparison (Pitfall 7)
- Active-tab class toggle mirrors `tabs.rs`: `border-primary`/`text-primary`/`font-semibold` ↔ `border-transparent`/`text-text-muted`/`hover:text-text` (D-12)
- `mod.rs`: `mod numpad; mod filters;` declared; `numpad::SOURCE` + `filters::SOURCE` concatenated after `tiles::SOURCE`
- `mod.rs`: dispatcher entries `setupNumpad();` and `setupFilters();` after `setupTiles()`
- `mod.rs`: both drift lists (`bundle_contains_all_setup_functions` + `dispatcher_invokes_every_setup`) extended with the two new setup functions
- `mod.rs`: SC-3 `runtime_exposes_numpad_and_filter_contract` inline-source test added

## Task Commits

1. **Task 1: tile_escapes_filter_text test** — `4e3a431a` (test)
2. **Task 2: runtime/numpad.rs** — `06f782e8` (feat)
3. **Task 3: runtime/filters.rs + mod.rs wiring** — `c2774ccf` (feat)

## Files Created/Modified

- `ferro-json-ui/src/render/atoms.rs` — added `tile_escapes_filter_text` test
- `ferro-json-ui/src/runtime/numpad.rs` — new; ES5 setupNumpad + helpers
- `ferro-json-ui/src/runtime/filters.rs` — new; ES5 setupFilters + helpers
- `ferro-json-ui/src/runtime/mod.rs` — mod declarations; concat; dispatcher; both drift lists; SC-3 test

## Decisions Made

- `data-filter-text` emission was pre-existing from Plan 255-01 — Task 1 only needed the escaping test (the render implementation was already complete).
- ES5 plan verify grep (`\bconst\b`) has a known false positive on the Rust `const SOURCE` declaration; this affects all runtime modules equally. The JavaScript content is verified ES5-clean by skipping the declaration line.
- No changes to `form_guards.rs` in this plan — D-05 (extending the number guard to include `data-numpad-input`) is scoped to Plan 255-05 per `files_modified` frontmatter.

## Deviations from Plan

### Auto-noted non-issues

**1. [Plan Check - False positive] ES5 grep check hits Rust const keyword**
- **Found during:** Task 2 verify
- **Issue:** The plan's verify command `grep -nE '...|\bconst \b|...'` matches `pub(super) const SOURCE` — the Rust declaration on line 1 of every runtime module file. The same false positive exists on `tiles.rs` and all other existing runtime modules.
- **Fix:** Verified ES5 compliance by checking only the JavaScript content (lines after the declaration). No JS `const`/`let`/arrow functions/template literals present in either new module.
- **Scope:** Plan check defect; no code change needed.

**2. [Pre-existing] data-filter-text already emitted from Plan 255-01**
- **Found during:** Task 1 research
- **Issue:** `render_tile` at line 1388 already had `data-filter-text="{name}"` and `tile_legacy_render_is_byte_identical` already asserted `data-filter-text="Espresso"` — both from Plan 255-01 work.
- **Fix:** Task 1 reduced to adding only the `tile_escapes_filter_text` test (the remaining planned artifact). The done criteria are fully met.

## Known Stubs

None — the data-attribute contract is fully specified. The Phase 256 render functions (`render_tile_grid`, `render_filter_tabs`, `render_numpad`) will emit the attributes this runtime reads; those renderers are the Phase 256 deliverable.

## Threat Flags

No new attack surface beyond the plan's threat model:
- **T-255-07** (data-filter-text XSS): mitigated by `html_escape(&props.name)` in `render_tile` (pre-existing from 255-01) + `tile_escapes_filter_text` test asserting escaping.
- **T-255-08** (client-side visibility toggle): accepted — filters.rs is a presentational UX affordance; server never trusts filter state.
- **T-255-09** (integer-cents money): mitigated — hidden field carries integer cents string; `numpadPriceDisplay` is display-only; module comment documents the contract.

## Self-Check

### Files exist:

- `ferro-json-ui/src/runtime/numpad.rs` with `function setupNumpad` ✓
- `ferro-json-ui/src/runtime/filters.rs` with `function setupFilters` ✓
- `ferro-json-ui/src/render/atoms.rs` with `fn tile_escapes_filter_text` ✓
- `ferro-json-ui/src/runtime/mod.rs` with `mod numpad; mod filters;` ✓
- `ferro-json-ui/src/runtime/mod.rs` with `numpad::SOURCE` + `filters::SOURCE` ✓
- `ferro-json-ui/src/runtime/mod.rs` with `"setupNumpad"` + `"setupFilters"` in both drift lists ✓
- `ferro-json-ui/src/runtime/mod.rs` with `runtime_exposes_numpad_and_filter_contract` test ✓

### Commits exist:

- `4e3a431a` ✓
- `06f782e8` ✓
- `c2774ccf` ✓

### Tests:

- `cargo test -p ferro-json-ui --all-features`: 750 tests, 0 failed ✓
- `tile_escapes_filter_text` ... ok ✓
- `tile_legacy_render_is_byte_identical` ... ok ✓
- `bundle_contains_all_setup_functions` ... ok ✓
- `dispatcher_invokes_every_setup` ... ok ✓
- `runtime_exposes_numpad_and_filter_contract` ... ok ✓

## Self-Check: PASSED

---
*Phase: 255-pos-runtime-modules-double-submit-protection*
*Completed: 2026-07-05*

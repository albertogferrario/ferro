---
phase: 254-props-contracts-touch-foundation-design-rules
plan: "02"
subsystem: ferro-json-ui
tags: [pos, touch, css, render, drift-guard, tdd]
dependency_graph:
  requires: [254-01]
  provides: [render::classes POS constants, pos-tap-highlight utility, render_product_tile migration, drift-guard test]
  affects: [ferro-json-ui, ferro-base.css]
tech_stack:
  added: []
  patterns: [pub-mod-dead_code-exemption, drift-guard-read_dir-scan, TDD red-green]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/render/classes.rs
    - ferro-json-ui/assets/input.css
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/assets/ferro-base.css
decisions:
  - "Test assertions use POS_TOUCH_ACTION/POS_HIT_TARGET_MIN constants (not raw literals) so the drift-guard scan does not trip on the test file"
  - "render::classes promoted to pub mod so unconsumed Phase-256 POS constants are dead_code-exempt without #[allow]"
metrics:
  duration_seconds: 1130
  completed: "2026-07-05T01:42:00Z"
  tasks_completed: 3
  files_modified: 5
---

# Phase 254 Plan 02: POS Touch Foundation + render_product_tile Migration Summary

Six pub POS touch constants in `render::classes`, drift-guard preventing future literal regression, `render_product_tile` migrated to constants with `data-product-categories` emission (HTML-escaped), `pos-tap-highlight` @utility, and a regenerated `ferro-base.css` — all CI gates green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | POS touch constants + pos-tap-highlight @utility | 2ac3f7f6 | render/mod.rs, render/classes.rs, assets/input.css |
| 2 | render_product_tile migration + data-product-categories + drift-guard | 0456e28c | render/atoms.rs, render/classes.rs |
| 3 | Regenerate ferro-base.css + full CI-exact gate | 08166f54 | assets/ferro-base.css, render/classes.rs (fmt) |

## What Was Built

### Task 1 — POS constants + @utility

`render::classes` promoted from `pub(crate) mod` to `pub mod` so the six new `pub const` constants are accessible from outside the crate (dead_code-exempt). Six constants added:

| Constant | Value |
|----------|-------|
| `POS_TOUCH_ACTION` | `"touch-manipulation"` |
| `POS_HIT_TARGET_MIN` | `"min-h-[44px] min-w-[44px]"` |
| `POS_HIT_TARGET_NUMPAD` | `"min-h-[56px] min-w-[56px]"` |
| `POS_PRESS_ACTIVE` | `"active:scale-95 active:bg-border"` |
| `POS_OVERSCROLL_CONTAIN` | `"overscroll-contain"` |
| `POS_TAP_HIGHLIGHT` | `"pos-tap-highlight"` |

`@utility pos-tap-highlight { -webkit-tap-highlight-color: transparent; }` added to `input.css` after the `duration-slow` block (Path B — guaranteed CSS generation).

### Task 2 — render_product_tile migration (TDD)

RED: 2 tests failed (`product_tile_emits_data_product_categories`, `product_tile_escapes_categories`). `product_tile_legacy_render_is_byte_identical` passed — the existing output already contained the touch-manipulation literal.

GREEN: Three inline literals removed from `render_product_tile`:
- outer div `touch-manipulation` → `{POS_TOUCH_ACTION}`
- both qty buttons `min-h-[44px] min-w-[44px]` → `{POS_HIT_TARGET_MIN}`

`categories_attr` computation added; guarded by `props.categories.is_empty()`; value passed through `html_escape`.

Drift-guard test `pos_render_functions_use_constants_not_literals` added to `classes.rs`: scans every `src/render/*.rs` except `classes.rs` for the three guarded literals — auto-covers Phase 256 new render files without re-enrollment.

**Deviation:** Test assertions in `product_tile_legacy_render_is_byte_identical` use `POS_TOUCH_ACTION` / `POS_HIT_TARGET_MIN` constants (via `use crate::render::classes::{...}`) instead of raw strings, because the drift-guard scans test source too. This keeps the guard accurate.

### Task 3 — CSS regen + CI gate

`scripts/gen-ferro-base-css.sh` ran once; `ferro-base.css` gained:
- `.pos-tap-highlight` utility
- `overscroll-contain`
- `active:scale-95` and `active:bg-border`

Full CI-exact gate green: `cargo fmt --all -- --check` (fmt fixed one test block reformatting), `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, `cargo doc --no-deps -p ferro-json-ui`. No schema churn.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test assertions used raw literals that tripped the drift-guard**
- **Found during:** Task 2 GREEN verification
- **Issue:** `product_tile_legacy_render_is_byte_identical` asserted `html.contains("touch-manipulation")` — the literal string appears in test source and the drift-guard scans all `src/render/*.rs` files; it found the literal and failed.
- **Fix:** Changed assertions to use `html.contains(POS_TOUCH_ACTION)` and `html.matches(POS_HIT_TARGET_MIN)` via `use crate::render::classes::{POS_HIT_TARGET_MIN, POS_TOUCH_ACTION}`.
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`
- **Commit:** included in 0456e28c

**2. [Rule 1 - Bug] cargo fmt reformatted the for-loop assert in classes.rs**
- **Found during:** Task 3 CI gate
- **Issue:** The `pos_constants_are_full_literals_and_token_compliant` test used a compact `assert!(!..., "...")` form that rustfmt reformatted to multi-line.
- **Fix:** Ran `cargo fmt --all` to apply canonical formatting before committing.
- **Files modified:** `ferro-json-ui/src/render/classes.rs`
- **Commit:** 08166f54

## Known Stubs

None. All constants and render changes are complete. The four constants `POS_HIT_TARGET_NUMPAD`, `POS_PRESS_ACTIVE`, `POS_OVERSCROLL_CONTAIN`, `POS_TAP_HIGHLIGHT` have no Phase-254 render consumer — that is by design (Phase 256 consumers). The `pub mod` visibility makes them dead_code-exempt.

## Threat Flags

None. The one threat boundary (`ProductTileProps.categories` → HTML attribute) is mitigated: `html_escape(&props.categories.join(" "))` with an escaping test (`product_tile_escapes_categories`). No new network endpoints or auth paths introduced.

## Self-Check

- [x] `ferro-json-ui/src/render/classes.rs` — 6 POS constants + drift-guard + composition tests exist
- [x] `ferro-json-ui/src/render/atoms.rs` — zero raw POS literals; data-product-categories guarded by is_empty, html_escaped
- [x] `ferro-json-ui/assets/input.css` — `@utility pos-tap-highlight` block present at line 103
- [x] `ferro-json-ui/assets/ferro-base.css` — pos-tap-highlight, overscroll-contain, active:scale-95/active:bg-border present
- [x] Commits 2ac3f7f6, 0456e28c, 08166f54 exist on master
- [x] Full CI-exact gate (fmt + clippy --all-features + test --all-features + doc) green

## Self-Check: PASSED

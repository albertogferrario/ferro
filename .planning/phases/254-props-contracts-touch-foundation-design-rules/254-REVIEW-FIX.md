---
phase: 254-props-contracts-touch-foundation-design-rules
fixed_at: 2026-07-05T01:54:39Z
review_path: .planning/phases/254-props-contracts-touch-foundation-design-rules/254-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 254: Code Review Fix Report

**Fixed at:** 2026-07-05T01:54:39Z
**Source review:** .planning/phases/254-props-contracts-touch-foundation-design-rules/254-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (fix_scope: critical_warning; 5 Info findings out of scope)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Space-separated `data-product-categories` contract silently corrupts category names containing spaces

**Files modified:** `ferro-json-ui/src/component.rs`, `ferro-json-ui/src/render/atoms.rs`
**Commit:** f91f6a40
**Applied fix:** Option (a) from the review — kept the CSS-selector-friendly space-separated token list and closed the ambiguity by normalizing spaces to hyphens at render time. `render_product_tile` now maps each category through `c.replace(' ', "-")` before joining and escaping, with a contract comment at the render site. The token-list constraint is documented in `ProductTileProps::categories` rustdoc (`"Bevande calde"` → token `Bevande-calde`; filter runtimes must apply the same normalization), and the matching-side requirement is recorded on `CategoryNavProps::items` for the Phase 255 `setupPosFilter` handoff. Added test `product_tile_normalizes_spaces_in_category_names` asserting `data-product-categories="Bevande-calde food"`. Legacy render path unchanged (empty `categories` still emits no attribute); existing categories/escaping fixtures pass unchanged. Verified: fmt clean, 10/10 `product_tile` tests green (`cargo test -p ferro-json-ui product_tile`).

### WR-02: `pos-grid-fill` false-positives on a `$data`-bound `fill` prop

**Files modified:** `ferro-json-ui/src/design/rules.rs`
**Commit:** 01397de6
**Applied fix:** `check_pos_grid_fill` now uses non-null acceptance mirroring the `list-empty-state`/`breadcrumb-on-subpages` regression-guard pattern: `fill` counts as set for any non-null value other than a literal `false` (`.map(|v| !v.is_null() && v.as_bool() != Some(false))`). A literal `fill: false` or absent `fill` still warns. Added fixture `pos_grid_fill_data_bound_fill_no_misfire` with `"fill": {"$data": "/ui/fill"}` on the root Grid asserting 0 findings — covering the gated prop itself, which the existing `pos_grid_fill_data_bound_no_misfire` (child-prop) fixture did not. Rule remains `Severity::Warning`, diagnostics-only, internal gates untouched. Verified: fmt clean, 4/4 `pos_grid_fill` tests green (`cargo test -p ferro-json-ui pos_grid_fill`), including the pre-existing violating fixture still firing.

## Skipped Issues

None — both in-scope findings were fixed. Info findings IN-01 through IN-05 were out of fix scope (`critical_warning`).

---

_Fixed: 2026-07-05T01:54:39Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
